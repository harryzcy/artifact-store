use std::{error, fmt, io, time};

use crate::database;

#[derive(Debug)]
pub enum HandleRequestError {
    IoError(io::Error),
    SystemTimeError(time::SystemTimeError),
    AxumError(axum::Error),
    RocksDBError(rocksdb::Error),
    Generic(String),
}

impl fmt::Display for HandleRequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HandleRequestError::IoError(e) => write!(f, "IO error: {}", e),
            HandleRequestError::SystemTimeError(e) => write!(f, "SystemTime error: {}", e),
            HandleRequestError::AxumError(e) => write!(f, "Axum error: {}", e),
            HandleRequestError::RocksDBError(e) => write!(f, "RocksDB error: {}", e),
            HandleRequestError::Generic(s) => write!(f, "Generic error: {}", s),
        }
    }
}

impl error::Error for HandleRequestError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            HandleRequestError::IoError(e) => Some(e),
            HandleRequestError::AxumError(e) => Some(e),
            HandleRequestError::RocksDBError(e) => Some(e),
            HandleRequestError::SystemTimeError(e) => Some(e),
            HandleRequestError::Generic(_) => None,
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

impl From<time::SystemTimeError> for HandleRequestError {
    fn from(e: time::SystemTimeError) -> Self {
        Self::SystemTimeError(e)
    }
}

impl From<database::Error> for HandleRequestError {
    fn from(e: database::Error) -> Self {
        match e {
            database::Error::RocksDB(e) => Self::RocksDBError(e),
            database::Error::Generic(s) => Self::Generic(s),
        }
    }
}

impl From<String> for HandleRequestError {
    fn from(e: String) -> Self {
        Self::Generic(e)
    }
}
