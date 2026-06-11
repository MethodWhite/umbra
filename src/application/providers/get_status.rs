// Zone 3 — Application
use crate::domain::ports::ProviderRepository;
use crate::domain::errors::AppError;

pub struct GetProviderStatusUseCase<R: ProviderRepository> {
    provider_repo: R,
}

impl<R: ProviderRepository> GetProviderStatusUseCase<R> {
    pub fn new(provider_repo: R) -> Self {
        Self { provider_repo }
    }

    pub fn execute(&self) -> Result<serde_json::Value, AppError> {
        let config = self.provider_repo.load_config();
        let map = self.provider_repo.get_provider_map();
        let primary_id = config.get("primary").and_then(|v| v.as_str());
        let secondary_id = config.get("secondary").and_then(|v| v.as_str());
        let providers_config = config.get("providers")
            .and_then(|v| v.as_table())
            .map(|t| {
                t.iter().map(|(pid, pc)| {
                    let has_key = pc.get("api_key").and_then(|v| v.as_str()).unwrap_or("") != "";
                    serde_json::json!({"id": pid, "has_key": has_key})
                }).collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Ok(serde_json::json!({
            "primary": primary_id.map(|id| {
                let p = map.get(id);
                serde_json::json!({
                    "id": id,
                    "name": p.map(|p| p.name),
                    "configured": providers_config.iter().any(|c| c["id"] == id && c["has_key"].as_bool().unwrap_or(false)),
                })
            }),
            "secondary": secondary_id.map(|id| {
                let p = map.get(id);
                serde_json::json!({
                    "id": id,
                    "name": p.map(|p| p.name),
                    "configured": providers_config.iter().any(|c| c["id"] == id && c["has_key"].as_bool().unwrap_or(false)),
                })
            }),
            "configured_providers": providers_config,
        }))
    }
}
