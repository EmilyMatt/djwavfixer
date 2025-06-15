use std::io::{Read, Seek, SeekFrom};

use crate::DWORD_SIZE;
use crate::errors::Result;
use crate::riff_parser::RIFF_CHUNK_HEADER_SIZE;

pub(crate) struct RiffSubchunk {
    position: u64,
    id: [u8; DWORD_SIZE],
    size: u32,
    data: Option<Vec<u8>>,
}

impl RiffSubchunk {
    pub(crate) fn scan_next<R: Read + Seek>(reader: &mut R) -> Result<Option<Self>> {
        let position = reader.stream_position()?;

        let mut id = [0; 4];
        if reader.read_exact(&mut id).is_err() {
            return Ok(None); // No more blocks to read
        }

        let mut size_buffer = [0; 4];
        reader.read_exact(&mut size_buffer)?;
        let size = u32::from_le_bytes(size_buffer);

        // Seek forward to the end of the subchunk
        reader.seek(SeekFrom::Current(size as i64))?;

        Ok(Some(Self {
            position,
            id,
            size,
            data: None,
        }))
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

    pub(crate) fn data<R: Read + Seek>(&mut self, reader: &mut R) -> Result<&[u8]> {
        if self.data.is_some() {
            return unsafe { Ok(self.data.as_deref().unwrap_unchecked()) }; // Data already read
        };

        reader.seek(SeekFrom::Start(
            self.position + RIFF_CHUNK_HEADER_SIZE as u64,
        ))?; // Skip block type and size

        let mut data = vec![0; self.size as usize];
        reader.read_exact(&mut data)?;

        self.data = Some(data);

        unsafe { Ok(self.data.as_deref().unwrap_unchecked()) }
    }
}
