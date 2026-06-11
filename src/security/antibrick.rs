use anyhow::Result;

pub struct AntiBrick;

impl AntiBrick {
    pub fn new() -> Self {
        Self
    }

    pub fn check(&self, cmd: &str, args: &[String]) -> Result<bool> {
        let full = format!("{} {}", cmd, args.join(" "));

        let destructive_patterns = [
            "dd if=", "mkfs", "fdisk", "parted", "format",
            "flashrom", "fastboot", "heimdall",
            "> /dev/sd", "> /dev/mmc", "> /dev/nvme",
            "pvcreate", "vgcreate", "lvcreate",
            "cryptsetup luksFormat",
        ];

        for pattern in &destructive_patterns {
            if full.contains(pattern) {
                tracing::warn!("[AntiBrick] Patrón destructivo detectado: {}", pattern);
                return Ok(false);
            }
        }

        Ok(true)
    }
}
