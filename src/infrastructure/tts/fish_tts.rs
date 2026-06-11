// Zone 4 — Infrastructure
use crate::domain::ports::tts_port::TtsPort;

pub struct FishTtsAdapter;

impl TtsPort for FishTtsAdapter {
    fn synthesize(&self, _text: &str, _voice: &str) -> Result<Vec<u8>, String> {
        todo!("Implement Fish TTS synthesis")
    }

    fn available_voices(&self) -> Vec<String> {
        todo!("Implement available voices")
    }
}
