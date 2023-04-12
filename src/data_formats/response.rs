use serde::{Deserialize, Serialize};

use crate::models::{Article, Comment, User};

use super::wrapper::Tags;
#[derive(Deserialize, Serialize, Debug)]
pub struct UserResponse {
    pub email: String,
    pub token: String,
    pub username: String,
    pub bio: String,
    pub image: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct ProfileResponse {
    pub username: String,
    pub bio: String,
    pub image: Option<String>,
    pub following: bool,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct ArticleResponse {
    slug: String,
    title: String,
    description: String,
    body: String,
    #[serde(flatten)]
    tag_list: Tags,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    favorited: bool,
    #[serde(rename = "favoritesCount")]
    favorites_count: i64,
    author: ProfileResponse,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CommentResponse {
    id: i64,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    body: String,
    author: ProfileResponse,
}

impl UserResponse {
    pub fn new(
        User {
            username,
            email,
            bio,
            image,
            ..
        }: User,
        token: String,
    ) -> Self {
        UserResponse {
            username,
            email,
            bio: bio.unwrap_or_default(),
            image,
            token,
        }
    }
}

impl ProfileResponse {
    pub fn new(
        User {
            username,
            bio,
            image,
            ..
        }: User,
        following: bool,
    ) -> Self {
        ProfileResponse {
            username,
            bio: bio.unwrap_or_default(),
            image,
            following,
        }
    }
}

impl CommentResponse {
    pub fn new(
        Comment {
            created_at,
            updated_at,
            body,
            id,
            ..
        }: Comment,
        author: ProfileResponse,
    ) -> Self {
        CommentResponse {
            id,
            created_at: created_at.to_string(),
            updated_at: updated_at.to_string(),
            body,
            author,
        }
    }
}

impl ArticleResponse {
    pub fn new(
        Article {
            slug,
            title,
            description,
            body,
            tag_list,
            created_at,
            updated_at,
            favorited,
            favorites_count,
            author_username,
            author_image,
            author_bio,
            following,
            ..
        }: Article,
    ) -> Self {
        ArticleResponse {
            slug,
            title,
            description,
            body,
            tag_list: Tags {
                tag_list: tag_list.split(',').map(|s| s.to_string()).collect(),
            },
            created_at: created_at.to_string(),
            updated_at: updated_at.to_string(),
            favorited,
            favorites_count,
            author: ProfileResponse {
                username: author_username,
                bio: author_bio.unwrap_or_default(),
                image: author_image,
                following,
                // ..Default::default()
            },
            ..Default::default()
        }
    }
}
