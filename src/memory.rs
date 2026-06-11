use anyhow::Result;
use std::sync::Arc;
use synapsis::{
    Observation, ObservationType, SessionId,
    Timestamp,
};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionId,
    pub project: String,
    pub cwd: String,
    pub started_at: Timestamp,
    pub ended_at: Option<Timestamp>,
    pub observation_count: i64,
}

impl Session {
    pub fn new(id: SessionId, project: String, cwd: String) -> Self {
        Self {
            id,
            project,
            cwd,
            started_at: Timestamp::now(),
            ended_at: None,
            observation_count: 0,
        }
    }
}

pub struct MemoryEngine {
    observations: Arc<RwLock<Vec<Observation>>>,
    sessions: Arc<RwLock<Vec<Session>>>,
    current_session: Arc<RwLock<Option<Session>>>,
}

impl MemoryEngine {
    pub fn new() -> Self {
        Self {
            observations: Arc::new(RwLock::new(Vec::new())),
            sessions: Arc::new(RwLock::new(Vec::new())),
            current_session: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start_session(&self, project: &str) -> Result<SessionId> {
        let session_id = SessionId::new(&uuid::Uuid::new_v4().to_string());
        let session = Session::new(
            session_id.clone(),
            project.to_string(),
            std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        );

        let mut sessions = self.sessions.write().await;
        sessions.push(session.clone());

        let mut current = self.current_session.write().await;
        *current = Some(session);

        Ok(session_id)
    }

    pub async fn end_session(&self) -> Result<()> {
        let mut current = self.current_session.write().await;
        if let Some(mut session) = current.take() {
            session.ended_at = Some(Timestamp::now());
            let mut sessions = self.sessions.write().await;
            if let Some(s) = sessions.iter_mut().find(|s| s.id == session.id) {
                s.ended_at = session.ended_at;
            }
        }
        Ok(())
    }

    pub async fn add_observation(
        &self,
        obs_type: ObservationType,
        title: String,
        content: String,
    ) -> Result<Observation> {
        let session_id = {
            let current = self.current_session.read().await;
            current
                .as_ref()
                .map(|s| s.id.clone())
                .unwrap_or_else(|| SessionId::new(&uuid::Uuid::new_v4().to_string()))
        };

        let observation = Observation::new(session_id, obs_type, title, content);

        let mut observations = self.observations.write().await;
        observations.push(observation.clone());

        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.iter_mut().find(|s| s.id.as_str() == observation.session_id) {
            session.observation_count += 1;
        }

        Ok(observation)
    }

    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<Observation>> {
        let observations = self.observations.read().await;
        let query_lower = query.to_lowercase();

        let mut results: Vec<(Observation, f64)> = observations
            .iter()
            .filter(|_obs| true)
            .filter_map(|obs| {
                let mut score = 0.0;
                let title_lower = obs.title.to_lowercase();
                let content_lower = obs.content.to_lowercase();

                if title_lower.contains(&query_lower) {
                    score += 10.0;
                }
                if content_lower.contains(&query_lower) {
                    score += 5.0;
                }

                for word in query_lower.split_whitespace() {
                    if title_lower.contains(word) {
                        score += 2.0;
                    }
                    if content_lower.contains(word) {
                        score += 1.0;
                    }
                }

                if score > 0.0 {
                    Some((obs.clone(), score))
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        Ok(results.into_iter().map(|(obs, _)| obs).collect())
    }

    pub async fn get_recent(&self, limit: usize) -> Result<Vec<Observation>> {
        let observations = self.observations.read().await;
        let mut recent: Vec<Observation> = observations
            .iter()
            .filter(|_obs| true)
            .cloned()
            .collect();

        recent.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        recent.truncate(limit);

        Ok(recent)
    }

    pub async fn get_context(&self, query: &str) -> Result<String> {
        let relevant = self.search(query, 5).await?;
        let recent = self.get_recent(3).await?;

        let mut context = String::new();

        if !relevant.is_empty() {
            context.push_str("## Memorias Relevantes\n");
            for obs in relevant {
                context.push_str(&format!("- **{}**: {}\n", obs.title, obs.content));
            }
            context.push('\n');
        }

        if !recent.is_empty() {
            context.push_str("## Memorias Recientes\n");
            for obs in recent {
                context.push_str(&format!("- **{}**: {}\n", obs.title, obs.content));
            }
        }

        Ok(context)
    }

    pub async fn stats(&self) -> (usize, usize) {
        let observations = self.observations.read().await;
        let sessions = self.sessions.read().await;
        (observations.len(), sessions.len())
    }
}
