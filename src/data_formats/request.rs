use serde::{Deserialize, Serialize};

use super::wrapper::Tags;

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
    pub title: String,
    pub description: String,
    pub body: String,
    #[serde(flatten)]
    pub tag_list: Option<Tags>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateArticleRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CommentRequest {
    pub body: String,
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        let test_string = r#"
             {
                "title": "How to train your dragon",
                "description": "Ever wonder how?",
                "body": "It takes a Jacobian",
                "tagList": ["dragons", "training"]
            }
        "#;
        let create_article_request =
            serde_json::from_str::<super::CreateArticleRequest>(test_string);
        println!("{:?}", create_article_request);
        assert!(create_article_request.is_ok());
    }
}
