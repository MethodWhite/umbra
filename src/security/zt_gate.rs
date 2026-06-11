// Zone 5 — Bridge/External
use anyhow::Result;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    High,
    Critical,
    Blocked,
}

impl RiskLevel {
    pub fn is_allowed(&self) -> bool {
        matches!(self, RiskLevel::Safe | RiskLevel::Low)
    }
}

pub struct ZeroTrustGate {
    session_id: String,
}

fn auth_token_path() -> Result<PathBuf> {
    let home =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("ZT: cannot determine home directory"))?;
    Ok(home.join(".umbra").join("auth_token"))
}

impl ZeroTrustGate {
    pub fn new() -> Self {
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    pub fn check_identity(&self) -> Result<bool> {
        let path = auth_token_path()?;
        if !path.exists() {
            tracing::warn!("ZT: auth_token not found at {}", path.display());
            return Ok(false);
        }
        let meta = fs::metadata(&path)?;
        let mode = meta.permissions().mode();
        if mode & 0o777 != 0o600 {
            tracing::warn!("ZT: auth_token permissions {:o} (expected 600)", mode);
            return Ok(false);
        }
        let _token = fs::read_to_string(&path)?;
        Ok(true)
    }

    pub fn check_permission(&self, cmd: &str) -> Result<bool> {
        let blocked = [
            "rm -rf /",
            "dd if=",
            "mkfs",
            "fdisk",
            "format",
            ":(){ :|:& };:",
            "wget",
            "curl",
            "chmod 777",
        ];
        Ok(!blocked.iter().any(|b| cmd.contains(b)))
    }

    pub fn check_context(&self, _cmd: &str) -> Result<bool> {
        if self.session_id.is_empty() {
            return Ok(false);
        }
        match uuid::Uuid::parse_str(&self.session_id) {
            Ok(u) => Ok(u.get_version() == Some(uuid::Version::Random)),
            Err(_) => Ok(false),
        }
    }

    pub fn analyze_risk(&self, cmd: &str) -> RiskLevel {
        if cmd.contains("rm -rf /") || cmd.contains("dd if=/dev/zero") {
            RiskLevel::Critical
        } else if cmd.contains("mkfs") || cmd.contains("fdisk") {
            RiskLevel::High
        } else if cmd.contains("sudo") || cmd.contains("chmod") {
            RiskLevel::Medium
        } else {
            RiskLevel::Safe
        }
    }
}
