use std::{fs, io::Write};

use axum::extract::BodyStream;
use futures_util::StreamExt;
use serde::Deserialize;

use crate::database;
use crate::error::HandleRequestError;

const DATA_DIR: &str = "data";

#[derive(Deserialize)]
pub struct UploadParams {
    commit: String,
    server: String,
    owner: String,
    repo: String,
    path: String,
}

pub async fn handle_file_upload(
    db: &database::Database,
    params: UploadParams,
    mut stream: BodyStream,
) -> Result<(), HandleRequestError> {
    let dir = format!(
        "{}/{}/{}/{}/{}",
        DATA_DIR, params.server, params.owner, params.repo, params.commit
    );
    let path = format!("{}/{}", dir, params.path);

    let txn = db.transaction();
    txn.create_commit(database::CreateCommitParams {
        commit: params.commit,
        server: params.server,
        owner: params.owner,
        repo: params.repo,
    })?;

    fs::create_dir_all(dir)?;
    let mut file = fs::File::create(path)?;

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(c) => file.write_all(&c)?,
            Err(e) => return Err(HandleRequestError::AxumError(e)),
        }
    }

    txn.commit()?;
    Ok(())
}
