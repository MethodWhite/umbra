// Zone 1 — Desktop/UI
use anyhow::Result;
use serde::Serialize;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::agent_memory::EmotionalState;

pub type MemoryId = u64;

#[derive(Debug, Clone, Serialize)]
pub struct MemoryEntry {
    pub id: MemoryId,
    pub agent_id: String,
    pub session_id: String,
    pub prompt: String,
    pub response: String,
    pub emotion: EmotionalState,
    pub agent_personality: String,
    pub tags: Vec<String>,
    pub created_at: u64,
}

impl MemoryEntry {
    pub fn content(&self) -> String {
        format!("Q: {}\nA: {}", self.prompt, self.response)
    }
}

pub trait MemoryRepository: Send + Sync {
    fn save(&self, entry: MemoryEntry) -> Result<()>;
    fn search(&self, query: &str, emotion: Option<&EmotionalState>, limit: usize) -> Result<Vec<MemoryEntry>>;
    fn recall_by_agent(&self, agent_id: &str, limit: usize) -> Result<Vec<MemoryEntry>>;
    fn recent(&self, limit: usize) -> Result<Vec<MemoryEntry>>;
    fn count(&self) -> usize;
    fn delete(&self, id: MemoryId) -> Result<()>;
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn text_relevance(query: &str, text: &str) -> f64 {
    let q = query.to_lowercase();
    let t = text.to_lowercase();
    let mut score = 0.0;
    if t.contains(&q) { score += 10.0; }
    if t.starts_with(&q) { score += 5.0; }
    for word in q.split_whitespace() {
        score += t.matches(word).count() as f64;
    }
    score
}

fn emotional_similarity(a: &EmotionalState, b: &EmotionalState) -> f64 {
    let val = 1.0_f64 - (a.valence - b.valence).abs() as f64;
    let aro = 1.0_f64 - (a.arousal - b.arousal).abs() as f64;
    val * 0.5 + aro * 0.3 + if a.primary == b.primary { 0.2 } else { 0.0 }
}

pub struct InMemoryMemoryStore {
    entries: RwLock<Vec<MemoryEntry>>,
    next_id: RwLock<MemoryId>,
}

impl InMemoryMemoryStore {
    pub fn new() -> Self {
        Self { entries: RwLock::new(Vec::new()), next_id: RwLock::new(1) }
    }
}

impl MemoryRepository for InMemoryMemoryStore {
    fn save(&self, mut entry: MemoryEntry) -> Result<()> {
        let mut id = self.next_id.write().unwrap();
        entry.id = *id;
        entry.created_at = now_secs();
        self.entries.write().unwrap().push(entry);
        *id += 1;
        Ok(())
    }

    fn search(&self, query: &str, emotion: Option<&EmotionalState>, limit: usize) -> Result<Vec<MemoryEntry>> {
        let entries = self.entries.read().unwrap();
        let mut scored: Vec<(f64, MemoryEntry)> = entries.iter().map(|e| {
            let text_score = text_relevance(query, &e.content());
            let emotion_score = emotion.map(|em| emotional_similarity(em, &e.emotion)).unwrap_or(0.0);
            (text_score * 0.7 + emotion_score * 0.3, e.clone())
        }).filter(|(s, _)| *s > 0.0).collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        Ok(scored.into_iter().take(limit).map(|(_, e)| e).collect())
    }

    fn recall_by_agent(&self, agent_id: &str, limit: usize) -> Result<Vec<MemoryEntry>> {
        let mut results: Vec<MemoryEntry> = self.entries.read().unwrap().iter()
            .filter(|e| e.agent_id == agent_id).cloned().collect();
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

    fn delete(&self, id: MemoryId) -> Result<()> {
        self.entries.write().unwrap().retain(|e| e.id != id);
        Ok(())
    }
}

pub struct MemoryEngine {
    repo: Arc<dyn MemoryRepository>,
}

impl MemoryEngine {
    pub fn new() -> Self {
        Self { repo: Arc::new(InMemoryMemoryStore::new()) }
    }

    pub fn with_repository(repo: Arc<dyn MemoryRepository>) -> Self {
        Self { repo }
    }

    pub fn save(&self, agent_id: &str, prompt: &str, response: &str, emotion: &EmotionalState, personality: &str) -> Result<MemoryEntry> {
        let entry = MemoryEntry {
            id: 0, agent_id: agent_id.to_string(), session_id: String::new(),
            prompt: prompt.to_string(), response: response.to_string(),
            emotion: emotion.clone(), agent_personality: personality.to_string(),
            tags: vec![], created_at: 0,
        };
        self.repo.save(entry.clone())?;
        Ok(entry)
    }

