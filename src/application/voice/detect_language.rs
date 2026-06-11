use crate::domain::ports::language_port::{LanguagePort, LanguageResult};

pub struct DetectLanguageUseCase;

impl DetectLanguageUseCase {
    pub async fn execute(&self, text: &str, detector: &dyn LanguagePort) -> Result<LanguageResult, String> {
        detector.detect(text)
    }
}
