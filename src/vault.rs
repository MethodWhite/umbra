// Zone 1 — Desktop/UI
use std::path::PathBuf;
use std::sync::Mutex;

/// Reads API keys from the Python vault file.
/// The vault is an AES-256-GCM encrypted JSON file at ~/.umbra/vault.enc
/// Only reads keys when vault is unlocked (checked via ~/.umbra/vault.lock)
pub struct VaultReader {
    _vault_path: PathBuf,
    lock_path: PathBuf,
    _cache: Mutex<Option<Vec<u8>>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VaultContents {
    pub version: u64,
    pub keys: std::collections::HashMap<String, VaultKeyEntry>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VaultKeyEntry {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub updated_at: u64,
}

impl VaultReader {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        VaultReader {
            _vault_path: PathBuf::from(&home).join(".umbra/vault.enc"),
            lock_path: PathBuf::from(&home).join(".umbra/vault.lock"),
            _cache: Mutex::new(None),
        }
    }

    pub fn is_unlocked(&self) -> bool {
        if !self.lock_path.exists() {
            return false;
        }
        match std::fs::read_to_string(&self.lock_path) {
            Ok(content) => content.trim().starts_with("unlocked:"),
            Err(_) => false,
        }
    }

    pub fn get_key(&self, provider_id: &str) -> Option<String> {
        if !self.is_unlocked() {
            return None;
        }
        let url = format!("http://127.0.0.1:8340/api/vault/key/{}", provider_id);
        match reqwest::blocking::get(&url) {
            Ok(resp) => resp.json::<serde_json::Value>().ok()
                .and_then(|j| j.get("key").and_then(|v| v.as_str().map(String::from))),
            Err(_) => None,
        }
    }

    pub fn check_key_available(&self, provider_id: &str) -> bool {
        let url = format!("http://127.0.0.1:8340/api/vault/key/{}", provider_id);
        match reqwest::blocking::get(&url) {
            Ok(resp) => resp.json::<serde_json::Value>().map(|j| {
                j.get("has_key").and_then(|v| v.as_bool()).unwrap_or(false)
            }).unwrap_or(false),
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_reader_not_panics() {
        let _ = VaultReader::new();
    }

    #[test]
    fn test_check_key_available_no_server() {
        let reader = VaultReader::new();
        // With no vault server running, should return false
        assert!(!reader.check_key_available("openai"));
    }
}
