use std::fmt::{Debug, Formatter, Write};
use std::path::PathBuf;

use crate::riff_parser::RiffFile;

pub(crate) use wav_format::{WaveFormatExtensible, WaveFormatType};

#[cfg(test)]
pub(crate) use wav_format::WaveAudioChannels;

mod wav_format;

pub(crate) enum WavFileLoadStatus<R> {
    Success {
        #[allow(unused)]
        riff_file: RiffFile<R>,
        wave_format_info: WaveFormatExtensible,
    },
    WavFileInvalid {
        #[allow(unused)]
        riff_file: RiffFile<R>,
        error: crate::DJWavFixerError,
    },
    RiffFileInvalid {
        error: crate::DJWavFixerError,
    },
}

// Must implement Debug manually because of the generic type R
impl<R> Debug for WavFileLoadStatus<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WavFileLoadStatus::Success {
                riff_file,
                wave_format_info,
            } => f
                .debug_struct("Success")
                .field("riff_file", riff_file)
                .field("wave_format_info", wave_format_info)
                .finish(),
            WavFileLoadStatus::WavFileInvalid { riff_file, error } => f
                .debug_struct("WavFileInvalid")
                .field("riff_file", riff_file)
                .field("error", error)
                .finish(),
            WavFileLoadStatus::RiffFileInvalid { error } => f
                .debug_struct("RiffFileInvalid")
                .field("error", error)
                .finish(),
        }
    }
}

pub struct WavFile<R> {
    pub(crate) path: PathBuf,
    pub(crate) load_status: WavFileLoadStatus<R>,
}

impl<R> WavFile<R> {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn needs_fixing(&self) -> Option<bool> {
        match self.load_status {
            WavFileLoadStatus::Success {
                ref wave_format_info,
                ..
            } => Some(
                !wave_format_info.is_sample_bits_supported_by_players()
                    || !wave_format_info.is_integer_pcm(),
            ),
            _ => None,
        }
    }

    pub fn can_fix(&self) -> Option<bool> {
        match self.load_status {
            WavFileLoadStatus::Success {
                ref wave_format_info,
                ..
            } => {
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
            _ => None,
        }
    }

    pub fn write_information(&self, mut writer: impl Write) -> crate::Result<()> {
        writeln!(writer, "  Path: {}", self.path.display())?;
        match self.load_status {
            WavFileLoadStatus::Success {
                ref wave_format_info,
                ..
            } => {
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
            WavFileLoadStatus::WavFileInvalid { ref error, .. } => {
                writeln!(writer, "  WAV file invalid: {}", error)?;
            }
            WavFileLoadStatus::RiffFileInvalid { ref error } => {
                writeln!(writer, "  Could not load RIFF structure: {}", error)?;
            }
        }

        Ok(())
    }
}
