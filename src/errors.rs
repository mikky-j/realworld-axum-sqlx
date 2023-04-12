use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::JsonResponse;

#[derive(Debug)]
pub enum RequestError {
    NotFound,
    NotAuthorized(&'static str),
    Forbidden,
    RunTimeError(&'static str),
    ServerError,
    DatabaseError(sqlx::Error),
}

#[derive(serde::Serialize)]
pub struct RequestErrorJsonWrapper {
    errors: RequestErrorJson,
}

#[derive(serde::Serialize)]
pub struct RequestErrorJson {
    body: Vec<String>,
}

impl RequestErrorJsonWrapper {
    pub fn new(error: &str) -> RequestErrorJsonWrapper {
        RequestErrorJsonWrapper {
            errors: RequestErrorJson {
                body: vec![error.to_string()],
            },
        }
    }
}

impl From<sqlx::Error> for RequestError {
    fn from(value: sqlx::Error) -> Self {
        Self::DatabaseError(value)
    }
}
impl IntoResponse for RequestError {
    fn into_response(self) -> axum::response::Response {
        self.to_json_response().into_response()
    }
}

impl RequestError {
    pub fn to_json_response(&self) -> JsonResponse<RequestErrorJsonWrapper> {
        let (status_code, json) = match self {
            RequestError::NotFound => (
                StatusCode::NOT_FOUND,
                RequestErrorJsonWrapper::new("Not Found"),
            ),
            RequestError::NotAuthorized(message) => (
                StatusCode::UNAUTHORIZED,
                RequestErrorJsonWrapper::new(message),
            ),
            RequestError::Forbidden => (
                StatusCode::FORBIDDEN,
                RequestErrorJsonWrapper::new("Forbidden"),
            ),
            RequestError::RunTimeError(message) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                RequestErrorJsonWrapper::new(message),
            ),
            RequestError::ServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                RequestErrorJsonWrapper::new("Internal Server Error"),
            ),
            RequestError::DatabaseError(e) => {
                eprintln!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    RequestErrorJsonWrapper::new("Internal Server Error"),
                )
            }
        };
        (status_code, Json(json))
    }
}
