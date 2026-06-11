pub struct GetTrainingStatsUseCase;

impl GetTrainingStatsUseCase {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, trainer: &crate::learning::trainer::TrainerEngine) -> serde_json::Value {
        trainer.stats()
    }
}
