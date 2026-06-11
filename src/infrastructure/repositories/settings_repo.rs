use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::domain::ports::SettingsRepository;

pub struct TomlSettingsRepository;

impl TomlSettingsRepository {
    pub fn new() -> Self {
        Self
    }
}

impl SettingsRepository for TomlSettingsRepository {
    fn config_path(&self) -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        PathBuf::from(&home).join(".umbra/config.toml")
    }

    fn load_config(&self) -> toml::map::Map<String, toml::Value> {
        let path = self.config_path();
        if !path.exists() { return toml::map::Map::new(); }
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        content.parse::<toml::Value>().ok()
            .and_then(|v| match v {
                toml::Value::Table(t) => Some(t),
                _ => None,
            })
            .unwrap_or_default()
    }

    fn save_config(&self, config: &toml::Value) -> Result<(), String> {
        let path = self.config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let toml_str = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
        std::fs::write(&path, &toml_str).map_err(|e| e.to_string())?;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
        Ok(())
    }
}
