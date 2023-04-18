// Basic hello world server with axum

use std::net::SocketAddr;

use realworld::{make_router, run_app};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    // init_db().await.unwrap();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    let router = make_router();
    println!("Server started on {}", addr);
    match run_app(router, addr).await {
        Ok(_) => (),
        Err(error) => println!("Error: {}", error),
    }
}
