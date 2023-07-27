use std::sync::{Arc, RwLock};

use axum::{
    extract::{BodyStream, Path},
    response::Html,
    routing::{get, put},
    Json, Router,
};
use serde::Serialize;

use crate::database;
use crate::storage;

type SharedState = Arc<RwLock<RouterState>>;

pub struct RouterState {
    pub db: database::Database,
}

pub fn router(db: database::Database) -> Router {
    let shared_state = SharedState::new(RwLock::new(RouterState { db }));

    Router::new()
        .route("/", get(index_handler))
        .route(
            "/upload/:server/:owner/:repo/:commit/*path",
            put(upload_handler),
        )
        .with_state(Arc::clone(&shared_state))
}

async fn index_handler() -> Html<&'static str> {
    Html("<h1>Artifact Store</h1>")
}

#[derive(Serialize)]
struct Response {
    code: u16,
    message: String,
}

async fn upload_handler(
    Path(params): Path<storage::UploadParams>,
    stream: BodyStream,
) -> Json<Response> {
    match storage::handle_file_upload(params, stream).await {
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn index_route() {
        let db = database::Database::new_mockdb();
        let app = router(db);

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"<h1>Artifact Store</h1>");
    }
}
