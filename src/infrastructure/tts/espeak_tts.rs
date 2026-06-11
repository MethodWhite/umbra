use crate::domain::ports::tts_port::TtsPort;

pub struct EspeakTtsAdapter;

impl TtsPort for EspeakTtsAdapter {
    fn synthesize(&self, _text: &str, _voice: &str) -> Result<Vec<u8>, String> {
        todo!("Implement espeak TTS synthesis")
    }

    fn available_voices(&self) -> Vec<String> {
        todo!("Implement available voices")
    }
}
