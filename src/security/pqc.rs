/// NOTE: This module provides conventional AES-256-GCM encryption + HMAC-SHA256.
/// Post-quantum cryptography (CRYSTALS-Kyber/Dilithium) is not yet implemented.
/// The pqc-crypto crate was removed due to broken implementation.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use hmac::{Hmac, Mac};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use zeroize::Zeroize;

const ITERATIONS: u32 = 600_000;

fn mlock_memory(ptr: *const u8, len: usize) -> bool {
    if ptr.is_null() || len == 0 {
        return false;
    }
    let ret = unsafe { libc::mlock(ptr as *const libc::c_void, len) };
    if ret != 0 {
        let err = std::io::Error::last_os_error();
        tracing::warn!("mlock failed ({}): {} — key material may be swapped to disk", ret, err);
        false
    } else {
        true
    }
}

fn munlock_memory(ptr: *const u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    unsafe {
        libc::munlock(ptr as *const libc::c_void, len);
    }
}

fn load_pqc_key() -> Result<String, anyhow::Error> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let key_path = PathBuf::from(home).join(".umbra/pqc_key");

    if !key_path.exists() {
        use rand::Rng;
        let key: String = (0..64)
            .map(|_| {
                let idx = rand::thread_rng().gen_range(0..16);
                format!("{:x}", idx)
            })
            .collect();
        std::fs::write(&key_path, &key)?;
        std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600))?;
        tracing::info!("Generated PQC key at {:?}", key_path);
    }

    Ok(std::fs::read_to_string(&key_path)?.trim().to_string())
}

pub struct CryptoEngine {
    cipher: Option<Aes256Gcm>,
    hmac_key: Vec<u8>,
}

impl Drop for CryptoEngine {
    fn drop(&mut self) {
        if !self.hmac_key.is_empty() {
            munlock_memory(self.hmac_key.as_ptr(), self.hmac_key.len());
            self.hmac_key.zeroize();
        }
    }
}

impl CryptoEngine {
    pub fn new() -> Self {
        match load_pqc_key() {
            Ok(val) if !val.is_empty() => {
                let salt = b"umbra-pqc-key-v1";
                let mut key_material = [0u8; 32];
                pbkdf2_hmac::<Sha256>(val.as_bytes(), salt, ITERATIONS, &mut key_material);

                mlock_memory(key_material.as_ptr(), key_material.len());

                let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_material);
                let cipher = Aes256Gcm::new(key);

                let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(&key_material)
                    .expect("HMAC key length OK");
                mac.update(b"pqc-hmac-key");
                let hmac_key = mac.finalize().into_bytes().to_vec();

                mlock_memory(hmac_key.as_ptr(), hmac_key.len());
                key_material.zeroize();

                Self {
                    cipher: Some(cipher),
                    hmac_key,
                }
            }
            _ => {
                tracing::warn!("PQC key not available — encryption operations will fail");
                Self {
                    cipher: None,
                    hmac_key: vec![],
                }
            }
        }
    }

    pub fn encrypt(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        let cipher = self
            .cipher
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("encryption key not available"))?;
        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| anyhow::anyhow!("AES-256-GCM encrypt failed: {}", e))?;
        let mut out = nonce_bytes.to_vec();
        out.extend(ciphertext);
        Ok(out)
    }

    pub fn decrypt(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        let cipher = self
            .cipher
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("encryption key not available"))?;
        if data.len() < 12 {
            anyhow::bail!("ciphertext too short");
        }
        let (raw_nonce, ct) = data.split_at(12);
        let nonce = Nonce::from_slice(raw_nonce);
        cipher
            .decrypt(nonce, ct)
            .map_err(|e| anyhow::anyhow!("AES-256-GCM decrypt failed: {}", e))
    }

    fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(key)
            .expect("HMAC key length OK");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }

    pub fn sign(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        if self.hmac_key.is_empty() {
            anyhow::bail!("HMAC key not available");
        }
        Ok(Self::hmac_sha256(&self.hmac_key, data))
    }

    pub fn verify(&self, data: &[u8], signature: &[u8]) -> anyhow::Result<bool> {
        if self.hmac_key.is_empty() {
            anyhow::bail!("HMAC key not available");
        }
        let expected = Self::hmac_sha256(&self.hmac_key, data);
        if expected.len() != signature.len() {
            return Ok(false);
        }
        Ok(expected
            .iter()
            .zip(signature)
            .fold(0u8, |acc, (a, b)| acc | (a ^ b))
            == 0)
    }
}
