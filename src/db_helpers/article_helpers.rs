use sqlx::{Sqlite, SqlitePool};

use crate::data_formats::request::CreateArticleRequest;
use crate::data_formats::wrapper::Tags;
use crate::data_formats::{request::UpdateArticleRequest, ArticleQueryParams};
use crate::errors::RequestError;
use crate::models::Article;
use crate::slugify;

use super::{get_user_by_username, QueryBuilder};

const ARTICLE_QUERY: &str = r#"
        SELECT articles.id as "id", articles.slug, articles.title, articles.author_id, articles.description, articles.body,
       (SELECT GROUP_CONCAT(tags.name, ',')
        FROM tags
        JOIN articletags ON articletags.tag_id = tags.id
        WHERE articletags.article_id = articles.id
        ) as "tag_list",
       articles.created_at as "created_at",
       articles.updated_at as "updated_at",
       EXISTS(SELECT 1 FROM favourite
              WHERE favourite.article_id = articles.id AND favourite.user_id = $1) as "favorited",
       COUNT(favourite.article_id) as "favorites_count",
       users.username as "author_username",
       users.image as "author_image",
       users.bio as "author_bio",
        EXISTS(SELECT 1 FROM follows WHERE followed_id = articles.author_id AND follower_id = $1) as "following"
        FROM articles
        JOIN users ON articles.author_id = users.id
        JOIN articletags ON articletags.article_id = articles.id
        LEFT JOIN favourite ON articles.id = favourite.article_id
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
    let article = sqlx::query_as!(
        Article,
        r#"
        Select DISTINCT articles.id as "id!: i64", 
        title as "title!",
        slug as "slug!",
        body as "body!",
        description as "description!",
        author_id as "author_id!: i64",
        articles.created_at as "created_at!",
        updated_at as "updated_at!",
        (SELECT 
            GROUP_CONCAT(tags.name, ',')
             from tags 
             join articletags on articletags.tag_id = tags.id
              where articletags.article_id = articles.id
            ) as "tag_list!",
        users.username as "author_username!",
        users.image as "author_image",
        users.bio as "author_bio",
        (select count(favourite.article_id) from favourite where favourite.article_id = articles.id) as "favorites_count!: i64",
        Exists(Select 1 from favourite where favourite.article_id = articles.id and favourite.user_id = $1) as "favorited!: bool",
        Exists(Select 1 from follows where followed_id = articles.author_id and follower_id = $1) as "following!: bool"
     from articles
     JOIN users ON articles.author_id = users.id
     LEFT JOIN follows ON follows.followed_id = articles.author_id
     LEFT JOIN articletags ON articletags.article_id = articles.id
     LEFT JOIN tags ON tags.id = articletags.tag_id
     WHERE (users.username = $2 OR $2 IS NULL)
     AND (tags.name = $3 OR $3 IS NULL)
     ORDER BY articles.created_at DESC
     LIMIT $4 OFFSET $5
     "#,
     id,
     author,
     tag, limit, offset
    )
    .fetch_all(&mut tx)
    .await?;

    tx.commit().await?;
    Ok(article)
}

pub async fn list_articles_in_db(
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

    let username = match favourited {
        Some(username) => {
            let user = get_user_by_username(pool, &username).await?;
            if let Some(user) = user {
                Some(user.id)
            } else {
                None
            }
        }
        None => None,
    };

    let (query, params) = QueryBuilder::new(
        String::from("WHERE "),
        Some(" AND "),
        Some(vec![id.map(|g| g.to_string()).unwrap_or_default()]),
    )
    .add_param("favourite.user_id", username.map(|id| id.to_string()))
    .add_param("tags.name", tag)
    .add_param("user.username", author)
    .build();

    let raw_initial_query = format!(
        "{} {} ORDER BY articles.updated_at DESC LIMIT {} OFFSET {}",
        ARTICLE_QUERY, query, limit, offset
    );
    // raw_initial_query.push_str(&query);

    let mut result = sqlx::query_as::<Sqlite, Article>(&raw_initial_query);
    for param in params {
        result = result.bind(param);
    }

    let result = result.fetch_all(&mut tx).await?;
    tx.commit().await?;
    Ok(result)
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

    let username = match favourited {
        Some(username) => {
            let user = get_user_by_username(pool, &username).await?;
            if let Some(user) = user {
                Some(user.id)
            } else {
                None
            }
        }
        None => None,
    };

    let (query, params) = QueryBuilder::new(
        String::from("WHERE "),
        Some(" AND "),
        Some(vec![id.to_string()]),
    )
    .add_param("favourite.user_id", username.map(|id| id.to_string()))
    .add_param("tags.name", tag)
    .add_param("user.username", author)
    .build();

    let raw_initial_query = format!(
        "{}\nJOIN follows ON folows.followed_id = users.id {} AND follows.follower_id = $1 ORDER BY articles.updated_at DESC LIMIT {} OFFSET {}",
        ARTICLE_QUERY, query, limit, offset
    );

    let mut result = sqlx::query_as::<Sqlite, Article>(&raw_initial_query);
    for param in params {
        result = result.bind(param);
    }

    let result = result.fetch_all(&mut tx).await?;

    Ok(result)
}

pub async fn get_article_by_slug_in_db(
    pool: &SqlitePool,
    slug: &str,
    id: Option<i64>,
) -> Result<Option<Article>, RequestError> {
    let mut tx = pool.begin().await?;

    let raw_initial_query = format!("{} WHERE articles.slug = $2", ARTICLE_QUERY);

    let result = sqlx::query_as::<Sqlite, Article>(&raw_initial_query)
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
    println!("I inserted the article");

    let article_id = result.id;
    if let Some(Tags { tag_list: tag }) = tag_list {
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

    println!("I inserted the tags of the article");
    let result = get_article_by_slug_in_db(pool, &result.slug, Some(id))
        .await?
        .unwrap();
    println!("I got the article");

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
        .add_param("title = ?", title)
        .add_param("description = ?", description)
        .add_param("body = ?", body)
        .add_param("slug = ?", new_slug.clone())
        .build();
    let (query_2, params_2) = QueryBuilder::new(String::from("WHERE "), Some(" AND "), None)
        .add_param("articles.slug = ?", Some(slug.to_owned()))
        .add_param("articles.author_id = ?", Some(id.to_string()))
        .build();

    let query = format!("UPDATE articles {query_1}, updated_at = CURRENT_TIMESTAMP {query_2}");
    let mut result = sqlx::query(&query);
    // .bind(params) .fetch_optional(&mut tx)
    // .await?;
    for param in params_1.iter().chain(params_2.iter()) {
        result = result.bind(param);
    }

    result.execute(&mut tx).await?;

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

    let _ = sqlx::query!(
        r#"
        DELETE FROM articles
        WHERE articles.slug = $1 AND articles.author_id = $2
        "#,
        slug,
        id
    )
    .execute(&mut tx)
    .await?;

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
        None => return Err(RequestError::RunTimeError("Article not found")),
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
    // article.favorited = true;

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
        None => return Err(RequestError::RunTimeError("Article not found")),
    };

    // let result = if article.favorited {
    //     sqlx::query!(
    //         r#"
    //         DELETE FROM favourite WHERE article_id = $1 AND user_id = $2
    //         "#,
    //         article.id,
    //         id
    //     )
    //     .execute(&mut tx)
    //     .await?;
    //     article.favorited = false;

    //     Ok(article)
    // } else {
    //     Err(RequestError::RunTimeError("Article was not liked before"))
    // };
    // tx.commit().await?;
    // result
    todo!()
}
