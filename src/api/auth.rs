use axum::{Json, extract::State, response::IntoResponse};
use axum::http::{header, StatusCode, HeaderValue};

use super::FrontendRouterState;

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "online",
        "name": "UMBRA Frontend",
        "version": env!("CARGO_PKG_VERSION"),
        "backend": "umbra",
    }))
}

pub async fn session(
    State(state): State<FrontendRouterState>,
) -> impl IntoResponse {
    let token = state.auth_token.clone();
    let cookie = format!(
        "umbra_session={}; HttpOnly; SameSite=Strict; Secure; Max-Age=86400; Path=/",
        token
    );
    let body = serde_json::json!({
        "status": "authenticated",
        "token": token,
    });
    let body_str = serde_json::to_string(&body).unwrap_or_default();
    let cookie_header = HeaderValue::from_str(&cookie).unwrap_or_else(|_| {
        HeaderValue::from_static("umbra_session=error; Path=/")
    });
    let headers = [
        (header::SET_COOKIE, cookie_header),
        (header::CONTENT_TYPE, HeaderValue::from_static("application/json")),
    ];
    (StatusCode::OK, headers, body_str)
}
