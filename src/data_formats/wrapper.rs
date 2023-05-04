use serde::{Deserialize, Serialize};

use super::response::{ArticleResponse, CommentResponse, ProfileResponse};

#[derive(Debug, Deserialize, Serialize)]
pub struct UserWrapper<T> {
    pub user: T,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProfileWrapper {
    pub profile: ProfileResponse,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CommentWrapper<T> {
    pub comment: T,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ArticleWrapper<T> {
    pub article: T,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MultipleArticlesWrapper {
    pub articles: Vec<ArticleResponse>,
    #[serde(rename = "articlesCount")]
    pub article_count: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MultipleCommentsWrapper {
    pub comments: Vec<CommentResponse>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Tags {
    #[serde(rename = "tagList")]
    pub tags: Vec<String>,
}

impl<T> UserWrapper<T> {
    pub fn wrap_with_user_data(request: T) -> UserWrapper<T> {
        UserWrapper { user: request }
    }
}
