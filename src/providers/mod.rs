use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

const DEFAULT_API_TYPE: &str = "openai";
const ANTHROPIC_API_VERSION: &str = "2023-06-01";
const OPENCODE_GO_ID: &str = "opencode-go";
const OPENCODE_GO_BASE_URL: &str = "https://opencode.ai/zen/go/v1";
const HTTP_TIMEOUT_SECONDS: u64 = 60;
/// Internal vault port
#[allow(dead_code)]
const INTERNAL_VAULT_PORT: u16 = 8340;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProvider {
    pub id: String,
    pub name: String,
    pub api_type: String,
    pub base_url: String,
    #[serde(skip)]
    pub api_key: String,
    #[serde(default)]
    pub models: Vec<String>,
}

impl ModelProvider {
    pub fn opencode_go(api_key: String) -> Self {
        Self {
            id: OPENCODE_GO_ID.into(),
            name: "OpenCode Go".into(),
            api_type: DEFAULT_API_TYPE.into(),
            base_url: OPENCODE_GO_BASE_URL.into(),
            api_key,
            models: crate::engine::scheduler::OPENCODE_GO_MODELS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }

    pub fn is_openai_compatible(&self) -> bool {
        matches!(self.api_type.as_str(), "openai" | "azure" | "openrouter")
    }

    pub fn auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }

    pub fn build_client(&self) -> reqwest::Client {
        let mut headers = HeaderMap::new();
        if let Ok(hv) = HeaderValue::from_str(&self.auth_header()) {
            headers.insert(AUTHORIZATION, hv);
        }
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        match self.api_type.as_str() {
            "anthropic" => {
                if let Ok(hv) = HeaderValue::from_str(&self.api_key) {
                    use std::str::FromStr;
                    if let Ok(name) = reqwest::header::HeaderName::from_str("x-api-key") {
                        headers.insert(name, hv);
                    }
                }
                use std::str::FromStr;
                if let Ok(ver) = reqwest::header::HeaderName::from_str("anthropic-version") {
                    headers.insert(ver, HeaderValue::from_static(ANTHROPIC_API_VERSION));
                }
            }
            "google" => {
                headers.remove(AUTHORIZATION);
            }
            _ => {}
        }
        reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SECONDS))
            .build()
            .unwrap_or_default()
    }

    pub async fn test_connection(&self) -> Result<String> {
        if self.is_openai_compatible() {
            let client = self.build_client();
            let resp = client
                .get(format!("{}/models", self.base_url.trim_end_matches('/')))
                .send()
                .await
                .context("Failed to reach provider")?;
            if resp.status().is_success() {
                let response_data: serde_json::Value = resp.json().await?;
                let model_count = response_data["data"].as_array().map(|array| array.len()).unwrap_or(0);
                Ok(format!("OK — {} models available", model_count))
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("HTTP {}: {}", status, body)
            }
        } else {
            anyhow::bail!("Test not implemented for api_type: {}", self.api_type)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersConfig {
    #[serde(default)]
    pub providers: Vec<ProviderEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderEntry {
    pub id: String,
    pub api_key: String,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub api_type: Option<String>,
    #[serde(default)]
    pub enabled: bool,
}

pub struct ProviderRegistry {
    pub providers: HashMap<String, ModelProvider>,
    pub config_path: PathBuf,
    pub enabled_ids: Vec<String>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        let config_path = PathBuf::from(home).join(".umbra/providers.toml");
        Self {
            providers: HashMap::new(),
            config_path,
            enabled_ids: Vec::new(),
        }
    }

    pub fn load() -> Self {
        let mut registry = Self::new();
        if registry.config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&registry.config_path) {
                if let Ok(cfg) = toml::from_str::<ProvidersConfig>(&content) {
                    for entry in cfg.providers {
                        let api_type = entry.api_type.unwrap_or_else(|| DEFAULT_API_TYPE.into());
                        let base_url = entry.base_url.clone().unwrap_or_else(|| {
                            match api_type.as_str() {
                                "anthropic" => "https://api.anthropic.com".into(),
                                "google" => "https://generativelanguage.googleapis.com".into(),
                                _ => "https://api.openai.com/v1".into(),
                            }
                        });
                        let provider = ModelProvider {
                            id: entry.id.clone(),
                            name: entry.id.clone(),
                            api_type,
                            base_url,
                            api_key: entry.api_key,
                            models: Vec::new(),
                        };
                        registry.providers.insert(entry.id.clone(), provider);
                        if entry.enabled {
                            registry.enabled_ids.push(entry.id.clone());
                        }
                    }
                }
            }
        }
        if let Ok(key) = std::env::var("OPENCODE_GO_API_KEY") {
            if !key.is_empty() {
                let opencode_provider = ModelProvider::opencode_go(key);
                registry.enabled_ids.push(opencode_provider.id.clone());
                registry.providers.entry(opencode_provider.id.clone()).or_insert(opencode_provider);
            }
        }
        registry
    }

    pub fn enabled_providers(&self) -> Vec<&ModelProvider> {
        self.enabled_ids
            .iter()
            .filter_map(|id| self.providers.get(id))
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<&ModelProvider> {
        self.providers.get(id)
    }

    pub fn is_configured(&self, id: &str) -> bool {
        self.providers.contains_key(id)
    }

    pub fn count_enabled(&self) -> usize {
        self.enabled_ids.len()
    }

    pub fn get_api_key(&self, provider_id: &str) -> Option<String> {
        let env_key = format!("{}_API_KEY", provider_id.to_uppercase().replace("-", "_"));
        if let Ok(key) = std::env::var(&env_key) {
            if !key.is_empty() { return Some(key); }
        }
        self.providers.get(provider_id).map(|p| p.api_key.clone())
            .filter(|k| !k.is_empty())
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
