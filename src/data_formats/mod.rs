pub mod request;
pub mod response;
pub mod wrapper;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct ArticleQueryParams {
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub favourited: Option<String>,
    #[serde(default = "get_default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
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
