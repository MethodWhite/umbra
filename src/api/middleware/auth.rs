use axum::{
    extract::{State, Request},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    Json,
    response::{IntoResponse, Response},
};

use crate::api::FrontendRouterState;

pub async fn auth_middleware(
    headers: HeaderMap,
    State(state): State<FrontendRouterState>,
    request: Request,
    next: Next,
) -> Response {
    let key = headers
        .get("x-umbra-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if key != state.auth_token {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Unauthorized"}))).into_response();
    }
    next.run(request).await
}

pub async fn frontend_auth_middleware(
    State(state): State<FrontendRouterState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    let key = headers
        .get("x-umbra-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let cookie_key = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|c| {
            for cookie in c.split(';') {
                let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
                if parts.len() == 2 && parts[0] == "umbra_session" {
                    return Some(parts[1].to_string());
                }
            }
            None
        });

    let authorized = key == state.auth_token
        || cookie_key.map(|c| c == state.auth_token).unwrap_or(false);

    if !authorized {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Unauthorized"}))).into_response();
    }

    next.run(request).await
}
