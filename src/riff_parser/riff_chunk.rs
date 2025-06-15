use indexmap::IndexMap;
use std::io::{Read, Seek};

use crate::errors::{DJWavFixerError, Result};
use crate::riff_parser::DWORD_SIZE;
use crate::riff_parser::riff_subchunk::RiffSubchunk;

pub(crate) struct RiffChunk {
    position: u64,
    id: [u8; DWORD_SIZE],
    size: u32,
    format: [u8; DWORD_SIZE],
    subchunks: IndexMap<[u8; DWORD_SIZE], RiffSubchunk>,
}

impl RiffChunk {
    fn scan_subchunks<R: Read + Seek>(
        reader: &mut R,
    ) -> Result<IndexMap<[u8; DWORD_SIZE], RiffSubchunk>> {
        let mut subchunks = IndexMap::new();
        while let Some(subchunk) = RiffSubchunk::scan_next(reader)? {
            let subchunk_id = subchunk.id();
            if subchunks.contains_key(&subchunk_id) {
                return Err(DJWavFixerError::RiffHeaderError(format!(
                    "Duplicate subchunk id found: {:?}{}",
                    subchunk_id,
                    String::from_utf8_lossy(&subchunk_id)
                )));
            }
            subchunks.insert(subchunk_id, subchunk);
        }
        Ok(subchunks)
    }

    pub(crate) fn scan_next<R: Read + Seek>(reader: &mut R) -> Result<Option<Self>> {
        let position = reader.stream_position()?;

        let mut id = [0; DWORD_SIZE];
        if reader.read_exact(&mut id).is_err() {
            return Ok(None); // No more chunks to read
        }

        let mut size_buffer = [0; DWORD_SIZE];
        reader.read_exact(&mut size_buffer)?;
        let size = u32::from_le_bytes(size_buffer);

        let mut format = [0; DWORD_SIZE];
        reader.read_exact(&mut format)?;

        let subchunks = Self::scan_subchunks(reader)?;

        let last_subchunk_end = subchunks
            .last()
            .map(|(_, subchunk)| subchunk.position() + subchunk.size() as u64)
            .unwrap_or(position + id.len() as u64 + DWORD_SIZE as u64 + format.len() as u64);

        if position + size as u64 != last_subchunk_end {
            return Err(DJWavFixerError::RiffHeaderError(format!(
                "Chunk size mismatch: expected {} from chunk, but last subchunk ends at {}",
                position + size as u64,
                last_subchunk_end
            )));
        }

        Ok(Some(Self {
            position,
            id,
            size,
            format,
            subchunks,
        }))
    }

    pub(crate) fn get_subchunk_mut(&mut self, id: [u8; DWORD_SIZE]) -> Option<&mut RiffSubchunk> {
        self.subchunks.get_mut(&id)
    }

    pub(crate) fn position(&self) -> u64 {
        self.position
    }

    pub(crate) fn id(&self) -> [u8; DWORD_SIZE] {
        self.id
    }

    pub(crate) fn size(&self) -> u32 {
        self.size
    }

    pub(crate) fn format(&self) -> [u8; DWORD_SIZE] {
        self.format
    }
}
