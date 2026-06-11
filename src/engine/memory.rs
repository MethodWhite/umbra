use anyhow::Result;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::agent_memory::EmotionalState;

const DEFAULT_SEARCH_LIMIT: usize = 5;

#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub id: u64,
    pub agent_id: String,
    pub prompt: String,
    pub response: String,
    pub emotion: EmotionalState,
    pub agent_personality: String,
    pub created_at: u64,
}

impl MemoryEntry {
    pub fn content(&self) -> String {
        format!("Q: {}\nA: {}", self.prompt, self.response)
    }
}

pub trait MemoryRepository: Send + Sync {
    fn save(&self, agent_id: &str, prompt: &str, response: &str, emotion: &EmotionalState, personality: &str) -> Result<()>;
    fn recall(&self, query: &str, emotion: Option<&EmotionalState>, limit: usize) -> Result<Vec<MemoryEntry>>;
    fn recall_by_agent(&self, agent_id: &str, limit: usize) -> Result<Vec<MemoryEntry>>;
    fn recent(&self, limit: usize) -> Result<Vec<MemoryEntry>>;
    fn count(&self) -> usize;
}

pub struct UnitOfWork {
    repo: Arc<dyn MemoryRepository>,
    pending: Vec<MemoryOperation>,
}

enum MemoryOperation {
    Save { agent_id: String, prompt: String, response: String, emotion: EmotionalState, personality: String },
}

impl UnitOfWork {
    pub fn new(repo: Arc<dyn MemoryRepository>) -> Self {
        Self { repo, pending: Vec::new() }
    }

    pub fn add_save(&mut self, agent_id: String, prompt: String, response: String, emotion: EmotionalState, personality: String) {
        self.pending.push(MemoryOperation::Save { agent_id, prompt, response, emotion, personality });
    }

    pub fn commit(&self) -> Result<()> {
        for op in &self.pending {
            match op {
                MemoryOperation::Save { agent_id, prompt, response, emotion, personality } => {
                    self.repo.save(agent_id, prompt, response, emotion, personality)?;
                }
            }
        }
        Ok(())
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

pub struct InMemoryMemoryRepository {
    entries: RwLock<Vec<MemoryEntry>>,
    next_id: RwLock<u64>,
}

impl InMemoryMemoryRepository {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
            next_id: RwLock::new(1),
        }
    }
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn text_relevance(query: &str, text: &str) -> f64 {
    let query_lower = query.to_lowercase();
    let text_lower = text.to_lowercase();
    let mut score = 0.0;

    if text_lower.contains(&query_lower) {
        score += 10.0;
    }
    if text_lower.starts_with(&query_lower) {
        score += 5.0;
    }

    for word in query_lower.split_whitespace() {
        if text_lower.contains(word) {
            score += 2.0;
        }
        let count = text_lower.matches(word).count();
        score += count as f64 * 0.5;
    }

    score
}

fn emotional_similarity(a: &EmotionalState, b: &EmotionalState) -> f64 {
    let val = 1.0_f64 - (a.valence - b.valence).abs() as f64;
    let aro = 1.0_f64 - (a.arousal - b.arousal).abs() as f64;
    val * 0.5 + aro * 0.3 + if a.label == b.label { 0.2 } else { 0.0 }
}

impl MemoryRepository for InMemoryMemoryRepository {
    fn save(&self, agent_id: &str, prompt: &str, response: &str, emotion: &EmotionalState, personality: &str) -> Result<()> {
        let mut id = self.next_id.write().unwrap();
        let entry = MemoryEntry {
            id: *id,
            agent_id: agent_id.to_string(),
            prompt: prompt.to_string(),
            response: response.to_string(),
            emotion: emotion.clone(),
            agent_personality: personality.to_string(),
            created_at: now_secs(),
        };
        self.entries.write().unwrap().push(entry);
        *id += 1;
        Ok(())
    }

    fn recall(&self, query: &str, emotion: Option<&EmotionalState>, limit: usize) -> Result<Vec<MemoryEntry>> {
        let entries = self.entries.read().unwrap();
        let mut scored: Vec<(f64, &MemoryEntry)> = entries
            .iter()
            .map(|e| {
                let text_score = text_relevance(query, &e.content());
                let emotion_score = emotion.map(|em| emotional_similarity(em, &e.emotion)).unwrap_or(0.0);
                (text_score * 0.7 + emotion_score * 0.3, e)
            })
            .filter(|(s, _)| *s > 0.0)
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        Ok(scored.into_iter().take(limit).map(|(_, e)| e.clone()).collect())
    }

    fn recall_by_agent(&self, agent_id: &str, limit: usize) -> Result<Vec<MemoryEntry>> {
        let entries = self.entries.read().unwrap();
        let mut results: Vec<MemoryEntry> = entries
            .iter()
            .filter(|e| e.agent_id == agent_id)
            .cloned()
            .collect();
        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        results.truncate(limit);
        Ok(results)
    }

    fn recent(&self, limit: usize) -> Result<Vec<MemoryEntry>> {
        let mut entries = self.entries.read().unwrap().clone();
        entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        entries.truncate(limit);
        Ok(entries)
    }

    fn count(&self) -> usize {
        self.entries.read().unwrap().len()
    }
}

pub struct SynapsisMemory {
    repo: Arc<dyn MemoryRepository>,
    current_personality: RwLock<String>,
}

impl SynapsisMemory {
    pub fn new() -> Self {
        let repo = Arc::new(InMemoryMemoryRepository::new());
        Self {
            repo,
            current_personality: RwLock::new("analytical".into()),
        }
    }

    pub fn with_repository(repo: Arc<dyn MemoryRepository>) -> Self {
        Self {
            repo,
            current_personality: RwLock::new("analytical".into()),
        }
    }

    pub fn set_personality(&self, personality: &str) {
        *self.current_personality.write().unwrap() = personality.to_string();
    }

    pub async fn save(&self, prompt: &str, response: &str, emotion: &EmotionalState) -> Result<()> {
        let personality = self.current_personality.read().unwrap().clone();
        self.repo.save("umbra-agent", prompt, response, emotion, &personality)
    }

    pub async fn recall(&self, query: &str, emotion: Option<&EmotionalState>) -> Result<Vec<MemoryEntry>> {
        self.repo.recall(query, emotion, DEFAULT_SEARCH_LIMIT)
    }

    pub async fn recall_formatted(&self, query: &str, emotion: Option<&EmotionalState>) -> Result<Vec<String>> {
        let entries = self.repo.recall(query, emotion, DEFAULT_SEARCH_LIMIT)?;
        Ok(entries.iter().map(|e| {
            format!("[{}] {} (emotion: {}, personality: {})",
                e.agent_id, e.content(), e.emotion.label, e.agent_personality)
        }).collect())
    }

    pub fn begin_unit_of_work(&self) -> UnitOfWork {
        UnitOfWork::new(self.repo.clone())
    }

    pub fn stats(&self) -> (usize, usize) {
        let total = self.repo.count();
        let unique = {
            let entries = self.repo.recent(1000).unwrap_or_default();
            let mut agents: Vec<String> = entries.into_iter().map(|e| e.agent_id).collect();
            agents.sort();
            agents.dedup();
            agents.len()
        };
        (total, unique)
    }
}
