use std::sync::Arc;
use tauri::{Manager, State};
use umbra::domain::ports::{ProviderRepository, VaultRepository};
use umbra::infrastructure::repositories::{
    provider_repo::TomlProviderRepository,
    vault_repo::EncryptedVaultRepository,
};

pub struct UmbraState {
    pub app: Arc<umbra::UmbraApp>,
}

fn get_frontend_dir() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("UMBRA_FRONTEND_DIR") {
        return std::path::PathBuf::from(dir);
    }
    if let Ok(exe) = std::env::current_exe() {
        let rel = exe.parent().unwrap_or(&exe).join("frontend");
        if rel.join("index.html").exists() { return rel; }
    }
    std::path::PathBuf::from("/mnt/external/projects/umbra/frontend/dist/browser")
}

fn serve_frontend(addr: std::net::SocketAddr, frontend_dir: std::path::PathBuf) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let app = axum::Router::new()
                .fallback_service(
                    tower_http::services::ServeDir::new(&frontend_dir)
                        .append_index_html_on_directories(true)
                );
            if let Err(e) = axum::serve(
                tokio::net::TcpListener::bind(addr).await.unwrap(),
                app
            ).await {
                eprintln!("Frontend server error: {}", e);
            }
        });
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let app = rt.block_on(async {
        umbra::init().await.expect("Failed to initialize Umbra")
    });

    let frontend_dir = get_frontend_dir();
    let frontend_port = 8340u16;
    let frontend_addr: std::net::SocketAddr = ([127, 0, 0, 1], frontend_port).into();

    if frontend_dir.join("index.html").exists() {
        serve_frontend(frontend_addr, frontend_dir.clone());
        println!("UMBRA frontend: http://{}", frontend_addr);
    } else {
        eprintln!("WARNING: Frontend not found at {:?}", frontend_dir);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(UmbraState { app: Arc::new(app) })
        .setup(move |app_handle| {
            if let Some(window) = app_handle.get_webview_window("main") {
                let url = format!("http://{}", frontend_addr);
                let _ = window.eval(&format!("window.location='{}'", url));
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_setup_status, get_providers, test_provider, get_provider_config_status,
            configure_provider, get_vault_status, unlock_vault, lock_vault, migrate_vault,
            get_system_status, get_models, get_hardware, trigger_training, get_training_stats,
            get_debugger_snapshot, get_sub_agents, synthesize_speech,
            minimize_window, toggle_maximize_window, close_window, get_backend_url,
        ])
        .run(tauri::generate_context!())
        .expect("Error running Umbra desktop app");
}

// ── Discovery ──
#[tauri::command]
async fn get_setup_status() -> Result<serde_json::Value, String> {
    let mut result = serde_json::json!({"ollama": false, "llamacpp": false, "models": [], "hardware": {}, "language": "en-US"});
    if let Ok(resp) = reqwest::get("http://localhost:11434/api/tags").await {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                result["ollama"] = serde_json::json!(true);
                result["models"] = serde_json::json!(data.get("models").and_then(|m| m.as_array()).map(|a| a.len()).unwrap_or(0));
            }
        }
    }
    if let Ok(resp) = reqwest::get("http://localhost:8080/v1/models").await {
        if resp.status().is_success() {
            result["llamacpp"] = serde_json::json!(true);
        }
    }
    if let Ok(locale) = std::env::var("LANG") {
        if locale.starts_with("es") { result["language"] = serde_json::json!("es-ES"); }
        else if locale.starts_with("zh") { result["language"] = serde_json::json!("zh-CN"); }
    }
    Ok(result)
}

// ── Providers ──
#[tauri::command]
async fn get_providers() -> Result<serde_json::Value, String> {
    serde_json::to_value(&*umbra::domain::models::provider::ALL_PROVIDERS).map_err(|e| e.to_string())
}

#[tauri::command]
async fn test_provider(provider_id: String, api_key: String) -> Result<serde_json::Value, String> {
    let map = umbra::domain::ports::provider_map();
    let provider = map.get(provider_id.as_str()).ok_or("Unknown provider")?;
    let uc = umbra::application::providers::test_connection::TestConnectionUseCase::new();
    Ok(uc.test(provider, &api_key, None).await)
}

#[tauri::command]
async fn get_provider_config_status() -> Result<serde_json::Value, String> {
    let repo = TomlProviderRepository;
    let uc = umbra::application::providers::get_status::GetProviderStatusUseCase::new(repo);
    uc.execute().map_err(|e| e.to_string())
}

