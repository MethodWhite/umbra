use axum::{Json, extract::{State, Path}, response::IntoResponse, http::StatusCode};

use crate::domain::models::ALL_PROVIDERS;
use crate::domain::ports::VaultRepository;
use crate::domain::ports::provider_port::ProviderRepository;
use crate::application::providers::{ConfigureProviderUseCase, TestConnectionUseCase, GetProviderStatusUseCase};
use crate::infrastructure::repositories::TomlProviderRepository;

use super::super::FrontendRouterState;

pub async fn list() -> Json<serde_json::Value> {
    let providers: Vec<serde_json::Value> = ALL_PROVIDERS.iter().map(|p| {
        serde_json::json!({
            "id": p.id,
            "name": p.name,
            "name_zh": p.name_zh,
            "api_type": p.api_type,
            "base_url": p.base_url,
            "models": p.models,
            "requires_key": p.requires_key,
            "is_chinese": p.is_chinese,
            "is_local": p.is_local,
            "supports_reasoning": p.supports_reasoning,
            "supports_vision": p.supports_vision,
        })
    }).collect();
    Json(serde_json::json!({"providers": providers}))
}

pub async fn get(
    Path(id): Path<String>,
) -> impl IntoResponse {
    let map = crate::domain::ports::provider_port::provider_map();
    match map.get(id.as_str()) {
        Some(p) => {
            let repo = TomlProviderRepository::new();
            let config = repo.load_config();
            let provider_config = config.get("providers")
                .and_then(|v| v.get(&id))
                .and_then(|v| v.as_table())
                .cloned()
                .unwrap_or_default();
            let base_url = provider_config.get("base_url")
                .and_then(|v| v.as_str())
                .unwrap_or(p.base_url);
            let api_key_configured = provider_config.get("api_key")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                != "";
            Json(serde_json::json!({
                "id": p.id,
                "name": p.name,
                "base_url": base_url,
                "api_key_configured": api_key_configured,
                "models": p.models,
            })).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Provider not found"}))).into_response(),
    }
}

#[derive(serde::Deserialize)]
pub struct ConfigureBody {
    pub provider_id: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub is_primary: Option<bool>,
    pub is_secondary: Option<bool>,
}

pub async fn configure(
    State(state): State<FrontendRouterState>,
    Json(body): Json<ConfigureBody>,
) -> impl IntoResponse {
    let repo = TomlProviderRepository::new();
    let use_case = ConfigureProviderUseCase::new(repo);
    let mut vault = state.vault.lock().await;
    match use_case.execute(
        &mut *vault, &body.provider_id, body.api_key, body.base_url,
        body.is_primary, body.is_secondary,
    ).await {
        Ok(val) => Json(val).into_response(),
        Err(e) => e.into_response(),
    }
}

#[derive(serde::Deserialize)]
pub struct TestBody {
    pub provider_id: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

pub async fn test(
    State(state): State<FrontendRouterState>,
    Json(body): Json<TestBody>,
) -> Json<serde_json::Value> {
    let map = crate::domain::ports::provider_port::provider_map();
    let provider = match map.get(body.provider_id.as_str()) {
        Some(p) => p,
        None => return Json(serde_json::json!({"valid": false, "error": "Unknown provider"})),
    };

    let api_key = if let Some(k) = body.api_key.clone() {
        k
    } else {
        let mut vault = state.vault.lock().await;
        vault.get_key(&body.provider_id).unwrap_or_default()
    };

    let base_url = if let Some(url) = body.base_url.clone() {
        Some(url)
    } else {
        let mut vault = state.vault.lock().await;
        vault.get_base_url(&body.provider_id)
    };

    let use_case = TestConnectionUseCase::new();
    let url = base_url.as_deref();
    let result = use_case.test(provider, &api_key, url).await;
    Json(result)
}

pub async fn test_all(
    State(state): State<FrontendRouterState>,
) -> impl IntoResponse {
    let keys = {
        let mut vault = state.vault.lock().await;
        if vault.is_locked() {
            return (StatusCode::LOCKED, Json(serde_json::json!({"error": "Vault is locked"}))).into_response();
        }
        vault.list_keys()
    };

    let map = crate::domain::ports::provider_port::provider_map();
    let mut results = std::collections::HashMap::new();
    let mut tested = 0usize;
    let mut passed = 0usize;

    for (pid, _) in &keys {
        let provider = match map.get(pid.as_str()) {
            Some(p) => p,
            None => {
                results.insert(pid.clone(), serde_json::json!({"valid": false, "error": "Unknown provider"}));
                continue;
            }
        };
        let api_key = {
            let mut vault = state.vault.lock().await;
            vault.get_key(pid)
        };
        let api_key = match api_key {
            Some(k) => k,
            None => {
                results.insert(pid.clone(), serde_json::json!({"valid": false, "error": "No key configured"}));
                continue;
            }
        };
        let use_case = TestConnectionUseCase::new();
        let r = use_case.test(provider, &api_key, None).await;
        if r.get("valid").and_then(|v| v.as_bool()).unwrap_or(false) {
            passed += 1;
        }
        tested += 1;
        results.insert(pid.clone(), r);
    }

    Json(serde_json::json!({
        "results": results,
        "tested": tested,
        "passed": passed,
    })).into_response()
}

pub async fn config_status() -> Json<serde_json::Value> {
    let repo = TomlProviderRepository::new();
    let use_case = GetProviderStatusUseCase::new(repo);
    match use_case.execute() {
        Ok(val) => Json(val),
        Err(_) => Json(serde_json::json!({"error": "Failed to read config"})),
    }
}
