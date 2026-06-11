// Zone 2 — Domain/Ports
pub trait TtsPort: Send + Sync {
    fn synthesize(&self, text: &str, voice: &str) -> Result<Vec<u8>, String>;
    fn available_voices(&self) -> Vec<String>;
}
