// Zone 4 — Infrastructure
use std::path::PathBuf;

use crate::domain::models::TrainingExample;

pub struct JsonlTrainingRepository {
    data_dir: PathBuf,
}

impl JsonlTrainingRepository {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    pub fn save_examples(&self, examples: &[TrainingExample], source: &str) -> Result<usize, String> {
        std::fs::create_dir_all(&self.data_dir).map_err(|e| e.to_string())?;
        let path = self.data_dir.join(format!("{}.jsonl", source));
        let mut count = 0usize;
        for example in examples {
            let line = serde_json::to_string(example).map_err(|e| e.to_string())?;
            std::fs::write(&path, format!("{}\n", line)).map_err(|e| e.to_string())?;
            count += 1;
        }
        Ok(count)
    }

    pub fn load_examples(&self, source: &str) -> Result<Vec<TrainingExample>, String> {
        let path = self.data_dir.join(format!("{}.jsonl", source));
        if !path.exists() {
            return Ok(vec![]);
        }
        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let mut examples = Vec::new();
        for line in content.lines() {
            if let Ok(ex) = serde_json::from_str::<TrainingExample>(line) {
                examples.push(ex);
            }
        }
        Ok(examples)
    }
}
