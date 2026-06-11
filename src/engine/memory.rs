use anyhow::{Result, anyhow};
use std::sync::Arc;
use synapsis::infrastructure::database::Database;
use synapsis::{Memory, MemoryPort, SessionId, StoragePort};

const DEFAULT_SEARCH_LIMIT: usize = 5;

pub trait MemoryRepository: Send + Sync {
    fn save(&self, agent_id: &str, prompt: &str, response: &str) -> Result<()>;
    fn recall(&self, query: &str, limit: usize) -> Result<Vec<String>>;
}

pub struct UnitOfWork {
    repo: Arc<dyn MemoryRepository>,
    pending: Vec<MemoryOperation>,
}

enum MemoryOperation {
    Save { agent_id: String, prompt: String, response: String },
}

impl UnitOfWork {
    pub fn new(repo: Arc<dyn MemoryRepository>) -> Self {
        Self {
            repo,
            pending: Vec::new(),
        }
    }

    pub fn add_save(&mut self, agent_id: String, prompt: String, response: String) {
        self.pending.push(MemoryOperation::Save { agent_id, prompt, response });
    }

    pub fn commit(&self) -> Result<()> {
        for op in &self.pending {
            match op {
                MemoryOperation::Save { agent_id, prompt, response } => {
                    self.repo.save(agent_id, prompt, response)?;
                }
            }
        }
        Ok(())
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

#[derive(Clone)]
pub struct SqliteMemoryRepository {
    db: Arc<Database>,
}

impl SqliteMemoryRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

impl MemoryRepository for SqliteMemoryRepository {
    fn save(&self, agent_id: &str, prompt: &str, response: &str) -> Result<()> {
        let content = format!("Q: {}\nA: {}", prompt, response);
        let session_id = SessionId::new(agent_id);
        let memory = Memory::new(session_id, content);
        self.db.save_memory(&memory).map_err(|e| anyhow!(e))?;
        Ok(())
    }

    fn recall(&self, query: &str, limit: usize) -> Result<Vec<String>> {
        let results = self.db.search_fts(query, None, limit as i32).map_err(|e| anyhow!(e))?;
        Ok(results
            .iter()
            .map(|entry| {
                let title = entry["title"].as_str().unwrap_or("");
                let content = entry["content"].as_str().unwrap_or("");
                format!("{}: {}", title, content)
            })
            .collect())
    }
}

#[derive(Clone)]
pub struct SynapsisMemory {
    repo: Arc<dyn MemoryRepository>,
}

impl SynapsisMemory {
    pub fn new() -> Self {
        let db = Arc::new(Database::new());
        db.init().ok();
        let repo = Arc::new(SqliteMemoryRepository::new(db));
        Self { repo }
    }

    pub fn with_repository(repo: Arc<dyn MemoryRepository>) -> Self {
        Self { repo }
    }

    pub async fn save(&self, prompt: &str, response: &str) -> Result<()> {
        self.repo.save("umbra-agent", prompt, response)
    }

    pub async fn recall(&self, query: &str) -> Result<Vec<String>> {
        self.repo.recall(query, DEFAULT_SEARCH_LIMIT)
    }

    pub fn begin_unit_of_work(&self) -> UnitOfWork {
        UnitOfWork::new(self.repo.clone())
    }
}
