use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use crate::DJWavFixerError;
use crate::errors::Result;
use crate::file_loader::get_distinct_wav_files;
use crate::riff_parser::{FMT_MAGIC, RIFF_MAGIC, RiffFile, WAVE_MAGIC};
use crate::wav_file::{WavFile, WavFileLoadStatus, WaveFormatExtensible};

fn _load_riff_file(path: &PathBuf) -> Result<RiffFile<BufReader<File>>> {
    let single_file = File::open(path)?;
    let file_size = single_file.metadata()?.len();

    let reader = BufReader::new(single_file);

    RiffFile::try_new(
        reader,
        file_size - RIFF_MAGIC.len() as u64 - FMT_MAGIC.len() as u64,
    )
}

fn parse_wav_format(riff_file: &mut RiffFile<BufReader<File>>) -> Result<WaveFormatExtensible> {
    let (reader, chunk) =
        riff_file
            .get_chunk_and_reader(&RIFF_MAGIC)
            .ok_or(DJWavFixerError::RiffHeaderError(
                "Missing 'RIFF' chunk".to_string(),
            ))?;

    if chunk.format() != WAVE_MAGIC {
        return Err(DJWavFixerError::RiffHeaderError(
            "Invalid RIFF format: expected 'WAVE'".to_string(),
        ));
    }

    let fmt_subchunk =
        chunk
            .get_subchunk_mut(&FMT_MAGIC)
            .ok_or(DJWavFixerError::RiffHeaderError(
                "Missing 'fmt ' subchunk".to_string(),
            ))?;

    WaveFormatExtensible::try_from(fmt_subchunk.read_data(reader)?)
}

pub fn load_wav_file(path: &PathBuf) -> WavFile<BufReader<File>> {
    let riff_file = _load_riff_file(path);

    WavFile {
        path: path.clone(),
        load_status: match riff_file {
            Ok(mut riff_file) => match parse_wav_format(&mut riff_file) {
                Ok(wave_format_info) => WavFileLoadStatus::Success {
                    riff_file,
                    wave_format_info,
                },
                Err(error) => WavFileLoadStatus::WavFileInvalid { riff_file, error },
            },
            Err(error) => WavFileLoadStatus::RiffFileInvalid { error },
        },
    }
}

pub fn load_wav_files(files: &[PathBuf]) -> Result<Vec<WavFile<BufReader<File>>>> {
    Ok(get_distinct_wav_files(files)?
        .iter()
        .map(load_wav_file)
        .collect())
}

#[cfg(feature = "parallel")]
pub fn load_wav_files_rayon(
    files: &[PathBuf],
    rayon_pool: &rayon::ThreadPool,
) -> Result<Vec<WavFile<BufReader<File>>>> {
    let distinct_files = get_distinct_wav_files(files)?;

    Ok(rayon_pool
        .install(|| distinct_files.par_iter().map(load_wav_file))
        .collect())
}

pub fn get_all_wav_files_in_directory(
    directory: &PathBuf,
    recursive: bool,
) -> Result<Vec<PathBuf>> {
    Ok(fs::read_dir(directory)?
        .map(|entry| {
            let entry = entry?;
            let path = entry.path();
            let file_type = entry.file_type()?;

            if file_type.is_dir() && recursive {
                // Recursively collect from subdirectories
                let dir_files = get_all_wav_files_in_directory(&path, true)?;
                Ok((!dir_files.is_empty()).then_some(dir_files))
            } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "wav") {
                Ok(Some(vec![path]))
            } else {
                Ok(None)
            }
        })
        .filter_map(Result::transpose)
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_loader::tests::readable_test_files;
    use crate::wav_file::WavFileLoadStatus;
    use std::path::PathBuf;
    use std::thread;

    #[test]
    fn test_get_all_wav_files_in_directory_nonrecursive() {
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("test")
            .join("audio_files");

        let mut files =
            get_all_wav_files_in_directory(&directory, false).expect("Failed to read directory");
        files.sort();

        let expected_files = ["as_alaw.wav", "as_ulaw.wav", "original.wav"]
            .map(PathBuf::from)
            .map(|path| directory.join(path))
            .to_vec();

        assert_eq!(files.len(), expected_files.len());
        assert_eq!(files, expected_files);
    }

    #[test]
    fn test_get_all_wav_files_in_directory_recursive() {
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("test")
            .join("audio_files");
        let mut files =
            get_all_wav_files_in_directory(&directory, true).expect("Failed to read directory");
        files.sort();

        let expected_files = [
            "as_alaw.wav",
            "as_ulaw.wav",
            "float_pcm/as_f32_pcm.wav",
            "float_pcm/as_f64_pcm.wav",
            "int_pcm/as_int16_pcm.wav",
            "int_pcm/as_int24_pcm.wav",
            "int_pcm/as_int32_pcm.wav",
            "int_pcm/uint/as_uint8_pcm.wav",
            "original.wav",
        ]
        .map(PathBuf::from)
        .map(|path| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("resources")
                .join("test")
                .join("audio_files")
                .join(path)
        })
        .to_vec();

        assert_eq!(files.len(), expected_files.len());
        assert_eq!(files, expected_files);
    }

    fn compare_files<R>(
        wav_files: &[WavFile<R>],
        readable_test_files: &[(PathBuf, WaveFormatExtensible)],
    ) {
        assert_eq!(wav_files.len(), readable_test_files.len());

        let compare_file = |(wav_file, (matching_path, expected_format)): (
            &WavFile<R>,
            &(PathBuf, WaveFormatExtensible),
        )| {
            assert_eq!(&wav_file.path, matching_path);
            let WavFileLoadStatus::Success {
                wave_format_info, ..
            } = &wav_file.load_status
            else {
                panic!(
                    "Expected WavFileLoadStatus::Success, got {:?}",
                    wav_file.load_status
                );
            };
            assert_eq!(wave_format_info, expected_format);
        };

        wav_files
            .iter()
            .zip(readable_test_files.iter())
            .for_each(compare_file);
    }

    #[test]
    fn test_load_single_file_all_types() {
        let readable_test_files = readable_test_files();

        let wav_files = readable_test_files
            .iter()
            .map(|(path, _)| load_wav_file(path))
            .collect::<Vec<_>>();

        compare_files(&wav_files, &readable_test_files);
    }

    #[test]
    fn test_load_all_files() {
        let readable_test_files = readable_test_files();

        let files_to_load = readable_test_files
            .iter()
            .map(|(path, _)| path.clone())
            .collect::<Vec<_>>();
        let wav_files = load_wav_files(&files_to_load).expect("Failed to load all files");

        compare_files(&wav_files, &readable_test_files);
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn test_load_all_files_with_rayon() {
        let readable_test_files = readable_test_files();

        let files_to_load = readable_test_files
            .iter()
            .map(|(path, _)| path.clone())
            .collect::<Vec<_>>();

        let rayon_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(thread::available_parallelism().unwrap().get())
            .build()
            .expect("Failed to create Rayon thread pool");

        let wav_files =
            load_wav_files_rayon(&files_to_load, &rayon_pool).expect("Failed to load all files");

        compare_files(&wav_files, &readable_test_files);
    }
}
