// Zone 0 — Config/Init
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct UmbraConfig {
    pub paths: PathsConfig,
    pub api: ApiConfig,
    pub audio: AudioConfig,
    pub ollama: OllamaConfig,
    pub training: TrainingConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PathsConfig {
    pub models_dir: PathBuf,
    pub subagents_dir: PathBuf,
    pub jarvis_dir: PathBuf,
    pub logs_dir: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiConfig {
    pub backend_port: u16,
    pub frontend_port: u16,
    pub backend_host: String,
    pub frontend_host: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AudioConfig {
    pub fish_api_url: String,
    pub default_voice_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OllamaConfig {
    pub base_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrainingConfig {
    pub auto_train_interval_mins: u32,
    pub max_examples: u32,
    pub jepa_epochs: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    pub auth_dir: PathBuf,
    pub env_file_perms: String,
}

impl UmbraConfig {
    pub fn load() -> Self {
        let config_path = Self::config_path();
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            UmbraConfig::default()
        }
    }

    fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        PathBuf::from(home).join(".umbra/config.toml")
    }
}

impl Default for UmbraConfig {
    fn default() -> Self {
        UmbraConfig {
            paths: PathsConfig {
                models_dir: PathBuf::from("/mnt/external/projects/umbra/models"),
                subagents_dir: PathBuf::from("/mnt/external/projects/umbra/sub_agents"),
                jarvis_dir: PathBuf::from("/mnt/external/projects/jarvis"),
                logs_dir: PathBuf::from("/mnt/external/projects/umbra/logs"),
            },
            api: ApiConfig {
                backend_port: 8484,
                frontend_port: 8340,
                backend_host: "127.0.0.1".into(),
                frontend_host: "127.0.0.1".into(),
            },
            audio: AudioConfig {
                fish_api_url: "https://api.fish.audio/v1/tts".into(),
                default_voice_id: "612b878b113047d9a770c069c8b4fdfe".into(),
            },
            ollama: OllamaConfig {
                base_url: "http://localhost:11434".into(),
            },
            training: TrainingConfig {
                auto_train_interval_mins: 30,
                max_examples: 1000,
                jepa_epochs: 50,
            },
            security: SecurityConfig {
                auth_dir: PathBuf::from("~/.umbra"),
                env_file_perms: "0600".into(),
            },
        }
    }
}
