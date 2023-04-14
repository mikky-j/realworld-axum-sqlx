use sqlx::{Sqlite, SqlitePool};

use crate::{errors::RequestError, models::User, ultra_fast_string_converter};

mod article_helpers;
mod comment_helpers;
mod profile_helpers;
mod tag_helpers;
mod user_helpers;

pub use article_helpers::*;
pub use comment_helpers::*;
pub use profile_helpers::*;
pub use tag_helpers::*;
pub use user_helpers::*;

struct QueryBuilder {
    query: String,
    params: Vec<String>,
    seperator: Option<&'static str>,
    counter: usize,
}

impl QueryBuilder {
    fn new(
        initial: String,
        seperator: Option<&'static str>,
        inital_params: Option<Vec<String>>,
    ) -> Self {
        Self {
            query: initial,
            params: inital_params.unwrap_or_default(),
            seperator,
            counter: 0,
        }
    }

    fn add_param(mut self, filter: &str, param: Option<String>) -> Self {
        if let Some(value) = param {
            let filter = format!("{} = ${} ", filter, self.params.len() + 1);
            self.query.push_str(&filter);
            if let Some(seperator) = self.seperator {
                self.query.push_str(seperator);
            }
            self.params.push(value);
            self.counter += 1;
        }
        self
    }

    fn trim(mut self) -> Self {
        if let Some(seperator) = self.seperator {
            self.query = self.query.trim_end_matches(seperator).to_string();
        }
        self
    }
    pub fn build(mut self) -> (String, Vec<String>) {
        self = self.trim();
        self.query = if !self.params.is_empty() && self.counter > 0 {
            self.query
        } else {
            String::new()
        };
        (self.query, self.params)
    }
}

// ----------------- Helper Functions -----------------

async fn get_user_by_username(
    pool: &SqlitePool,
    username: &str,
) -> Result<Option<User>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let result = sqlx::query_as!(
        User,
        r#"
        SELECT id as "id!", created_at as 'created_at!', username, email, image, bio, password  FROM users WHERE username = $1
        "#,
        username
    )
    .fetch_optional(&mut tx)
    .await?;
    tx.commit().await?;
    Ok(result)
}

pub async fn get_user_by_email(
    pool: &SqlitePool,
    email: &str,
) -> Result<Option<User>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let result = sqlx::query_as!(
        User,
        r#"
        SELECT id as "id!", created_at as 'created_at!', username, email, image, bio, password  FROM users WHERE email = $1
        "#,
        email
    )
    .fetch_optional(&mut tx)
    .await?;
    tx.commit().await?;
    Ok(result)
}

// TODO: Need to add testing to ensure if this is faster than the individual query method
#[allow(dead_code)]
pub async fn get_users_by_id(pool: &SqlitePool, id: &[i64]) -> Result<Vec<User>, RequestError> {
    let mut tx = pool.begin().await?;
    let ids = ultra_fast_string_converter(id);
    let query = format!("SELECT id as 'id!', created_at as 'created_at!', username, email, image, bio, password  FROM users WHERE id IN {}", ids);
    let result = sqlx::query_as::<Sqlite, User>(&query)
        .fetch_all(&mut tx)
        .await?;
    tx.commit().await?;
    Ok(result)
}

pub async fn get_user_by_id(pool: &SqlitePool, id: i64) -> Result<Option<User>, RequestError> {
    let mut tx = pool.begin().await?;
    let result = sqlx::query_as!(
        User,
        r#"
        SELECT id as "id!", created_at as 'created_at!', username, email, image, bio, password  FROM users WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&mut tx)
    .await?;
    tx.commit().await?;
    Ok(result)
}

pub async fn get_article_id_by_slug_in_db(
    pool: &SqlitePool,
    slug: &str,
) -> Result<i64, RequestError> {
    let mut tx = pool.begin().await?;
    let article = sqlx::query!(r#"SELECT id as "id!" from articles WHERE slug = $1"#, slug)
        .fetch_optional(&mut tx)
        .await?;
    match article {
        Some(record) => Ok(record.id),
        None => Err(RequestError::NotFound("Article not found")),
    }
}
