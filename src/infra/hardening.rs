// Zone 6 — Research/Stubs (server-gated)
use std::path::Path;

pub struct OpenVentus;

impl OpenVentus {
    pub fn new() -> Self {
        Self
    }

    pub fn check_file_permissions(path: &str) -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            Path::new(path)
                .metadata()
                .map(|m| m.permissions().mode() & 0o077 == 0)
                .unwrap_or(false)
        }
        #[cfg(not(unix))]
        {
            true
        }
    }

    pub fn suggest_hardening() -> Vec<String> {
        let mut suggestions = Vec::new();
        if Path::new("/proc/sys/kernel/core_pattern").exists() {
            suggestions.push("Verificar core dump limits (ulimit -c 0)".into());
        }
        if Path::new("/proc/sys/kernel/yama/ptrace_scope").exists() {
            suggestions.push("Verificar ptrace_scope (3 = solo root)".into());
        }
        suggestions.push("Mantener /home/$USER/.umbra con permisos 0700".into());
        suggestions
    }
}
