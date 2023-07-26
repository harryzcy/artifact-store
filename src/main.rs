use std::net::SocketAddr;

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
        .await
        .unwrap();
}
