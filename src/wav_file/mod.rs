use std::fmt::Write;
use std::path::PathBuf;

use crate::riff_parser::RiffFile;

pub(crate) use wav_format::{WaveFormatExtensible, WaveFormatType};

#[cfg(test)]
pub(crate) use wav_format::WaveAudioChannels;

mod wav_format;

pub struct WavFile<R> {
    pub(crate) path: PathBuf,
    #[allow(unused)]
    pub(crate) riff_file: crate::Result<RiffFile<R>>,
    pub(crate) wave_format_info: crate::Result<WaveFormatExtensible>,
}

impl<R> WavFile<R> {
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

    pub fn write_information(&self, mut writer: impl Write) -> crate::Result<()> {
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
