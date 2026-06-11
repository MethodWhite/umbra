// Zone 2 — Domain/Ports
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Vault locked")]
    VaultLocked,
    #[error("External API error: {0}")]
    ExternalApi(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
            AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m.clone()),
            AppError::Validation(m) => (StatusCode::BAD_REQUEST, m.clone()),
            AppError::VaultLocked => (StatusCode::LOCKED, "Vault is locked".into()),
            AppError::ExternalApi(m) => (StatusCode::BAD_GATEWAY, m.clone()),
            AppError::Internal(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };
        (status, Json(serde_json::json!({"error": message}))).into_response()
    }
}
