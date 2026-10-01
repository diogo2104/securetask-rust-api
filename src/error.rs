use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("bad request")]
    BadRequest(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("not found")]
    NotFound,
    #[error("conflict")]
    Conflict(String),
    #[error("internal server error")]
    Internal,
}

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorMessage,
}

#[derive(Serialize)]
struct ErrorMessage {
    code: &'static str,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, "bad_request", message),
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "invalid or missing credentials".into(),
            ),
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found", "resource not found".into()),
            Self::Conflict(message) => (StatusCode::CONFLICT, "conflict", message),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "unexpected server error".into(),
            ),
        };

        (status, Json(ErrorBody { error: ErrorMessage { code, message } })).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        tracing::error!(error = %error, "database operation failed");
        Self::Internal
    }
}
