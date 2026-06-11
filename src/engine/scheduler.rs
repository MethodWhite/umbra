// Zone 6 — Research/Stubs (server-gated)
use anyhow::{Context, Result};

const KILOBYTES_PER_MEGABYTE: u64 = 1024;
const DEFAULT_RAM_MB: u64 = 8192;
const LLAMACPP_DEFAULT_URL: &str = "http://localhost:8080";
const OLLAMA_DEFAULT_MODEL: &str = "llama3.2:latest";
const DEFAULT_OPENCODE_MODEL: &str = "deepseek-v4-flash";
const VRAM_F16_THRESHOLD_MB: u64 = 16_000;
const VRAM_Q8_THRESHOLD_MB: u64 = 8_000;
const VRAM_Q5_THRESHOLD_MB: u64 = 6_000;
const RAM_32GB_THRESHOLD_MB: u64 = 32_000;
const RAM_16GB_THRESHOLD_MB: u64 = 16_000;
const RAM_8GB_THRESHOLD_MB: u64 = 8_000;
const CONTEXT_32K: usize = 32768;
const CONTEXT_16K: usize = 16384;
const CONTEXT_8K: usize = 8192;
const CONTEXT_4K: usize = 4096;

pub const OPENCODE_GO_BASE_URL: &str = "https://opencode.ai/zen/go/v1";

pub const OPENCODE_GO_MODELS: &[&str] = &[
    "glm-5.1", "glm-5",
    "kimi-k2.6", "kimi-k2.5",
    "deepseek-v4-pro", "deepseek-v4-flash",
    "mimo-v2.5", "mimo-v2.5-pro",
    "minimax-m3", "minimax-m2.7", "minimax-m2.5",
    "qwen3.7-max", "qwen3.7-plus", "qwen3.6-plus",
];

#[derive(Debug, Clone)]
pub enum QuantLevel {
    Q4KM,
    Q5KM,
    Q8_0,
    F16,
}

