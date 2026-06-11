use crate::domain::ports::VaultRepository;
use crate::domain::errors::AppError;

pub struct MigrateUseCase;

impl MigrateUseCase {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, vault: &mut dyn VaultRepository) -> Result<serde_json::Value, AppError> {
        if vault.is_locked() {
            return Err(AppError::VaultLocked);
        }
        let migrated = vault.migrate_from_env()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Migration failed: {}", e)))?;
        Ok(serde_json::json!({"migrated": migrated, "count": migrated.len()}))
    }
}
