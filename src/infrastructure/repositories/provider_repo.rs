use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::domain::ports::ProviderRepository;
use crate::domain::models::{ApiProvider, ALL_PROVIDERS};

pub struct TomlProviderRepository;

impl TomlProviderRepository {
    pub fn new() -> Self {
        Self
    }

    fn get_config_path_inner(&self) -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        PathBuf::from(&home).join(".umbra/providers.toml")
    }
}

impl ProviderRepository for TomlProviderRepository {
    fn get_config_path(&self) -> PathBuf {
        self.get_config_path_inner()
    }

    fn load_config(&self) -> toml::Value {
        let path = self.get_config_path_inner();
        if !path.exists() { return toml::Value::Table(Default::default()); }
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        content.parse::<toml::Value>().unwrap_or(toml::Value::Table(Default::default()))
    }

    fn save_config(&self, config: &toml::Value) -> Result<(), String> {
        let path = self.get_config_path_inner();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let toml_str = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
        std::fs::write(&path, &toml_str).map_err(|e| e.to_string())?;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_provider_map(&self) -> HashMap<&'static str, &'static ApiProvider> {
        let mut map = HashMap::new();
        for p in ALL_PROVIDERS {
            map.insert(p.id, p);
        }
        map
    }
}
