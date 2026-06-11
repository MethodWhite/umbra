use anyhow::{Result, anyhow};
use std::time::Instant;

use super::zt_gate::ZeroTrustGate;
use super::antibrick::AntiBrick;

pub struct RuntimeEnforcer {
    zt_gate: ZeroTrustGate,
    antibrick: AntiBrick,
}

impl RuntimeEnforcer {
    pub fn new() -> Self {
        Self {
            zt_gate: ZeroTrustGate::new(),
            antibrick: AntiBrick::new(),
        }
    }

    pub async fn validate(&self, cmd: &str, args: &[String]) -> Result<bool> {
        let start = Instant::now();

        // 1. Zero-Trust: identidad + permiso + contexto
        let identity = self.zt_gate.check_identity()?;
        if !identity {
            return Err(anyhow!("[Thoth] Fallo verificación de identidad"));
        }

        let permission = self.zt_gate.check_permission(cmd)?;
        if !permission {
            return Err(anyhow!("[Thoth] Sin permiso para: {}", cmd));
        }

        // 2. AntiBrick: análisis de riesgo
        let safe = self.antibrick.check(cmd, args)?;
        if !safe {
            return Err(anyhow!("[Thoth] Comando bloqueado por AntiBrick: {}", cmd));
        }

        // 3. Contexto
        let coherent = self.zt_gate.check_context(cmd)?;
        if !coherent {
            return Err(anyhow!("[Thoth] Contexto inválido para: {}", cmd));
        }

        let elapsed = start.elapsed();
        tracing::debug!("[Thoth] Validación completada en {:?}", elapsed);

        if elapsed.as_micros() > 100_000 {
            tracing::warn!("[Thoth] Validación excedió 100ms: {:?}", elapsed);
        }

        Ok(true)
    }
}
