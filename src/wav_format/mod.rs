use crate::errors::{DJWavFixerError, Result};
use std::fmt::Write;
use std::fmt::{Display, Formatter};

#[repr(u16)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum WaveAudioChannels {
    Mono = 1,
    Stereo = 2,
    Other(u16),
}

impl WaveAudioChannels {
    fn as_u16(&self) -> u16 {
        match self {
            WaveAudioChannels::Mono => 1,
            WaveAudioChannels::Stereo => 2,
            WaveAudioChannels::Other(value) => *value,
        }
    }
}

impl From<u16> for WaveAudioChannels {
    fn from(value: u16) -> Self {
        match value {
            1 => WaveAudioChannels::Mono,
            2 => WaveAudioChannels::Stereo,
            _ => WaveAudioChannels::Other(value),
        }
    }
}

#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum WaveFormatType {
    IntegerPCM = 1,
    MicrosoftADPCM = 2,
    FloatPCM = 3,
    ALaw = 6,
    ImaAdpcm = 17,
    ULaw = 7,
    WaveFormatExtensible = 0xFFFE,
}

impl TryFrom<u16> for WaveFormatType {
    type Error = DJWavFixerError;

    fn try_from(value: u16) -> Result<Self> {
        match value {
            1 => Ok(WaveFormatType::IntegerPCM),
            2 => Ok(WaveFormatType::MicrosoftADPCM),
            3 => Ok(WaveFormatType::FloatPCM),
            6 => Ok(WaveFormatType::ALaw),
            7 => Ok(WaveFormatType::ULaw),
            17 => Ok(WaveFormatType::ImaAdpcm),
            0xFFFE => Ok(WaveFormatType::WaveFormatExtensible),
            _ => Err(DJWavFixerError::WaveFormatError(format!(
                "{} is not a supported WaveFormatType",
                value
            ))),
        }
    }
}

impl Display for WaveFormatType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WaveFormatType::IntegerPCM => write!(f, "Integer PCM"),
            WaveFormatType::MicrosoftADPCM => write!(f, "Microsoft ADPCM"),
            WaveFormatType::FloatPCM => write!(f, "Float PCM"),
            WaveFormatType::ALaw => write!(f, "A-Law"),
            WaveFormatType::ImaAdpcm => write!(f, "IMA ADPCM"),
            WaveFormatType::ULaw => write!(f, "U-Law"),
            WaveFormatType::WaveFormatExtensible => write!(f, "Wave Format Extensible"),
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct WaveFormatExtensible {
    pub(crate) format_tag: WaveFormatType,
    pub(crate) channels: WaveAudioChannels,
    pub(crate) sample_rate: u32,
    pub(crate) avg_bytes_per_second: u32,
    pub(crate) block_align: u16,
    pub(crate) bits_per_sample: u16,
    pub(crate) valid_bits_per_sample: Option<u16>,
    pub(crate) channel_mask: u32,
    pub(crate) subformat_data: Vec<u8>,
}

impl WaveFormatExtensible {
    pub(crate) fn is_integer_pcm(&self) -> bool {
        self.format_tag == WaveFormatType::IntegerPCM
    }

    pub(crate) fn is_sample_bits_supported_by_players(&self) -> bool {
        self.bits_per_sample == 16 || self.bits_per_sample == 24
    }

    pub(crate) fn write_information(&self, mut writer: impl Write) -> Result<()> {
        writeln!(writer, "  Format Tag: {}", self.format_tag)?;
        writeln!(writer, "  Channels: {}", self.channels.as_u16())?;
        writeln!(writer, "  Sample Rate: {}", self.sample_rate)?;
        writeln!(
            writer,
            "  Average Bytes Per Second: {}",
            self.avg_bytes_per_second
        )?;
        writeln!(writer, "  Block Align: {}", self.block_align)?;
        writeln!(writer, "  Bits Per Sample: {}", self.bits_per_sample)?;
        writeln!(
            writer,
            "  Valid Bits Per Sample: {}",
            self.valid_bits_per_sample.unwrap_or(self.bits_per_sample)
        )?;

        if self.format_tag == WaveFormatType::WaveFormatExtensible {
            writeln!(writer, "  Channel Mask: {:#X}", self.channel_mask)?;
        }

        if !self.subformat_data.is_empty() {
            writeln!(
                writer,
                "  Subformat Data Length: {} bytes",
                self.subformat_data.len()
            )?;
        }

        Ok(())
    }
}

