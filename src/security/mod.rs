// Zone 5 — Bridge/External
pub mod enforcer;
pub mod pqc;
pub mod zt_gate;
pub mod antibrick;
pub mod audit;
pub mod ssrf;

pub use enforcer::RuntimeEnforcer;
pub use pqc::CryptoEngine;
pub use zt_gate::ZeroTrustGate;
pub use antibrick::AntiBrick;
pub use audit::AuditWorm;

use anyhow::Result;
use tokio::sync::Mutex;
use std::sync::Arc;

pub struct SecurityGate {
    pub enforcer: Arc<RuntimeEnforcer>,
    pub crypto: Arc<Mutex<CryptoEngine>>,
    pub zt_gate: Arc<ZeroTrustGate>,
    pub antibrick: Arc<AntiBrick>,
    pub audit: Arc<AuditWorm>,
}

impl SecurityGate {
    pub fn new() -> Self {
        Self {
            enforcer: Arc::new(RuntimeEnforcer::new()),
            crypto: Arc::new(Mutex::new(CryptoEngine::new())),
            zt_gate: Arc::new(ZeroTrustGate::new()),
            antibrick: Arc::new(AntiBrick::new()),
            audit: Arc::new(AuditWorm::new()),
        }
    }

    pub async fn validate_tool_call(&self, cmd: &str, args: &[String]) -> Result<bool> {
        let result = self.enforcer.validate(cmd, args).await?;

        let _ = self.audit.log(
            if result { "INFO" } else { "BLOCKED" },
            "security",
            &format!("tool_call: {} {:?} -> {}", cmd, args, result),
        );

        Ok(result)
    }
}
