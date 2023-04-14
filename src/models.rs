use chrono::NaiveDateTime;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password: String,
    pub image: Option<String>,
    pub bio: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Article {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub tag_list: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub favorited: bool,
    pub favorites_count: i64,
    pub author_id: i64,
    pub author_username: String,
    pub author_image: Option<String>,
    pub author_bio: Option<String>,
    pub following: bool,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Comment {
    pub id: i64,
    pub body: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub article_id: i64,
    pub author_id: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tag {
    pub id: i64,
    pub tag: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ArticleTag {
    pub article_id: i64,
    pub tag_id: i64,
}
