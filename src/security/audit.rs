use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use chrono::Utc;
use std::sync::Mutex;
use std::path::PathBuf;

type HmacSha256 = Hmac<Sha256>;

fn get_signing_key() -> Vec<u8> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let key_path = PathBuf::from(home).join(".umbra/pqc_key");
    std::fs::read_to_string(&key_path).unwrap_or_default().trim().as_bytes().to_vec()
}

fn sign_entry(data: &str) -> String {
    let key = get_signing_key();
    let mut mac = HmacSha256::new_from_slice(&key).expect("HMAC key");
    mac.update(data.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub id: u64,
    pub timestamp: String,
    pub level: String,
    pub source: String,
    pub data: String,
    pub entry_id: String,
    pub prev_hash: String,
    pub signature: String,
    pub data_hash: String,
}

struct Inner {
    next_id: u64,
    entries: Vec<AuditEntry>,
}

pub struct AuditWorm {
    inner: Mutex<Inner>,
}

impl AuditWorm {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner {
                next_id: 1,
                entries: Vec::new(),
            }),
        }
    }

    pub fn log(&self, level: &str, source: &str, data: &str) -> u64 {
        let mut inner = self.inner.lock().unwrap();
        let id = inner.next_id;
        inner.next_id += 1;

        let timestamp = Utc::now().to_rfc3339();
        let prev_hash = inner.entries.last().map(|e| e.entry_id.clone()).unwrap_or_default();
        let data_hash = format!("{:x}", Sha256::digest(data.as_bytes()));
        let entry_id = format!("{:x}", Sha256::digest(
            format!("{}{}{}{}{}", prev_hash, timestamp, level, source, data_hash)
        ));
        let sig = sign_entry(&entry_id);

        let entry = AuditEntry {
            id,
            timestamp: timestamp.clone(),
            level: level.to_string(),
            source: source.to_string(),
            data: data.to_string(),
            entry_id: entry_id.clone(),
            prev_hash,
            signature: sig,
            data_hash,
        };

        if let Ok(json) = serde_json::to_string(&entry) {
            let audit_path = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()))
                .join(".umbra/audit.log");
            if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&audit_path) {
                use std::io::Write;
                writeln!(file, "{}", json).ok();
            }
        }

        inner.entries.push(entry);
        id
    }

    pub fn export(&self) -> Vec<AuditEntry> {
        self.inner.lock().unwrap().entries.clone()
    }

    pub fn verify_chain(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        let mut prev = String::new();
        for entry in &inner.entries {
            let raw = format!("{}{}{}{}{}", prev, entry.timestamp, entry.level, entry.source, entry.data_hash);
            let mut hasher = Sha256::new();
            hasher.update(raw.as_bytes());
            let computed = hex::encode(hasher.finalize());
            if computed != entry.entry_id {
                return false;
            }
            prev = entry.entry_id.clone();
        }
        true
    }
}
