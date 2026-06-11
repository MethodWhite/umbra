pub mod server;
pub mod auth;
pub mod routes;
pub mod middleware;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::domain::ports::VaultRepository;
use crate::infrastructure::repositories::EncryptedVaultRepository;

#[derive(Clone)]
pub struct FrontendRouterState {
    pub vault: Arc<Mutex<EncryptedVaultRepository>>,
    pub auth_token: String,
    pub backend_url: String,
    pub start_time: std::time::Instant,
    pub frontend_dir: PathBuf,
}

#[derive(Clone)]
pub struct BackendRouterState {
    pub agent: Arc<crate::learning::AgentEngine>,
    pub security: Arc<crate::security::SecurityGate>,
    pub ironclaw: Arc<crate::ironclaw::IronClaw>,
    pub memory: Arc<crate::memory::MemoryEngine>,
    pub audio: crate::audio::AudioEngine,
    pub sub_agents: crate::sub_agents::SubAgentManager,
    pub resources: Arc<Mutex<crate::resource::ResourceManager>>,
    pub debugger: Arc<crate::debugger::Debugger>,
    pub auth_token: String,
    pub start_time: std::time::Instant,
    pub models_dir: PathBuf,
}

impl BackendRouterState {
    pub fn engine(&self) -> &crate::engine::MateriaCore {
        &self.agent.agent.engine
    }
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub conversation_id: Option<String>,
    pub context: Option<ContextSnapshot>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub conversation_id: String,
    pub actions: Vec<ExecutedAction>,
    pub tokens_used: TokenUsage,
}

#[derive(Debug, Deserialize)]
pub struct ContextSnapshot {
    pub screen: Option<String>,
    pub calendar: Option<String>,
    pub mail: Option<String>,
    pub active_tasks: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExecutedAction {
    pub name: String,
    pub status: String,
    pub result: String,
}

#[derive(Debug, Serialize)]
pub struct TokenUsage {
    pub input: u64,
    pub output: u64,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub mode: String,
    pub active: bool,
    pub uptime_seconds: u64,
    pub memory_count: u64,
    pub task_count: u64,
    pub ironclaw: crate::ironclaw::ClawStatsSnapshot,
}

#[derive(Debug, Deserialize)]
pub struct ActionRequest {
    pub action: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ActionResponse {
    pub status: String,
    pub result: String,
    pub blocked_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JepaTrainRequest {
    pub name: Option<String>,
    pub input_dim: usize,
    pub latent_dim: usize,
    pub steps: Option<usize>,
    pub learning_rate: Option<f32>,
    pub seed: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct JepaPredictRequest {
    pub model: String,
    pub input: Vec<f32>,
}

#[derive(Debug, Deserialize)]
pub struct JepaConvertRequest {
    pub model: String,
    pub output_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HsaqAnalyzeRequest {
    pub layers: Vec<HsaqLayerInfo>,
}

#[derive(Debug, Deserialize)]
pub struct HsaqLayerInfo {
    pub name: String,
    pub size: u64,
    pub importance: f32,
}
