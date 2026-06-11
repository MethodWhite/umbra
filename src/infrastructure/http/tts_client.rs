use crate::domain::errors::AppError;
use futures::StreamExt;

const NVIDIA_FUNCTIONS: &[(&str, &str)] = &[
    ("magpie-tts-multilingual", "5624e4ce-ca44-407f-bf0e-2e76190c8ca4"),
    ("magpie-tts-zeroshot", "7285e78e-30dd-4353-b4b0-5f5f6cce14a5"),
];

fn nvidia_function_id(model_id: &str) -> Result<&'static str, AppError> {
    NVIDIA_FUNCTIONS.iter()
        .find(|(name, _)| *name == model_id)
        .map(|(_, id)| *id)
        .ok_or_else(|| AppError::Validation(format!("Unknown NVIDIA TTS model: {}", model_id)))
}

pub struct TtsClient;

impl TtsClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn synthesize_fish(&self, text: &str, api_key: &str, voice_id: &str) -> Result<Vec<u8>, AppError> {
        let fish_api_url = std::env::var("FISH_API_URL")
            .unwrap_or_else(|_| "https://api.fish.audio/v1/tts".into());
        let client = reqwest::Client::new();
        let resp = client
            .post(&fish_api_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "text": text,
                "reference_id": voice_id,
                "format": "mp3",
            }))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Fish TTS request failed: {}", e)))?;
        if resp.status().is_success() {
            let bytes = resp.bytes().await
                .map_err(|e| AppError::ExternalApi(format!("Fish TTS read failed: {}", e)))?;
            Ok(bytes.to_vec())
        } else {
            Err(AppError::ExternalApi(format!("Fish TTS error: HTTP {}", resp.status())))
        }
    }

    pub async fn synthesize_nvidia(&self, text: &str, api_key: &str, model_id: &str) -> Result<Vec<u8>, AppError> {
        let function_id = nvidia_function_id(model_id)?;
        let url = format!("https://api.nvcf.nvidia.com/v2/nvcf/pexec/functions/{}", function_id);
        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "text": text,
                "language": "en-US",
                "voice": "default",
            }))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("NVIDIA TTS request failed: {}", e)))?;
        if resp.status() == 200 {
            let bytes = resp.bytes().await
                .map_err(|e| AppError::ExternalApi(format!("NVIDIA TTS read failed: {}", e)))?;
            Ok(bytes.to_vec())
        } else {
            Err(AppError::ExternalApi(format!("NVIDIA TTS error: HTTP {}", resp.status())))
        }
    }

    pub async fn synthesize_nvidia_stream(
        &self,
        text: &str,
        api_key: &str,
        model_id: &str,
    ) -> Result<impl futures::Stream<Item = Result<Vec<u8>, AppError>>, AppError> {
        let function_id = nvidia_function_id(model_id)?;
        let url = format!("https://api.nvcf.nvidia.com/v2/nvcf/pexec/stream/functions/{}", function_id);
        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Accept", "audio/wav")
            .json(&serde_json::json!({
                "text": text,
                "language": "en-US",
                "voice": "default",
            }))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("NVIDIA TTS stream request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::ExternalApi(format!("NVIDIA TTS stream error: HTTP {}", resp.status())));
        }

        let stream = resp.bytes_stream().map(|chunk| {
            chunk
                .map(|b| b.to_vec())
                .map_err(|e| AppError::ExternalApi(format!("NVIDIA TTS stream read error: {}", e)))
        });

        Ok(stream)
    }

    pub async fn synthesize_nvidia_realtime(
        &self,
        text: &str,
        api_key: &str,
        model_id: &str,
    ) -> Result<impl futures::Stream<Item = Result<Vec<u8>, AppError>>, AppError> {
        let function_id = nvidia_function_id(model_id)?;
        let url = format!("https://api.nvcf.nvidia.com/v2/nvcf/pexec/stream/functions/{}", function_id);
        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Accept", "audio/wav")
            .header("X-NVCF-POLL-INTERVAL", "100")
            .json(&serde_json::json!({
                "text": text,
                "language": "en-US",
                "voice": "default",
                "encode": "wav",
                "sample_rate": 24000,
            }))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("NVIDIA real-time TTS request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::ExternalApi(format!("NVIDIA real-time TTS error: HTTP {}", resp.status())));
        }

        let stream = resp.bytes_stream().map(|chunk| {
            chunk
                .map(|b| b.to_vec())
                .map_err(|e| AppError::ExternalApi(format!("NVIDIA real-time TTS stream error: {}", e)))
        });

        Ok(stream)
    }

    pub async fn list_nvidia_models(&self, api_key: &str) -> Result<Vec<String>, AppError> {
        let client = reqwest::Client::new();
        let resp = client
            .get("https://api.nvcf.nvidia.com/v2/nvcf/functions")
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("NVIDIA list models failed: {}", e)))?;

        if resp.status().is_success() {
            let data: serde_json::Value = resp.json().await
                .map_err(|e| AppError::ExternalApi(format!("NVIDIA list models parse failed: {}", e)))?;
            let models = data["functions"].as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|f| f["name"].as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            Ok(models)
        } else {
            Err(AppError::ExternalApi(format!("NVIDIA list models error: HTTP {}", resp.status())))
        }
    }
}
