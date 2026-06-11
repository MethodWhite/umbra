use crate::domain::models::voice::Transcription;
use crate::domain::ports::stt_port::SttPort;

pub struct TranscribeSpeechUseCase;

impl TranscribeSpeechUseCase {
    pub async fn execute(&self, audio_data: &[u8], format: &str, stt: &dyn SttPort) -> Result<Transcription, String> {
        stt.transcribe(audio_data, format)
    }
}
