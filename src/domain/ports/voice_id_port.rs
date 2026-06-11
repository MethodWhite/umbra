// Zone 2 — Domain/Ports
use crate::domain::models::security::Identity;

pub trait VoiceIdPort: Send + Sync {
    fn enroll(&self, audio_data: &[u8], user_id: &str) -> Result<Identity, String>;
    fn verify(&self, audio_data: &[u8], user_id: &str) -> Result<bool, String>;
    fn similarity(&self, audio_data: &[u8], user_id: &str) -> Result<f32, String>;
}
