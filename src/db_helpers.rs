use sqlx::SqlitePool;

use crate::{
    authentication::hash_password_argon2,
    errors::RequestError,
    models::{Article, User},
    RegisterRequest, UpdateUserRequest,
};

struct QueryBuilder {
    query: String,
    params: Vec<String>,
    seperator: Option<&'static str>,
}

impl QueryBuilder {
    fn new(initial: String, seperator: Option<&'static str>) -> Self {
        Self {
            query: initial,
            params: vec![],
            seperator,
        }
    }

    fn add_param(mut self, filter: &str, param: Option<String>) -> Self {
        if let Some(value) = param {
            self.query.push_str(filter);
            if let Some(seperator) = self.seperator {
                self.query.push_str(seperator);
            }
            self.params.push(value);
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

pub async fn get_user_by_id(pool: &SqlitePool, id: i64) -> Result<Option<User>, sqlx::Error> {
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
// ----------------- End Helper Functions -----------------

// ----------------- User Queries -----------------

pub async fn insert_user(pool: &SqlitePool, user: &RegisterRequest) -> Result<i64, RequestError> {
    let mut tx = pool.begin().await?;
    let result = sqlx::query!(
        r#"
        INSERT INTO users (email, username, password)
        VALUES ($1, $2, $3)
        RETURNING id, created_at
        "#,
        user.email,
        user.username,
        user.password
    )
    .fetch_one(&mut tx)
    .await?;
    tx.commit().await?;
    Ok(result.id)
    // Ok(user)
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

    let (query, params) = QueryBuilder::new("UPDATE users SET ".to_owned(), Some(", "))
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

// ----------------- End User Queries -----------------

// ----------------- Profile Queries -----------------

pub async fn get_profile_from_db(
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
// ----------------- End Profile Queries -----------------

// ----------------- Article Queries -----------------
pub async fn list_articles_in_db(
    pool: &SqlitePool,
    id: Option<i64>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<Article>, RequestError> {
    let mut tx = pool.begin().await?;
    let limit = limit.unwrap_or(20);
    let offset = offset.unwrap_or(0);
    let result = sqlx::query!(
        r#"
        SELECT articles.id as "id!", articles.slug, articles.title, articles.description, articles.body,
       (SELECT GROUP_CONCAT(tags.name, ',')
        FROM tags
        JOIN articletags ON articletags.tag_id = tags.id
        WHERE articletags.article_id = articles.id) as "tag_list!",
       articles.created_at as "created_at!",
       EXISTS(SELECT 1 FROM favourite
              WHERE favourite.article_id = articles.id AND favourite.user_id = ?) as "favorited!",
       COUNT(favourite.article_id) as "favorites_count!"
        FROM articles
        LEFT JOIN favourite ON articles.id = favourite.article_id
        WHERE articles.slug = ?
        GROUP BY articles.id
        "#,
        1,
        "slug"
    );

    // let result = sqlx::query_as!(
    //     Article,
    //     r#"
    //     SELECT articles.id as "id!", articles.created_at as 'created_at!', articles.slug, articles.title,  articles.body, articles.author_id as "author_id!",  users.username, users.email, users.image, users.bio, users.password FROM articles
    //     INNER JOIN users ON articles.author_id = users.id
    //     ORDER BY articles.created_at DESC
    //     LIMIT $1 OFFSET $2
    //     "#,
    //     limit,
    //     offset
    // )
    // .fetch_all(&mut tx)
    // .await?;

    // let result = if let Some(id) = id {
    //     let mut result = result;
    //     for article in result.iter_mut() {
    //         let result = sqlx::query!(
    //             r#"
    //             SELECT * FROM favorites WHERE user_id = $1 AND article_id = $2
    //             "#,
    //             id,
    //             article.id
    //         )
    //         .fetch_optional(&mut tx)
    //         .await?;
    //         article.favorited = result.is_some();
    //     }
    //     result
    // } else {
    //     result
    // };

    tx.commit().await?;
    todo!()
    // Ok(result)
}
