// Zone 3 — Application
use crate::domain::models::ApiProvider;

pub struct TestConnectionUseCase;

impl TestConnectionUseCase {
    pub fn new() -> Self {
        Self
    }

    pub async fn test(
        &self,
        provider: &ApiProvider,
        api_key: &str,
        base_url: Option<&str>,
    ) -> serde_json::Value {
        let url = base_url.unwrap_or(provider.base_url);
        self.test_connection(provider.api_type, url, api_key).await
    }

    async fn test_connection(&self, api_type: &str, base_url: &str, api_key: &str) -> serde_json::Value {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .ok();
        let client = match client {
            Some(c) => c,
            None => return serde_json::json!({"valid": false, "error": "Failed to create HTTP client"}),
        };

        match api_type {
            "openai" => {
                let resp = client
                    .get(format!("{}/models", base_url.trim_end_matches('/')))
                    .header("Authorization", format!("Bearer {}", api_key))
                    .send().await;
                match resp {
                    Ok(r) if r.status() == 200 => {
                        match r.json::<serde_json::Value>().await {
                            Ok(data) => {
                                let count = data["data"].as_array().map(|a| a.len()).unwrap_or(0);
                                serde_json::json!({"valid": true, "models_available": count})
                            }
                            Err(_) => serde_json::json!({"valid": true}),
                        }
                    }
                    Ok(r) if r.status() == 401 => serde_json::json!({"valid": false, "error": "Invalid API key"}),
                    Ok(r) => serde_json::json!({"valid": false, "error": format!("HTTP {}", r.status())}),
                    Err(e) => serde_json::json!({"valid": false, "error": e.to_string()}),
                }
            }
            "anthropic" => {
                let resp = client
                    .post(format!("{}/messages", base_url.trim_end_matches('/')))
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .json(&serde_json::json!({
                        "model": "claude-3-haiku-20240307",
                        "max_tokens": 10,
                        "messages": [{"role": "user", "content": "ping"}],
                    }))
                    .send().await;
                match resp {
                    Ok(r) => serde_json::json!({"valid": r.status() == 200}),
                    Err(e) => serde_json::json!({"valid": false, "error": e.to_string()}),
                }
            }
            "google" => {
                let resp = client
                    .get(format!("{}/models", base_url.trim_end_matches('/')))
                    .header("x-goog-api-key", api_key)
                    .send().await;
                match resp {
                    Ok(r) => serde_json::json!({"valid": r.status() == 200}),
                    Err(e) => serde_json::json!({"valid": false, "error": e.to_string()}),
                }
            }
            "baidu" => {
                let resp = client
                    .post("https://aip.baidubce.com/oauth/2.0/token")
                    .query(&[("grant_type", "client_credentials"), ("client_id", api_key), ("client_secret", base_url)])
                    .send().await;
                match resp {
                    Ok(r) => serde_json::json!({"valid": r.status() == 200}),
                    Err(e) => serde_json::json!({"valid": false, "error": e.to_string()}),
                }
            }
            _ => serde_json::json!({"valid": false, "error": format!("Unsupported API type: {}", api_type)}),
        }
    }
}
