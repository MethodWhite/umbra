// Zone 0 — Config/Init
pub mod config;
pub mod desktop;
pub mod sphere;
pub mod agent_memory;
pub mod agent_personality;
pub mod vault;
pub mod memory;
pub mod providers;
pub mod audio;
pub mod rate_limiter;
pub mod job_queue;
pub mod security;
pub mod ai_client;
pub mod domain;
pub mod application;

// Server-only: CLI API server
#[cfg(feature = "server")]
pub mod infrastructure;
#[cfg(feature = "server")]
pub mod engine;
#[cfg(feature = "server")]
pub mod bridge;
#[cfg(feature = "server")]
pub mod infra;
#[cfg(feature = "server")]
pub mod persona;
#[cfg(feature = "server")]
pub mod jarvis;
#[cfg(feature = "server")]
pub mod ironclaw;
#[cfg(feature = "server")]
pub mod api;
#[cfg(feature = "server")]
pub mod learning;
#[cfg(feature = "server")]
pub mod sub_agents;
#[cfg(feature = "server")]
pub mod resource;
#[cfg(feature = "server")]
pub mod debugger;
#[cfg(feature = "server")]
pub mod frontend;
#[cfg(feature = "server")]
pub mod cache;

#[cfg(feature = "server")]
pub use config::UmbraConfig;
#[cfg(feature = "server")]
pub use engine::MateriaCore;
#[cfg(feature = "server")]
pub use security::SecurityGate;
#[cfg(feature = "server")]
pub use learning::{AgentEngine, UmbraAgent, AgentEvent, trainer::{TrainingExample, TrainReport}};
#[cfg(feature = "server")]
pub use bridge::Mt4Bridge;
#[cfg(feature = "server")]
pub use persona::JarvisPersona;
#[cfg(feature = "server")]
pub use jarvis::{JarvisManager, bridge::JarvisApi};
#[cfg(feature = "server")]
pub use ironclaw::IronClaw;
#[cfg(feature = "server")]
pub use memory::MemoryEngine;
#[cfg(feature = "server")]
pub use audio::AudioEngine;
#[cfg(feature = "server")]
pub use sub_agents::SubAgentManager;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "UMBRA";

#[cfg(feature = "server")]
pub async fn init() -> anyhow::Result<UmbraApp> {
    tracing::info!("{} v{} inicializando", NAME, VERSION);

    let config = UmbraConfig::load();

    let engine = MateriaCore::new(config.paths.models_dir.clone()).await?;
    let engine = std::sync::Arc::new(engine);

    let security = SecurityGate::new();
    let security = std::sync::Arc::new(security);

    let ironclaw = IronClaw::new();
    let ironclaw = std::sync::Arc::new(ironclaw);

    let memory = MemoryEngine::new();
    let memory = std::sync::Arc::new(memory);

    let persona = JarvisPersona::new();
    let mut agent = AgentEngine::new(engine.clone(), security.clone(), persona, config.paths.models_dir.clone());
    agent.agent = agent.agent.with_memory(memory.clone());

    let mut sub_agents = SubAgentManager::new(config.paths.subagents_dir.clone());
    let _ = sub_agents.scan().await;

    let audio = AudioEngine::new(config.paths.models_dir.clone());

    let mut resource_manager = resource::ResourceManager::new();
    resource_manager.register("umbra-core", resource::ResourceKind::Model, 4096, 1);
    resource_manager.register("audio-pipeline", resource::ResourceKind::AudioPipeline, 256, 3);
    resource_manager.register("synapsis-cache", resource::ResourceKind::Cache, 512, 2);
    for sa in sub_agents.list() {
        resource_manager.register(&format!("sub-agent:{}", sa.name),
            resource::ResourceKind::SubAgent, 128, 4);
    }

    let jarvis = JarvisManager::new(config.paths.jarvis_dir.clone(), config.api.frontend_port);
    let jarvis = std::sync::Arc::new(jarvis);

    let infra = crate::infra::Infrastructure::new();

    let providers = providers::ProviderRegistry::load();

    let debugger = debugger::Debugger::new();
    debugger.run_background();

    debugger.report(debugger::Severity::Info, "startup",
        &format!("{} v{} iniciado con {} modulos", NAME, VERSION, 6),
        None);

    tracing::info!("{} v{} lista — {} sub-agentes, {} modelos locales, {} MB allocados, {} proveedores cloud",
        NAME, VERSION, sub_agents.list().len(), engine.models.local_models.len(),
        resource_manager.total_allocated_mb, providers.count_enabled());

    Ok(UmbraApp { config, engine, security, ironclaw, memory, agent, jarvis, audio, sub_agents, resource_manager, debugger, providers, infra })
}

#[cfg(feature = "server")]
pub struct UmbraApp {
    pub config: UmbraConfig,
    pub engine: std::sync::Arc<MateriaCore>,
    pub security: std::sync::Arc<SecurityGate>,
    pub ironclaw: std::sync::Arc<IronClaw>,
    pub memory: std::sync::Arc<MemoryEngine>,
    pub agent: AgentEngine,
    pub jarvis: std::sync::Arc<JarvisManager>,
    pub audio: AudioEngine,
    pub sub_agents: SubAgentManager,
    pub resource_manager: resource::ResourceManager,
    pub debugger: std::sync::Arc<debugger::Debugger>,
    pub providers: providers::ProviderRegistry,
    pub infra: crate::infra::Infrastructure,
}
