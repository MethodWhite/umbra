use axum::{Json, extract::State, response::IntoResponse};

use crate::application::training::{TriggerTrainingUseCase, GetTrainingStatsUseCase};

use super::super::BackendRouterState;

pub async fn training_stats(
    State(state): State<BackendRouterState>,
) -> impl IntoResponse {
    let use_case = GetTrainingStatsUseCase::new();
    Json(use_case.execute(&state.agent.trainer))
}

#[derive(serde::Deserialize)]
pub struct IngestTrainingRequest {
    pub examples: Vec<crate::learning::trainer::TrainingExample>,
    pub source: String,
}

pub async fn ingest_training_data(
    State(state): State<BackendRouterState>,
    Json(req): Json<IngestTrainingRequest>,
) -> Json<serde_json::Value> {
    let count = state.agent.trainer.ingest_from_browser(req.examples, &req.source);
    Json(serde_json::json!({
        "status": "ok",
        "ingested": count,
    }))
}

pub async fn trigger_training(
    State(state): State<BackendRouterState>,
) -> Json<serde_json::Value> {
    let use_case = TriggerTrainingUseCase::new();
    match use_case.execute(&state.agent.trainer).await {
        Ok(val) => Json(val),
        Err(_) => Json(serde_json::json!({"status": "error", "error": "Training failed"})),
    }
}
