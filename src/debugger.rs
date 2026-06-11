use serde::Serialize;
use std::collections::{BTreeMap, VecDeque};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const MAX_DIAGNOSTICS: usize = 256;
const CLEANUP_INTERVAL_SECS: u64 = 60;
const LATENCY_WINDOW: usize = 100;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARN"),
            Severity::Error => write!(f, "ERROR"),
            Severity::Critical => write!(f, "CRIT"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub id: u64,
    pub severity: Severity,
    pub category: String,
    pub message: String,
    pub source: String,
    #[serde(skip)]
    pub timestamp: Instant,
    pub suggestion: Option<String>,
    pub context: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DebuggerSnapshot {
    pub uptime_secs: u64,
    pub total_diagnostics: usize,
    pub active_warnings: usize,
    pub active_errors: usize,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub error_rate: f64,
    pub categories: BTreeMap<String, usize>,
    pub recent_issues: Vec<String>,
    pub health: String,
}

#[derive(Debug, Clone)]
struct LatencySample {
    value: Duration,
    _endpoint: String,
}

pub struct Debugger {
    inner: Mutex<DebuggerInner>,
    _active: AtomicBool,
}

struct DebuggerInner {
    diagnostics: VecDeque<Diagnostic>,
    latencies: VecDeque<LatencySample>,
    next_id: u64,
    startup: Instant,
    total_requests: usize,
    error_count: usize,
}

impl Debugger {
    pub fn new() -> Arc<Self> {
        let d = Arc::new(Debugger {
            inner: Mutex::new(DebuggerInner {
                diagnostics: VecDeque::with_capacity(MAX_DIAGNOSTICS),
                latencies: VecDeque::with_capacity(LATENCY_WINDOW),
                next_id: 1,
                startup: Instant::now(),
                total_requests: 0,
                error_count: 0,
            }),
            _active: AtomicBool::new(true),
        });
        d.report(Severity::Info, "debugger", "Debugger iniciado", Some("Sistema de linting y diagnostico activo"));
        d
    }

    pub fn report(self: &Arc<Self>, severity: Severity, category: &str, message: &str, suggestion: Option<&str>) -> u64 {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let id = inner.next_id;
        inner.next_id += 1;
        let diag = Diagnostic {
            id,
            severity,
            category: category.to_string(),
            message: message.to_string(),
            source: "debugger".to_string(),
            timestamp: Instant::now(),
            suggestion: suggestion.map(|s| s.to_string()),
            context: BTreeMap::new(),
        };
        if inner.diagnostics.len() >= MAX_DIAGNOSTICS {
            inner.diagnostics.pop_front();
        }
        inner.diagnostics.push_back(diag);
        id
    }

    pub fn record_latency(self: &Arc<Self>, endpoint: &str, duration: Duration) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.total_requests += 1;
        if inner.latencies.len() >= LATENCY_WINDOW {
            inner.latencies.pop_front();
        }
        inner.latencies.push_back(LatencySample {
            value: duration,
            _endpoint: endpoint.to_string(),
        });
        if duration > Duration::from_secs(10) {
            self.report(
                Severity::Warning,
                "latency",
                &format!("Latencia alta en {}: {:.1}s", endpoint, duration.as_secs_f64()),
                Some("Considere reducir el tamano del contexto o usar un modelo mas rapido"),
            );
        }
    }

    pub fn record_error(self: &Arc<Self>, category: &str, error: &str) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.error_count += 1;
        drop(inner);
        self.report(Severity::Error, category, error, Some("Revise logs para mas detalles"));
    }

    pub fn lint_response(self: &Arc<Self>, response: &str, endpoint: &str) {
        let min_len = 10;
        if response.len() < min_len {
            self.report(
                Severity::Warning,
                "quality",
                &format!("Respuesta muy corta ({} chars < {}) de {}", response.len(), min_len, endpoint),
                Some("El agente debe generar respuestas sustanciales"),
            );
        }
        let indicators = ["error", "undefined", "null", "None", "failed", "Exception"];
        for indicator in &indicators {
            if response.contains(indicator) {
                self.report(
                    Severity::Warning,
                    "coherence",
                    &format!("Posible incoherencia: '{}' en respuesta de {}", indicator, endpoint),
                    Some("Revise la logica del agente para este caso"),
                );
                break;
            }
        }
    }

    pub fn snapshot(self: &Arc<Self>) -> DebuggerSnapshot {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();
        let active_warnings = inner.diagnostics.iter().filter(|d| matches!(d.severity, Severity::Warning)).count();
        let active_errors = inner.diagnostics.iter().filter(|d| matches!(d.severity, Severity::Error | Severity::Critical)).count();
        let avg_latency_ms = if !inner.latencies.is_empty() {
            let sum: Duration = inner.latencies.iter().map(|s| s.value).sum();
            sum.as_secs_f64() / inner.latencies.len() as f64 * 1000.0
        } else {
            0.0
        };
        let mut sorted: Vec<Duration> = inner.latencies.iter().map(|s| s.value).collect();
        sorted.sort();
        let p95 = if !sorted.is_empty() {
            let idx = (sorted.len() as f64 * 0.95) as usize;
            sorted[idx.min(sorted.len() - 1)].as_secs_f64() * 1000.0
        } else {
            0.0
        };
        let error_rate = if inner.total_requests > 0 {
            inner.error_count as f64 / inner.total_requests as f64
        } else {
            0.0
        };
        let mut categories = BTreeMap::new();
        for d in &inner.diagnostics {
            *categories.entry(d.category.clone()).or_insert(0) += 1;
        }
        let recent_issues: Vec<String> = inner.diagnostics.iter().rev().take(5).map(|d| d.message.clone()).collect();
        let health = if active_errors > 0 { "degraded" } else if active_warnings > 5 { "warning" } else { "healthy" };

        DebuggerSnapshot {
            uptime_secs: now.duration_since(inner.startup).as_secs(),
            total_diagnostics: inner.diagnostics.len(),
            active_warnings,
            active_errors,
            avg_latency_ms,
            p95_latency_ms: p95,
            error_rate,
            categories,
            recent_issues,
            health: health.to_string(),
        }
    }

    pub fn run_background(self: &Arc<Self>) {
        let debugger = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(CLEANUP_INTERVAL_SECS));
            loop {
                interval.tick().await;
                debugger.cleanup_old();
            }
        });
    }

    pub fn recent_diagnostics(self: &Arc<Self>, count: usize) -> Vec<Diagnostic> {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.diagnostics.iter().rev().take(count).cloned().collect()
    }

    fn cleanup_old(self: &Arc<Self>) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let cutoff = Instant::now() - Duration::from_secs(3600);
        inner.diagnostics.retain(|diag| {
            diag.severity == Severity::Error || diag.severity == Severity::Critical || diag.timestamp > cutoff
        });
    }
}
