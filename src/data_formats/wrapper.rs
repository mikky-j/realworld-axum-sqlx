use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct UserWrapper<T> {
    pub user: T,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProfileWrapper {
    pub profile: ProfileResponse,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CommentWrapper {
    pub comment: CommentResponse,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ArticleWrapper {
    pub article: ArticleResponse,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MultipleArticlesWrapper {
    pub articles: Vec<CommentResponse>,
    #[serde(rename = "articlesCount")]
    pub article_count: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MultipleCommentsWrapper {
    pub comments: Vec<CommentResponse>,
}

impl<T> UserWrapper<T> {
    pub fn wrap_with_user_data(request: T) -> UserWrapper<T> {
        UserWrapper { user: request }
    }
}
