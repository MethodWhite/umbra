// Zone 6 — Research/Stubs (server-gated)
use anyhow::Result;
use std::collections::HashMap;
use std::time::Instant;
use std::sync::atomic::{AtomicBool, Ordering};

const IDLE_UNLOAD_SECS: u64 = 300;
const MEMORY_HIGH_WATERMARK: f32 = 0.85;
const MEMORY_CRITICAL_WATERMARK: f32 = 0.95;
const _MIN_FREE_MB: u64 = 1024;

#[derive(Debug, Clone)]
pub struct ResourceHandle {
    pub name: String,
    pub kind: ResourceKind,
    pub size_mb: u64,
    pub priority: u8,
    pub last_used: Instant,
    pub loaded: bool,
    pub can_unload: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResourceKind {
    Model,
    SubAgent,
    Cache,
    AudioPipeline,
    MemoryBuffer,
    JepaModel,
}

impl std::fmt::Display for ResourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Model => write!(f, "modelo"),
            Self::SubAgent => write!(f, "sub-agente"),
            Self::Cache => write!(f, "caché"),
            Self::AudioPipeline => write!(f, "audio"),
            Self::MemoryBuffer => write!(f, "buffer"),
            Self::JepaModel => write!(f, "jepa"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceManager {
    pub resources: HashMap<String, ResourceHandle>,
    pub total_allocated_mb: u64,
    pub auto_unload: bool,
    pub last_cleanup: Instant,
    cleanup_count: u64,
    memory_pressure: f32,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            total_allocated_mb: 0,
            auto_unload: true,
            last_cleanup: Instant::now(),
            cleanup_count: 0,
            memory_pressure: 0.0,
        }
    }

    pub fn register(&mut self, name: &str, kind: ResourceKind, size_mb: u64, priority: u8) {
        self.resources.insert(name.to_string(), ResourceHandle {
            name: name.to_string(),
            kind,
            size_mb,
            priority,
            last_used: Instant::now(),
            loaded: true,
            can_unload: true,
        });
        self.total_allocated_mb += size_mb;
        tracing::debug!("[ResourceManager] Registrado: {} ({} MB, prioridad {})", name, size_mb, priority);
    }

    pub fn use_resource(&mut self, name: &str) {
        if let Some(res) = self.resources.get_mut(name) {
            res.last_used = Instant::now();
        }
    }

    pub fn mark_unloadable(&mut self, name: &str, can_unload: bool) {
        if let Some(res) = self.resources.get_mut(name) {
            res.can_unload = can_unload;
        }
    }

    pub fn unload(&mut self, name: &str) -> Result<u64> {
        if let Some(res) = self.resources.get_mut(name) {
            if !res.can_unload {
                return Err(anyhow::anyhow!("{} es persistente", name));
            }
            if !res.loaded {
                return Ok(0);
            }
            res.loaded = false;
            self.total_allocated_mb = self.total_allocated_mb.saturating_sub(res.size_mb);
            tracing::info!("[ResourceManager] Descargado: {} ({} MB)", name, res.size_mb);
            self.cleanup_count += 1;
            Ok(res.size_mb)
        } else {
            Err(anyhow::anyhow!("Recurso '{}' no registrado", name))
        }
    }

    pub fn reload(&mut self, name: &str) -> Result<u64> {
        if let Some(res) = self.resources.get_mut(name) {
            if res.loaded { return Ok(0); }
            res.loaded = true;
            res.last_used = Instant::now();
            self.total_allocated_mb += res.size_mb;
            tracing::info!("[ResourceManager] Recargado: {} ({} MB)", name, res.size_mb);
            Ok(res.size_mb)
        } else {
            Err(anyhow::anyhow!("{} no registrado", name))
        }
    }

    pub fn pressure_level(&self) -> PressureLevel {
        if self.memory_pressure >= MEMORY_CRITICAL_WATERMARK {
            PressureLevel::Critical
        } else if self.memory_pressure >= MEMORY_HIGH_WATERMARK {
            PressureLevel::High
        } else {
            PressureLevel::Normal
        }
    }

