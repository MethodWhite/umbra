use crate::domain::ports::VaultRepository;
use crate::domain::errors::AppError;

pub struct UnlockUseCase;

impl UnlockUseCase {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, vault: &mut dyn VaultRepository, passphrase: &str) -> Result<serde_json::Value, AppError> {
        match vault.unlock(passphrase) {
            Ok(true) => {
                let s = vault.status();
                Ok(serde_json::json!({"success": true, "status": s}))
            }
            Err(e) if e == "wrong_passphrase" => {
                Err(AppError::Unauthorized(format!("Unlock failed: {}", e)))
            }
            Err(e) => {
                Err(AppError::Validation(format!("Unlock failed: {}", e)))
            }
            _ => Err(AppError::Internal(anyhow::anyhow!("Unknown unlock error"))),
        }
    }
}
