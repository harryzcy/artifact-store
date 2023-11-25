use std::{
    fs,
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::body::Body;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::database;
use crate::error::HandleRequestError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListReposResponse {
    pub repos: Vec<database::RepoData>,
}

pub async fn list_repos(db: &database::Database) -> Result<ListReposResponse, HandleRequestError> {
    let repos = db.list_repos()?;
    Ok(ListReposResponse { repos })
}

#[derive(Deserialize)]
pub struct ListCommitsParams {
    server: String,
    owner: String,
    repo: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCommitsResponse {
    pub server: String,
    pub owner: String,
    pub repo: String,
    pub commits: Vec<database::CommitData>,
}

pub async fn list_commits(
    db: &database::Database,
    params: ListCommitsParams,
) -> Result<ListCommitsResponse, HandleRequestError> {
    let commits = db.list_repo_commits(database::ListRepoCommitsParams {
        server: &params.server,
        owner: &params.owner,
        repo: &params.repo,
    })?;

    Ok(ListCommitsResponse {
        server: params.server,
        owner: params.owner,
        repo: params.repo,
        commits,
    })
}

#[derive(Deserialize)]
pub struct ListArtifactsParams {
    server: String,
    owner: String,
    repo: String,
    commit: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListArtifactsResponse {
    pub server: String,
    pub owner: String,
    pub repo: String,
    pub commit: String,
    pub artifacts: Vec<database::ArtifactData>,
}

pub async fn list_artifacts(
    db: &database::Database,
    params: ListArtifactsParams,
) -> Result<ListArtifactsResponse, HandleRequestError> {
    let commit = get_or_verify_commit(
        db,
        GetOrVerifyCommitParams {
            server: &params.server,
            owner: &params.owner,
            repo: &params.repo,
            commit: &params.commit,
        },
    )?;

    let artifacts = db.list_artifacts(database::ListArtifactsParams {
        server: &params.server,
        owner: &params.owner,
        repo: &params.repo,
        commit: &commit,
    })?;

    Ok(ListArtifactsResponse {
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
    base_dir: &String,
    db: &database::Database,
    params: UploadParams,
    body: Body,
) -> Result<(), HandleRequestError> {
    let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let dir = format!(
        "{}/{}/{}/{}/{}",
        base_dir, params.server, params.owner, params.repo, params.commit
    );

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

    let path = Path::new(&dir).join(&params.path);
    fs::create_dir_all(path.parent().unwrap())?;
    let mut file = fs::File::create(path)?;

    let mut stream = body.into_data_stream();
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
) -> Result<(String, Body), HandleRequestError> {
    let commit = get_or_verify_commit(
        db,
        GetOrVerifyCommitParams {
            server: &params.server,
            owner: &params.owner,
            repo: &params.repo,
            commit: &params.commit,
        },
    )?;

    let exists = db.exists_artifact(database::ExistsArtifactParams {
        server: &params.server,
        owner: &params.owner,
        repo: &params.repo,
        commit: &commit,
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
        data_dir, params.server, params.owner, params.repo, commit, params.path
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
    let body = Body::from_stream(stream);
    Ok((params.path, body))
}

#[derive(Clone)]
pub struct GetOrVerifyCommitParams<'a> {
    pub server: &'a String,
    pub owner: &'a String,
    pub repo: &'a String,
    pub commit: &'a String,
}

/// Get the latest commit if `commit` is "@latest", otherwise verify that `commit` exists.
fn get_or_verify_commit(
    db: &database::Database,
    params: GetOrVerifyCommitParams<'_>,
) -> Result<String, HandleRequestError> {
    let is_latest = params.commit == "@latest";
    if is_latest {
        let commit = db.get_latest_commit(database::GetLatestCommitParams {
            server: params.server,
            owner: params.owner,
            repo: params.repo,
        })?;
        return Ok(commit);
    }

    let exists = db.exists_commit(database::ExistsCommitParams {
        server: params.server,
        owner: params.owner,
        repo: params.repo,
        commit: params.commit,
    })?;
    if !exists {
        return Err(HandleRequestError::NotFound(format!(
            "commit {} not found",
            params.commit
        )));
    }
    Ok(params.commit.clone())
}
