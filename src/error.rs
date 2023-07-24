#[derive(Debug)]
pub enum CreateFileError {
    IoError(std::io::Error),
    AxumError(axum::Error),
}

impl std::fmt::Display for CreateFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CreateFileError::IoError(e) => write!(f, "IO error: {}", e),
            CreateFileError::AxumError(e) => write!(f, "Axum error: {}", e),
        }
    }
}

impl std::error::Error for CreateFileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CreateFileError::IoError(e) => Some(e),
            CreateFileError::AxumError(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for CreateFileError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}
impl From<axum::Error> for CreateFileError {
    fn from(e: axum::Error) -> Self {
        Self::AxumError(e)
    }
}