impl TryFrom<&[u8]> for WaveFormatExtensible {
    type Error = DJWavFixerError;

    fn try_from(data: &[u8]) -> Result<Self> {
        if data.len() < 16 {
            return Err(DJWavFixerError::WaveFormatError(format!(
                "Data is too short to contain WaveFormatExtensible (data size is {})",
                data.len()
            )));
        }

        let (
            format_tag,
            channels,
            sample_rate,
            avg_bytes_per_second,
            block_align,
            bits_per_sample,
            cb_size,
        ) = unsafe {
            (
                WaveFormatType::try_from(u16::from_le_bytes(
                    data[0..2].try_into().unwrap_unchecked(),
                ))?,
                WaveAudioChannels::from(u16::from_le_bytes(
                    data[2..4].try_into().unwrap_unchecked(),
                )),
                u32::from_le_bytes(data[4..8].try_into().unwrap_unchecked()),
                u32::from_le_bytes(data[8..12].try_into().unwrap_unchecked()),
                u16::from_le_bytes(data[12..14].try_into().unwrap_unchecked()),
                u16::from_le_bytes(data[14..16].try_into().unwrap_unchecked()),
                // cbSize is does not exist in non-extensible formats, so we read it conditionally
                if data.len() > 16 {
                    u16::from_le_bytes(data[16..18].try_into().unwrap_unchecked())
                } else {
                    0
                },
            )
        };

        let (valid_bits_per_sample, channel_mask, subformat_data) = match format_tag {
            WaveFormatType::IntegerPCM | WaveFormatType::FloatPCM => {
                if cb_size != 0 {
                    return Err(DJWavFixerError::WaveFormatError(
                        "cbSize should be 0 for Integer/Float PCM".to_string(),
                    ));
                }
                (None, 0, vec![])
            }
            WaveFormatType::MicrosoftADPCM => {
                return Err(DJWavFixerError::WaveFormatError(
                    "Microsoft ADPCM format is not supported".to_string(),
                ));
            }
            WaveFormatType::ImaAdpcm => {
                return Err(DJWavFixerError::WaveFormatError(
                    "IMA_ADPCM format is not supported".to_string(),
                ));
            }
            WaveFormatType::WaveFormatExtensible => {
                if cb_size < 22 {
                    return Err(DJWavFixerError::WaveFormatError(
                        "cbSize is too small for WaveFormatExtensible".to_string(),
                    ));
                }

                if data.len() < 18 + cb_size as usize {
                    return Err(DJWavFixerError::WaveFormatError(
                        "Data is too short for additional subformat information".to_string(),
                    ));
                }

                let valid_bits_per_sample =
                    unsafe { u16::from_le_bytes(data[18..20].try_into().unwrap_unchecked()) };

                let channel_mask =
                    unsafe { u32::from_le_bytes(data[20..24].try_into().unwrap_unchecked()) };

                // subformat_data is the rest of the fmt subchunk, it is not guaranteed to be a specific size or format
                let subformat_data = data[24..].to_vec();

                (Some(valid_bits_per_sample), channel_mask, subformat_data)
            }
            _ => {
                // For other formats, we don't really parse other information
                (None, 0, vec![])
            }
        };

        let bits_per_sample_storage =
            valid_bits_per_sample.unwrap_or(bits_per_sample).div_ceil(8) * 8; // Round up to nearest byte
        let calculated_block_align = channels.as_u16() * (bits_per_sample_storage / 8);
        if block_align != calculated_block_align {
            return Err(DJWavFixerError::WaveFormatError(format!(
                "Block align mismatch: expected {}, got {}",
                calculated_block_align, block_align
            )));
        }

        let calculated_avg_bytes_per_second = sample_rate * block_align as u32;
        if avg_bytes_per_second != calculated_avg_bytes_per_second {
            return Err(DJWavFixerError::WaveFormatError(format!(
                "Average bytes per second mismatch: expected {}, got {}",
                calculated_avg_bytes_per_second, avg_bytes_per_second
            )));
        }

        Ok(Self {
            format_tag,
            channels,
            sample_rate,
            avg_bytes_per_second,
            block_align,
            bits_per_sample,
            valid_bits_per_sample,
            channel_mask,
            subformat_data,
        })
    }
}
