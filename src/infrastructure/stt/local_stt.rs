use crate::domain::models::voice::Transcription;
use crate::domain::ports::stt_port::SttPort;

pub struct LocalSttAdapter;

impl SttPort for LocalSttAdapter {
    fn transcribe(&self, audio_data: &[u8], format: &str) -> Result<Transcription, String> {
        todo!("Implement local STT")
    }

    fn supported_formats(&self) -> Vec<String> {
        vec!["wav".into()]
    }
}
