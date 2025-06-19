mod errors;
mod file_loader;
mod riff_parser;
mod wav_file;

pub use errors::{DJWavFixerError, Result};
pub use file_loader::*;
pub use wav_file::WavFile;

const DWORD_SIZE: usize = 4;
