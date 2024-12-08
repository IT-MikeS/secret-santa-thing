use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Group not found")]
    GroupNotFound,

    #[error("Name already taken in this group")]
    NameTaken,

    #[error("Cannot join group - Secret Santas have already been generated")]
    GroupAlreadyGenerated,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Websocket error: {0}")]
    WebSocket(#[from] axum::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            AppError::GroupNotFound => (StatusCode::NOT_FOUND, "Group not found".to_string()),
            AppError::NameTaken => (
                StatusCode::BAD_REQUEST,
                "Name already taken in this group".to_string(),
            ),
            AppError::GroupAlreadyGenerated => (
                StatusCode::BAD_REQUEST,
                "Cannot join group - Secret Santas have already been generated".to_string(),
            ),
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::WebSocket(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        };

        let body = serde_json::json!({
            "error": message
        });

        (status, axum::Json(body)).into_response()
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::InvalidInput(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
