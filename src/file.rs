use axum::extract::BodyStream;
use futures_util::StreamExt;
use serde::Deserialize;
use std::{fs, io::Write};

const DATA_DIR: &str = "data";

#[derive(Deserialize)]
pub struct UploadParams {
    server: String,
    owner: String,
    repo: String,
    commit: String,
    path: String,
}

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

pub async fn create_file(
    params: UploadParams,
    mut stream: BodyStream,
) -> Result<(), CreateFileError> {
    let dir = format!(
        "{}/{}/{}/{}/{}",
        DATA_DIR, params.server, params.owner, params.repo, params.commit
    );
    let path = format!("{}/{}", dir, params.path);

    fs::create_dir_all(dir)?;
    let mut file = fs::File::create(path)?;

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(c) => file.write_all(&c)?,
            Err(e) => return Err(CreateFileError::AxumError(e)),
        }
    }

    Ok(())
}
