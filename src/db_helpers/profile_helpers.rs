use sqlx::SqlitePool;

use crate::{errors::RequestError, models::User};

use super::{get_user_by_id, get_user_by_username};

pub async fn get_profile_by_id_in_db(
    pool: &SqlitePool,
    id: Option<i64>,
    profile_id: i64,
) -> Result<(User, bool), RequestError> {
    let mut tx = pool.begin().await?;
    let result = get_user_by_id(pool, profile_id).await?;

    let result = if let Some(profile) = result {
        if let Some(id) = id {
            let result = sqlx::query!(
                r#"
                SELECT * FROM follows WHERE follower_id = $1 AND followed_id = $2
                "#,
                id,
                profile.id
            )
            .fetch_optional(&mut tx)
            .await?;
            tx.commit().await?;
            (profile, result.is_some())
        } else {
            (profile, false)
        }
    } else {
        //? Throw error if user not found
        return Err(RequestError::NotFound);
    };
    Ok(result)
}

pub async fn get_profile_by_username_in_db(
    pool: &SqlitePool,
    id: Option<i64>,
    profile: &str,
) -> Result<(User, bool), RequestError> {
    let mut tx = pool.begin().await?;
    let result = get_user_by_username(pool, profile).await?;

    let result = if let Some(profile) = result {
        if let Some(id) = id {
            let result = sqlx::query!(
                r#"
                SELECT * FROM follows WHERE follower_id = $1 AND followed_id = $2
                "#,
                id,
                profile.id
            )
            .fetch_optional(&mut tx)
            .await?;
            tx.commit().await?;
            (profile, result.is_some())
        } else {
            (profile, false)
        }
    } else {
        //? Throw error if user not found
        return Err(RequestError::NotFound);
    };
    Ok(result)
}

pub async fn follow_user_in_db(
    pool: &SqlitePool,
    follower_id: i64,
    profile: &str,
) -> Result<User, RequestError> {
    let mut tx = pool.begin().await?;
    let profile_result = match get_user_by_username(pool, profile).await? {
        Some(user) => user,
        None => return Err(RequestError::RunTimeError("User not found")),
    };
    sqlx::query!(
        r#"
        INSERT INTO follows (follower_id, followed_id)
        VALUES ($1, $2)
        "#,
        follower_id,
        profile_result.id
    )
    .execute(&mut tx)
    .await?;
    tx.commit().await?;

    Ok(profile_result)
}

pub async fn unfollow_user_in_db(
    pool: &SqlitePool,
    follower_id: i64,
    profile: &str,
) -> Result<User, RequestError> {
    let mut tx = pool.begin().await?;
    let profile_result = match get_user_by_username(pool, profile).await? {
        Some(user) => user,
        None => return Err(RequestError::RunTimeError("User not found")),
    };

    sqlx::query!(
        r#"
        DELETE FROM follows WHERE follower_id = $1 AND followed_id = $2
        "#,
        follower_id,
        profile_result.id
    )
    .execute(&mut tx)
    .await?;
    tx.commit().await?;

    Ok(profile_result)
}
