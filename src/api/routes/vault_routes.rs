use axum::{Json, extract::State, response::IntoResponse, http::StatusCode};

use crate::domain::models::VaultStatus;
use crate::domain::ports::VaultRepository;
use crate::application::vault::{UnlockUseCase, LockUseCase, MigrateUseCase};

use super::super::FrontendRouterState;

#[derive(serde::Deserialize)]
pub struct UnlockBody {
    pub passphrase: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct VaultKeyBody {
    pub provider_id: String,
    pub api_key: String,
    pub base_url: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct AutoLockBody {
    pub minutes: Option<u64>,
}

pub async fn status(
    State(state): State<FrontendRouterState>,
) -> Json<VaultStatus> {
    let mut vault = state.vault.lock().await;
    Json(vault.status())
}

pub async fn unlock(
    State(state): State<FrontendRouterState>,
    Json(body): Json<UnlockBody>,
) -> impl IntoResponse {
    let passphrase = body.passphrase.unwrap_or_default();
    let mut vault = state.vault.lock().await;
    let use_case = UnlockUseCase::new();
    match use_case.execute(&mut *vault, &passphrase).await {
        Ok(val) => Json(val).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn lock(
    State(state): State<FrontendRouterState>,
) -> Json<serde_json::Value> {
    let mut vault = state.vault.lock().await;
    let use_case = LockUseCase::new();
    use_case.execute(&mut *vault);
    Json(serde_json::json!({"success": true}))
}

pub async fn list_keys(
    State(state): State<FrontendRouterState>,
) -> impl IntoResponse {
    let mut vault = state.vault.lock().await;
    if vault.is_locked() {
        return (StatusCode::LOCKED, Json(serde_json::json!({"error": "Vault is locked"}))).into_response();
    }
    let keys = vault.list_keys();
    (StatusCode::OK, Json(serde_json::json!({"keys": keys}))).into_response()
}

pub async fn get_key(
    State(state): State<FrontendRouterState>,
    axum::extract::Path(provider_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let mut vault = state.vault.lock().await;
    if vault.is_locked() {
        return (StatusCode::LOCKED, Json(serde_json::json!({"error": "Vault is locked"}))).into_response();
    }
    let has_key = vault.get_key(&provider_id).is_some();
    (StatusCode::OK, Json(serde_json::json!({"provider_id": provider_id, "has_key": has_key}))).into_response()
}

pub async fn set_key(
    State(state): State<FrontendRouterState>,
    Json(body): Json<VaultKeyBody>,
) -> impl IntoResponse {
    let mut vault = state.vault.lock().await;
    if vault.is_locked() {
        return (StatusCode::LOCKED, Json(serde_json::json!({"error": "Vault is locked"}))).into_response();
    }
    match vault.set_key(&body.provider_id, &body.api_key, &body.base_url.unwrap_or_default()) {
        Ok(()) => Json(serde_json::json!({"success": true})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e}))).into_response(),
    }
}

pub async fn delete_key(
    State(state): State<FrontendRouterState>,
    axum::extract::Path(provider_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let mut vault = state.vault.lock().await;
    if vault.is_locked() {
        return (StatusCode::LOCKED, Json(serde_json::json!({"error": "Vault is locked"}))).into_response();
    }
    match vault.delete_key(&provider_id) {
        Ok(()) => Json(serde_json::json!({"success": true})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e}))).into_response(),
    }
}

pub async fn migrate(
    State(state): State<FrontendRouterState>,
) -> impl IntoResponse {
    let use_case = MigrateUseCase::new();
    let mut vault = state.vault.lock().await;
    match use_case.execute(&mut *vault).await {
        Ok(val) => Json(val).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn auto_lock(
    State(state): State<FrontendRouterState>,
    Json(body): Json<AutoLockBody>,
) -> Json<serde_json::Value> {
    let mut vault = state.vault.lock().await;
    let minutes = body.minutes.unwrap_or(15);
    vault.set_auto_lock(minutes);
    Json(serde_json::json!({"success": true, "auto_lock_minutes": minutes}))
}
