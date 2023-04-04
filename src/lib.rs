mod authentication;
mod data_formats;
mod db_helpers;
mod errors;
mod handlers;
mod models;

use anyhow::Context;
pub use anyhow::Result;
use axum::http::StatusCode;
use axum::{routing::*, Extension, Json, Router};
pub use data_formats::*;
use handlers::*;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::{
    net::{SocketAddr, TcpListener},
    sync::Arc,
};
pub type JsonResponse<T> = (StatusCode, Json<T>);

pub async fn run_app(app: Router, address: SocketAddr) -> Result<()> {
    let db = init_db().await?;
    let app = app.layer(Extension(Arc::new(db)));
    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

pub async fn init_db() -> Result<SqlitePool> {
    let db_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        println!("Creating database {}", db_url);
        match Sqlite::create_database(&db_url).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }
    let pool = SqlitePool::connect(&db_url).await?;
    println!("Running Migrations");
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("Failed to run migrations")?;
    println!("Migrations completed");
    Ok(pool)
}

pub fn get_random_free_port() -> (u16, SocketAddr) {
    let listener = TcpListener::bind("localhost:0").unwrap();
    match listener.local_addr() {
        Ok(addr) => (addr.port(), addr),
        Err(_) => panic!("Could not get a free port"),
    }
}
pub fn make_router() -> Router {
    Router::new()
        .route("/check_health", get(alive))
        .route("/users/login", post(login_user))
        .route("/users", post(register_user))
        .route("/user", get(get_current_user).put(update_user))
        .route("/profiles/:username", get(get_profile))
        .fallback(not_found)
}
