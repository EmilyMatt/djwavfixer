use indexmap::IndexMap;
use std::io::{Read, Seek};

use crate::errors::Result;
use crate::riff_parser::RiffChunk;
use crate::{DJWavFixerError, DWORD_SIZE};

pub(crate) struct RiffFile<R> {
    reader: R,
    chunks: IndexMap<[u8; DWORD_SIZE], RiffChunk>,
}

impl<R: Read + Seek> RiffFile<R> {
    fn scan_chunks(reader: &mut R) -> Result<IndexMap<[u8; DWORD_SIZE], RiffChunk>> {
        let mut chunks = IndexMap::new();
        while let Some(chunk) = RiffChunk::scan_next(reader)? {
            let chunk_id = chunk.id();
            if chunks.contains_key(&chunk_id) {
                return Err(DJWavFixerError::RiffHeaderError(format!(
                    "Duplicate chunk id found: {:?}{}",
                    chunk_id,
                    String::from_utf8_lossy(&chunk_id)
                )));
            }
            chunks.insert(chunk_id, chunk);
        }
        Ok(chunks)
    }

    pub fn try_new(mut reader: R, data_size: u64) -> Result<Self> {
        let chunks = Self::scan_chunks(&mut reader)?;

        let last_chunk_end = chunks
            .last()
            .map(|(_, chunk)| chunk.position() + chunk.size() as u64)
            .unwrap_or(0);

        if last_chunk_end != data_size {
            return Err(DJWavFixerError::RiffHeaderError(format!(
                "Data size mismatch: expected {} from file, but last chunk ends at {}",
                data_size, last_chunk_end
            )));
        }

        Ok(Self { reader, chunks })
    }

    pub fn get_chunk_mut(&mut self, id: [u8; DWORD_SIZE]) -> Option<(&mut R, &mut RiffChunk)> {
        self.chunks
            .get_mut(&id)
            .map(|chunk| (&mut self.reader, chunk))
    }
}
