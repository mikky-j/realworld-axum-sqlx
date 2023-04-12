use sqlx::SqlitePool;

use crate::{
    authentication::hash_password_argon2,
    data_formats::request::{RegisterRequest, UpdateUserRequest},
    errors::RequestError,
    models::User,
};

use super::{get_user_by_id, QueryBuilder};

pub async fn insert_user(pool: &SqlitePool, user: &RegisterRequest) -> Result<User, RequestError> {
    let mut tx = pool.begin().await?;
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (email, username, password)
        VALUES ($1, $2, $3)
        RETURNING id, created_at as "created_at!", username as "username!", email as "email!", image, bio, password as "password!"
        "#,
        user.email,
        user.username,
        user.password
    )
    .fetch_one(&mut tx)
    .await?;
    tx.commit().await?;
    Ok(user)
}

pub async fn update_user_in_db(
    pool: &SqlitePool,
    id: i64,
    UpdateUserRequest {
        email,
        bio,
        image,
        username,
        password,
    }: UpdateUserRequest,
) -> Result<User, RequestError> {
    let mut tx = pool.begin().await?;
    let password = if let Some(password) = password {
        let hashed_password = hash_password_argon2(password)
            .await
            .map_err(|_| RequestError::ServerError)?;
        Some(hashed_password)
    } else {
        None
    };

    let (query, params) = QueryBuilder::new("UPDATE users SET ".to_owned(), Some(", "), None)
        .add_param("email = ?", email)
        .add_param("bio = ?", bio)
        .add_param("image = ?", image)
        .add_param("username = ?", username)
        .add_param("password = ?", password)
        .trim()
        .add_param(" WHERE id = ?", Some(id.to_string()))
        .build();

    let mut query = sqlx::query(&query);
    for i in params {
        query = query.bind(i);
    }
    query.execute(&mut tx).await?;

    tx.commit().await?;

    let result = match get_user_by_id(pool, id).await? {
        Some(user) => user,
        None => return Err(RequestError::NotFound),
    };

    Ok(result)
}
