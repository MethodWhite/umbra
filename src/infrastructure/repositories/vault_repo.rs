// Zone 4 — Infrastructure
use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use rand::Rng;

use crate::domain::models::{VaultContents, VaultKeyEntry, VaultStatus, ProviderKeyStatus, now_secs};
use crate::domain::ports::VaultRepository;
use crate::infrastructure::persistence::{derive_key, encrypt_vault, decrypt_vault};

pub struct EncryptedVaultRepository {
    vault_path: PathBuf,
    lock_path: PathBuf,
    contents: Option<VaultContents>,
    derived_key: [u8; 32],
    passphrase: String,
    salt: [u8; 16],
    locked: bool,
    unlocked_at: u64,
    auto_lock_minutes: u64,
    last_access: u64,
}

impl EncryptedVaultRepository {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        EncryptedVaultRepository {
            vault_path: PathBuf::from(&home).join(".umbra/vault.enc"),
            lock_path: PathBuf::from(&home).join(".umbra/vault.lock"),
            contents: None,
            derived_key: [0u8; 32],
            passphrase: String::new(),
            salt: [0u8; 16],
            locked: true,
            unlocked_at: 0,
            auto_lock_minutes: 15,
            last_access: 0,
        }
    }

    fn check_auto_lock(&mut self) {
        if !self.locked && self.auto_lock_minutes > 0 {
            let elapsed = now_secs().saturating_sub(self.last_access);
            if elapsed > self.auto_lock_minutes * 60 {
                self.locked = true;
                self.derived_key = [0u8; 32];
                self.contents = None;
                self.passphrase.clear();
                self.write_lock_state(true);
            }
        }
    }

    fn write_lock_state(&self, locked: bool) {
        if let Some(parent) = self.lock_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        if locked {
            std::fs::write(&self.lock_path, "locked").ok();
        } else {
            std::fs::write(&self.lock_path, format!("unlocked:{}", now_secs())).ok();
        }
        std::fs::set_permissions(&self.lock_path, std::fs::Permissions::from_mode(0o600)).ok();
    }

    fn touch_access(&mut self) {
        self.last_access = now_secs();
    }

    fn save(&mut self) -> Result<(), String> {
        let contents = self.contents.as_ref().ok_or("Vault not unlocked")?;
        let json = serde_json::to_vec(contents).map_err(|e| e.to_string())?;
        let salt = rand::thread_rng().gen::<[u8; 16]>();
        let out = encrypt_vault(&json, &self.passphrase, &salt);
        std::fs::create_dir_all(self.vault_path.parent().unwrap()).ok();
        std::fs::write(&self.vault_path, &out).map_err(|e| e.to_string())?;
        std::fs::set_permissions(&self.vault_path, std::fs::Permissions::from_mode(0o600)).ok();
        self.derived_key = derive_key(&self.passphrase, &salt);
        self.salt = salt;
        Ok(())
    }
}

impl VaultRepository for EncryptedVaultRepository {
    fn unlock(&mut self, passphrase: &str) -> Result<bool, String> {
        if !self.vault_path.exists() {
            let salt = rand::thread_rng().gen::<[u8; 16]>();
            let contents = VaultContents {
                version: 1,
                keys: HashMap::new(),
                created_at: now_secs(),
                updated_at: now_secs(),
            };
            let json = serde_json::to_vec(&contents).map_err(|e| e.to_string())?;
            let out = encrypt_vault(&json, passphrase, &salt);
            std::fs::create_dir_all(self.vault_path.parent().unwrap()).ok();
            std::fs::write(&self.vault_path, &out).map_err(|e| e.to_string())?;
            std::fs::set_permissions(&self.vault_path, std::fs::Permissions::from_mode(0o600)).ok();
            self.contents = Some(contents);
            self.derived_key = derive_key(passphrase, &salt);
            self.passphrase = passphrase.to_string();
            self.salt = salt;
            self.locked = false;
            self.unlocked_at = now_secs();
            self.last_access = now_secs();
            self.write_lock_state(false);
            return Ok(true);
        }

        let data = std::fs::read(&self.vault_path).map_err(|e| e.to_string())?;
        let plain = decrypt_vault(&data, passphrase).map_err(|_| "wrong_passphrase".to_string())?;
        let contents: VaultContents = serde_json::from_slice(&plain).map_err(|_| "corrupted".to_string())?;
        let mut salt = [0u8; 16];
        salt.copy_from_slice(&data[..16]);
        self.contents = Some(contents);
        self.derived_key = derive_key(passphrase, &salt);
        self.passphrase = passphrase.to_string();
        self.salt = salt;
        self.locked = false;
        self.unlocked_at = now_secs();
        self.last_access = now_secs();
        self.write_lock_state(false);
        Ok(true)
    }

    fn lock(&mut self) {
        self.locked = true;
        self.derived_key = [0u8; 32];
        self.salt = [0u8; 16];
        self.contents = None;
        self.passphrase.clear();
        self.write_lock_state(true);
    }

    fn is_locked(&self) -> bool {
        self.locked
    }

