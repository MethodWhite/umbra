// Zone 3 — Application
use crate::domain::ports::tts_port::TtsPort;

pub struct SynthesizeSpeechUseCase;

impl SynthesizeSpeechUseCase {
    pub async fn execute(&self, text: &str, tone: &str, tts: &dyn TtsPort) -> Result<Vec<u8>, String> {
        tts.synthesize(text, tone)
    }
}
