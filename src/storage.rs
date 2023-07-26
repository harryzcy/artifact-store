use axum::extract::BodyStream;
use futures_util::StreamExt;
use serde::Deserialize;
use std::{fs, io::Write};

use crate::database;
use crate::error::CreateFileError;

const DATA_DIR: &str = "data";

#[derive(Deserialize)]
pub struct UploadParams {
    server: String,
    owner: String,
    repo: String,
    commit: String,
    path: String,
}

/// Create a file on disk and record the result in the database.
pub async fn handle_file_create(
    conn: &mut database::Connection,
    params: UploadParams,
    mut stream: BodyStream,
) -> Result<(), CreateFileError> {
    let dir = format!(
        "{}/{}/{}/{}/{}",
        DATA_DIR, params.server, params.owner, params.repo, params.commit
    );
    let path = format!("{}/{}", dir, params.path);

    let tx = database::Tx::new(conn);

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
