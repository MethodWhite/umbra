use crate::domain::errors::AppError;

pub struct OllamaClient;

impl OllamaClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn check_available(&self) -> Result<serde_json::Value, AppError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to build HTTP client: {}", e)))?;

        let resp = client.get("http://localhost:11434/api/tags")
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
}
