use crate::domain::models::emotion::EmotionalState;

pub struct DetectEmotionUseCase;

impl DetectEmotionUseCase {
    pub async fn execute(&self, _text: &str, _tone: &str) -> Result<EmotionalState, String> {
        todo!("Implement emotion detection from voice/text")
    }
}
