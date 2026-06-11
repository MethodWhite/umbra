use axum::{
    extract::{State, Request},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    Json,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::time::Instant;

use crate::api::FrontendRouterState;

struct RateLimitState {
    attempts: HashMap<String, Vec<Instant>>,
}

impl RateLimitState {
    fn new() -> Self {
        Self { attempts: HashMap::new() }
    }

    fn check(&mut self, key: &str, max_attempts: u32, window_secs: u64) -> bool {
        let now = Instant::now();
        let cutoff = now - std::time::Duration::from_secs(window_secs);

        let attempts = self.attempts.entry(key.to_string()).or_default();
        attempts.retain(|&t| t > cutoff);

        if attempts.len() >= max_attempts as usize {
            false
        } else {
            attempts.push(now);
            true
        }
    }
}

static RATE_LIMITER: LazyLock<Mutex<RateLimitState>> = LazyLock::new(|| Mutex::new(RateLimitState::new()));

fn check_rate_limit(ip: &str) -> bool {
    RATE_LIMITER.lock().unwrap_or_else(|e| e.into_inner()).check(ip, 10, 60)
}

pub async fn auth_middleware(
    headers: HeaderMap,
    State(state): State<FrontendRouterState>,
    request: Request,
    next: Next,
) -> Response {
    let client_ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    if !check_rate_limit(client_ip) {
        return (StatusCode::TOO_MANY_REQUESTS, Json(serde_json::json!({"error": "Rate limit exceeded"}))).into_response();
    }

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
    let client_ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    if !check_rate_limit(client_ip) {
        return (StatusCode::TOO_MANY_REQUESTS, Json(serde_json::json!({"error": "Rate limit exceeded"}))).into_response();
    }

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
