use std::{
    fs,
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{body::StreamBody, extract::BodyStream};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::database;
use crate::error::HandleRequestError;

#[derive(Deserialize)]
pub struct GetCommitsParams {
    server: String,
    owner: String,
    repo: String,
}

#[derive(Serialize)]
pub struct GetCommitsResponse {
    pub server: String,
    pub owner: String,
    pub repo: String,
    pub commits: Vec<database::CommitData>,
}

pub async fn list_commits(
    db: &database::Database,
    params: GetCommitsParams,
) -> Result<GetCommitsResponse, HandleRequestError> {
    let commits = db.list_repo_commits(database::GetRepoCommitsParams {
        server: &params.server,
        owner: &params.owner,
        repo: &params.repo,
    })?;

    Ok(GetCommitsResponse {
        server: params.server,
        owner: params.owner,
        repo: params.repo,
        commits,
    })
}

#[derive(Deserialize)]
pub struct GetArtifactsParams {
    server: String,
    owner: String,
    repo: String,
    commit: String,
}

#[derive(Serialize)]
pub struct GetArtifactsResponse {
    pub server: String,
    pub owner: String,
    pub repo: String,
    pub commit: String,
    pub artifacts: Vec<database::ArtifactData>,
}

pub async fn list_artifacts(
    db: &database::Database,
    params: GetArtifactsParams,
) -> Result<GetArtifactsResponse, HandleRequestError> {
    let is_latest = params.commit == "@latest";
    let commit: String;
    if is_latest {
        commit = db.get_latest_commit(database::GetLatestCommitParams {
            server: &params.server,
            owner: &params.owner,
            repo: &params.repo,
        })?;
    } else {
        let exists = db.exists_commit(database::ExistsCommitParams {
            server: &params.server,
            owner: &params.owner,
            repo: &params.repo,
            commit: &params.commit,
        })?;
        if !exists {
            return Err(HandleRequestError::NotFound(format!(
                "commit {} not found",
                params.commit
            )));
        }
        commit = params.commit.clone();
    }

    let artifacts = db.list_artifacts(database::GetArtifactsParams {
        server: &params.server,
        owner: &params.owner,
        repo: &params.repo,
        commit: &commit,
    })?;

    Ok(GetArtifactsResponse {
        server: params.server,
        owner: params.owner,
        repo: params.repo,
        commit,
        artifacts,
    })
}

#[derive(Deserialize)]
pub struct UploadParams {
    commit: String,
    server: String,
    owner: String,
    repo: String,
    path: String,
}

pub async fn store_file(
    data_dir: &String,
    db: &database::Database,
    params: UploadParams,
    mut stream: BodyStream,
) -> Result<(), HandleRequestError> {
    let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let dir = format!(
        "{}/{}/{}/{}/{}",
        data_dir, params.server, params.owner, params.repo, params.commit
    );
    let path = format!("{}/{}", dir, params.path);

    let txn = db.transaction();

    txn.create_repo_if_not_exists(
        time,
        database::CreateRepositoryParams {
            server: &params.server,
            owner: &params.owner,
            repo: &params.repo,
        },
    )?;

    txn.create_commit_if_not_exists(
        time,
        database::CreateCommitParams {
            commit: &params.commit,
            server: &params.server,
            owner: &params.owner,
            repo: &params.repo,
        },
    )?;

    txn.create_artifact(
        time,
        database::CreateArtifactParams {
            commit: &params.commit,
            path: &params.path,
        },
    )?;

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
    data_dir: &String,
    db: &database::Database,
    params: DownloadParams,
) -> Result<(String, StreamBody<ReaderStream<File>>), HandleRequestError> {
    let exists = db.exists_artifact(database::ExistsArtifactParams {
        server: &params.server,
        owner: &params.owner,
        repo: &params.repo,
        commit: &params.commit,
        path: &params.path,
    })?;
    if !exists {
        return Err(HandleRequestError::NotFound(format!(
            "file {} not found",
            params.path
        )));
    }

    let path = format!(
        "{}/{}/{}/{}/{}/{}",
        data_dir, params.server, params.owner, params.repo, params.commit, params.path
    );
    let file = match File::open(path).await {
        Ok(file) => file,
        Err(_) => {
            return Err(HandleRequestError::NotFound(format!(
                "file {} not found",
                params.path
            )))
        }
    };
    let stream: ReaderStream<File> = ReaderStream::new(file);
    let body = StreamBody::new(stream);
    Ok((params.path, body))
}
