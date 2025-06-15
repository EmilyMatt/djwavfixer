mod errors;
mod file_loader;
mod riff_parser;
mod wav_format;

pub use errors::{DJWavFixerError, Result};
pub use file_loader::*;

const DWORD_SIZE: usize = 4;
