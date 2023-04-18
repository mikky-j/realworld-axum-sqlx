mod authentication;
mod data_formats;
mod db_helpers;
mod errors;
mod handlers;
mod models;

use anyhow::Context;
pub use anyhow::Result;
use axum::http::Method;
use axum::http::StatusCode;
use axum::{routing::*, Extension, Json, Router};
use handlers::*;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::fmt::Write;
use std::{
    net::{SocketAddr, TcpListener},
    sync::Arc,
};
use tower_http::cors::Any;
use tower_http::cors::CorsLayer;
pub type JsonResponse<T> = (StatusCode, Json<T>);

pub fn slugify(title: &str) -> String {
    title.to_lowercase().replace(' ', "-")
}

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
    }
    let pool = SqlitePool::connect(&db_url).await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("Failed to run migrations")?;
    Ok(pool)
}

pub fn ultra_fast_string_converter(v: &[i64]) -> String {
    let buf_size = v.len() * 3; // length of each number + separator
    let mut s = String::with_capacity(buf_size);
    for (i, item) in v.iter().enumerate() {
        match i {
            0 => write!(&mut s, "({}", item).unwrap(),
            index if index == v.len() - 1 => write!(&mut s, ",{})", item).unwrap(),
            _ => write!(&mut s, ",{}", item).unwrap(),
        }
    }
    s
}

pub fn get_random_free_port() -> (u16, SocketAddr) {
    let listener = TcpListener::bind("localhost:0").unwrap();
    match listener.local_addr() {
        Ok(addr) => (addr.port(), addr),
        Err(_) => panic!("Could not get a free port"),
    }
}
pub fn make_router() -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        // .allow_credentials(true)
        .allow_origin(Any);
    Router::new()
        .route("/check_health", get(alive))
        .route("/users/login", post(login_user))
        .route("/users", post(register_user))
        .route("/user", get(get_current_user).put(update_user))
        .route("/profiles/:username", get(get_profile))
        .route(
            "/profiles/:username/follow",
            post(follow_profile).delete(unfollow_profile),
        )
        .route("/articles", get(list_articles).post(create_article))
        .route("/articles/feed", get(get_article_feed))
        .route(
            "/articles/:slug",
            get(get_article).put(update_article).delete(delete_article),
        )
        .route(
            "/articles/:slug/comments",
            get(get_comments).post(add_comment),
        )
        .route(
            "/articles/:slug/comments/:id",
            get(get_comment).delete(delete_comment),
        )
        .route(
            "/articles/:slug/favorite",
            post(favourite_article).delete(unfavourite_article),
        )
        .route("/tags", get(get_tags))
        .layer(cors)
        .fallback(not_found)
}
