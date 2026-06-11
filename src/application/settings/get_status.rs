// Zone 3 — Application
use crate::domain::ports::VaultRepository;

pub struct GetSettingsStatusUseCase;

impl GetSettingsStatusUseCase {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, vault: &mut dyn VaultRepository, backend_url: &str, start_time: std::time::Instant) -> serde_json::Value {
        let uptime = start_time.elapsed().as_secs();

        let has_anthropic = std::env::var("ANTHROPIC_API_KEY").ok()
            .map(|s| !s.is_empty()).unwrap_or(false)
            || vault.get_key("anthropic").is_some();

        let has_fish = std::env::var("FISH_API_KEY").ok()
            .map(|s| !s.is_empty()).unwrap_or(false)
            || vault.get_key("fish_audio").is_some();

        let models = get_backend_models(backend_url).await;

        serde_json::json!({
            "calendar_accessible": false,
            "mail_accessible": false,
            "memory_count": 0,
            "task_count": 0,
            "server_port": 8340,
            "models": models,
            "claude_code_installed": has_anthropic,
            "notes_accessible": false,
            "uptime_seconds": uptime,
            "keys_configured": has_anthropic && has_fish,
        })
    }
}

async fn get_backend_models(backend_url: &str) -> serde_json::Value {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build().ok();
    let client = match client {
        Some(c) => c,
        None => return serde_json::json!({"primary": "N/A", "secondary": "N/A"}),
    };
    match client.get(format!("{}/api/v1/models", backend_url)).send().await {
        Ok(resp) if resp.status() == 200 => {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                serde_json::json!({
                    "primary": data.get("primary").and_then(|v| v.get("name")).and_then(|v| v.as_str()).unwrap_or("N/A"),
                    "secondary": data.get("secondary").and_then(|v| v.get("name")).and_then(|v| v.as_str()).unwrap_or("N/A"),
                })
            } else {
                serde_json::json!({"primary": "N/A", "secondary": "N/A"})
            }
        }
        _ => serde_json::json!({"primary": "N/A", "secondary": "N/A"}),
    }
}
