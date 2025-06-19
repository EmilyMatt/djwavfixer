use indexmap::IndexSet;
use std::path;
use std::path::PathBuf;

use crate::errors::Result;

pub use blocking_loader::{
    get_all_wav_files_in_directory, load_wav_file, load_wav_files, load_wav_files_rayon,
};

pub(crate) mod blocking_loader;

fn get_distinct_wav_files(files: &[PathBuf]) -> Result<Vec<PathBuf>> {
    files
        .iter()
        .map(|f| path::absolute(f).map_err(Into::into))
        .collect::<Result<IndexSet<_>>>()
        .map(|set| set.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use crate::wav_file::{WaveAudioChannels, WaveFormatExtensible, WaveFormatType};
    use std::path::PathBuf;

    pub(crate) fn readable_test_files() -> [(PathBuf, WaveFormatExtensible); 9] {
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
                    cb_size: 0,
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
                    cb_size: 0,
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
                    cb_size: 0,
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
                    cb_size: 0,
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
                    cb_size: 0,
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
                    cb_size: 0,
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
                    cb_size: 0,
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
                    cb_size: 0,
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
                    cb_size: 22,
                    valid_bits_per_sample: Some(24),
                    channel_mask: 3,
                    subformat_data: vec![1, 0, 0, 0, 0, 0, 16, 0, 128, 0, 0, 170, 0, 56, 155, 113],
                },
            ),
        ]
        .map(|(file_path, format)| {
            (
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("resources")
                    .join("test")
                    .join("audio_files")
                    .join(file_path)
                    .with_extension("wav"),
                format,
            )
        })
    }
}
