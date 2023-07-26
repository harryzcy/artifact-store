use std::net::SocketAddr;
use tokio::signal;

mod error;
mod router;
mod storage;

#[tokio::main]
async fn main() {
    let db = storage::create_db().unwrap();

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(router::router().into_make_service())
        .with_graceful_shutdown(shutdown_signal(db))
        .await
        .unwrap();
}

async fn shutdown_signal(db: storage::Connection) {
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

    println!("signal received, starting graceful shutdown");

    storage::shutdown_db(db).unwrap();
}
