use axum::{extract::Request, response::Response, serve::IncomingStream};
use std::{convert::Infallible, net::SocketAddr};
use tokio::{net::TcpListener, signal};
use tracing::info;

mod config;
mod database;
mod error;
mod router;
mod storage;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_target(false).json().init();

    let conf = config::load();
    let db = database::Database::new_rocksdb(&conf.rocksdb_path).unwrap();

    let port = 3001;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(message = "starting server", port = addr.port());

    let listener = TcpListener::bind(&addr).await.unwrap();
    let router = router::router(conf.data_path, conf.artifact_path, db).into_make_service();
    axum::serve(listener, router).await.unwrap();
}
