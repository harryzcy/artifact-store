use std::net::SocketAddr;

use axum::extract::Request;
use hyper::body::Incoming;
use hyper_util::rt::TokioIo;
use tokio::{net::TcpListener, signal, sync::watch};
use tower_service::Service;
use tracing::{debug, info};

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
    let app = router::router(conf.data_path, conf.artifact_path, db);

    let (close_tx, close_rx) = watch::channel(());

    loop {
        let (socket, remote_addr) = tokio::select! {
            result = listener.accept() => {
                result.unwrap()
            }
            _ = shutdown_signal() => {
                debug!("signal received, not accepting new connections");
                break;
            }
        };

        debug!("connection {remote_addr} accepted");

        let tower_service = app.clone();
        let close_rx = close_rx.clone();

        tokio::spawn(async move {
            let socket = TokioIo::new(socket);

            let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                tower_service.clone().call(request)
            });

            let conn = hyper::server::conn::http1::Builder::new()
                .serve_connection(socket, hyper_service)
                .with_upgrades();

            let mut conn = std::pin::pin!(conn);

            loop {
                tokio::select! {
                    result = conn.as_mut() => {
                        if let Err(err) = result {
                            debug!("failed to serve connection: {err:#}");
                        }
                        break;
                    }
                    _ = shutdown_signal() => {
                        debug!("signal received, starting graceful shutdown");
                        conn.as_mut().graceful_shutdown();
                    }
                }
            }

            debug!("connection {remote_addr} closed");

            drop(close_rx);
        });
    }

    drop(close_rx);
    drop(listener);

    debug!("waiting for {} tasks to finish", close_tx.receiver_count());
    close_tx.closed().await;
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
}
