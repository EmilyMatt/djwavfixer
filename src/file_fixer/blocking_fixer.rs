use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, Cursor, Seek, SeekFrom, Write};

use crate::Result;
use crate::riff_parser::RIFF_CHUNK_HEADER_SIZE;
use crate::wav_file::{ValidWavFile, WaveFormatType};

pub fn fix_wav_file(wav_file: &mut ValidWavFile<BufReader<File>>) -> Result<()> {
    let riff_file = &mut wav_file.riff_file;
    let wave_format_info = &wav_file.wave_format_info;

    let mut new_file = BufWriter::new(tempfile::tempfile()?);

    {
        let reader = riff_file.reader();
        reader.seek(SeekFrom::Start(0))?;
        io::copy(riff_file.reader(), &mut new_file)?;
    }

    // Seek back on the writer side
    new_file.seek(SeekFrom::Start(0))?;

    // Create a fix path based on the unsupported format
    if wave_format_info.is_sample_bits_supported_by_players()
        && wave_format_info.are_channels_supported_by_players()
        && matches!(
            wave_format_info.format_tag,
            WaveFormatType::WaveFormatExtensible
        )
    {
        let riff_chunk = riff_file.get_chunk(b"RIFF").unwrap();
        let fmt_subchunk = riff_chunk.get_subchunk(b"fmt ").unwrap();

        let mut new_data = vec![0u8; 18];
        // use mut_slice so we don't exceed bounds
        let mut data_cursor = Cursor::new(new_data.as_mut_slice());

        // Write the Integer PCM format
        data_cursor.write_all(&(WaveFormatType::IntegerPCM as u16).to_le_bytes())?;
        data_cursor.write_all(&wave_format_info.channels.as_u16().to_le_bytes())?;
        data_cursor.write_all(&wave_format_info.sample_rate.to_le_bytes())?;
        data_cursor.write_all(&wave_format_info.avg_bytes_per_second.to_le_bytes())?;
        data_cursor.write_all(&wave_format_info.block_align.to_le_bytes())?;
        data_cursor.write_all(&wave_format_info.bits_per_sample.to_le_bytes())?;
        data_cursor.write_all(&0u16.to_le_bytes())?; // cbSize is 0 for Integer PCM

        // Everything else stays the same as it was and we can write it in place
        new_file.seek(SeekFrom::Start(
            fmt_subchunk.position() + RIFF_CHUNK_HEADER_SIZE as u64,
        ))?;
        new_file.write_all(&new_data)?;

        log::info!("Fixed RIFF file with Integer PCM format.");

        File::create("fixed_wav_file.wav").and_then(|mut file| {
            let mut inner_file = new_file.into_inner()?;
            inner_file.flush()?;
            inner_file.seek(SeekFrom::Start(0))?;

            io::copy(&mut inner_file, &mut file)?;
            Ok(())
        })?;
    }

    riff_file.chunks().iter().for_each(|(id, chunk)| {
        log::info!("Chunk {:?}", String::from_utf8_lossy(id).to_string());

        chunk.subchunks().iter().for_each(|(subchunk_id, _)| {
            log::info!(
                "Subchunk {:?}",
                String::from_utf8_lossy(subchunk_id).to_string()
            );
        });
    });

    Ok(())
}