    pub async fn sample_pressure(&mut self) {
        let hardware = crate::engine::HardwareMonitor::sample().await;
        let ram_pressure = if hardware.ram_total_mb > 0 {
            hardware.ram_used_mb as f32 / hardware.ram_total_mb as f32
        } else { 0.5 };
        let vram_pressure = if hardware.vram_total_mb > 0 {
            hardware.vram_used_mb as f32 / hardware.vram_total_mb as f32
        } else { 0.0 };
        self.memory_pressure = ram_pressure.max(vram_pressure);
    }

    pub async fn intelligent_cleanup(&mut self) -> CleanupReport {
        self.sample_pressure().await;
        let level = self.pressure_level();

        if level == PressureLevel::Normal {
            return CleanupReport { freed_mb: 0, unloaded: vec![], pressure: level };
        }

        let now = Instant::now();
        let idle_seconds = match level {
            PressureLevel::Critical => 10,
            PressureLevel::High => IDLE_UNLOAD_SECS / 2,
            PressureLevel::Normal => IDLE_UNLOAD_SECS,
        };

        let mut candidates: Vec<(String, u64, u8, Instant)> = self.resources.iter()
            .filter(|(_, r)| r.loaded && r.can_unload)
            .map(|(n, r)| (n.clone(), r.size_mb, r.priority, r.last_used))
            .collect();
        candidates.sort_by(|a, b| {
            let a_idle = now.duration_since(a.3).as_secs();
            let b_idle = now.duration_since(b.3).as_secs();
            a.2.cmp(&b.2).reverse().then(b_idle.cmp(&a_idle))
        });

        let mut freed_mb = 0u64;
        let mut unloaded = Vec::new();

        for (name, _size, _priority, last_used) in &candidates {
            let idle = now.duration_since(*last_used).as_secs();
            let should_unload = level == PressureLevel::Critical || idle > idle_seconds;

            if should_unload {
                if let Ok(freed) = self.unload(name) {
                    freed_mb += freed;
                    unloaded.push(name.clone());
                    self.last_cleanup = now;
                    self.sample_pressure().await;
                    if self.pressure_level() == PressureLevel::Normal {
                        break;
                    }
                }
            }
        }

        CleanupReport { freed_mb, unloaded, pressure: level }
    }

    pub async fn periodic_cleanup(running: &AtomicBool) {
        tracing::info!("[ResourceManager] Cleanup loop cada 30s");
        while running.load(Ordering::Relaxed) {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            tracing::debug!("[ResourceManager] Cleanup tick");
        }
    }

    pub fn stats(&self) -> serde_json::Value {
        let total = self.resources.len();
        let loaded = self.resources.values().filter(|r| r.loaded).count();
        let unloaded = self.resources.values().filter(|r| !r.loaded).count();
        let by_kind: std::collections::BTreeMap<&str, usize> = self.resources.values()
            .fold(std::collections::BTreeMap::new(), |mut acc, r| {
                let label = match r.kind {
                    ResourceKind::Model => "modelos",
                    ResourceKind::SubAgent => "sub-agentes",
                    ResourceKind::Cache => "cache",
                    ResourceKind::AudioPipeline => "audio",
                    ResourceKind::MemoryBuffer => "buffers",
                    ResourceKind::JepaModel => "jepa",
                };
                *acc.entry(label).or_insert(0) += 1;
                acc
            });

        serde_json::json!({
            "total_recursos": total,
            "cargados": loaded,
            "descargados": unloaded,
            "memoria_allocada_mb": self.total_allocated_mb,
            "presion_memoria": format!("{:.1}%", self.memory_pressure * 100.0),
            "cleanups_realizados": self.cleanup_count,
            "por_tipo": by_kind,
            "auto_unload": self.auto_unload,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PressureLevel {
    Normal,
    High,
    Critical,
}

impl std::fmt::Display for PressureLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "normal"),
            Self::High => write!(f, "alta"),
            Self::Critical => write!(f, "crítica"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CleanupReport {
    pub freed_mb: u64,
    pub unloaded: Vec<String>,
    pub pressure: PressureLevel,
}
