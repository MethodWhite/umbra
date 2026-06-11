use crate::domain::errors::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct OllamaError {
    error: String,
}

pub struct OllamaClient {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaClient {
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:11434".into(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    pub fn with_url(url: &str) -> Self {
        Self {
            base_url: url.trim_end_matches('/').to_string(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    pub async fn check_available(&self) -> Result<serde_json::Value, AppError> {
        let resp = self.client.get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Ollama check failed: {}", e)))?;

        if resp.status() == 200 {
            let data = resp.json::<serde_json::Value>().await
                .map_err(|e| AppError::ExternalApi(format!("Ollama parse failed: {}", e)))?;
            Ok(data)
        } else {
            Err(AppError::ExternalApi(format!("Ollama HTTP {}", resp.status())))
        }
    }

    pub async fn chat_completion(&self, model: &str, messages: Vec<ChatMessage>) -> Result<String, AppError> {
        let body = ChatRequest {
            model: model.to_string(),
            messages,
            stream: false,
        };

        let resp = self.client.post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Ollama request failed: {}", e)))?;

        if resp.status() == 200 {
            let data = resp.json::<ChatResponse>().await
                .map_err(|e| AppError::ExternalApi(format!("Ollama parse failed: {}", e)))?;
            Ok(data.message.content)
        } else {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            if let Ok(err) = serde_json::from_str::<OllamaError>(&text) {
                Err(AppError::ExternalApi(format!("Ollama error ({}): {}", status, err.error)))
            } else {
                Err(AppError::ExternalApi(format!("Ollama HTTP {}: {}", status, text)))
            }
        }
    }
}
