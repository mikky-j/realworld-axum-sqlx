pub mod request;
pub mod response;
pub mod wrapper;

use chrono::{DateTime, NaiveDateTime, Utc};
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

fn datetime_to_string(date: NaiveDateTime) -> String {
    let date: DateTime<Utc> = DateTime::from_utc(date, Utc);
    date.to_rfc3339()
}
