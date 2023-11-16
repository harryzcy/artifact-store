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
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

use crate::storage;
use crate::{database, error::HandleRequestError};

type SharedState = Arc<RwLock<RouterState>>;

pub struct RouterState {
    pub data_path: String,
    pub artifact_path: String,
    pub db: database::Database,
}

pub fn router(data_path: String, artifact_path: String, db: database::Database) -> Router {
    let shared_state = SharedState::new(RwLock::new(RouterState {
        data_path,
        artifact_path,
        db,
    }));

    Router::new()
        .route("/", get(index_handler))
        .route("/ping", get(ping_handler))
        .route("/repositories", get(list_repos_handler))
        .route("/:server/:owner/:repo", get(list_commits_handler))
        .route("/:server/:owner/:repo/:commit", get(list_artifacts_handler))
        .route("/:server/:owner/:repo/:commit/*path", put(upload_handler))
        .route("/:server/:owner/:repo/:commit/*path", get(download_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Micros),
                ),
        )
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
    let artifact_path = &state.read().await.artifact_path;
    let db = &state.read().await.db;
    match storage::store_file(artifact_path, db, params, stream).await {
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
    let artifact_path = &state.read().await.artifact_path;
    let db = &state.read().await.db;
    let (filename, body) = match storage::prepare_download_file(artifact_path, db, params).await {
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
    use http_body_util::BodyExt;
    use tower::Service;
    use tower::ServiceExt;

    #[tokio::test]
    async fn index_route() {
        let data_dir = String::from("data");
        let artifact_path = String::from("data/artifacts");
        let db = database::Database::new_rocksdb("data/router/test_index_route").unwrap();
        let app = router(data_dir, artifact_path, db);

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"<h1>Artifact Store</h1>");

        std::fs::remove_dir_all("data/router/test_index_route").unwrap();
    }

    #[tokio::test]
    async fn ping_route() {
        let data_dir = String::from("data");
        let artifact_path = String::from("data/artifacts");
        let db = database::Database::new_rocksdb("data/router/test_ping_route").unwrap();
        let app = router(data_dir, artifact_path, db);

        let response = app
            .oneshot(Request::builder().uri("/ping").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body = response.body().collect().await.unwrap();
        assert_eq!(&body[..], b"pong");

        std::fs::remove_dir_all("data/router/test_ping_route").unwrap();
    }

    async fn send_request(
        mut app: &mut Router,
        method: &str,
        uri: &str,
        body: Body,
    ) -> http::Response<http_body_util::combinators::UnsyncBoxBody<bytes::Bytes, axum::Error>> {
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
        let data_dir = String::from("data");
        let artifact_path = String::from("data/artifacts");
        let db = database::Database::new_rocksdb("data/router/test_upload_download_empty").unwrap();
        let mut app = router(data_dir, artifact_path, db);

        let response = send_request(
            &mut app,
            "PUT",
            "/git.example.dev/owner/repo/commit/dir/test_upload_download_empty.txt",
            Body::empty(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = BodyExt::collect(response.into_body())
            .await
            .unwrap()
            .to_bytes();
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

        let body = BodyExt::collect(response.into_body())
            .await
            .unwrap()
            .to_bytes();
        assert!(body.is_empty());

        std::fs::remove_dir_all("data/router/test_upload_download_empty").unwrap();
    }

    #[tokio::test]
    async fn upload_download_binary() {
        let data_dir = String::from("data");
        let artifact_path = String::from("data/artifacts");
        let db =
            database::Database::new_rocksdb("data/router/test_upload_download_binary").unwrap();
        let mut app = router(data_dir, artifact_path, db);

        let response = send_request(
            &mut app,
            "PUT",
            "/git.example.dev/owner/repo/commit/dir/test_upload_download_binary.txt",
            Body::from("test_upload_download_binary"),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = BodyExt::collect(response.into_body())
            .await
            .unwrap()
            .to_bytes();
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

        let body = BodyExt::collect(response.into_body())
            .await
            .unwrap()
            .to_bytes();
        assert!(!body.is_empty());
        assert!(&body[..] == b"test_upload_download_binary");

        std::fs::remove_dir_all("data/router/test_upload_download_binary").unwrap();
    }

    #[tokio::test]
    async fn upload_download_latest() {
        let data_dir = String::from("data");
        let artifact_path = String::from("data/artifacts");
        let db =
            database::Database::new_rocksdb("data/router/test_upload_download_latest").unwrap();
        let mut app = router(data_dir, artifact_path, db);

        let response = send_request(
            &mut app,
            "PUT",
            "/git.example.dev/owner/repo-latest/commit/dir/test_upload_download_latest.txt",
            Body::from("test_upload_download_latest"),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = BodyExt::collect(response.into_body())
            .await
            .unwrap()
            .to_bytes();
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

        let body = BodyExt::collect(response.into_body())
            .await
            .unwrap()
            .to_bytes();
        assert!(!body.is_empty());
        assert!(&body[..] == b"test_upload_download_latest");

        std::fs::remove_dir_all("data/router/test_upload_download_latest").unwrap();
    }

    #[tokio::test]
    async fn download_not_exist() {
        let data_dir = String::from("data");
        let artifact_path = String::from("data/artifacts");
        let db = database::Database::new_rocksdb("data/router/test_download_not_exist").unwrap();
        let mut app = router(data_dir, artifact_path, db);

        let response = send_request(
            &mut app,
            "GET",
            "/git.example.dev/owner/repo/commit/dir/test_download_not_exist.txt",
            Body::from("test_download_not_exist"),
        )
        .await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = BodyExt::collect(response.into_body())
            .await
            .unwrap()
            .to_bytes();
        assert!(!body.is_empty());

        std::fs::remove_dir_all("data/router/test_download_not_exist").unwrap();
    }

    #[tokio::test]
    async fn list_repo() {
        let data_dir = String::from("data");
        let artifact_path = String::from("data/artifacts");
        let db = database::Database::new_rocksdb("data/router/test_list_repo").unwrap();
        let mut app = router(data_dir, artifact_path, db);

        let response = send_request(
            &mut app,
            "PUT",
            "/git.example.dev/owner/repo/commit/dir/test_list_repo.txt",
            Body::from("test_list_repo"),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let response = send_request(&mut app, "GET", "/repositories", Body::empty()).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = BodyExt::collect(response.into_body())
            .await
            .unwrap()
            .to_bytes();
        assert!(!body.is_empty());

        let value: serde_json::Value = serde_json::from_slice(&body[..]).unwrap();
        assert_eq!(value["repos"].as_array().unwrap().len(), 1);
        assert_eq!(value["repos"][0]["server"], "git.example.dev");
        assert_eq!(value["repos"][0]["owner"], "owner");
        assert_eq!(value["repos"][0]["repo"], "repo");

        std::fs::remove_dir_all("data/router/test_list_repo").unwrap();
    }

    #[tokio::test]
    async fn list_repo_multiple() {
        let data_dir = String::from("data");
        let artifact_path = String::from("data/artifacts");
        let db = database::Database::new_rocksdb("data/router/test_list_repo_multiple").unwrap();
        let mut app = router(data_dir, artifact_path, db);

        let response = send_request(
            &mut app,
            "PUT",
            "/git.example.dev/owner/repo/commit/dir/test_list_repo_multiple.txt",
            Body::from("test_list_repo_multiple"),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let response = send_request(
            &mut app,
            "PUT",
            "/git.example.dev/owner/repo-2/commit-2/dir/test_list_repo_multiple.txt",
            Body::from("test_list_repo_multiple-2"),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let response = send_request(&mut app, "GET", "/repositories", Body::empty()).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = BodyExt::collect(response.into_body())
            .await
            .unwrap()
            .to_bytes();
        assert!(!body.is_empty());

        let value: serde_json::Value = serde_json::from_slice(&body[..]).unwrap();
        assert_eq!(value["repos"].as_array().unwrap().len(), 2);

        std::fs::remove_dir_all("data/router/test_list_repo_multiple").unwrap();
    }

    #[tokio::test]
    async fn list_commits() {
        let data_dir = String::from("data");
        let artifact_path = String::from("data/artifacts");
        let db = database::Database::new_rocksdb("data/router/test_list_commit").unwrap();
        let mut app = router(data_dir, artifact_path, db);

        let response = send_request(
            &mut app,
            "PUT",
            "/git.example.dev/owner/repo/commit/dir/test_list_commit.txt",
            Body::from("test_list_commit"),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let response = send_request(
            &mut app,
            "GET",
            "/git.example.dev/owner/repo",
            Body::empty(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = BodyExt::collect(response.into_body())
            .await
            .unwrap()
            .to_bytes();
        assert!(!body.is_empty());

        let value: serde_json::Value = serde_json::from_slice(&body[..]).unwrap();
        assert_eq!(value["server"], "git.example.dev");
        assert_eq!(value["owner"], "owner");
        assert_eq!(value["repo"], "repo");
        assert_eq!(value["commits"].as_array().unwrap().len(), 1);
        assert_eq!(value["commits"][0]["commit"], "commit");

        std::fs::remove_dir_all("data/router/test_list_commit").unwrap();
    }

    #[tokio::test]
    async fn list_commits_multiple() {
        let data_dir = String::from("data");
        let artifact_path = String::from("data/artifacts");
        let db = database::Database::new_rocksdb("data/router/test_list_commit_multiple").unwrap();
        let mut app = router(data_dir, artifact_path, db);

        let response = send_request(
            &mut app,
            "PUT",
            "/git.example.dev/owner/repo/commit-1/dir/test-1.txt",
            Body::empty(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let response = send_request(
            &mut app,
            "PUT",
            "/git.example.dev/owner/repo/commit-2/dir/test-2.txt",
            Body::empty(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let response = send_request(
            &mut app,
            "GET",
            "/git.example.dev/owner/repo",
            Body::empty(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = BodyExt::collect(response.into_body())
            .await
            .unwrap()
            .to_bytes();
        assert!(!body.is_empty());

        let value: serde_json::Value = serde_json::from_slice(&body[..]).unwrap();
        assert_eq!(value["server"], "git.example.dev");
        assert_eq!(value["owner"], "owner");
        assert_eq!(value["repo"], "repo");
        assert_eq!(value["commits"].as_array().unwrap().len(), 2);

        std::fs::remove_dir_all("data/router/test_list_commit_multiple").unwrap();
    }

    #[tokio::test]
    async fn list_artifacts() {
        let data_dir = String::from("data");
        let artifact_path = String::from("data/artifacts");
        let db = database::Database::new_rocksdb("data/router/test_list_artifacts").unwrap();
        let mut app = router(data_dir, artifact_path, db);

        let response = send_request(
            &mut app,
            "PUT",
            "/git.example.dev/owner/repo/commit/dir/test_list_artifacts.txt",
            Body::from("test_list_artifacts"),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let response = send_request(
            &mut app,
            "GET",
            "/git.example.dev/owner/repo/commit",
            Body::empty(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert!(!body.is_empty());

        let value: serde_json::Value = serde_json::from_slice(&body[..]).unwrap();
        assert_eq!(value["server"], "git.example.dev");
        assert_eq!(value["owner"], "owner");
        assert_eq!(value["repo"], "repo");
        assert_eq!(value["commit"], "commit");
        assert_eq!(value["artifacts"].as_array().unwrap().len(), 1);
        assert_eq!(value["artifacts"][0]["path"], "dir/test_list_artifacts.txt");

        std::fs::remove_dir_all("data/router/test_list_artifacts").unwrap();
    }

    #[tokio::test]
    async fn list_artifacts_multiple() {
        let data_dir = String::from("data");
        let artifact_path = String::from("data/artifacts");
        let db = database::Database::new_rocksdb("data/router/test_list_artifacts_multi").unwrap();
        let mut app = router(data_dir, artifact_path, db);

        let response = send_request(
            &mut app,
            "PUT",
            "/git.example.dev/owner/repo/commit/dir/test_list_artifacts_multi.txt",
            Body::empty(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let response = send_request(
            &mut app,
            "PUT",
            "/git.example.dev/owner/repo/commit/dir/test_list_artifacts_multi-2.txt",
            Body::empty(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let response = send_request(
            &mut app,
            "GET",
            "/git.example.dev/owner/repo/commit",
            Body::empty(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert!(!body.is_empty());

        let value: serde_json::Value = serde_json::from_slice(&body[..]).unwrap();
        assert_eq!(value["server"], "git.example.dev");
        assert_eq!(value["owner"], "owner");
        assert_eq!(value["repo"], "repo");
        assert_eq!(value["commit"], "commit");
        assert_eq!(value["artifacts"].as_array().unwrap().len(), 2);

        std::fs::remove_dir_all("data/router/test_list_artifacts_multi").unwrap();
    }
}
