use axum::{Json, response::IntoResponse, http::StatusCode};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(&home).join(".umbra/config.toml")
}

fn vault_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(&home).join(".umbra/vault.enc")
}

const SYSTEM_MODES: &[(&str, &str, &str, &str, u64)] = &[
    ("secure", "🔒 Secure", "🔒 Seguro", "Full TLS, vault required, auto-lock 5min, restricted CORS, Firefox hardening", 5),
    ("balanced", "⚖️ Balanced", "⚖️ Balanceado", "TLS enabled, vault optional, auto-lock 15min", 15),
    ("unrestricted", "🚀 Unrestricted", "🚀 Sin restricciones", "Local-only, no TLS needed, no vault, full API access, no hardening", 0),
];

fn get_current_mode() -> String {
    let path = config_path();
    if !path.exists() { return "balanced".into(); }
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    if let Ok(config) = content.parse::<toml::Value>() {
        config.get("system")
            .and_then(|v| v.get("mode"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "balanced".into())
    } else {
        "balanced".into()
    }
}

fn is_first_run() -> bool {
    if config_path().exists() { return false; }
    if vault_path().exists() { return false; }
    true
}

async fn auto_discover() -> serde_json::Value {
    let mut providers: Vec<String> = Vec::new();
    let mut models: Vec<String> = Vec::new();
    let mut gpu_info: Option<String> = None;
    let mut language = "en-US".to_string();

    if let Ok(client) = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
    {
        if let Ok(resp) = client.get("http://localhost:11434/api/tags").send().await {
            if resp.status() == 200 {
                if let Ok(data) = resp.json::<serde_json::Value>().await {
                    if let Some(m) = data.get("models").and_then(|v| v.as_array()) {
                        for m in m.iter().take(10) {
                            if let Some(name) = m.get("name").and_then(|v| v.as_str()) {
                                models.push(name.to_string());
                            }
                        }
                    }
                }
                providers.push("ollama".into());
            }
        }

        if let Ok(resp) = client.get("http://localhost:8080/v1/models").send().await {
            if resp.status() == 200 {
                providers.push("llamacpp".into());
            }
        }
    }

    if let Ok(output) = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=name,memory.total,driver_version", "--format=csv,noheader"])
        .output()
    {
        if output.status.success() {
            if let Ok(s) = String::from_utf8(output.stdout) {
                gpu_info = Some(s.trim().to_string());
            }
        }
    }

    if let Ok(lang) = std::env::var("LANG") {
        if lang.starts_with("es") {
            language = "es-ES".into();
        } else if lang.starts_with("zh") {
            language = "zh-CN".into();
        }
    }

    serde_json::json!({
        "providers": providers,
        "models": models,
        "hardware": { "gpu": gpu_info, },
        "language": language,
    })
}

pub async fn status() -> Json<serde_json::Value> {
    let first = is_first_run();
    let mode = get_current_mode();
    let discovery = auto_discover().await;

    let modes: serde_json::Value = SYSTEM_MODES.iter().map(|(id, label, label_es, desc_es, lock_min)| {
        (id.to_string(), serde_json::json!({
            "label": label,
            "label_es": label_es,
            "description_es": desc_es,
            "auto_lock_minutes": lock_min,
        }))
    }).collect();

    Json(serde_json::json!({
        "first_run": first,
        "current_mode": mode,
        "modes": modes,
        "discovery": discovery,
    }))
}

#[derive(serde::Deserialize)]
pub struct SetModeBody {
    pub mode: String,
}

pub async fn set_mode(
    Json(body): Json<SetModeBody>,
) -> impl IntoResponse {
    let mode = &body.mode;
    let mode_config = SYSTEM_MODES.iter().find(|(id, _, _, _, _)| id == mode);
    let mode_config = match mode_config {
        Some(m) => m,
        None => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"success": false, "error": "Invalid mode"}))).into_response(),
    };

    let path = config_path();
    let mut config: toml::Value = if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        content.parse().unwrap_or(toml::Value::Table(Default::default()))
    } else {
        toml::Value::Table(Default::default())
    };

    let system_toml = toml::Value::Table({
        let mut table = toml::map::Map::new();
        table.insert("mode".into(), toml::Value::String(mode.clone()));
        table.insert("auto_lock_minutes".into(), toml::Value::Integer(mode_config.4 as i64));
        table.insert("require_vault".into(), toml::Value::Boolean(mode == &"secure"));
        table.insert("cors_origin".into(), toml::Value::String(
            if mode == &"secure" { "strict".into() } else if mode == &"balanced" { "local".into() } else { "any".into() }
        ));
        table.insert("firefox_hardening".into(), toml::Value::Boolean(mode == &"secure" || mode == &"balanced"));
        table.insert("system_hardening".into(), toml::Value::Boolean(mode == &"secure"));
        table
    });

    if let Some(table) = config.as_table_mut() {
        table.insert("system".into(), system_toml);
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let toml_str = toml::to_string_pretty(&config).unwrap_or_default();
    let tmp_path = path.with_extension("tmp");
    std::fs::write(&tmp_path, &toml_str).ok();
    std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o600)).ok();
    std::fs::rename(&tmp_path, &path).ok();

    Json(serde_json::json!({"success": true, "mode": mode})).into_response()
}
