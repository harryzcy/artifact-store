use std::net::SocketAddr;

mod error;
mod file;
mod router;

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(router::router().into_make_service())
        .await
        .unwrap();
}
