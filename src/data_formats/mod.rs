mod request;
mod response;
mod wrapper;

pub use request::*;
pub use response::*;
pub use wrapper::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Tags {
    #[serde(rename = "tagList")]
    pub tag_list: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ArticleQueryParams {
    #[serde(default)]
    tag: Option<String>,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    favourited: Option<String>,
    #[serde(default = "get_default_limit")]
    limit: u32,
    #[serde(default)]
    offset: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FeedQueryParams {
    #[serde(default = "get_default_limit")]
    limit: u32,
    #[serde(default)]
    offset: u32,
}

fn get_default_limit() -> u32 {
    20
}
