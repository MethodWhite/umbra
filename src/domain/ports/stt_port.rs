// Zone 2 — Domain/Ports
use crate::domain::models::voice::Transcription;

pub trait SttPort: Send + Sync {
    fn transcribe(&self, audio_data: &[u8], format: &str) -> Result<Transcription, String>;
    fn supported_formats(&self) -> Vec<String>;
}
