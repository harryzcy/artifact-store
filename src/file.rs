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

pub async fn create_file(
    params: UploadParams,
    mut stream: BodyStream,
) -> Result<(), CreateFileError> {
    let dir = format!(
        "{}/{}/{}/{}/{}",
        DATA_DIR, params.server, params.owner, params.repo, params.commit
    );
    let path = format!("{}/{}", dir, params.path);

    match fs::create_dir_all(dir) {
        Ok(_) => (),
        Err(e) => return Err(CreateFileError::IoError(e)),
    }

    let mut file = match fs::File::create(path) {
        Ok(f) => f,
        Err(e) => return Err(CreateFileError::IoError(e)),
    };

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(c) => match file.write_all(&c) {
                Ok(_) => (),
                Err(e) => return Err(CreateFileError::IoError(e)),
            },
            Err(e) => return Err(CreateFileError::AxumError(e)),
        }
    }

    Ok(())
}
