use anyhow::{Result, anyhow};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingExample {
    pub url: String,
    pub title: String,
    pub content: String,
    pub content_type: String,
    pub metadata: HashMap<String, String>,
    pub source: String,
    pub collected_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainReport {
    pub epochs: usize,
    pub final_loss: f32,
    pub accuracy: f32,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JepaModelHeader {
    pub magic: String,
    pub version: String,
    pub model_type: String,
    pub input_dim: usize,
    pub latent_dim: usize,
    pub num_layers: usize,
    pub hsaq_version: Option<String>,
    pub compression: Option<f32>,
    pub created: String,
    pub training: Option<JepaTrainingMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JepaTrainingMeta {
    pub dataset: String,
    pub steps: usize,
    pub final_loss: f32,
    pub accuracy: f32,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct JepaModel {
    pub header: JepaModelHeader,
    pub encoder_weights: Vec<f32>,
    pub predictor_weights: Vec<f32>,
    pub decoder_weights: Vec<f32>,
    pub embedding: Vec<f32>,
    pub compressed_weights: Option<HashMap<String, Vec<u8>>>,
}

impl JepaModel {
    pub fn new(input_dim: usize, latent_dim: usize) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            header: JepaModelHeader {
                magic: "NUM_JEPA_V3".into(),
                version: "3.0.0".into(),
                model_type: "jepa".into(),
                input_dim,
                latent_dim,
                num_layers: 4,
                hsaq_version: None,
                compression: None,
                created: now,
                training: None,
            },
            encoder_weights: Vec::new(),
            predictor_weights: Vec::new(),
            decoder_weights: Vec::new(),
            embedding: vec![0.0; latent_dim],
            compressed_weights: None,
        }
    }

    pub fn save_jepa(&self, path: &Path) -> Result<()> {
        let bytes = rmp_serde::to_vec(&self.header)?;
        let enc = rmp_serde::to_vec(&self.encoder_weights)?;
        let pred = rmp_serde::to_vec(&self.predictor_weights)?;
        let dec = rmp_serde::to_vec(&self.decoder_weights)?;
        let emb = rmp_serde::to_vec(&self.embedding)?;

        let mut buf = Vec::new();
        buf.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
        buf.extend_from_slice(&bytes);
        buf.extend_from_slice(&(enc.len() as u64).to_le_bytes());
        buf.extend_from_slice(&enc);
        buf.extend_from_slice(&(pred.len() as u64).to_le_bytes());
        buf.extend_from_slice(&pred);
        buf.extend_from_slice(&(dec.len() as u64).to_le_bytes());
        buf.extend_from_slice(&dec);
        buf.extend_from_slice(&(emb.len() as u64).to_le_bytes());
        buf.extend_from_slice(&emb);

        std::fs::write(path, &buf)?;
        Ok(())
    }

    pub fn load_jepa(path: &Path) -> Result<Self> {
        let buf = std::fs::read(path)?;
        let mut pos = 0;

        let read_section = |buf: &[u8], pos: &mut usize| -> Result<Vec<u8>> {
            if *pos + 8 > buf.len() {
                return Err(anyhow!("Formato .jepa inválido"));
            }
            let len = u64::from_le_bytes(buf[*pos..*pos + 8].try_into()?);
            *pos += 8;
            if *pos + len as usize > buf.len() {
                return Err(anyhow!("Sección .jepa truncada"));
            }
            let section = buf[*pos..*pos + len as usize].to_vec();
            *pos += len as usize;
            Ok(section)
        };

        let header_bytes = read_section(&buf, &mut pos)?;
        let header: JepaModelHeader = rmp_serde::from_slice(&header_bytes)?;

        let enc_bytes = read_section(&buf, &mut pos)?;
        let encoder_weights: Vec<f32> = rmp_serde::from_slice(&enc_bytes)?;

        let pred_bytes = read_section(&buf, &mut pos)?;
        let predictor_weights: Vec<f32> = rmp_serde::from_slice(&pred_bytes)?;

        let dec_bytes = read_section(&buf, &mut pos)?;
        let decoder_weights: Vec<f32> = rmp_serde::from_slice(&dec_bytes)?;

        let emb_bytes = read_section(&buf, &mut pos)?;
        let embedding: Vec<f32> = rmp_serde::from_slice(&emb_bytes)?;

        Ok(Self { header, encoder_weights, predictor_weights, decoder_weights, embedding, compressed_weights: None })
    }

    pub fn predict(&self, input: &[f32]) -> Vec<f32> {
        if input.len() != self.header.input_dim {
            return vec![0.0; self.header.latent_dim];
        }

        let mut latent = self.encoder(input);
        latent = self.predictor(&latent);
        let output = self.decoder(&latent);
        output
    }

    pub fn encoder(&self, input: &[f32]) -> Vec<f32> {
        let mut output = vec![0.0; self.header.latent_dim];
        for latent_idx in 0..self.header.latent_dim.min(self.encoder_weights.len()) {
            let mut sum = 0.0f32;
            for input_idx in 0..input.len().min(self.header.input_dim) {
                let weight_idx = latent_idx * self.header.input_dim + input_idx;
                if weight_idx < self.encoder_weights.len() {
                    sum += input[input_idx] * self.encoder_weights[weight_idx];
                }
            }
            output[latent_idx] = sum.tanh();
        }
        output
    }

    pub fn predictor(&self, latent: &[f32]) -> Vec<f32> {
        let mut output = latent.to_vec();
        for latent_idx in 0..latent.len().min(self.predictor_weights.len().min(latent.len())) {
            if latent_idx < self.predictor_weights.len() {
                output[latent_idx] = (latent[latent_idx] * self.predictor_weights[latent_idx]).tanh();
            }
        }
        output
    }

    pub fn decoder(&self, latent: &[f32]) -> Vec<f32> {
        let mut output = vec![0.0f32; self.header.input_dim];
        for input_idx in 0..self.header.input_dim {
            let mut sum = 0.0f32;
            for latent_idx in 0..latent.len().min(self.header.latent_dim) {
                let weight_idx = input_idx * self.header.latent_dim + latent_idx;
                if weight_idx < self.decoder_weights.len() {
                    sum += latent[latent_idx] * self.decoder_weights[weight_idx];
                }
            }
            output[input_idx] = sum;
        }
        output
    }

    pub fn train_epoch(&mut self, inputs: &[Vec<f32>], targets: &[Vec<f32>], learning_rate: f32) -> f32 {
        let mut total_loss = 0.0f32;
        let encoder_gradients = vec![0.0f32; self.encoder_weights.len()];
        let predictor_gradients = vec![0.0f32; self.predictor_weights.len()];
        let mut decoder_gradients = vec![0.0f32; self.decoder_weights.len()];

        for (input, target) in inputs.iter().zip(targets.iter()) {
            let latent = self.encoder(input);
            let predicted = self.predictor(&latent);
            let output = self.decoder(&predicted);

            let mut loss = 0.0f32;
            for (output_val, target_val) in output.iter().zip(target.iter()) {
                let diff = output_val - target_val;
                loss += diff * diff;
            }
            loss /= target.len() as f32;
            total_loss += loss;

            for output_idx in 0..output.len().min(self.decoder_weights.len().min(latent.len())) {
                for latent_idx in 0..self.header.latent_dim.min(predicted.len()) {
                    let weight_idx = output_idx * self.header.latent_dim + latent_idx;
                    if weight_idx < decoder_gradients.len() {
                        decoder_gradients[weight_idx] += (output[output_idx] - target[output_idx]) * predicted[latent_idx];
                    }
                }
            }
        }

        let batch_size = inputs.len() as f32;
        for (weight, gradient) in self.encoder_weights.iter_mut().zip(encoder_gradients.iter()) {
            *weight -= learning_rate * gradient / batch_size;
        }
        for (weight, gradient) in self.predictor_weights.iter_mut().zip(predictor_gradients.iter()) {
            *weight -= learning_rate * gradient / batch_size;
        }
        for (weight, gradient) in self.decoder_weights.iter_mut().zip(decoder_gradients.iter()) {
            *weight -= learning_rate * gradient / batch_size;
        }

        total_loss / batch_size
    }

    pub fn convert_to_materia(&self, name: &str, models_dir: &Path) -> Result<String> {
        let output_path = models_dir.join(format!("{}.materia", name));

        let mut materia_content = serde_json::json!({
            "materia": "umbra_sub_agent",
            "version": "1.0",
            "converted_from": "jepa",
            "source_model": name,
            "jepa_header": self.header,
            "agent": {
                "name": name,
                "version": "1.0.0",
                "description": format!("Modelo JEPA convertido a .materia — dims: {}→{}", self.header.input_dim, self.header.latent_dim),
                "model": {
                    "name": name,
                    "familia": "jepa",
                    "context": 2048,
                },
                "capabilities": ["prediccion", "analisis"],
            },
            "weights_encoded": base64::engine::general_purpose::STANDARD.encode(
                rmp_serde::to_vec(&self.encoder_weights).unwrap_or_default()
            ),
        });

        if let Some(ref compressed) = self.compressed_weights {
            let hsaq_map: serde_json::Map<String, serde_json::Value> = compressed.iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(base64::engine::general_purpose::STANDARD.encode(v))))
                .collect();
            materia_content["hsaq_compressed"] = serde_json::Value::Object(hsaq_map);
        }

        let content = serde_json::to_string_pretty(&materia_content)?;
        std::fs::write(&output_path, &content)
            .map_err(|e| anyhow!("Error escribiendo .materia: {}", e))?;

        tracing::info!("Modelo JEPA '{}' convertido a .materia en {}", name, output_path.display());
        Ok(output_path.to_string_lossy().to_string())
    }

    pub fn train_from_examples(&mut self, examples: &[TrainingExample]) -> Result<TrainReport> {
        const TRAINING_EPOCHS: usize = 10;
        const LEARNING_RATE: f32 = 0.01;
        let epochs = TRAINING_EPOCHS;
        let input_dim = self.header.input_dim;

        let inputs: Vec<Vec<f32>> = examples.iter()
            .map(|example| {
                let combined = format!("{} {} {}", example.title, example.content, example.content_type);
                Self::text_to_embedding(&combined, input_dim)
            })
            .collect();

        let mut final_loss = 0.0;
        for epoch in 0..epochs {
            let loss = self.train_epoch(&inputs, &inputs, LEARNING_RATE);
            final_loss = loss;
            tracing::debug!("JEPA epoch {}/{} — loss: {:.6}", epoch + 1, epochs, loss);
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let output_name = format!("jepa-trained-{}", now);
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let models_dir = PathBuf::from(&home).join(".umbra/models");
        std::fs::create_dir_all(&models_dir).ok();
        let output_path = self.convert_to_materia(&output_name, &models_dir)?;

        Ok(TrainReport {
            epochs,
            final_loss,
            accuracy: (-final_loss).exp(),
            output_path,
        })
    }

    fn text_to_embedding(text: &str, embedding_dim: usize) -> Vec<f32> {
        let mut embedding = vec![0.0f32; embedding_dim];
        let bytes = text.as_bytes();
        const HASH_MULTIPLIER: u64 = 31;
        for (byte_idx, &byte_val) in bytes.iter().enumerate() {
            let bucket = (byte_idx as u64 * HASH_MULTIPLIER + byte_val as u64) % embedding_dim as u64;
            embedding[bucket as usize] += 1.0;
        }
        let norm: f32 = embedding.iter().map(|val| val * val).sum::<f32>().sqrt();
        if norm > 0.0 {
            for component in embedding.iter_mut() {
                *component /= norm;
            }
        }
        embedding
    }

    pub fn random_init(&mut self, seed: u64) {
        use rand::{Rng, SeedableRng};
        const WEIGHT_RANGE_MIN: f32 = -0.1;
        const WEIGHT_RANGE_MAX: f32 = 0.1;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let encoder_count = self.header.input_dim * self.header.latent_dim;
        self.encoder_weights = (0..encoder_count)
            .map(|_| rng.gen_range(WEIGHT_RANGE_MIN..WEIGHT_RANGE_MAX)).collect();
        self.predictor_weights = (0..self.header.latent_dim)
            .map(|_| rng.gen_range(WEIGHT_RANGE_MIN..WEIGHT_RANGE_MAX)).collect();
        self.decoder_weights = (0..encoder_count)
            .map(|_| rng.gen_range(WEIGHT_RANGE_MIN..WEIGHT_RANGE_MAX)).collect();
    }
}
