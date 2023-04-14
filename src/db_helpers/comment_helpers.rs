use sqlx::SqlitePool;

use crate::{data_formats::request::CommentRequest, errors::RequestError, models::Comment};

use super::get_article_id_by_slug_in_db;

pub async fn add_comments_to_article_in_db(
    pool: &SqlitePool,
    id: i64,
    slug: &str,
    CommentRequest { body }: CommentRequest,
) -> Result<Comment, RequestError> {
    let mut tx = pool.begin().await?;

    let article = sqlx::query!(
        r#"
    SELECT id as "id!" from articles WHERE slug = $1
    "#,
        slug
    )
    .fetch_optional(&mut tx)
    .await?;

    let article = match article {
        Some(record) => record,
        None => return Err(RequestError::NotFound("Article not found")),
    };

    let result = sqlx::query_as!(
        Comment,
        r#"
        INSERT INTO comments (body, author_id, article_id)
        VALUES ($1, $2, $3)
        RETURNING id as "id!",
         body as "body!",
         created_at as "created_at!",
         updated_at as "updated_at!",
         article_id as "article_id!",
         author_id as "author_id!"
        "#,
        body,
        id,
        article.id
    )
    .fetch_one(&mut tx)
    .await?;
    tx.commit().await?;

    Ok(result)
}

pub async fn delete_comment_in_db(
    pool: &SqlitePool,
    user_id: i64,
    comment_id: i64,
    slug: &str,
) -> Result<(), RequestError> {
    let mut tx = pool.begin().await?;
    let article_id = get_article_id_by_slug_in_db(pool, slug).await?;
    sqlx::query!(
        r#"
        DELETE FROM comments WHERE author_id = $1 AND article_id = $2 AND id = $3
        "#,
        user_id,
        article_id,
        comment_id,
    )
    .execute(&mut tx)
    .await?;
    tx.commit().await?;
    Ok(())
}
pub async fn get_comment_for_article_in_db(
    pool: &SqlitePool,
    id: i64,
    slug: &str,
) -> Result<Comment, RequestError> {
    let mut tx = pool.begin().await?;
    let article_id = get_article_id_by_slug_in_db(pool, slug).await?;
    let result = sqlx::query_as!(
        Comment,
        r#"
        SELECT id as "id!", 
        body, 
        created_at as "created_at!", 
        updated_at as "updated_at!", 
        article_id, 
        author_id
         from comments 
         WHERE article_id = $1 AND id = $2
        "#,
        article_id,
        id
    )
    .fetch_optional(&mut tx)
    .await?;
    tx.commit().await?;
    let result = match result {
        Some(record) => record,
        None => return Err(RequestError::NotFound("Comment not found")),
    };
    Ok(result)
}

pub async fn get_comments_for_article_in_db(
    pool: &SqlitePool,
    slug: &str,
) -> Result<Vec<Comment>, RequestError> {
    let mut tx = pool.begin().await?;
    let article_id = get_article_id_by_slug_in_db(pool, slug).await?;
    let result = sqlx::query_as!(
        Comment,
        r#"
        SELECT id as "id!",
         body,
         created_at as "created_at!",
         updated_at as "updated_at!",
         article_id,
         author_id
            from comments 
              WHERE article_id = $1
        
        "#,
        article_id
    )
    .fetch_all(&mut tx)
    .await?;
    tx.commit().await?;
    Ok(result)
}
