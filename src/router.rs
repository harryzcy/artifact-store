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
    pub data_dir: String,
    pub db: database::Database,
}

pub fn router(data_dir: String, db: database::Database) -> Router {
    let shared_state = SharedState::new(RwLock::new(RouterState { data_dir, db }));

    Router::new()
        .route("/", get(index_handler))
        .route("/ping", get(ping_handler))
        .route("/repositories", get(list_repos_handler))
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
struct SimpleResponse {
    code: u16,
    message: String,
}

async fn list_repos_handler(State(state): State<SharedState>) -> impl IntoResponse {
    let db = &state.read().await.db;
    let response = match storage::list_repos(db).await {
        Ok(res) => res,
        Err(e) => {
            let response = SimpleResponse {
                code: 500,
                message: format!("{}", e),
            };
            return serde_json::to_string(&response).unwrap();
        }
    };

    serde_json::to_string(&response).unwrap()
}

async fn list_commits_handler(
    Path(params): Path<storage::ListCommitsParams>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let db = &state.read().await.db;
    let response = match storage::list_commits(db, params).await {
        Ok(res) => res,
        Err(e) => {
            let response = SimpleResponse {
                code: 500,
                message: format!("{}", e),
            };
            return serde_json::to_string(&response).unwrap();
        }
    };

    serde_json::to_string(&response).unwrap()
}

async fn list_artifacts_handler(
    Path(params): Path<storage::ListArtifactsParams>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let db = &state.read().await.db;
    let response = match storage::list_artifacts(db, params).await {
        Ok(res) => res,
        Err(e) => match e {
            HandleRequestError::NotFound(message) => {
                let response = SimpleResponse { code: 404, message };
                return serde_json::to_string(&response).unwrap();
            }
            _ => {
                let response = SimpleResponse {
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
) -> Json<SimpleResponse> {
    println!("upload_handler");
    let data_dir = &state.read().await.data_dir;
    let db = &state.read().await.db;
    match storage::store_file(data_dir, db, params, stream).await {
        Ok(_) => (),
        Err(e) => {
            let response = SimpleResponse {
                code: 500,
                message: format!("{}", e),
            };
            return Json(response);
        }
    }

    let response = SimpleResponse {
        code: 200,
        message: String::from("OK"),
    };
    Json(response)
}

async fn download_handler(
    Path(params): Path<storage::DownloadParams>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let data_dir = &state.read().await.data_dir;
    let db = &state.read().await.db;
    let (filename, body) = match storage::prepare_download_file(data_dir, db, params).await {
        Ok(result) => result,
        Err(e) => match e {
            HandleRequestError::NotFound(message) => {
                return Err((StatusCode::NOT_FOUND, message.to_string()))
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
    use axum::http::Request;
    use tower::Service;
    use tower::ServiceExt;

    #[tokio::test]
    async fn index_route() {
        let data_dir = "data".to_string();
        let db = database::Database::new_rocksdb("data/test_index_route").unwrap();
        let app = router(data_dir, db);

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
        let data_dir = "data".to_string();
        let db = database::Database::new_rocksdb("data/test_ping_route").unwrap();
        let app = router(data_dir, db);

        let response = app
            .oneshot(Request::builder().uri("/ping").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"pong");

        std::fs::remove_dir_all("data/test_ping_route").unwrap();
    }

    async fn send_request(
        mut app: &mut Router,
        method: &str,
        uri: &str,
        body: Body,
    ) -> http::Response<http_body::combinators::UnsyncBoxBody<bytes::Bytes, axum::Error>> {
        let request = Request::builder()
            .uri(uri)
            .method(method)
            .body(body)
            .unwrap();
        ServiceExt::<Request<Body>>::ready(&mut app)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn upload_download_empty() {
        let data_dir = "data".to_string();
        let db = database::Database::new_rocksdb("data/test_upload_download_empty").unwrap();
        let mut app = router(data_dir, db);

        let response = send_request(
            &mut app,
            "PUT",
            "/git.example.dev/owner/repo/commit/dir/test_upload_download_empty.txt",
            Body::empty(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert!(!body.is_empty());
        let value: serde_json::Value = serde_json::from_slice(&body[..]).unwrap();
        assert_eq!(value["code"], 200);
        assert_eq!(value["message"], "OK");

        let response = send_request(
            &mut app,
            "GET",
            "/git.example.dev/owner/repo/commit/dir/test_upload_download_empty.txt",
            Body::empty(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert!(body.is_empty());

        std::fs::remove_dir_all("data/test_upload_download_empty").unwrap();
    }

    #[tokio::test]
    async fn upload_download_binary() {
        let data_dir = "data".to_string();
        let db = database::Database::new_rocksdb("data/test_upload_download_binary").unwrap();
        let mut app = router(data_dir, db);

        let response = send_request(
            &mut app,
            "PUT",
            "/git.example.dev/owner/repo/commit/dir/test_upload_download_binary.txt",
            Body::from("test_upload_download_binary"),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert!(!body.is_empty());
        let value: serde_json::Value = serde_json::from_slice(&body[..]).unwrap();
        assert_eq!(value["code"], 200);
        assert_eq!(value["message"], "OK");

        let response = send_request(
            &mut app,
            "GET",
            "/git.example.dev/owner/repo/commit/dir/test_upload_download_binary.txt",
            Body::empty(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert!(!body.is_empty());
        assert!(body.starts_with(b"test_upload_download_binary"));

        std::fs::remove_dir_all("data/test_upload_download_binary").unwrap();
    }

    #[tokio::test]
    async fn upload_download_latest() {
        let data_dir = "data".to_string();
        let db = database::Database::new_rocksdb("data/test_upload_download_latest").unwrap();
        let mut app = router(data_dir, db);

        let response = send_request(
            &mut app,
            "PUT",
            "/git.example.dev/owner/repo-latest/commit/dir/test_upload_download_latest.txt",
            Body::from("test_upload_download_latest"),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert!(!body.is_empty());
        let value: serde_json::Value = serde_json::from_slice(&body[..]).unwrap();
        assert_eq!(value["code"], 200);
        assert_eq!(value["message"], "OK");

        let response = send_request(
            &mut app,
            "GET",
            "/git.example.dev/owner/repo-latest/@latest/dir/test_upload_download_latest.txt",
            Body::empty(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert!(!body.is_empty());
        assert!(body.starts_with(b"test_upload_download_latest"));

        std::fs::remove_dir_all("data/test_upload_download_latest").unwrap();
    }

    #[tokio::test]
    async fn download_not_exist() {
        let data_dir = "data".to_string();
        let db = database::Database::new_rocksdb("data/test_download_not_exist").unwrap();
        let mut app = router(data_dir, db);

        let response = send_request(
            &mut app,
            "GET",
            "/git.example.dev/owner/repo/commit/dir/test_download_not_exist.txt",
            Body::from("test_download_not_exist"),
        )
        .await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert!(!body.is_empty());

        std::fs::remove_dir_all("data/test_download_not_exist").unwrap();
    }
}
