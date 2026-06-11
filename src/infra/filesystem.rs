use std::path::{Path, PathBuf};

/// File system operations for project navigation
pub struct FileSystem {
    current_dir: PathBuf,
    home_dir: PathBuf,
    projects_dir: PathBuf,
}

impl FileSystem {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let projects = home.join("Proyectos");
        std::fs::create_dir_all(&projects).ok();
        FileSystem {
            current_dir: projects.clone(),
            home_dir: home,
            projects_dir: projects,
        }
    }

    pub fn list_directory(&self, path: &Path) -> Result<Vec<String>, String> {
        let mut items = Vec::new();
        for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name().to_string_lossy().to_string();
            let file_type = entry.file_type().map_err(|e| e.to_string())?;
            if file_type.is_dir() {
                items.push(format!("📁 {}", name));
            } else {
                items.push(format!("📄 {}", name));
            }
        }
        items.sort();
        Ok(items)
    }

    pub fn read_file(&self, path: &Path) -> Result<String, String> {
        std::fs::read_to_string(path).map_err(|e| e.to_string())
    }

    pub fn write_file(&self, path: &Path, content: &str) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(path, content).map_err(|e| e.to_string())
    }

    pub fn change_directory(&mut self, path: &Path) -> Result<(), String> {
        if path.exists() && path.is_dir() {
            self.current_dir = path.to_path_buf();
            Ok(())
        } else {
            Err("Directory not found".to_string())
        }
    }

    pub fn current_dir(&self) -> &Path {
        &self.current_dir
    }
}
