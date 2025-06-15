use indexmap::IndexSet;
use std::fmt::Write;
use std::path;
use std::path::PathBuf;

pub(crate) mod blocking_loader;

use crate::errors::Result;
use crate::wav_format::{WaveFormatExtensible, WaveFormatType};
pub use blocking_loader::{
    get_all_wav_files_in_directory, load_wav_file, load_wav_files, load_wav_files_rayon,
};

fn get_distinct_wav_files(files: &[PathBuf]) -> Result<Vec<PathBuf>> {
    files
        .iter()
        .map(|f| path::absolute(f).map_err(Into::into))
        .collect::<Result<IndexSet<_>>>()
        .map(|set| set.into_iter().collect())
}

#[derive(Clone, Debug, PartialEq)]
pub struct WavFile {
    pub(crate) path: PathBuf,
    pub(crate) wave_format_info: Result<WaveFormatExtensible>,
}

impl WavFile {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn needs_fixing(&self) -> Option<bool> {
        match self.wave_format_info {
            Ok(ref wave_format_info) => Some(
                !wave_format_info.is_sample_bits_supported_by_players()
                    || !wave_format_info.is_integer_pcm(),
            ),
            Err(_) => None,
        }
    }

    pub fn can_fix(&self) -> Option<bool> {
        match self.wave_format_info {
            Ok(ref wave_format_info) => {
                if wave_format_info.is_sample_bits_supported_by_players() {
                    Some(
                        self.path.is_file()
                            && wave_format_info.format_tag == WaveFormatType::WaveFormatExtensible
                            && wave_format_info.bits_per_sample
                                == wave_format_info
                                    .valid_bits_per_sample
                                    .unwrap_or(wave_format_info.bits_per_sample),
                    )
                } else {
                    Some(false)
                }
            }
            Err(_) => None,
        }
    }

    pub fn write_information(&self, mut writer: impl Write) -> Result<()> {
        writeln!(writer, "  Path: {}", self.path.display())?;
        match self.wave_format_info.as_ref() {
            Ok(wave_format_info) => {
                wave_format_info.write_information(&mut writer)?;
                if let Some(needs_fixing) = self.needs_fixing() {
                    writeln!(writer, "  Needs Fixing: {}", needs_fixing)?;
                    if needs_fixing {
                        if let Some(can_fix) = self.can_fix() {
                            writeln!(writer, "  Can Fix: {}", can_fix)?;
                        }
                    }
                }
            }
            Err(err) => {
                writeln!(writer, "  Error: {}", err)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::file_loader::WavFile;
    use crate::wav_format::{WaveAudioChannels, WaveFormatExtensible, WaveFormatType};
    use std::path::PathBuf;

    pub(crate) fn readable_test_files() -> [(&'static str, WaveFormatExtensible); 9] {
        [
            (
                "int_pcm/uint/as_uint8_pcm",
                WaveFormatExtensible {
                    format_tag: WaveFormatType::IntegerPCM,
                    channels: WaveAudioChannels::Stereo,
                    sample_rate: 44100,
                    avg_bytes_per_second: 88200,
                    block_align: 2,
                    bits_per_sample: 8,
                    valid_bits_per_sample: None,
                    channel_mask: 0,
                    subformat_data: vec![],
                },
            ),
            (
                "int_pcm/as_int16_pcm",
                WaveFormatExtensible {
                    format_tag: WaveFormatType::IntegerPCM,
                    channels: WaveAudioChannels::Stereo,
                    sample_rate: 44100,
                    avg_bytes_per_second: 176400,
                    block_align: 4,
                    bits_per_sample: 16,
                    valid_bits_per_sample: None,
                    channel_mask: 0,
                    subformat_data: vec![],
                },
            ),
            (
                "int_pcm/as_int24_pcm",
                WaveFormatExtensible {
                    format_tag: WaveFormatType::IntegerPCM,
                    channels: WaveAudioChannels::Stereo,
                    sample_rate: 44100,
                    avg_bytes_per_second: 264600,
                    block_align: 6,
                    bits_per_sample: 24,
                    valid_bits_per_sample: None,
                    channel_mask: 0,
                    subformat_data: vec![],
                },
            ),
            (
                "int_pcm/as_int32_pcm",
                WaveFormatExtensible {
                    format_tag: WaveFormatType::IntegerPCM,
                    channels: WaveAudioChannels::Stereo,
                    sample_rate: 44100,
                    avg_bytes_per_second: 352800,
                    block_align: 8,
                    bits_per_sample: 32,
                    valid_bits_per_sample: None,
                    channel_mask: 0,
                    subformat_data: vec![],
                },
            ),
            (
                "float_pcm/as_f32_pcm",
                WaveFormatExtensible {
                    format_tag: WaveFormatType::FloatPCM,
                    channels: WaveAudioChannels::Stereo,
                    sample_rate: 44100,
                    avg_bytes_per_second: 352800,
                    block_align: 8,
                    bits_per_sample: 32,
                    valid_bits_per_sample: None,
                    channel_mask: 0,
                    subformat_data: vec![],
                },
            ),
            (
                "float_pcm/as_f64_pcm",
                WaveFormatExtensible {
                    format_tag: WaveFormatType::FloatPCM,
                    channels: WaveAudioChannels::Stereo,
                    sample_rate: 44100,
                    avg_bytes_per_second: 705600,
                    block_align: 16,
                    bits_per_sample: 64,
                    valid_bits_per_sample: None,
                    channel_mask: 0,
                    subformat_data: vec![],
                },
            ),
            (
                "as_alaw",
                WaveFormatExtensible {
                    format_tag: WaveFormatType::ALaw,
                    channels: WaveAudioChannels::Stereo,
                    sample_rate: 44100,
                    avg_bytes_per_second: 88200,
                    block_align: 2,
                    bits_per_sample: 8,
                    valid_bits_per_sample: None,
                    channel_mask: 0,
                    subformat_data: vec![],
                },
            ),
            (
                "as_ulaw",
                WaveFormatExtensible {
                    format_tag: WaveFormatType::ULaw,
                    channels: WaveAudioChannels::Stereo,
                    sample_rate: 44100,
                    avg_bytes_per_second: 88200,
                    block_align: 2,
                    bits_per_sample: 8,
                    valid_bits_per_sample: None,
                    channel_mask: 0,
                    subformat_data: vec![],
                },
            ),
            (
                "original",
                WaveFormatExtensible {
                    format_tag: WaveFormatType::WaveFormatExtensible,
                    channels: WaveAudioChannels::Stereo,
                    sample_rate: 44100,
                    avg_bytes_per_second: 264600,
                    block_align: 6,
                    bits_per_sample: 24,
                    valid_bits_per_sample: Some(24),
                    channel_mask: 3,
                    subformat_data: vec![1, 0, 0, 0, 0, 0, 16, 0, 128, 0, 0, 170, 0, 56, 155, 113],
                },
            ),
        ]
    }

    pub(crate) fn create_wav_files_for_test<const N: usize>(
        inputs: [(&str, WaveFormatExtensible); N],
    ) -> Vec<WavFile> {
        let root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("test")
            .join("audio_files");

        inputs
            .map(|(path, format)| WavFile {
                path: root_path.join(path).with_extension("wav"),
                wave_format_info: Ok(format),
            })
            .into_iter()
            .collect::<Vec<_>>()
    }
}
