use axum::{
    extract::Path,
    response::Html,
    routing::{get, put},
    Json, Router,
};
use serde::Serialize;

use crate::file;

pub fn router() -> Router {
    Router::new().route("/", get(index_handler)).route(
        "/upload/:server/:owner/:repo/:commit/*path",
        put(upload_handler),
    )
}

async fn index_handler() -> Html<&'static str> {
    Html("<h1>Artifact Store</h1>")
}

#[derive(Serialize)]
struct Response {
    code: u16,
    message: String,
}

async fn upload_handler(Path(params): Path<file::UploadParams>) -> Json<Response> {
    match file::create_file(params) {
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
        let app = router();

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"<h1>Artifact Store</h1>");
    }
}
