use std::{
    fs,
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};

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
    let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let dir = format!(
        "{}/{}/{}/{}/{}",
        DATA_DIR, params.server, params.owner, params.repo, params.commit
    );
    let path = format!("{}/{}", dir, params.path);

    let txn = db.transaction();

    txn.create_commit(database::CreateCommitParams {
        commit: &params.commit,
        server: &params.server,
        owner: &params.owner,
        repo: &params.repo,
    })
    .or_else(|e| {
        if e.kind() != Some(database::ErrorKind::CommitExists) {
            return Err(HandleRequestError::from(e));
        }
        Ok(())
    })?;

    txn.create_artifact(database::CreateArtifactParams {
        time: &time,
        commit: &params.commit,
        path: &params.path,
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

#[derive(Deserialize)]
pub struct DownloadParams {
    server: String,
    owner: String,
    repo: String,
    commit: String,
    path: String,
}

pub async fn prepare_download_file(
    db: &database::Database,
    params: DownloadParams,
) -> Result<String, HandleRequestError> {
    let txn = db.transaction();

    Ok("".to_string())
}
