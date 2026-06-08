//! The HTTP error boundary. Every handler returns `Result<_, ApiError>` and the
//! `IntoResponse` impl maps variants to status codes plus a typed JSON body so
//! the dashboard can branch on `error` codes.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use wienerenvoy_core::PowerError;

#[derive(Debug)]
pub enum ApiError {
    Unauthorized,
    Forbidden,
    NotImplemented(String),
    Conflict(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "missing or invalid bearer token".to_string(),
            ),
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                "forbidden",
                "caller is not on loopback or the tailnet".to_string(),
            ),
            Self::NotImplemented(m) => (StatusCode::NOT_IMPLEMENTED, "not_implemented", m),
            Self::Conflict(m) => (StatusCode::CONFLICT, "conflict", m),
            Self::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, "internal", m),
        };
        (status, Json(json!({ "error": code, "message": message }))).into_response()
    }
}

impl From<PowerError> for ApiError {
    fn from(err: PowerError) -> Self {
        match err {
            PowerError::NotPermitted => Self::Internal(
                "power action not permitted; the daemon must run as root".to_string(),
            ),
            PowerError::Disabled(action) => {
                Self::Conflict(format!("action '{action}' is disabled"))
            }
            other => Self::Internal(other.to_string()),
        }
    }
}
