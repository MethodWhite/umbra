use std::collections::HashMap;
use crate::domain::models::VaultStatus;

pub trait VaultRepository: Send + Sync {
    fn unlock(&mut self, passphrase: &str) -> Result<bool, String>;
    fn lock(&mut self);
    fn is_locked(&self) -> bool;
    fn status(&mut self) -> VaultStatus;
    fn get_key(&mut self, provider_id: &str) -> Option<String>;
    fn get_base_url(&mut self, provider_id: &str) -> Option<String>;
    fn list_keys(&mut self) -> HashMap<String, bool>;
    fn set_key(&mut self, provider_id: &str, api_key: &str, base_url: &str) -> Result<(), String>;
    fn delete_key(&mut self, provider_id: &str) -> Result<(), String>;
    fn migrate_from_env(&mut self) -> Result<Vec<String>, String>;
    fn set_auto_lock(&mut self, minutes: u64);
    fn auto_lock_minutes(&self) -> u64;
}

impl<T: VaultRepository> VaultRepository for &mut T {
    fn unlock(&mut self, passphrase: &str) -> Result<bool, String> { T::unlock(&mut **self, passphrase) }
    fn lock(&mut self) { T::lock(&mut **self) }
    fn is_locked(&self) -> bool { T::is_locked(&**self) }
    fn status(&mut self) -> VaultStatus { T::status(&mut **self) }
    fn get_key(&mut self, provider_id: &str) -> Option<String> { T::get_key(&mut **self, provider_id) }
    fn get_base_url(&mut self, provider_id: &str) -> Option<String> { T::get_base_url(&mut **self, provider_id) }
    fn list_keys(&mut self) -> HashMap<String, bool> { T::list_keys(&mut **self) }
    fn set_key(&mut self, provider_id: &str, api_key: &str, base_url: &str) -> Result<(), String> {
        T::set_key(&mut **self, provider_id, api_key, base_url)
    }
    fn delete_key(&mut self, provider_id: &str) -> Result<(), String> { T::delete_key(&mut **self, provider_id) }
    fn migrate_from_env(&mut self) -> Result<Vec<String>, String> { T::migrate_from_env(&mut **self) }
    fn set_auto_lock(&mut self, minutes: u64) { T::set_auto_lock(&mut **self, minutes) }
    fn auto_lock_minutes(&self) -> u64 { T::auto_lock_minutes(&**self) }
}
