// Zone 2 — Domain/Ports
pub trait SettingsRepository: Send + Sync {
    fn load_config(&self) -> toml::map::Map<String, toml::Value>;
    fn save_config(&self, config: &toml::Value) -> Result<(), String>;
    fn config_path(&self) -> std::path::PathBuf;
}
