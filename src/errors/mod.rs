use std::sync::Arc;
use std::{fmt, io, string};

pub type Result<T> = std::result::Result<T, DJWavFixerError>;

#[derive(Clone, Debug, thiserror::Error)]
pub enum DJWavFixerError {
    #[error("General error: {0}")]
    GeneralError(String),
    #[error("IO Error: {0}")]
    IoError(Arc<io::Error>),
    #[error("FMT error: {0}")]
    FmtError(#[from] fmt::Error),
    #[error("RIFF header error: {0}")]
    RiffHeaderError(String),
    #[error("Invalid WAV format: {0}")]
    WaveFormatError(String),
    #[error("Invalid UTF8 string: {0}")]
    FromUtf8Error(#[from] string::FromUtf8Error),
    #[cfg(feature = "parallel")]
    #[error("Error building thread pool: {0}")]
    ThreadPoolError(Arc<rayon::ThreadPoolBuildError>),
}

impl PartialEq for DJWavFixerError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DJWavFixerError::GeneralError(a), DJWavFixerError::GeneralError(b)) => a == b,
            (DJWavFixerError::IoError(a), DJWavFixerError::IoError(b)) => Arc::ptr_eq(a, b),
            (DJWavFixerError::FmtError(_), DJWavFixerError::FmtError(_)) => true,
            (DJWavFixerError::RiffHeaderError(a), DJWavFixerError::RiffHeaderError(b)) => a == b,
            (DJWavFixerError::WaveFormatError(a), DJWavFixerError::WaveFormatError(b)) => a == b,
            (DJWavFixerError::FromUtf8Error(_), DJWavFixerError::FromUtf8Error(_)) => true,
            #[cfg(feature = "parallel")]
            (DJWavFixerError::ThreadPoolError(a), DJWavFixerError::ThreadPoolError(b)) => {
                Arc::ptr_eq(a, b)
            }
            _ => false,
        }
    }
}

impl From<io::Error> for DJWavFixerError {
    fn from(err: io::Error) -> Self {
        DJWavFixerError::IoError(Arc::new(err))
    }
}

#[cfg(feature = "parallel")]
impl From<rayon::ThreadPoolBuildError> for DJWavFixerError {
    fn from(err: rayon::ThreadPoolBuildError) -> Self {
        DJWavFixerError::ThreadPoolError(Arc::new(err))
    }
}
