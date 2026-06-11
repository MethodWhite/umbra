// Zone 6 — Research/Stubs (server-gated)
#[derive(Clone)]
pub struct VoiceEngine;

impl VoiceEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn speak(&self, _text: &str) {
        // TODO: integrar TTS local (piper-rs o similar)
    }

    pub fn listen(&self) -> Option<String> {
        // TODO: integrar STT local (whisper-rs)
        None
    }
}
