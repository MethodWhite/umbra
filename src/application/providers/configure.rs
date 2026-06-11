// Zone 3 — Application
use crate::domain::ports::ProviderRepository;
use crate::domain::ports::VaultRepository;
use crate::domain::errors::AppError;

pub struct ConfigureProviderUseCase<R: ProviderRepository> {
    provider_repo: R,
}

impl<R: ProviderRepository> ConfigureProviderUseCase<R> {
    pub fn new(provider_repo: R) -> Self {
        Self { provider_repo }
    }

    pub async fn execute(
        &self,
        vault: &mut dyn VaultRepository,
        provider_id: &str,
        api_key: Option<String>,
        base_url: Option<String>,
        is_primary: Option<bool>,
        is_secondary: Option<bool>,
    ) -> Result<serde_json::Value, AppError> {
        let map = self.provider_repo.get_provider_map();
        if !map.contains_key(provider_id) {
            return Err(AppError::Validation("Unknown provider".into()));
        }

        let mut config = self.provider_repo.load_config();

        let vault_used = if !vault.is_locked() {
            if let Some(key) = &api_key {
                vault.set_key(provider_id, key, base_url.as_deref().unwrap_or(""))
                    .map_err(|e| AppError::Internal(anyhow::anyhow!("Vault error: {}", e)))?;
            }
            true
        } else {
            false
        };

        if !vault_used {
            if let Some(api_key) = api_key {
                let providers_table = config.get_mut("providers")
                    .and_then(|v| v.as_table_mut());
                if let Some(table) = providers_table {
                    let entry = table.entry(provider_id.to_string()).or_insert(toml::Value::Table(Default::default()));
                    if let Some(t) = entry.as_table_mut() {
                        t.insert("api_key".into(), toml::Value::String(api_key));
                        if let Some(bu) = &base_url {
                            t.insert("base_url".into(), toml::Value::String(bu.clone()));
                        }
                    }
                }
            }
        }

        if is_primary.unwrap_or(false) {
            if let Some(table) = config.as_table_mut() {
                table.insert("primary".into(), toml::Value::String(provider_id.to_string()));
            }
        }
        if is_secondary.unwrap_or(false) {
            if let Some(table) = config.as_table_mut() {
                table.insert("secondary".into(), toml::Value::String(provider_id.to_string()));
            }
        }

        self.provider_repo.save_config(&config)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Save failed: {}", e)))?;

        Ok(serde_json::json!({"success": true}))
    }
}
