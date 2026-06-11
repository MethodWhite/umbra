use crate::domain::errors::AppError;

pub struct TriggerTrainingUseCase;

impl TriggerTrainingUseCase {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, trainer: &crate::learning::trainer::TrainerEngine) -> Result<serde_json::Value, AppError> {
        match trainer.auto_train("umbra-base").await {
            Ok(path) => Ok(serde_json::json!({ "status": "ok", "model": path })),
            Err(e) => Ok(serde_json::json!({ "status": "error", "error": e.to_string() })),
        }
    }
}
