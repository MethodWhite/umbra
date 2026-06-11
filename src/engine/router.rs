// Zone 6 — Research/Stubs (server-gated)
use anyhow::Result;

const OLLAMA_TAGS_URL: &str = "http://localhost:11434/api/tags";
const LLAMACPP_HEALTH_URL: &str = "http://localhost:8080/health";
const SERVER_TIMEOUT_SECONDS: u64 = 2;
const OLLAMA_TIMEOUT_SECONDS: u64 = 3;

#[derive(Debug, Clone)]
pub struct LocalModel {
    pub name: String,
    pub source: String,
    pub is_vision: bool,
    pub size_rank: u32,
}

pub struct ModelRouter {
    pub models: Vec<LocalModel>,
    pub reasoning_models: Vec<String>,
    pub vision_models: Vec<String>,
    pub ollama_running: bool,
    pub llamacpp_running: bool,
}

impl ModelRouter {
    pub async fn scan() -> Self {
        let mut models = Vec::new();
        let mut reasoning = Vec::new();
        let mut vision = Vec::new();
        let ollama_running = Self::check_server(OLLAMA_TAGS_URL).await;
        let llamacpp_running = Self::check_server(LLAMACPP_HEALTH_URL).await;

        if llamacpp_running {
            models.push(LocalModel {
                name: "llama.cpp-server".into(),
                source: "llama.cpp".into(),
                is_vision: false,
                size_rank: 100,
            });
            reasoning.push("llama.cpp-server".into());
        }

        if ollama_running {
            if let Ok(tags) = Self::fetch_ollama_models().await {
                for tag in &tags {
                    let is_vision = Self::is_vision_model(tag);
                    let rank = Self::model_rank(tag, 0);
                    models.push(LocalModel {
                        name: tag.clone(),
                        source: "ollama".into(),
                        is_vision,
                        size_rank: rank,
                    });
                    if is_vision {
                        vision.push(tag.clone());
                    } else {
                        reasoning.push(tag.clone());
                    }
                }
            }
        }

        reasoning.sort_by(|a, b| Self::model_rank(b, 0).cmp(&Self::model_rank(a, 0)));
        vision.sort_by(|a, b| Self::model_rank(b, 0).cmp(&Self::model_rank(a, 0)));

        Self { models, reasoning_models: reasoning, vision_models: vision, ollama_running, llamacpp_running }
    }

    pub fn best_reasoning_model(&self) -> Option<String> {
        if self.llamacpp_running {
            return Some("llama.cpp-server".into());
        }
        self.reasoning_models.first().cloned()
    }

    pub fn best_vision_model(&self) -> Option<String> {
        self.vision_models.first().cloned()
    }

    pub fn best_secondary_model(&self) -> Option<String> {
        if !self.vision_models.is_empty() {
            return self.vision_models.first().cloned();
        }
        if self.reasoning_models.len() > 1 {
            return self.reasoning_models.get(1).cloned();
        }
        self.reasoning_models.first().cloned()
    }

    fn is_vision_model(name: &str) -> bool {
        let lower = name.to_lowercase();
        lower.contains("vision") || lower.contains("llava") || lower.contains("bakllava")
            || lower.contains("cogvlm") || lower.contains("minicpm-v")
    }

    fn model_rank(name: &str, fallback: usize) -> u32 {
        let lower = name.to_lowercase();
        if lower.contains("70b") || lower.contains("qwen2.5:72b") { return 90 }
        if lower.contains("32b") || lower.contains("qwen2.5:32b") { return 80 }
        if lower.contains("13b") { return 70 }
        if lower.contains("8b") || lower.contains("qwen2.5:7b") { return 60 }
        if lower.contains("7b") { return 50 }
        if lower.contains("3b") || lower.contains("qwen2.5:3b") || lower.contains("1.5b") { return 40 }
        if lower.contains("llama3.2:latest") { return 45 }
        if lower.contains("llama3") || lower.contains("llama-3") { return 65 }
        if lower.contains("mistral") || lower.contains("mixtral") { return 60 }
        if lower.contains("qwen") { return 55 }
        if lower.contains("gemma") { return 50 }
        if lower.contains("phi") || lower.contains("tinyllama") { return 35 }
        if lower.contains("deepseek") || lower.contains("codellama") || lower.contains("codegemma") { return 55 }
        30u32.saturating_sub(fallback as u32)
    }

    async fn check_server(url: &str) -> bool {
        reqwest::Client::new()
            .get(url)
            .timeout(std::time::Duration::from_secs(SERVER_TIMEOUT_SECONDS))
            .send()
            .await
            .is_ok()
    }

    async fn fetch_ollama_models() -> Result<Vec<String>> {
        let resp = reqwest::Client::new()
            .get(OLLAMA_TAGS_URL)
            .timeout(std::time::Duration::from_secs(OLLAMA_TIMEOUT_SECONDS))
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;
        let models = data["models"]
            .as_array()
            .map(|array| {
                array.iter()
                    .filter_map(|model| model["name"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(models)
    }
}
