pub struct LanguageResult {
    pub language: String,
    pub confidence: f32,
}

pub trait LanguagePort: Send + Sync {
    fn detect(&self, text: &str) -> Result<LanguageResult, String>;
    fn detect_from_audio(&self, audio_data: &[u8], format: &str) -> Result<LanguageResult, String>;
    fn supported_languages(&self) -> Vec<String>;
}
