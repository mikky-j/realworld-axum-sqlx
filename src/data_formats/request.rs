use serde::{Deserialize, Serialize};

use crate::Tags;

// ----------------- User Request -----------------
#[derive(Deserialize, Serialize, Debug)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub username: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(default)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

// ----------------- Article Request -----------------
#[derive(Deserialize, Serialize, Debug)]
pub struct CreateArticleRequest {
    title: String,
    description: String,
    body: String,
    #[serde(flatten)]
    tag_list: Option<Tags>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateArticleRequest {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    body: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CommentRequest {
    body: String,
}
