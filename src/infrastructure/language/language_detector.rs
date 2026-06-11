use crate::domain::ports::language_port::{LanguagePort, LanguageResult};

pub struct LanguageDetectorAdapter;

impl LanguagePort for LanguageDetectorAdapter {
    fn detect(&self, _text: &str) -> Result<LanguageResult, String> {
        todo!("Implement language detection")
    }

    fn detect_from_audio(&self, _audio_data: &[u8], _format: &str) -> Result<LanguageResult, String> {
        todo!("Implement language detection from audio")
    }

    fn supported_languages(&self) -> Vec<String> {
        vec!["en".into(), "es".into(), "fr".into(), "de".into(), "pt".into()]
    }
}
