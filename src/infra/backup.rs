use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct BackupEngine {
    backup_dir: PathBuf,
    last_backup: Option<Instant>,
    interval: Duration,
}

impl BackupEngine {
    pub fn new(backup_dir: Option<PathBuf>) -> Self {
        let dir = backup_dir.unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            PathBuf::from(home).join(".umbra/backups")
        });
        std::fs::create_dir_all(&dir).ok();
        BackupEngine {
            backup_dir: dir,
            last_backup: None,
            interval: Duration::from_secs(300),
        }
    }

    pub fn backup_file(&self, path: &str) -> std::io::Result<PathBuf> {
        let src = PathBuf::from(path);
        if !src.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file not found",
            ));
        }
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let dest = self.backup_dir.join(format!(
            "{}_{}.bak",
            src.file_name().unwrap_or_default().to_string_lossy(),
            timestamp
        ));
        std::fs::copy(&src, &dest)?;
        Ok(dest)
    }

    pub fn should_backup(&self) -> bool {
        self.last_backup
            .map_or(true, |t| t.elapsed() >= self.interval)
    }

    pub fn mark_backup_done(&mut self) {
        self.last_backup = Some(Instant::now());
    }

    pub fn list_backups(&self) -> std::io::Result<Vec<PathBuf>> {
        let mut backups = Vec::new();
        for entry in std::fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            if entry.path().extension().map_or(false, |e| e == "bak") {
                backups.push(entry.path());
            }
        }
        backups.sort_by_key(|p| std::fs::metadata(p).ok().and_then(|m| m.modified().ok()));
        backups.reverse();
        Ok(backups)
    }

    pub fn restore_latest(&self, original: &str) -> std::io::Result<()> {
        let dest = PathBuf::from(original);
        let dest_canonical = dest.canonicalize().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Invalid restore path: {e}"))
        })?;
        let backup_canonical = self.backup_dir.canonicalize().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("Invalid backup dir: {e}"))
        })?;
        if !dest_canonical.starts_with(&backup_canonical) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!("Restore target '{}' is outside backup directory", dest_canonical.display()),
            ));
        }
        let backups = self.list_backups()?;
        if let Some(latest) = backups.first() {
            std::fs::copy(latest, &dest_canonical)?;
        }
        Ok(())
    }
}
