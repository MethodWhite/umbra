// Zone 1 — Desktop/UI
pub mod input;
pub mod playback;
pub mod vad;

use anyhow::{Result, anyhow};
use std::path::PathBuf;

fn get_fish_api_key() -> Option<String> {
    if let Ok(key) = std::env::var("FISH_API_KEY") {
        if !key.is_empty() { return Some(key); }
    }
    let url = "http://127.0.0.1:8340/api/internal/key/fish";
    if let Ok(resp) = reqwest::blocking::get(url) {
        if resp.status().is_success() {
            if let Ok(json) = resp.json::<serde_json::Value>() {
                if let Some(key) = json.get("key").and_then(|v| v.as_str()) {
                    if !key.is_empty() { return Some(key.to_string()); }
                }
            }
        }
    }
    None
}

#[derive(Clone)]
pub struct AudioEngine {
    pub models_dir: PathBuf,
    pub whisper_model: String,
    pub piper_voice: String,
    pub use_local: bool,
}

impl AudioEngine {
    pub fn new(models_dir: PathBuf) -> Self {
        Self {
            models_dir,
            whisper_model: "tiny".into(),
            piper_voice: "en_US-lessac-medium".into(),
            use_local: get_fish_api_key().map_or(true, |k| k.is_empty()),
        }
    }

    pub fn set_whisper_model(&mut self, model: &str) {
        self.whisper_model = model.to_string();
    }

    pub fn set_piper_voice(&mut self, voice: &str) {
        self.piper_voice = voice.to_string();
    }

    pub async fn transcribe(&self, audio_path: &str) -> Result<String> {
        if self.use_local {
            self.transcribe_local(audio_path).await
        } else {
            self.transcribe_api(audio_path).await
        }
    }

    async fn transcribe_local(&self, audio_path: &str) -> Result<String> {
        let model_path = self.models_dir.join(format!("ggml-{}.en.bin", self.whisper_model));

        if !model_path.exists() {
            return Err(anyhow!("Modelo whisper no encontrado. Usa: umbra models download whisper:{}", self.whisper_model));
        }

        let output = std::process::Command::new("whisper")
            .args(["--model", model_path.to_str().unwrap_or("tiny"), audio_path, "--output-txt"])
            .output()
            .map_err(|e| anyhow!("whisper.cpp no instalado: {}", e))?;

        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(text.trim().to_string())
        } else {
            Err(anyhow!("whisper error: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }

    async fn transcribe_api(&self, audio_path: &str) -> Result<String> {
        let api_key = get_fish_api_key()
            .ok_or_else(|| anyhow!("FISH_API_KEY no configurada"))?;

        let client = reqwest::Client::new();
        let audio_bytes = tokio::fs::read(audio_path).await?;

        let form = reqwest::multipart::Form::new()
            .part("audio", reqwest::multipart::Part::bytes(audio_bytes)
                .file_name("audio.wav")
                .mime_str("audio/wav")?);

        let resp = client
            .post("https://api.fish.audio/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", api_key))
            .multipart(form)
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;
        data["text"].as_str()
            .map(String::from)
            .ok_or_else(|| anyhow!("Error en transcripción API"))
    }

    pub async fn synthesize(&self, text: &str) -> Result<Vec<u8>> {
        if self.use_local {
            self.synthesize_local(text).await
        } else {
            self.synthesize_api(text).await
        }
    }

    async fn synthesize_local(&self, text: &str) -> Result<Vec<u8>> {
        let voice_path = self.models_dir.join(format!("{}.onnx", self.piper_voice));

        if !voice_path.exists() {
            return Err(anyhow!("Voz Piper no encontrada. Usa: umbra models download piper:{}", self.piper_voice));
        }

        let mut child = std::process::Command::new("piper")
            .args([
                "--model", voice_path.to_str().unwrap_or(""),
                "--output-raw",
            ])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| anyhow!("Piper TTS no instalado: {}", e))?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(text.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        if output.status.success() {
            Ok(output.stdout)
        } else {
            Err(anyhow!("Piper TTS error"))
        }
    }

    async fn synthesize_api(&self, text: &str) -> Result<Vec<u8>> {
        let api_key = get_fish_api_key()
            .ok_or_else(|| anyhow!("FISH_API_KEY no configurada"))?;
        let voice_id = std::env::var("FISH_VOICE_ID")
            .unwrap_or_else(|_| "612b878b113047d9a770c069c8b4fdfe".into());

        let client = reqwest::Client::new();
        let resp = client
            .post("https://api.fish.audio/v1/tts")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "text": text,
                "reference_id": voice_id,
                "format": "mp3",
            }))
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(resp.bytes().await?.to_vec())
        } else {
            Err(anyhow!("TTS API error: {}", resp.status()))
        }
    }

    pub fn is_local_available(&self) -> bool {
        let whisper = self.models_dir.join(format!("ggml-{}.en.bin", self.whisper_model));
        let piper = self.models_dir.join(format!("{}.onnx", self.piper_voice));
        whisper.exists() || piper.exists()
    }
}
