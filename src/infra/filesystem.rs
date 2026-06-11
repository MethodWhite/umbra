// Zone 6 — Research/Stubs (server-gated)
use std::path::{Path, PathBuf};

fn resolve_safe(path: &Path, sandbox: &Path) -> Result<PathBuf, String> {
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("Cannot resolve path '{}': {}", path.display(), e))?;
    let sandbox_canonical = sandbox
        .canonicalize()
        .map_err(|e| format!("Cannot resolve sandbox '{}': {}", sandbox.display(), e))?;
    if !canonical.starts_with(&sandbox_canonical) {
        return Err(format!(
            "Path '{}' is outside the allowed sandbox '{}'",
            canonical.display(),
            sandbox_canonical.display()
        ));
    }
    Ok(canonical)
}

/// File system operations for project navigation (sandboxed)
#[allow(dead_code)]
pub struct FileSystem {
    current_dir: PathBuf,
    home_dir: PathBuf,
    projects_dir: PathBuf,
    sandbox: PathBuf,
}

impl FileSystem {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let projects = home.join("Proyectos");
        let sandbox = projects.clone();
        std::fs::create_dir_all(&projects).ok();
        FileSystem {
            current_dir: projects.clone(),
            home_dir: home,
            projects_dir: projects,
            sandbox,
        }
    }

    pub fn list_directory(&self, path: &Path) -> Result<Vec<String>, String> {
        let safe_path = resolve_safe(path, &self.sandbox)?;
        let mut items = Vec::new();
        for entry in std::fs::read_dir(&safe_path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name().to_string_lossy().to_string();
            let file_type = entry.file_type().map_err(|e| e.to_string())?;
            if file_type.is_dir() {
                items.push(format!("[dir] {}", name));
            } else {
                items.push(format!("[file] {}", name));
            }
        }
        items.sort();
        Ok(items)
    }

    pub fn read_file(&self, path: &Path) -> Result<String, String> {
        let safe_path = resolve_safe(path, &self.sandbox)?;
        std::fs::read_to_string(&safe_path).map_err(|e| e.to_string())
    }

    pub fn write_file(&self, path: &Path, content: &str) -> Result<(), String> {
        let safe_path = resolve_safe(path, &self.sandbox)?;
        if let Some(parent) = safe_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&safe_path, content).map_err(|e| e.to_string())
    }

    pub fn change_directory(&mut self, path: &Path) -> Result<(), String> {
        let safe_path = resolve_safe(path, &self.sandbox)?;
        if safe_path.exists() && safe_path.is_dir() {
            self.current_dir = safe_path;
            Ok(())
        } else {
            Err("Directory not found".to_string())
        }
    }

    pub fn current_dir(&self) -> &Path {
        &self.current_dir
    }
}
