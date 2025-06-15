use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use std::fs;
use std::io::BufReader;
use std::path::PathBuf;

use crate::DJWavFixerError;
use crate::errors::Result;
use crate::file_loader::{WavFile, get_distinct_wav_files};
use crate::riff_parser::{FMT_MAGIC, RIFF_MAGIC, RiffFile, WAVE_MAGIC};
use crate::wav_format::WaveFormatExtensible;

fn _load_wav_file(path: &PathBuf) -> Result<WaveFormatExtensible> {
    let single_file = fs::File::open(path)?;
    let file_size = single_file.metadata()?.len();

    let mut reader = BufReader::new(single_file);

    let mut riff_file = RiffFile::try_new(
        &mut reader,
        file_size - RIFF_MAGIC.len() as u64 - FMT_MAGIC.len() as u64,
    )?;

    let (reader, chunk) =
        riff_file
            .get_chunk_mut(RIFF_MAGIC)
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
            .get_subchunk_mut(FMT_MAGIC)
            .ok_or(DJWavFixerError::RiffHeaderError(
                "Missing 'fmt ' subchunk".to_string(),
            ))?;

    let fmt_data = fmt_subchunk.data(reader)?;

    WaveFormatExtensible::try_from(fmt_data)
}

pub fn load_wav_file(path: &PathBuf) -> WavFile {
    WavFile {
        path: path.clone(),
        wave_format_info: _load_wav_file(path),
    }
}

pub fn load_wav_files(files: &[PathBuf]) -> Result<Vec<WavFile>> {
    Ok(get_distinct_wav_files(files)?
        .iter()
        .map(load_wav_file)
        .collect())
}

#[cfg(feature = "parallel")]
pub fn load_wav_files_rayon(
    files: &[PathBuf],
    rayon_pool: &rayon::ThreadPool,
) -> Result<Vec<WavFile>> {
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
    use crate::file_loader::tests::{create_wav_files_for_test, readable_test_files};
    use std::path::PathBuf;
    use std::thread;

    fn evaluate_wav_files(wav_files: &[WavFile]) {
        for wav_file in wav_files {
            let loaded_file = load_wav_file(&wav_file.path);
            assert_eq!(wav_file, &loaded_file);
        }
    }

    #[test]
    fn test_load_single_file_all_types() {
        let wav_files = create_wav_files_for_test(readable_test_files());

        evaluate_wav_files(&wav_files);
    }

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

    #[test]
    fn test_load_all_files() {
        let readable_test_files = create_wav_files_for_test(readable_test_files());

        let files_to_load = readable_test_files
            .iter()
            .map(|x| x.path.clone())
            .collect::<Vec<_>>();

        let wav_files = load_wav_files(&files_to_load).expect("Failed to load all files");
        assert_eq!(readable_test_files.len(), wav_files.len());

        for (wav_file, expected_file) in wav_files.iter().zip(&readable_test_files) {
            assert_eq!(wav_file, expected_file);
        }
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn test_load_all_files_with_rayon() {
        let readable_test_files = create_wav_files_for_test(readable_test_files());

        let files_to_load = readable_test_files
            .iter()
            .map(|x| x.path.clone())
            .collect::<Vec<_>>();

        let rayon_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(thread::available_parallelism().unwrap().get())
            .build()
            .expect("Failed to create Rayon thread pool");

        let wav_files =
            load_wav_files_rayon(&files_to_load, &rayon_pool).expect("Failed to load all files");
        assert_eq!(readable_test_files.len(), wav_files.len());

        for (wav_file, expected_file) in wav_files.iter().zip(&readable_test_files) {
            assert_eq!(wav_file, expected_file);
        }
    }
}
