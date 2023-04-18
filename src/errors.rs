use axum::{extract::rejection::JsonRejection, http::StatusCode, response::IntoResponse, Json};

use crate::JsonResponse;

#[derive(Debug)]
pub enum RequestError {
    NotFound(&'static str),
    NotAuthorized(&'static str),
    BadRequest(&'static str),
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

impl From<JsonRejection> for RequestError {
    fn from(value: JsonRejection) -> Self {
        match value {
            JsonRejection::JsonDataError(_) => RequestError::RunTimeError(
                "Invalid Request Json.\nPlease check the documentation for the correct format",
            ),
            JsonRejection::JsonSyntaxError(_) => RequestError::BadRequest("Invalid Request Body."),
            JsonRejection::MissingJsonContentType(_) => {
                RequestError::RunTimeError("Missing JSON Content Type.")
            }
            JsonRejection::BytesRejection(_) => RequestError::BadRequest("Invalid Request Body."),
            _ => RequestError::RunTimeError(
                "There was an error with the data associated with this request",
            ),
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
            RequestError::NotFound(message) => {
                (StatusCode::NOT_FOUND, RequestErrorJsonWrapper::new(message))
            }
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
            RequestError::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                RequestErrorJsonWrapper::new(message),
            ),
        };
        (status_code, Json(json))
    }
}
