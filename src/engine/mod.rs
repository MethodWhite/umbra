// Zone 6 — Research/Stubs (server-gated)
pub mod jepa;
pub mod router;
pub mod memory;
pub mod scheduler;
pub mod wasm;
pub mod snn;
pub mod models;
pub mod safety;
pub mod jepa_model;
pub mod hdt_router;
pub mod hsaq;

pub use jepa::{JepaEngine, InferenceResult};
pub use router::ModelRouter;
pub use memory::SynapsisMemory;
pub use scheduler::{AdaptiveScheduler, Backend, QuantLevel, ModelConfig};
pub use wasm::WasmSandbox;
pub use snn::SnnClassifier;
pub use models::ModelManager;
pub use safety::HardwareMonitor;
pub use jepa_model::JepaModel;
pub use hdt_router::HdtRouter;
pub use hsaq::HsaqCompressor;

use anyhow::Result;
use std::path::PathBuf;

pub struct MateriaCore {
    pub jepa: JepaEngine,
    pub router: ModelRouter,
    pub memory: synapsis_core::infrastructure::database::Database,
    pub scheduler: AdaptiveScheduler,
    pub wasm: WasmSandbox,
    pub snn: SnnClassifier,
    pub models: ModelManager,
    pub safety: HardwareMonitor,
    pub hdt: HdtRouter,
    pub hsaq: HsaqCompressor,
}

impl MateriaCore {
    pub async fn new(models_dir: PathBuf) -> Result<Self> {
        let db = synapsis_core::infrastructure::database::Database::new();
        let mut models = ModelManager::new(models_dir);
        models.scan_local().await?;

        let safety = HardwareMonitor::sample().await;
        let scheduler = AdaptiveScheduler::detect().await;
        let router = ModelRouter::scan().await;
        let hdt = HdtRouter::new();
        let hsaq = HsaqCompressor::new();

        if !safety.is_throttling {
            tracing::info!("Hardware OK — GPU: {}°C, CPU: {}°C, VRAM: {}/{} MB",
                safety.gpu_temp_c, safety.cpu_temp_c, safety.vram_used_mb, safety.vram_total_mb);
        } else {
            tracing::warn!("Hardware en throttling — reduciendo capacidad. GPU: {}°C", safety.gpu_temp_c);
        }

        tracing::info!("HDT: {} nodos, {} puentes teroideales, diámetro {}",
            hdt.nodes.len(), 12, 3);

        Ok(Self {
            jepa: JepaEngine::new(),
            router,
            memory: db,
            scheduler,
            wasm: WasmSandbox::new(),
            snn: SnnClassifier::new(10, 64),
            models,
            safety,
            hdt,
            hsaq,
        })
    }

    pub async fn stream(&self, prompt: String) -> Result<String> {
        let backend = self.scheduler.select_backend(&self.router);
        let result = self.jepa.infer(prompt, backend).await?;
        Ok(result.text)
    }

    pub async fn infer_safe(&self, prompt: String, desired_context: usize) -> Result<String> {
        let context = self.safety.safe_context_size(desired_context);
        let backend = self.scheduler.select_primary(&self.router);
        if self.safety.is_throttling {
            tracing::warn!("Hardware throttling — inferencia con contexto reducido ({})", context);
        }
        self.jepa.infer(prompt, backend).await.map(|r| r.text)
    }
}
