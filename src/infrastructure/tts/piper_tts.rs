use crate::domain::ports::tts_port::TtsPort;

pub struct PiperTtsAdapter;

impl TtsPort for PiperTtsAdapter {
    fn synthesize(&self, _text: &str, _voice: &str) -> Result<Vec<u8>, String> {
        todo!("Implement Piper TTS synthesis")
    }

    fn available_voices(&self) -> Vec<String> {
        todo!("Implement available voices")
    }
}
