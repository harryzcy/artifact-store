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

    loop {
        // When an incoming TCP connection is received grab a TCP stream for
        // client<->server communication.
        let (tcp, remote_address) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(tcp);

        // Print the remote address connecting to our server.
        println!("accepted connection from {:?}", remote_address);

        // Clone the connection_timeouts so they can be passed to the new task.
        let connection_timeouts_clone = connection_timeouts.clone();

        // Spin up a new task in Tokio so we can continue to listen for new TCP connection on the
        // current task without waiting for the processing of the HTTP1 connection we just received
        // to finish
        tokio::task::spawn(async move {
            // Pin the connection object so we can use tokio::select! below.
            let conn = http1::Builder::new().serve_connection(io, router);
            pin!(conn);

            // Iterate the timeouts.  Use tokio::select! to wait on the
            // result of polling the connection itself,
            // and also on tokio::time::sleep for the current timeout duration.
            for (iter, sleep_duration) in connection_timeouts_clone.iter().enumerate() {
                println!("iter = {} sleep_duration = {:?}", iter, sleep_duration);
                tokio::select! {
                    res = conn.as_mut() => {
                        // Polling the connection returned a result.
                        // In this case print either the successful or error result for the connection
                        // and break out of the loop.
                        match res {
                            Ok(()) => println!("after polling conn, no error"),
                            Err(e) =>  println!("error serving connection: {:?}", e),
                        };
                        break;
                    }
                    _ = tokio::time::sleep(*sleep_duration) => {
                        // tokio::time::sleep returned a result.
                        // Call graceful_shutdown on the connection and continue the loop.
                        println!("iter = {} got timeout_interval, calling conn.graceful_shutdown", iter);
                        conn.as_mut().graceful_shutdown();
                    }
                }
            }
        })
    }
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
