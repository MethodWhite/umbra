use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use rand::Rng;

pub fn derive_key(passphrase: &str, salt: &[u8]) -> [u8; 32] {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let auth_path = std::path::PathBuf::from(&home).join(".umbra/auth_token");
    let token = if auth_path.exists() {
        std::fs::read_to_string(&auth_path).unwrap_or_default().trim().to_string()
    } else {
        String::new()
    };
    let raw = format!("{}:{}", token, passphrase);
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(raw.as_bytes(), salt, 600_000, &mut key);
    key
}

pub fn encrypt_vault(plaintext: &[u8], passphrase: &str, salt: &[u8]) -> Vec<u8> {
    let key = derive_key(passphrase, salt);
    let aes_key = Key::<Aes256Gcm>::from_slice(&key);
    let cipher = Aes256Gcm::new(aes_key);
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher.encrypt(nonce, plaintext).expect("AES-256-GCM encrypt failed");
    let mut out = Vec::new();
    out.extend_from_slice(salt);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);
    out
}

pub fn decrypt_vault(data: &[u8], passphrase: &str) -> Result<Vec<u8>, ()> {
    if data.len() < 28 { return Err(()); }
    let salt = &data[..16];
    let nonce_slice = &data[16..28];
    let ct = &data[28..];
    let key = derive_key(passphrase, salt);
    let aes_key = Key::<Aes256Gcm>::from_slice(&key);
    let cipher = Aes256Gcm::new(aes_key);
    let nonce = Nonce::from_slice(nonce_slice);
    cipher.decrypt(nonce, ct).map_err(|_| ())
}

pub fn derive_customization_key() -> [u8; 32] {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let auth_path = std::path::PathBuf::from(&home).join(".umbra/auth_token");
    let token = std::fs::read_to_string(&auth_path).unwrap_or_default();
    let raw = format!("{}:customization", token.trim());
    let mut key = [0u8; 32];
    let salt = b"umbra-customization-salt";
    pbkdf2_hmac::<Sha256>(raw.as_bytes(), salt, 600_000, &mut key);
    key
}
