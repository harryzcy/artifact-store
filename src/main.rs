use axum::{extract::Request, response::Response, serve::IncomingStream};
use futures_util::{future::poll_fn, FutureExt};
use std::{convert::Infallible, net::SocketAddr};
use tokio::{net::TcpListener, signal};
use tracing::info;

mod config;
mod database;
mod error;
mod router;
mod serve;
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
    serve::serve(listener, router).await.unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!(message = "starting graceful shutdown");
}