    fn status(&mut self) -> VaultStatus {
        self.check_auto_lock();
        if self.locked {
            return VaultStatus {
                locked: true,
                providers_with_keys: vec![],
                auto_lock_minutes: self.auto_lock_minutes,
                auto_lock_remaining: 0,
                unlocked_at: None,
                key_count: 0,
            };
        }
        self.touch_access();
        let keys = self.contents.as_ref().map(|c| c.keys.clone()).unwrap_or_default();
        let remaining = if self.auto_lock_minutes > 0 {
            let elapsed = now_secs().saturating_sub(self.last_access);
            (self.auto_lock_minutes * 60).saturating_sub(elapsed) / 60
        } else { 0 };
        VaultStatus {
            locked: false,
            providers_with_keys: keys.iter().map(|(id, e)| ProviderKeyStatus {
                id: id.clone(),
                has_key: e.api_key.is_some(),
            }).collect(),
            auto_lock_minutes: self.auto_lock_minutes,
            auto_lock_remaining: remaining,
            unlocked_at: Some(self.unlocked_at),
            key_count: keys.len(),
        }
    }

    fn get_key(&mut self, provider_id: &str) -> Option<String> {
        self.check_auto_lock();
        if self.locked { return None; }
        self.touch_access();
        self.contents.as_ref()?.keys.get(provider_id)
            .and_then(|e| e.api_key.clone())
    }

    fn get_base_url(&mut self, provider_id: &str) -> Option<String> {
        self.check_auto_lock();
        if self.locked { return None; }
        self.touch_access();
        self.contents.as_ref()?.keys.get(provider_id)
            .and_then(|e| e.base_url.clone())
    }

    fn list_keys(&mut self) -> HashMap<String, bool> {
        self.check_auto_lock();
        if self.locked { return HashMap::new(); }
        self.touch_access();
        let mut result = HashMap::new();
        if let Some(contents) = &self.contents {
            for (pid, entry) in &contents.keys {
                result.insert(pid.clone(), entry.api_key.is_some());
            }
        }
        result
    }

    fn set_key(&mut self, provider_id: &str, api_key: &str, base_url: &str) -> Result<(), String> {
        self.check_auto_lock();
        if self.locked { return Err("Vault is locked".into()); }
        self.touch_access();
        let contents = self.contents.as_mut().ok_or("Vault not unlocked")?;
        contents.keys.insert(provider_id.to_string(), VaultKeyEntry {
            api_key: Some(api_key.to_string()),
            base_url: if base_url.is_empty() { None } else { Some(base_url.to_string()) },
            updated_at: now_secs(),
        });
        contents.updated_at = now_secs();
        self.save()
    }

    fn delete_key(&mut self, provider_id: &str) -> Result<(), String> {
        self.check_auto_lock();
        if self.locked { return Err("Vault is locked".into()); }
        self.touch_access();
        let contents = self.contents.as_mut().ok_or("Vault not unlocked")?;
        contents.keys.remove(provider_id);
        contents.updated_at = now_secs();
        self.save()
    }

    fn migrate_from_env(&mut self) -> Result<Vec<String>, String> {
        self.check_auto_lock();
        if self.locked { return Err("Vault is locked".into()); }
        self.touch_access();
        let mut migrated = Vec::new();

        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());

        let env_path = PathBuf::from(&home).join(".umbra/.env");
        if env_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&env_path) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') || !line.contains('=') { continue; }
                    let mut parts = line.splitn(2, '=');
                    let key_name = parts.next().unwrap_or("").trim().to_string();
                    let value = parts.next().unwrap_or("").trim().trim_matches('"').trim_matches('\'').to_string();
                    if value.is_empty() { continue; }
                    if key_name.ends_with("_API_KEY") || key_name.ends_with("_KEY") {
                        let provider_id = key_name
                            .replace("_API_KEY", "")
                            .replace("_KEY", "")
                            .to_lowercase();
                        if !provider_id.is_empty() {
                            self.set_key(&provider_id, &value, "")?;
                            migrated.push(key_name);
                        }
                    }
                }
            }
        }

        let prov_path = PathBuf::from(&home).join(".umbra/providers.toml");
        if prov_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&prov_path) {
                if let Ok(data) = content.parse::<toml::Value>() {
                    if let Some(providers) = data.get("providers").and_then(|v| v.as_table()) {
                        for (pid, config) in providers {
                            let api_key = config.get("api_key").and_then(|v| v.as_str()).unwrap_or("");
                            let base_url = config.get("base_url").and_then(|v| v.as_str()).unwrap_or("");
                            if !api_key.is_empty() {
                                self.set_key(pid, api_key, base_url)?;
                                migrated.push(pid.clone());
                            }
                        }
                    }
                }
            }
        }

        Ok(migrated)
    }

    fn set_auto_lock(&mut self, minutes: u64) {
        self.auto_lock_minutes = minutes;
    }

    fn auto_lock_minutes(&self) -> u64 {
        self.auto_lock_minutes
    }
}
