use anyhow::{Context, Result, anyhow};
use serde_json::json;
use super::scheduler::Backend;

const DEFAULT_TEMPERATURE: f64 = 0.7;
const DEFAULT_TOP_P: f64 = 0.9;
const DEFAULT_REPEAT_PENALTY: f64 = 1.1;
const DEFAULT_MAX_TOKENS: u32 = 2048;
const API_TIMEOUT_SECONDS: u64 = 120;

#[derive(Debug, Clone)]
pub struct InferenceResult {
    pub text: String,
    pub backend: String,
    pub latency_ms: u64,
    pub tokens: u32,
}

pub struct JepaEngine;

impl JepaEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn infer(
        &self,
        prompt: String,
        backend: Backend,
    ) -> Result<InferenceResult> {
        match backend {
            Backend::LlamaCppServer { url } => {
                Self::stream_llamacpp(&prompt, &url).await
            }
            Backend::Ollama { model } => {
                Self::stream_ollama(&prompt, &model).await
            }
            Backend::Cloud { provider, model } => {
                Self::stream_cloud(&prompt, &provider, &model).await
            }
            Backend::OpenCodeGo { model, api_key } => {
                let messages = vec![
                    serde_json::json!({"role": "user", "content": prompt}),
                ];
                let inference_client = reqwest::Client::new();
                let request_body = serde_json::json!({
                    "model": model,
                    "messages": messages,
                    "stream": false,
                });
                let response = inference_client
                    .post(format!("{}/chat/completions", crate::engine::scheduler::OPENCODE_GO_BASE_URL))
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Content-Type", "application/json")
                    .json(&request_body)
                    .timeout(std::time::Duration::from_secs(API_TIMEOUT_SECONDS))
                    .send()
                    .await
                    .context("OpenCode Go API request failed")?;
                let response_data: serde_json::Value = response.json().await?;
                let text = response_data["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                Ok(InferenceResult { text, backend: "opencode-go".into(), latency_ms: 0, tokens: 0 })
            }
            Backend::Unavailable => {
                Err(anyhow!("No inference backend available. Start llama.cpp or Ollama."))
            }
        }
    }

    async fn stream_llamacpp(prompt: &str, url: &str) -> Result<InferenceResult> {
        let start = std::time::Instant::now();
        let llama_client = reqwest::Client::new();
        let request_body = json!({
            "prompt": prompt,
            "n_predict": DEFAULT_MAX_TOKENS,
            "stream": false,
            "temperature": DEFAULT_TEMPERATURE,
            "top_p": DEFAULT_TOP_P,
            "repeat_penalty": DEFAULT_REPEAT_PENALTY,
        });

        let response = llama_client
            .post(format!("{}/completion", url))
            .json(&request_body)
            .send()
            .await
            .map_err(|error| anyhow!("Error connecting to llama.cpp: {}", error))?;

        let response_data: serde_json::Value = response
            .json()
            .await
            .map_err(|error| anyhow!("Error parsing llama.cpp response: {}", error))?;

        let text = response_data["content"].as_str().unwrap_or("").to_string();
        let elapsed = start.elapsed().as_millis() as u64;

        Ok(InferenceResult {
            text,
            backend: format!("llama.cpp @ {}", url),
            latency_ms: elapsed,
            tokens: response_data["tokens_predicted"].as_u64().unwrap_or(0) as u32,
        })
    }

    async fn stream_ollama(prompt: &str, model: &str) -> Result<InferenceResult> {
        let start = std::time::Instant::now();
        let ollama_client = reqwest::Client::new();
        let request_body = json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": DEFAULT_TEMPERATURE,
                "top_p": DEFAULT_TOP_P,
            }
        });

        let response = ollama_client
            .post("http://localhost:11434/api/generate")
            .json(&request_body)
            .send()
            .await
            .map_err(|error| anyhow!("Error connecting to Ollama: {}", error))?;

        let response_data: serde_json::Value = response
            .json()
            .await
            .map_err(|error| anyhow!("Error parsing Ollama response: {}", error))?;

        let text = response_data["response"].as_str().unwrap_or("").to_string();
        let elapsed = start.elapsed().as_millis() as u64;

        Ok(InferenceResult {
            text,
            backend: format!("Ollama/{}", model),
            latency_ms: elapsed,
            tokens: response_data["eval_count"].as_u64().unwrap_or(0) as u32,
        })
    }

    async fn stream_cloud(prompt: &str, provider: &str, model: &str) -> Result<InferenceResult> {
        let start = std::time::Instant::now();
        let api_url = match provider {
            "openai" => "https://api.openai.com/v1/chat/completions",
            "openrouter" => "https://openrouter.ai/api/v1/chat/completions",
            _ => return Err(anyhow!("Cloud provider not supported: {}", provider)),
        };

        let cloud_client = reqwest::Client::new();
        let request_body = json!({
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": DEFAULT_TEMPERATURE,
        });

        let response = cloud_client
            .post(api_url)
            .json(&request_body)
            .send()
            .await
            .map_err(|error| anyhow!("Error connecting to {}: {}", provider, error))?;

        let response_data: serde_json::Value = response
            .json()
            .await
            .map_err(|error| anyhow!("Error parsing {} response: {}", provider, error))?;

        let text = response_data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let elapsed = start.elapsed().as_millis() as u64;

        Ok(InferenceResult {
            text,
            backend: format!("{}/{}", provider, model),
            latency_ms: elapsed,
            tokens: 0,
        })
    }
}
