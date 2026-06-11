use std::collections::HashMap;

use crate::domain::models::{ApiProvider, ALL_PROVIDERS};

pub fn provider_map() -> HashMap<&'static str, &'static ApiProvider> {
    let mut map = HashMap::new();
    for p in ALL_PROVIDERS {
        map.insert(p.id, p);
    }
    map
}

pub trait ProviderRepository: Send + Sync {
    fn load_config(&self) -> toml::Value;
    fn save_config(&self, config: &toml::Value) -> Result<(), String>;
    fn get_provider_map(&self) -> HashMap<&'static str, &'static ApiProvider>;
    fn get_config_path(&self) -> std::path::PathBuf;
}
