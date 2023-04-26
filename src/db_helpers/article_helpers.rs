use sqlx::{Sqlite, SqlitePool};

use crate::data_formats::request::CreateArticleRequest;
use crate::data_formats::wrapper::Tags;
use crate::data_formats::{request::UpdateArticleRequest, ArticleQueryParams};
use crate::errors::RequestError;
use crate::models::Article;
use crate::slugify;

use super::{get_user_by_username, QueryBuilder};

const ARTICLE_QUERY: &str = r#"
            SELECT DISTINCT articles.id                                   AS "id",
                            title                                         AS "title",
                            slug                                          AS "slug",
                            body                                          AS "body",
                            description                                   AS "description",
                            author_id                                     AS "author_id",
                            articles.created_at                           AS "created_at",
                            updated_at                                    AS "updated_at",
                            (SELECT Group_concat(tags.name, ',')
                            FROM   tags
                                    JOIN articletags
                                    ON articletags.tag_id = tags.id
                            WHERE  articletags.article_id = articles.id) AS "tag_list",
                            users.username                                AS
                            "author_username",
                            users.image                                   AS "author_image",
                            users.bio                                     AS "author_bio",
                            (SELECT Count(favourite.article_id)
                            FROM   favourite
                            WHERE  favourite.article_id = articles.id)   AS
                            "favorites_count",
                            EXISTS (SELECT 1
                                    FROM   favourite
                                    WHERE  favourite.article_id = articles.id
                                        AND favourite.user_id = $1)    AS "favorited",
                            EXISTS (SELECT 1
                                    FROM   follows
                                    WHERE  followed_id = articles.author_id
                                        AND follower_id = $1)          AS "following"
            FROM   articles
                JOIN users
                    ON articles.author_id = users.id
                LEFT JOIN favourite
                        ON favourite.article_id = articles.id
                LEFT JOIN follows
                        ON follows.followed_id = articles.author_id
                            AND ( follows.follower_id = $1
                                    OR $1 IS NULL )
                LEFT JOIN articletags
                        ON articletags.article_id = articles.id
                LEFT JOIN tags
                        ON tags.id = articletags.tag_id
            WHERE  ( users.username = $2
                    OR $2 IS NULL )
                AND ( favourite.user_id = $6
                        OR $6 IS NULL )
                AND ( tags.name = $3
                        OR $3 IS NULL )
                AND ( follows.follower_id = $7
                        OR $7 IS NULL )
            ORDER  BY articles.created_at DESC
            LIMIT  $4 offset $5 
     "#;

const SINGLE_ARTICLE_QUERY: &str = r#"
            SELECT DISTINCT articles.id                                   AS "id",
                            title                                         AS "title",
                            slug                                          AS "slug",
                            body                                          AS "body",
                            description                                   AS "description",
                            author_id                                     AS "author_id",
                            articles.created_at                           AS "created_at",
                            updated_at                                    AS "updated_at",
                            (SELECT Group_concat(tags.NAME, ',')
                            FROM   tags
                                    JOIN articletags
                                    ON articletags.tag_id = tags.id
                            WHERE  articletags.article_id = articles.id) AS "tag_list",
                            users.username                                AS
                            "author_username",
                            users.image                                   AS "author_image",
                            users.bio                                     AS "author_bio",
                            (SELECT Count(favourite.article_id)
                            FROM   favourite
                            WHERE  favourite.article_id = articles.id)   AS
                            "favorites_count",
                            EXISTS (SELECT 1
                                    FROM   favourite
                                    WHERE  favourite.article_id = articles.id
                                        AND favourite.user_id = $1)    AS "favorited",
                            EXISTS (SELECT 1
                                    FROM   follows
                                    WHERE  followed_id = articles.author_id
                                        AND follower_id = $1)          AS "following"
            FROM   articles
                JOIN users
                    ON articles.author_id = users.id
                LEFT JOIN follows
                        ON follows.followed_id = articles.author_id
                            AND ( follows.follower_id = $1
                                    OR $1 IS NULL )
                LEFT JOIN articletags
                        ON articletags.article_id = articles.id
                LEFT JOIN tags
                        ON tags.id = articletags.tag_id
            WHERE  ( articles.slug = $2
                    OR $2 IS NULL )  
"#;

pub async fn list_all_articles(
    pool: &SqlitePool,
    id: Option<i64>,
    ArticleQueryParams {
        tag,
        author,
        favourited,
        limit,
        offset,
    }: ArticleQueryParams,
) -> Result<Vec<Article>, RequestError> {
    let mut tx = pool.begin().await?;
    let favourite_id = match &favourited {
        Some(username) => get_user_by_username(pool, username)
            .await?
            .map(|user| user.id),
        None => None,
    };
    let article = sqlx::query_as::<Sqlite, Article>(ARTICLE_QUERY)
        .bind(id)
        .bind(author)
        .bind(tag)
        .bind(limit)
        .bind(offset)
        .bind(favourite_id)
        //? This is set to none coz we want to get all the articles not just the articles that a user is following
        .bind(Option::<String>::None)
        .fetch_all(&mut tx)
        .await?;

    tx.commit().await?;
    Ok(article)
}

pub async fn list_articles_feed_in_db(
    pool: &SqlitePool,
    id: i64,
    ArticleQueryParams {
        tag,
        author,
        favourited,
        limit,
        offset,
    }: ArticleQueryParams,
) -> Result<Vec<Article>, RequestError> {
    let mut tx = pool.begin().await?;
    let article = sqlx::query_as::<Sqlite, Article>(ARTICLE_QUERY)
        .bind(id)
        .bind(author)
        .bind(tag)
        .bind(limit)
        .bind(offset)
        .bind(favourited)
        .bind(id)
        .fetch_all(&mut tx)
        .await?;

    Ok(article)
}

