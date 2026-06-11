// Zone 3 — Application
use crate::domain::ports::voice_id_port::VoiceIdPort;

pub struct VerifyIdentityUseCase;

impl VerifyIdentityUseCase {
    pub async fn execute(&self, audio_data: &[u8], user_id: &str, voice_id: &dyn VoiceIdPort) -> Result<bool, String> {
        voice_id.verify(audio_data, user_id)
    }
}
