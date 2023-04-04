use serde::{Deserialize, Serialize};

use crate::Tags;
#[derive(Deserialize, Serialize, Debug)]
pub struct UserResponse {
    pub email: String,
    pub token: String,
    pub username: String,
    pub bio: String,
    pub image: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProfileResponse {
    pub username: String,
    pub bio: String,
    pub image: Option<String>,
    pub following: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ArticleResponse {
    slug: String,
    title: String,
    decscription: String,
    body: String,
    #[serde(flatten)]
    tag_list: Tags,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    favorited: bool,
    #[serde(rename = "favoritesCount")]
    favorites_count: u32,
    author: ProfileResponse,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CommentResponse {
    id: u32,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    body: String,
    author: ProfileResponse,
}