pub async fn get_article_by_slug_in_db(
    pool: &SqlitePool,
    slug: &str,
    id: Option<i64>,
) -> Result<Option<Article>, RequestError> {
    let mut tx = pool.begin().await?;

    let result = sqlx::query_as::<Sqlite, Article>(SINGLE_ARTICLE_QUERY)
        .bind(id)
        .bind(slug)
        .fetch_optional(&mut tx)
        .await?;

    tx.commit().await?;

    Ok(result)
}

pub async fn create_article_in_db(
    pool: &SqlitePool,
    id: i64,
    CreateArticleRequest {
        title,
        description,
        body,
        tag_list,
    }: CreateArticleRequest,
) -> Result<Article, RequestError> {
    let mut tx = pool.begin().await?;

    let slug = slugify(&title);

    let result = sqlx::query!(
        r#"

        INSERT INTO articles (slug, title, description, body, author_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, slug as "slug!" 
        "#,
        slug,
        title,
        description,
        body,
        id
    )
    .fetch_one(&mut tx)
    .await?;

    let article_id = result.id;
    if let Some(Tags { tags: tag }) = tag_list {
        for tag in tag {
            let tag_id = sqlx::query!(
                r#"
            INSERT INTO tags (name)
            VALUES ($1)
            ON CONFLICT (name) DO UPDATE SET name = $1
            RETURNING id
            "#,
                tag,
            )
            .fetch_one(&mut tx)
            .await?
            .id;

            sqlx::query!(
                r#"
            INSERT INTO articletags (article_id, tag_id)
            VALUES ($1, $2)
            "#,
                article_id,
                tag_id
            )
            .execute(&mut tx)
            .await?;
        }
    }
    tx.commit().await?;

    let result = get_article_by_slug_in_db(pool, &result.slug, Some(id))
        .await?
        .unwrap();

    Ok(result)
}

pub async fn update_article_in_db(
    pool: &SqlitePool,
    id: i64,
    slug: &str,
    UpdateArticleRequest {
        title,
        description,
        body,
    }: UpdateArticleRequest,
) -> Result<Article, RequestError> {
    let mut tx = pool.begin().await?;
    let new_slug = title.as_ref().map(|title| slugify(title));
    let (query_1, params_1) = QueryBuilder::new(String::from("SET "), Some(", "), None)
        .add_param("title", title)
        .add_param("description", description)
        .add_param("body", body)
        .add_param("slug", new_slug.clone())
        .build();
    let query = format!("UPDATE articles {query_1}, updated_at = CURRENT_TIMESTAMP WHERE articles.slug = {slug} AND articles.author_id = {id}");
    let mut result = sqlx::query(&query);

    for param in params_1 {
        result = result.bind(param);
    }

    let result = result.execute(&mut tx).await?;
    if result.rows_affected() == 0 {
        return Err(RequestError::Forbidden);
    }

    let slug = new_slug.unwrap_or(slug.to_owned());

    let result = get_article_by_slug_in_db(pool, &slug, Some(id))
        .await?
        .unwrap();

    tx.commit().await?;
    Ok(result)
}

pub async fn delete_article_in_db(
    pool: &SqlitePool,
    id: i64,
    slug: &str,
) -> Result<(), RequestError> {
    let mut tx = pool.begin().await?;

    let result = sqlx::query!(
        r#"
        DELETE FROM articles
        WHERE articles.slug = $1 AND articles.author_id = $2
        "#,
        slug,
        id
    )
    .execute(&mut tx)
    .await?;

    if result.rows_affected() == 0 {
        return Err(RequestError::Forbidden);
    }

    tx.commit().await?;
    Ok(())
}

pub async fn favourite_article_in_db(
    pool: &SqlitePool,
    slug: &str,
    id: i64,
) -> Result<Article, RequestError> {
    let mut tx = pool.begin().await?;

    let article = get_article_by_slug_in_db(pool, slug, Some(id)).await?;

    let mut article = match article {
        Some(article) => article,
        None => return Err(RequestError::NotFound("Article not found")),
    };

    sqlx::query!(
        r#"
            INSERT INTO favourite (article_id, user_id)
            VALUES ($1, $2) 
        "#,
        article.id,
        id
    )
    .execute(&mut tx)
    .await?;

    tx.commit().await?;
    article.favorited = true;

    Ok(article)
}

pub async fn unfavourite_article_in_db(
    pool: &SqlitePool,
    id: i64,
    slug: &str,
) -> Result<Article, RequestError> {
    let mut tx = pool.begin().await?;

    let article = get_article_by_slug_in_db(pool, slug, Some(id)).await?;

    let mut article = match article {
        Some(article) => article,
        None => return Err(RequestError::NotFound("Article not found")),
    };

    let result = if article.favorited {
        sqlx::query!(
            r#"
            DELETE FROM favourite WHERE article_id = $1 AND user_id = $2
            "#,
            article.id,
            id
        )
        .execute(&mut tx)
        .await?;
        article.favorited = false;

        Ok(article)
    } else {
        Err(RequestError::RunTimeError("Article was not liked before"))
    };
    tx.commit().await?;
    result
}

// This is a function to allow the frontend(NextJS getStaticPath) to get all slugs
pub async fn get_all_slugs_in_db(pool: &SqlitePool) -> Result<Vec<String>, RequestError> {
    let mut tx = pool.begin().await?;

    let result = sqlx::query!(
        r#"
        SELECT slug FROM articles
        "#
    )
    .fetch_all(&mut tx)
    .await?;

    let result = result
        .into_iter()
        .map(|article| article.slug)
        .collect::<Vec<String>>();

    tx.commit().await?;

    Ok(result)
}
