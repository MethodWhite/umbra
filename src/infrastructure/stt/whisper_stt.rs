use crate::domain::models::voice::Transcription;
use crate::domain::ports::stt_port::SttPort;

pub struct WhisperSttAdapter;

impl SttPort for WhisperSttAdapter {
    fn transcribe(&self, audio_data: &[u8], format: &str) -> Result<Transcription, String> {
        todo!("Implement Whisper STT")
    }

    fn supported_formats(&self) -> Vec<String> {
        vec!["wav".into(), "mp3".into(), "ogg".into()]
    }
}
