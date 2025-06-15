use crate::DWORD_SIZE;
pub(crate) use riff_chunk::RiffChunk;
pub(crate) use riff_file::RiffFile;

mod riff_chunk;
mod riff_file;
mod riff_subchunk;

pub(crate) const RIFF_MAGIC: [u8; DWORD_SIZE] = *b"RIFF";
pub(crate) const FMT_MAGIC: [u8; DWORD_SIZE] = *b"fmt ";
pub(crate) const WAVE_MAGIC: [u8; DWORD_SIZE] = *b"WAVE";
#[allow(dead_code)]
pub(crate) const DATA_MAGIC: [u8; DWORD_SIZE] = *b"data";
const RIFF_CHUNK_HEADER_SIZE: usize = 8; // 4 bytes for type + 4 bytes for size
