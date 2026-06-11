use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use rand::Rng;
use sha2::{Digest, Sha256};

pub use crate::engine::jepa_model::{TrainingExample, TrainReport};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserExample {
    pub url: String,
    pub title: String,
    pub content: String,
    pub content_type: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct TrainingFromBrowser {
    pub examples: Vec<BrowserExample>,
    pub source: String,
}

#[derive(Clone)]
pub struct TrainerEngine {
    inner: Arc<Mutex<TrainerInner>>,
    _models_dir: PathBuf,
    jsonl_path: PathBuf,
}

struct TrainerInner {
    pub examples: Vec<TrainingExample>,
    pub total_examples_processed: usize,
    pub last_train_time: Option<Instant>,
}

fn derive_encryption_key() -> [u8; 32] {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let auth_path = PathBuf::from(home).join(".umbra/auth_token");
    let token = std::fs::read_to_string(&auth_path).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(format!("umbra-data:{}", token.trim()));
    let hash = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash[..32]);
    key
}

fn encrypt_aes256gcm(key: &[u8; 32], plaintext: &[u8]) -> Vec<u8> {
    let cipher = Aes256Gcm::new_from_slice(key).expect("valid AES-256 key");
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill(&mut nonce);
    let ciphertext = cipher.encrypt(&Nonce::from_slice(&nonce), plaintext)
        .expect("AES-256-GCM encryption failed");
    [&nonce[..], &ciphertext[..]].concat()
}

fn decrypt_aes256gcm(key: &[u8; 32], data: &[u8]) -> Vec<u8> {
    if data.len() < 12 {
        return data.to_vec();
    }
    let cipher = Aes256Gcm::new_from_slice(key).expect("valid AES-256 key");
    let (nonce, ct) = data.split_at(12);
    cipher.decrypt(&Nonce::from_slice(nonce), ct)
        .unwrap_or_else(|_| data.to_vec())
}

