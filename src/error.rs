use std::{error, fmt, io};

#[derive(Debug)]
pub enum HandleRequestError {
    IoError(std::io::Error),
    AxumError(axum::Error),
    RocksDBError(rocksdb::Error),
}

impl fmt::Display for HandleRequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HandleRequestError::IoError(e) => write!(f, "IO error: {}", e),
            HandleRequestError::AxumError(e) => write!(f, "Axum error: {}", e),
            HandleRequestError::RocksDBError(e) => write!(f, "RocksDB error: {}", e),
        }
    }
}

impl error::Error for HandleRequestError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            HandleRequestError::IoError(e) => Some(e),
            HandleRequestError::AxumError(e) => Some(e),
            HandleRequestError::RocksDBError(e) => Some(e),
        }
    }
}

impl From<io::Error> for HandleRequestError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<axum::Error> for HandleRequestError {
    fn from(e: axum::Error) -> Self {
        Self::AxumError(e)
    }
}

impl From<rocksdb::Error> for HandleRequestError {
    fn from(e: rocksdb::Error) -> Self {
        Self::RocksDBError(e)
    }
}
