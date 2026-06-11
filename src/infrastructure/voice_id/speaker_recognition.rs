use crate::domain::models::security::Identity;
use crate::domain::ports::voice_id_port::VoiceIdPort;

pub struct SpeakerRecognitionAdapter;

impl VoiceIdPort for SpeakerRecognitionAdapter {
    fn enroll(&self, _audio_data: &[u8], _user_id: &str) -> Result<Identity, String> {
        todo!("Implement speaker enrollment")
    }

    fn verify(&self, _audio_data: &[u8], _user_id: &str) -> Result<bool, String> {
        todo!("Implement speaker verification")
    }

    fn similarity(&self, _audio_data: &[u8], _user_id: &str) -> Result<f32, String> {
        todo!("Implement speaker similarity")
    }
}