impl TrainerEngine {
    pub fn new(models_dir: PathBuf) -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let jsonl_path = PathBuf::from(&home).join(".umbra/training_data.jsonl");
        if let Some(parent) = jsonl_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let examples = Self::load_jsonl(&jsonl_path);
        let total = examples.len();
        Self {
            inner: Arc::new(Mutex::new(TrainerInner {
                examples,
                total_examples_processed: total,
                last_train_time: None,
            })),
            _models_dir: models_dir,
            jsonl_path,
        }
    }

    fn load_jsonl(path: &PathBuf) -> Vec<TrainingExample> {
        let encrypted = match std::fs::read(path) {
            Ok(content) => content,
            Err(_) => return Vec::new(),
        };
        let key = derive_encryption_key();
        let decrypted = decrypt_aes256gcm(&key, &encrypted);
        let content = String::from_utf8_lossy(&decrypted);
        content.lines()
            .filter_map(|line| serde_json::from_str::<TrainingExample>(line).ok())
            .collect()
    }

    fn append_jsonl(&self, examples: &[TrainingExample]) {
        let key = derive_encryption_key();
        let existing = if self.jsonl_path.exists() {
            let encrypted = std::fs::read(&self.jsonl_path).unwrap_or_default();
            decrypt_aes256gcm(&key, &encrypted)
        } else {
            Vec::new()
        };
        let mut content = String::from_utf8_lossy(&existing).to_string();
        for example in examples {
            if let Ok(line) = serde_json::to_string(example) {
                content.push_str(&line);
                content.push('\n');
            }
        }
        let encrypted = encrypt_aes256gcm(&key, content.as_bytes());
        if let Err(e) = std::fs::write(&self.jsonl_path, &encrypted) {
            tracing::warn!("No se pudo escribir {}: {}", self.jsonl_path.display(), e);
        }
    }

    pub fn ingest_from_browser(&self, examples: Vec<TrainingExample>, source: &str) -> usize {
        let current_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let mut ingested: Vec<TrainingExample> = examples.into_iter().map(|mut example| {
            if example.source.is_empty() {
                example.source = source.to_string();
            }
            if example.collected_at == 0 {
                example.collected_at = current_timestamp;
            }
            example
        }).collect();
        let ingested_count = ingested.len();
        self.append_jsonl(&ingested);
        let mut inner = self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        inner.total_examples_processed += ingested_count;
        inner.examples.append(&mut ingested);
        tracing::info!("Ingested {} training examples from '{}'", ingested_count, source);
        ingested_count
    }

    pub fn ingest_browser_data(&self, examples: Vec<BrowserExample>, source: &str) -> usize {
        let current_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let training_examples: Vec<TrainingExample> = examples.into_iter().map(|browser_example| TrainingExample {
            url: browser_example.url,
            title: browser_example.title,
            content: browser_example.content,
            content_type: browser_example.content_type,
            metadata: browser_example.metadata,
            source: source.to_string(),
            collected_at: current_timestamp,
        }).collect();
        self.ingest_from_browser(training_examples, source)
    }

    pub async fn auto_train(&self, _model_name: &str) -> Result<String> {
        let examples = {
            let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            if inner.examples.is_empty() {
                return Err(anyhow!(
                    "No hay datos de entrenamiento. Umbra debe operar primero."
                ));
            }
            inner.examples.clone()
        };
        let num_examples = examples.len();

        const DEFAULT_INPUT_DIM: usize = 64;
        const DEFAULT_LATENT_DIM: usize = 32;
        const TRAIN_SEED: u64 = 42;
        let input_dim = DEFAULT_INPUT_DIM;
        let latent_dim = DEFAULT_LATENT_DIM;
        let mut model = crate::engine::JepaModel::new(input_dim, latent_dim);
        model.random_init(TRAIN_SEED);

        let report = model.train_from_examples(&examples)?;

        // --- HSAQ Compression ---
        use crate::engine::hsaq::{HsaqCompressor, LayerPrecision};
        use std::collections::HashMap;

        let encoder_size = (model.encoder_weights.len() * 4) as u64;
        let predictor_size = (model.predictor_weights.len() * 4) as u64;
        let decoder_size = (model.decoder_weights.len() * 4) as u64;
        let embedding_size = (model.embedding.len() * 4) as u64;

        let mut compressor = HsaqCompressor::new();
        compressor.analyze(&[
            ("encoder".into(), encoder_size, 0.9),
            ("predictor".into(), predictor_size, 0.6),
            ("decoder".into(), decoder_size, 0.6),
            ("embedding".into(), embedding_size, 0.15),
        ]);

        let compressed_encoder = compressor.compress_weights(&model.encoder_weights, &LayerPrecision::FP16);
        let compressed_predictor = compressor.compress_weights(&model.predictor_weights, &LayerPrecision::INT8);
        let compressed_decoder = compressor.compress_weights(&model.decoder_weights, &LayerPrecision::INT8);
        let compressed_embedding = compressor.compress_weights(&model.embedding, &LayerPrecision::BIN4);

        let compression_ratio = compressor.compression_ratio;

        let mut hsaq_weights = HashMap::new();
        hsaq_weights.insert("encoder".into(), compressed_encoder);
        hsaq_weights.insert("predictor".into(), compressed_predictor);
        hsaq_weights.insert("decoder".into(), compressed_decoder);
        hsaq_weights.insert("embedding".into(), compressed_embedding);
        model.compressed_weights = Some(hsaq_weights);

        model.header.hsaq_version = Some("3.0.0".into());
        model.header.compression = Some(compression_ratio);
        model.header.training = Some(crate::engine::jepa_model::JepaTrainingMeta {
            dataset: format!("{} ejemplos", num_examples),
            steps: report.epochs,
            final_loss: report.final_loss,
            accuracy: report.accuracy,
            timestamp: chrono::Utc::now().to_rfc3339(),
        });

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let output_name = format!("jepa-trained-{}", now);
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let models_dir = PathBuf::from(&home).join(".umbra/models");
        std::fs::create_dir_all(&models_dir).ok();
        let output_path = model.convert_to_materia(&output_name, &models_dir)?;

        {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.last_train_time = Some(Instant::now());
        }

        tracing::info!(
            "Modelo entrenado desde {} ejemplos: {} (loss: {:.6}, accuracy: {:.4}, hsaq: {:.2}x)",
            num_examples, output_path, report.final_loss, report.accuracy, compression_ratio
        );

        Ok(output_path)
    }

    pub fn stats(&self) -> serde_json::Value {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let mut sources: HashMap<String, usize> = HashMap::new();
        for example in &inner.examples {
            *sources.entry(example.source.clone()).or_insert(0) += 1;
        }
        const TOKEN_ESTIMATE_DIVISOR: usize = 4;
        let estimated_tokens: usize = inner.examples.iter()
            .map(|example| example.content.len() / TOKEN_ESTIMATE_DIVISOR + example.title.len() / TOKEN_ESTIMATE_DIVISOR)
            .sum();

        serde_json::json!({
            "total_examples": inner.examples.len(),
            "total_examples_processed": inner.total_examples_processed,
            "sources": sources,
            "estimated_tokens": estimated_tokens,
            "last_train_time": inner.last_train_time.map(|t| t.elapsed().as_secs_f64()),
        })
    }

    pub fn get_training_data(&self) -> Vec<TrainingExample> {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.examples.clone()
    }
}