#[tauri::command]
async fn configure_provider(provider_id: String, api_key: String, base_url: String, is_primary: bool, is_secondary: bool) -> Result<serde_json::Value, String> {
    let mut vault = EncryptedVaultRepository::new();
    if !vault.is_locked() {
        vault.set_key(&provider_id, &api_key, &base_url).map_err(|e| e.to_string())?;
    } else {
        let repo = TomlProviderRepository;
        let cfg = repo.load_config();
        if let Some(table) = cfg.as_table() {
            let mut map = table.clone();
            map.insert(provider_id.clone(), toml::Value::Table({
                let mut m = toml::map::Map::new();
                m.insert("api_key".into(), toml::Value::String(api_key.clone()));
                if !base_url.is_empty() { m.insert("base_url".into(), toml::Value::String(base_url.clone())); }
                m
            }));
            repo.save_config(&toml::Value::Table(map)).map_err(|e| e.to_string())?;
        }
    }
    Ok(serde_json::json!({"success": true}))
}

// ── Vault ──
#[tauri::command]
async fn get_vault_status() -> Result<serde_json::Value, String> {
    let mut vault = EncryptedVaultRepository::new();
    serde_json::to_value(vault.status()).map_err(|e| e.to_string())
}

#[tauri::command]
async fn unlock_vault(passphrase: String) -> Result<serde_json::Value, String> {
    let mut vault = EncryptedVaultRepository::new();
    vault.unlock(&passphrase).map_err(|e| format!("wrong_passphrase: {}", e))?;
    serde_json::to_value(vault.status()).map_err(|e| e.to_string())
}

#[tauri::command]
async fn lock_vault() -> Result<serde_json::Value, String> {
    let mut vault = EncryptedVaultRepository::new();
    vault.lock();
    serde_json::to_value(vault.status()).map_err(|e| e.to_string())
}

#[tauri::command]
async fn migrate_vault() -> Result<serde_json::Value, String> {
    let mut vault = EncryptedVaultRepository::new();
    let migrated = vault.migrate_from_env().map_err(|e| e.to_string())?;
    Ok(serde_json::json!({"migrated": migrated}))
}

// ── System ──
#[tauri::command]
async fn get_system_status(state: State<'_, UmbraState>) -> Result<serde_json::Value, String> {
    let snap = state.app.debugger.snapshot();
    Ok(serde_json::json!({
        "models": {"local_count": state.app.engine.models.local_models.len()},
        "sub_agents": state.app.sub_agents.list().len(),
        "debugger": {"health": snap.health, "warnings": snap.active_warnings, "errors": snap.active_errors},
    }))
}

#[tauri::command]
async fn get_models(state: State<'_, UmbraState>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({"local_models": state.app.engine.models.local_models}))
}

#[tauri::command]
async fn get_hardware(state: State<'_, UmbraState>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "recommended_quant": state.app.engine.safety.recommended_quant(),
        "safe_context_size": state.app.engine.safety.safe_context_size(4096),
    }))
}

#[tauri::command]
async fn trigger_training(state: State<'_, UmbraState>) -> Result<serde_json::Value, String> {
    let result = state.app.agent.trainer.auto_train("umbra-train").await.map_err(|e| e.to_string())?;
    Ok(serde_json::json!({"trained": result}))
}

#[tauri::command]
async fn get_training_stats(state: State<'_, UmbraState>) -> Result<serde_json::Value, String> {
    serde_json::to_value(state.app.agent.trainer.stats()).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_debugger_snapshot(state: State<'_, UmbraState>) -> Result<serde_json::Value, String> {
    serde_json::to_value(state.app.debugger.snapshot()).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_sub_agents(state: State<'_, UmbraState>) -> Result<serde_json::Value, String> {
    let agents: Vec<serde_json::Value> = state.app.sub_agents.list().iter().map(|a| {
        serde_json::json!({"name": a.name, "version": a.version, "model": a.model, "active": false})
    }).collect();
    Ok(serde_json::json!({"sub_agents": agents}))
}

// ── TTS ──
#[tauri::command]
async fn synthesize_speech(text: String, api_key: String, voice_id: String) -> Result<Vec<u8>, String> {
    let client = umbra::infrastructure::http::tts_client::TtsClient::new();
    client.synthesize_fish(&text, &api_key, &voice_id).await.map_err(|e| e.to_string())
}

// ── Window Controls ──
#[tauri::command]
async fn minimize_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(w) = app_handle.get_webview_window("main") { w.minimize().map_err(|e| e.to_string())?; }
    Ok(())
}

#[tauri::command]
async fn toggle_maximize_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(w) = app_handle.get_webview_window("main") {
        if w.is_maximized().unwrap_or(false) { w.unmaximize().map_err(|e| e.to_string())?; }
        else { w.maximize().map_err(|e| e.to_string())?; }
    }
    Ok(())
}

#[tauri::command]
async fn close_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(w) = app_handle.get_webview_window("main") { w.hide().map_err(|e| e.to_string())?; }
    Ok(())
}

#[tauri::command]
async fn get_backend_url() -> String { "http://127.0.0.1:8484".to_string() }

