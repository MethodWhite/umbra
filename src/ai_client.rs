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

#[derive(Debug)]
pub enum AiError {
    Request(String),
    Http(u16, String),
    Parse(String),
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiError::Request(e) => write!(f, "Ollama request failed: {}", e),
            AiError::Http(code, text) => write!(f, "Ollama HTTP {}: {}", code, text),
            AiError::Parse(e) => write!(f, "Ollama parse failed: {}", e),
        }
    }
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

    pub async fn check_available(&self) -> Result<serde_json::Value, AiError> {
        let resp = self.client.get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| AiError::Request(format!("Ollama check failed: {}", e)))?;

        if resp.status() == 200 {
            let data = resp.json::<serde_json::Value>().await
                .map_err(|e| AiError::Parse(format!("Ollama parse failed: {}", e)))?;
            Ok(data)
        } else {
            Err(AiError::Http(resp.status().as_u16(), "non-200".into()))
        }
    }

    pub async fn chat_completion(&self, model: &str, messages: Vec<ChatMessage>) -> Result<String, AiError> {
        let body = ChatRequest {
            model: model.to_string(),
            messages,
            stream: false,
        };

        let resp = self.client.post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::Request(format!("Ollama request failed: {}", e)))?;

        if resp.status() == 200 {
            let data = resp.json::<ChatResponse>().await
                .map_err(|e| AiError::Parse(format!("Ollama parse failed: {}", e)))?;
            Ok(data.message.content)
        } else {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            if let Ok(err) = serde_json::from_str::<OllamaError>(&text) {
                Err(AiError::Http(status, err.error))
            } else {
                Err(AiError::Http(status, text))
            }
        }
    }
}

// ── STT client (whisper.cpp / OpenAI Whisper) ───────────────────────────────

pub struct SttClient {
    base_url: String,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl SttClient {
    pub fn new_local() -> Self {
        Self {
            base_url: "http://localhost:8080".into(),
            api_key: None,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build().expect("Failed to build HTTP client"),
        }
    }

    pub fn new_openai(api_key: String) -> Self {
        Self {
            base_url: "https://api.openai.com/v1".into(),
            api_key: Some(api_key),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build().expect("Failed to build HTTP client"),
        }
    }

    /// Transcribe audio bytes to text using whisper.cpp server API
    pub async fn transcribe(&self, audio_data: &[u8]) -> Result<String, AiError> {
        if let Some(key) = &self.api_key {
            self.transcribe_openai(audio_data, key).await
        } else {
            self.transcribe_local(audio_data).await
        }
    }

    async fn transcribe_local(&self, audio_data: &[u8]) -> Result<String, AiError> {
        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(audio_data.to_vec())
                .file_name("audio.wav")
                .mime_str("audio/wav").unwrap())
            .text("model", "ggml-base.en")
            .text("response_format", "text");

        let resp = self.client.post(format!("{}/inference", self.base_url))
            .multipart(form)
            .send().await
            .map_err(|e| AiError::Request(format!("STT request failed: {}", e)))?;

        if resp.status() == 200 {
            let text = resp.text().await
                .map_err(|e| AiError::Parse(format!("STT parse failed: {}", e)))?;
            Ok(text.trim().to_string())
        } else {
            Err(AiError::Http(resp.status().as_u16(), resp.text().await.unwrap_or_default()))
        }
    }

    async fn transcribe_openai(&self, audio_data: &[u8], api_key: &str) -> Result<String, AiError> {
        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(audio_data.to_vec())
                .file_name("audio.wav")
                .mime_str("audio/wav").unwrap())
            .text("model", "whisper-1");

        let resp = self.client.post(format!("{}/audio/transcriptions", self.base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .multipart(form)
            .send().await
            .map_err(|e| AiError::Request(format!("OpenAI STT failed: {}", e)))?;

        if resp.status() == 200 {
            #[derive(Deserialize)]
            struct WhisperResponse { text: String }
            let data = resp.json::<WhisperResponse>().await
                .map_err(|e| AiError::Parse(format!("STT parse failed: {}", e)))?;
            Ok(data.text.trim().to_string())
        } else {
            Err(AiError::Http(resp.status().as_u16(), resp.text().await.unwrap_or_default()))
        }
    }
}