impl std::fmt::Display for QuantLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Q4KM => write!(f, "Q4_K_M"),
            Self::Q5KM => write!(f, "Q5_K_M"),
            Self::Q8_0 => write!(f, "Q8_0"),
            Self::F16 => write!(f, "F16"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Backend {
    LlamaCppServer { url: String },
    Ollama { model: String },
    Cloud { provider: String, model: String },
    OpenCodeGo { model: String, api_key: String },
    Unavailable,
}

impl Backend {
    pub fn name(&self) -> &str {
        match self {
            Self::LlamaCppServer { .. } => "llama.cpp",
            Self::Ollama { .. } => "Ollama",
            Self::Cloud { provider, .. } => provider,
            Self::OpenCodeGo { .. } => "OpenCodeGo",
            Self::Unavailable => "N/A",
        }
    }

    pub fn is_local(&self) -> bool {
        matches!(self, Self::LlamaCppServer { .. } | Self::Ollama { .. })
    }

    pub fn is_cloud(&self) -> bool {
        matches!(self, Self::Cloud { .. } | Self::OpenCodeGo { .. })
    }

    pub fn opencodego_config(&self) -> Option<(&str, &str)> {
        match self {
            Self::OpenCodeGo { model, api_key } => Some((model, api_key)),
            _ => None,
        }
    }

    pub async fn chat_completion(&self, messages: Vec<serde_json::Value>) -> Result<String> {
        match self {
            Self::OpenCodeGo { model, api_key } => {
                let client = reqwest::Client::new();
                let body = serde_json::json!({
                    "model": model,
                    "messages": messages,
                    "stream": false,
                });
                let resp = client
                    .post(format!("{}/chat/completions", OPENCODE_GO_BASE_URL))
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Content-Type", "application/json")
                    .json(&body)
                    .timeout(std::time::Duration::from_secs(120))
                    .send()
                    .await
                    .context("OpenCode Go API request failed")?;
                let data: serde_json::Value = resp.json().await?;
                data["choices"][0]["message"]["content"]
                    .as_str()
                    .map(String::from)
                    .context("OpenCode Go response missing content")
            }
            _ => anyhow::bail!("chat_completion called on non-OpenCodeGo backend"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub primary: Backend,
    pub secondary: Backend,
}

impl ModelConfig {
    pub fn both_available(&self) -> bool {
        !matches!(self.primary, Backend::Unavailable)
    }
}

pub struct AdaptiveScheduler {
    pub vram_mb: u64,
    pub ram_mb: u64,
    pub quant_level: QuantLevel,
    pub context_size: usize,
}

impl AdaptiveScheduler {
    pub async fn detect() -> Self {
        let vram_mb = Self::detect_vram();
        let ram_mb = Self::detect_ram();

        let quant_level = match vram_mb {
            vram if vram >= VRAM_F16_THRESHOLD_MB => QuantLevel::F16,
            vram if vram >= VRAM_Q8_THRESHOLD_MB => QuantLevel::Q8_0,
            vram if vram >= VRAM_Q5_THRESHOLD_MB => QuantLevel::Q5KM,
            _ => QuantLevel::Q4KM,
        };

        let context_size = match ram_mb {
            ram if ram >= RAM_32GB_THRESHOLD_MB => CONTEXT_32K,
            ram if ram >= RAM_16GB_THRESHOLD_MB => CONTEXT_16K,
            ram if ram >= RAM_8GB_THRESHOLD_MB => CONTEXT_8K,
            _ => CONTEXT_4K,
        };

        Self { vram_mb, ram_mb, quant_level, context_size }
    }

    pub fn select_backend(&self, router: &super::router::ModelRouter) -> Backend {
        self.select_primary(router)
    }

    pub fn select_primary(&self, router: &super::router::ModelRouter) -> Backend {
        let api_key = std::env::var("OPENCODE_GO_API_KEY").ok();
        if let Some(key) = &api_key {
            if !key.is_empty() {
                return Backend::OpenCodeGo {
                    model: DEFAULT_OPENCODE_MODEL.into(),
                    api_key: key.clone(),
                };
            }
        }

        if router.llamacpp_running {
            Backend::LlamaCppServer { url: LLAMACPP_DEFAULT_URL.into() }
        } else if router.ollama_running {
            if let Some(model) = router.best_reasoning_model() {
                Backend::Ollama { model }
            } else {
                Backend::Ollama { model: OLLAMA_DEFAULT_MODEL.into() }
            }
        } else {
            if let Some(model) = router.best_reasoning_model() {
                Backend::Ollama { model }
            } else {
                Backend::Unavailable
            }
        }
    }

    pub fn select_secondary(&self, router: &super::router::ModelRouter) -> Backend {
        if let Some(model) = router.best_secondary_model() {
            if router.llamacpp_running {
                Backend::LlamaCppServer { url: LLAMACPP_DEFAULT_URL.into() }
            } else {
                Backend::Ollama { model }
            }
        } else {
            self.select_primary(router)
        }
    }

    pub fn select_opencodego(&self, model: &str, api_key: &str) -> Backend {
        Backend::OpenCodeGo {
            model: model.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub fn select_config(&self, router: &super::router::ModelRouter) -> ModelConfig {
        ModelConfig {
            primary: self.select_primary(router),
            secondary: self.select_secondary(router),
        }
    }

    fn detect_vram() -> u64 {
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .args(["--query-gpu=memory.total", "--format=csv,noheader,nounits"])
            .output()
        {
            let output_string = String::from_utf8_lossy(&output.stdout);
            if let Ok(vram_mb) = output_string.trim().parse::<u64>() {
                return vram_mb;
            }
        }
        if let Ok(output) = std::process::Command::new("rocm-smi")
            .args(["--showmeminfo", "vram"])
            .output()
        {
            let output_string = String::from_utf8_lossy(&output.stdout);
            for line in output_string.lines() {
                if line.contains("Total") && line.contains("VRAM") {
                    if let Some(vram_mb) = line.split_whitespace().filter_map(|part| part.parse::<u64>().ok()).next() {
                        return vram_mb;
                    }
                }
            }
        }
        0
    }

    fn detect_ram() -> u64 {
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    if let Some(kilobytes) = line.split_whitespace().nth(1) {
                        if let Ok(kilobytes) = kilobytes.parse::<u64>() {
                            return kilobytes / KILOBYTES_PER_MEGABYTE;
                        }
                    }
                }
            }
        }
        DEFAULT_RAM_MB
    }
}
