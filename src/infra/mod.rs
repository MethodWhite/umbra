// Zone 6 — Research/Stubs (server-gated)
pub mod filesystem;
pub mod hardening;
pub mod ghost;
pub mod backup;

pub use hardening::OpenVentus;
pub use ghost::GhostMonitor;
pub use backup::BackupEngine;

pub struct Infrastructure {
    pub hardening: OpenVentus,
    pub ghost: GhostMonitor,
    pub backup: BackupEngine,
}

impl Infrastructure {
    pub fn new() -> Self {
        Self {
            hardening: OpenVentus::new(),
            ghost: GhostMonitor::new(),
            backup: BackupEngine::new(None),
        }
    }
}
