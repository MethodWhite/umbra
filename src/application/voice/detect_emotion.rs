use crate::domain::models::emotion::EmotionalState;

pub struct DetectEmotionUseCase;

impl DetectEmotionUseCase {
    pub async fn execute(&self, text: &str, tone: &str) -> Result<EmotionalState, String> {
        todo!("Implement emotion detection from voice/text")
    }
}
