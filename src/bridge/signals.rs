// Zone 5 — Bridge/External
use anyhow::{Result, anyhow};
use super::types::Signal;
use std::sync::atomic::{AtomicU64, Ordering};

static SIGNAL_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct SignalPipeline {
    allowed_symbols: Vec<String>,
    _max_risk_percent: f64,
    min_confidence: f64,
}

impl SignalPipeline {
    pub fn new() -> Self {
        Self {
            allowed_symbols: vec![],
            _max_risk_percent: 2.0,
            min_confidence: 0.65,
        }
    }

    pub fn validate(&self, signal: &Signal) -> Result<Signal> {
        if signal.confidence < self.min_confidence {
            return Err(anyhow!("Confianza baja: {:.2} < {:.2}", signal.confidence, self.min_confidence));
        }
        if signal.volume <= 0.0 {
            return Err(anyhow!("Volumen inválido: {}", signal.volume));
        }
        if signal.symbol.is_empty() {
            return Err(anyhow!("Símbolo vacío"));
        }
        if !self.allowed_symbols.is_empty() && !self.allowed_symbols.contains(&signal.symbol) {
            return Err(anyhow!("Símbolo no permitido: {}", signal.symbol));
        }
        Ok(signal.clone())
    }

    pub fn generate_signal_id(&self) -> String {
        let n = SIGNAL_COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("umbra_sig_{:x}_{}", chrono::Utc::now().timestamp(), n)
    }
}
