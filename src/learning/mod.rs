pub mod agent_loop;
pub mod skills;
pub mod trainer;
pub mod messaging;

pub use agent_loop::{UmbraAgent, AgentEvent};
pub use skills::SkillManager;
pub use trainer::TrainerEngine;
pub use messaging::MessagingGateway;

use std::sync::Arc;
use anyhow::Result;
use tokio::sync::mpsc;

use crate::engine::MateriaCore;
use crate::security::SecurityGate;
use crate::persona::JarvisPersona;
use std::path::PathBuf;

#[derive(Clone)]
pub struct AgentEngine {
    pub agent: UmbraAgent,
    pub skills: SkillManager,
    pub trainer: TrainerEngine,
    pub messaging: MessagingGateway,
}

impl AgentEngine {
    pub fn new(engine: Arc<MateriaCore>, security: Arc<SecurityGate>, persona: JarvisPersona, models_dir: PathBuf) -> Self {
        Self {
            agent: UmbraAgent::new(engine, security, persona),
            skills: SkillManager::new(),
            trainer: TrainerEngine::new(models_dir),
            messaging: MessagingGateway::new(),
        }
    }

    pub async fn run(&self, input: String, tx: mpsc::Sender<AgentEvent>) -> Result<String> {
        self.agent.run(input, tx).await
    }
}
