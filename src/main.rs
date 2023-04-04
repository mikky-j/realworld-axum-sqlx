// Basic hello world server with axum

use std::net::SocketAddr;

use realworld::{make_router, run_app};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    // init_db().await.unwrap();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let router = make_router();
    match run_app(router, addr).await {
        Ok(_) => println!("Server started on {}", addr),
        Err(error) => println!("Error: {}", error),
    }
}
