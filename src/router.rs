use std::sync::Arc;

use axum::{
    extract::{BodyStream, Path, State},
    response::{Html, IntoResponse},
    routing::{get, put},
    Json, Router,
};
use hyper::{header, StatusCode};
use serde::Serialize;
use tokio::sync::RwLock;

use crate::storage;
use crate::{database, error::HandleRequestError};

type SharedState = Arc<RwLock<RouterState>>;

pub struct RouterState {
    pub db: database::Database,
}

pub fn router(db: database::Database) -> Router {
    let shared_state = SharedState::new(RwLock::new(RouterState { db }));

    Router::new()
        .route("/", get(index_handler))
        .route("/ping", get(ping_handler))
        .route("/:server/:owner/:repo", get(list_commits_handler))
        .route("/:server/:owner/:repo/:commit", get(list_artifacts_handler))
        .route("/:server/:owner/:repo/:commit/*path", put(upload_handler))
        .route("/:server/:owner/:repo/:commit/*path", get(download_handler))
        .with_state(Arc::clone(&shared_state))
}

async fn index_handler() -> Html<&'static str> {
    Html("<h1>Artifact Store</h1>")
}

async fn ping_handler() -> &'static str {
    "pong"
}

#[derive(Serialize)]
struct Response {
    code: u16,
    message: String,
}

async fn list_commits_handler(
    Path(params): Path<storage::GetCommitsParams>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let db = &state.read().await.db;
    let response = match storage::get_commits(db, params).await {
        Ok(res) => res,
        Err(e) => {
            let response = Response {
                code: 500,
                message: format!("{}", e),
            };
            return serde_json::to_string(&response).unwrap();
        }
    };

    serde_json::to_string(&response).unwrap()
}

async fn list_artifacts_handler(
    Path(params): Path<storage::GetArtifactsParams>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let db = &state.read().await.db;
    let response = match storage::get_artifacts(db, params).await {
        Ok(res) => res,
        Err(e) => match e {
            HandleRequestError::NotFound(message) => {
                let response = Response { code: 404, message };
                return serde_json::to_string(&response).unwrap();
            }
            _ => {
                let response = Response {
                    code: 500,
                    message: format!("{}", e),
                };
                return serde_json::to_string(&response).unwrap();
            }
        },
    };

    serde_json::to_string(&response).unwrap()
}

async fn upload_handler(
    Path(params): Path<storage::UploadParams>,
    State(state): State<SharedState>,
    stream: BodyStream,
) -> Json<Response> {
    let db = &state.read().await.db;
    match storage::store_file(db, params, stream).await {
        Ok(_) => (),
        Err(e) => {
            let response = Response {
                code: 500,
                message: format!("{}", e),
            };
            return Json(response);
        }
    }

    let response = Response {
        code: 200,
        message: String::from("OK"),
    };
    Json(response)
}

async fn download_handler(
    Path(params): Path<storage::DownloadParams>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let db = &state.read().await.db;
    let (filename, body) = match storage::prepare_download_file(db, params).await {
        Ok(result) => result,
        Err(e) => match e {
            HandleRequestError::NotFound(message) => {
                return Err((StatusCode::NOT_FOUND, format!("{}", message)))
            }
            _ => return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
        },
    };

    let headers = [(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", filename),
    )];

    Ok((headers, body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn index_route() {
        let db = database::Database::new_rocksdb("data/test_index_route").unwrap();
        let app = router(db);

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"<h1>Artifact Store</h1>");

        std::fs::remove_dir_all("data/test_index_route").unwrap();
    }

    #[tokio::test]
    async fn ping_route() {
        let db = database::Database::new_rocksdb("data/test_ping_route").unwrap();
        let app = router(db);

        let response = app
            .oneshot(Request::builder().uri("/ping").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"pong");

        std::fs::remove_dir_all("data/test_ping_route").unwrap();
    }
}
