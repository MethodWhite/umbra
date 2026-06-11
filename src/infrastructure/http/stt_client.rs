/// Simple STT client that can use Whisper.cpp locally or API-based STT
use reqwest::Client;

pub struct SttClient {
    client: Client,
    api_key: String,
    use_local: bool,
}

impl SttClient {
    pub fn new(api_key: &str, use_local: bool) -> Self {
        SttClient {
            client: Client::new(),
            api_key: api_key.to_string(),
            use_local,
        }
    }

    /// Transcribe audio bytes to text
    pub async fn transcribe(&self, audio_data: &[u8]) -> Result<String, String> {
        if self.use_local {
            // Local whisper.cpp via HTTP API
            let resp = self.client.post("http://localhost:8080/v1/audio/transcriptions")
                .multipart(reqwest::multipart::Form::new()
                    .part("file", reqwest::multipart::Part::bytes(audio_data.to_vec())
                        .file_name("audio.wav")
                        .mime_str("audio/wav").unwrap())
                    .text("model", "whisper-1"))
                .send().await.map_err(|e| e.to_string())?;
            let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            Ok(data["text"].as_str().unwrap_or("").to_string())
        } else if !self.api_key.is_empty() {
            // OpenAI Whisper API
            let resp = self.client.post("https://api.openai.com/v1/audio/transcriptions")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .multipart(reqwest::multipart::Form::new()
                    .part("file", reqwest::multipart::Part::bytes(audio_data.to_vec())
                        .file_name("audio.wav")
                        .mime_str("audio/wav").unwrap())
                    .text("model", "whisper-1"))
                .send().await.map_err(|e| e.to_string())?;
            let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            Ok(data["text"].as_str().unwrap_or("").to_string())
        } else {
            Err("No STT method available".to_string())
        }
    }
}
