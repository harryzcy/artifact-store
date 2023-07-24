use std::net::SocketAddr;

mod router;
mod file;

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(router::router().into_make_service())
        .await
        .unwrap();
}