    pub fn search(&self, query: &str, emotion: Option<&EmotionalState>) -> Result<Vec<MemoryEntry>> {
        self.repo.search(query, emotion, 5)
    }

    pub fn recent(&self, limit: usize) -> Result<Vec<MemoryEntry>> {
        self.repo.recent(limit)
    }

    pub fn recall_by_agent(&self, agent_id: &str, limit: usize) -> Result<Vec<MemoryEntry>> {
        self.repo.recall_by_agent(agent_id, limit)
    }

    pub fn get_context(&self, query: &str, emotion: Option<&EmotionalState>) -> Result<String> {
        let entries = self.repo.search(query, emotion, 5)?;
        if entries.is_empty() {
            return Ok(String::new());
        }
        let mut ctx = String::from("## Emotional Memory Context\n");
        for e in &entries {
            ctx.push_str(&format!("- [{}] {} (emotion: {}, personality: {})\n",
                e.agent_id, e.content(), e.emotion.label, e.agent_personality));
        }
        Ok(ctx)
    }

    pub fn stats(&self) -> (usize, usize) {
        let total = self.repo.count();
        let entries = self.repo.recent(1000).unwrap_or_default();
        let mut agents: Vec<String> = entries.into_iter().map(|e| e.agent_id).collect();
        agents.sort();
        agents.dedup();
        (total, agents.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_memory::EmotionalState;

    fn test_repo() -> impl MemoryRepository {
        InMemoryMemoryStore::new()
    }

    #[test]
    fn test_save_and_count() {
        let repo = test_repo();
        assert_eq!(repo.count(), 0);
        let entry = MemoryEntry {
            id: 0, agent_id: "test".into(), session_id: String::new(),
            prompt: "hello".into(), response: "world".into(),
            emotion: EmotionalState::calm(), agent_personality: "neutral".into(),
            tags: vec![], created_at: 0,
        };
        repo.save(entry).unwrap();
        assert_eq!(repo.count(), 1);
    }

    #[test]
    fn test_search_finds_relevant() {
        let repo = test_repo();
        let entry = MemoryEntry {
            id: 0, agent_id: "test".into(), session_id: String::new(),
            prompt: "What is the capital of France?".into(),
            response: "Paris".into(),
            emotion: EmotionalState::calm(), agent_personality: "neutral".into(),
            tags: vec![], created_at: 0,
        };
        repo.save(entry).unwrap();
        let results = repo.search("France", None, 5).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_no_match() {
        let repo = test_repo();
        let entry = MemoryEntry {
            id: 0, agent_id: "test".into(), session_id: String::new(),
            prompt: "hello".into(), response: "world".into(),
            emotion: EmotionalState::calm(), agent_personality: "neutral".into(),
            tags: vec![], created_at: 0,
        };
        repo.save(entry).unwrap();
        let results = repo.search("nonexistent", None, 5).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_recall_by_agent() {
        let repo = test_repo();
        for i in 0..3 {
            let entry = MemoryEntry {
                id: 0, agent_id: format!("agent{}", i), session_id: String::new(),
                prompt: "test".into(), response: "data".into(),
                emotion: EmotionalState::calm(), agent_personality: "neutral".into(),
                tags: vec![], created_at: 0,
            };
            repo.save(entry).unwrap();
        }
        let results = repo.recall_by_agent("agent1", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].agent_id, "agent1");
    }

    #[test]
    fn test_delete_removes_entry() {
        let repo = test_repo();
        let mut entry = MemoryEntry {
            id: 0, agent_id: "test".into(), session_id: String::new(),
            prompt: "test".into(), response: "data".into(),
            emotion: EmotionalState::calm(), agent_personality: "neutral".into(),
            tags: vec![], created_at: 0,
        };
        repo.save(entry.clone()).unwrap();
        let entries = repo.recent(10).unwrap();
        let id = entries[0].id;
        repo.delete(id).unwrap();
        assert_eq!(repo.count(), 0);
    }

    #[test]
    fn test_engine_save_and_stats() {
        let engine = MemoryEngine::new();
        engine.save("agent1", "hello", "world", &EmotionalState::calm(), "analytical").unwrap();
        let (total, agents) = engine.stats();
        assert_eq!(total, 1);
        assert_eq!(agents, 1);
    }
}
