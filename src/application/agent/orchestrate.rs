// Zone 3 — Application

use crate::agent_memory::{AgentParams, EmotionalState};


/// Result from an agent task execution
#[derive(Debug, Clone)]
pub struct AgentResult {
    pub agent_id: String,
    pub output: String,
    pub emotion: EmotionalState,
    pub confidence: f32,
    pub tokens_used: u32,
}

/// Task to be executed by an agent
#[derive(Debug, Clone)]
pub struct AgentTask {
    pub id: String,
    pub description: String,
    pub context: String,
    pub required_capability: Option<String>,
}

pub struct AgentOrchestrator;

impl AgentOrchestrator {
    pub fn new() -> Self {
        Self
    }

    /// Select the best agent for a task based on capabilities and emotional state.
    pub fn select_agent<'a>(&self, agents: &'a [AgentParams], task: &AgentTask) -> Option<&'a AgentParams> {
        if agents.is_empty() {
            return None;
        }

        let mut scored: Vec<(f32, &AgentParams)> = agents.iter().map(|a| {
            let mut score = a.success_rate;

            // Prefer agents with appropriate capabilities
            if let Some(ref cap) = task.required_capability {
                match cap.as_str() {
                    "analysis" => score += a.capabilities.analysis * 0.3,
                    "quality" => score += a.capabilities.quality * 0.3,
                    "speed" => score += a.capabilities.speed * 0.3,
                    "creativity" => score += a.capabilities.creativity * 0.3,
                    _ => {}
                }
            }

            // Prefer agents in a positive emotional state
            if a.emotional_state.valence > 0.5 {
                score += 0.2;
            }

            // Prefer agents with more experience
            score += (a.total_tasks as f32).min(100.0) / 500.0;

            (score, a)
        }).collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.first().map(|(_, a)| *a)
    }

    /// Evaluate task result and determine follow-up.
    pub fn evaluate_result(&self, result: &AgentResult) -> &'static str {
        if result.confidence > 0.8 && result.tokens_used < 1000 {
            "complete"
        } else if result.confidence > 0.5 {
            "review"
        } else {
            "reassign"
        }
    }

    /// Create an appropriate emotional response based on task outcome.
    pub fn emotional_response(&self, success: bool) -> EmotionalState {
        if success {
            EmotionalState::happy()
        } else {
            EmotionalState::curious()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_memory::{AgentCapabilities, PerformanceHistory};

    fn make_test_agent(name: &str, success_rate: f32) -> AgentParams {
        AgentParams {
            id: name.into(), name: name.into(), agent_type: AgentType::LLM,
            capabilities: AgentCapabilities { analysis: 0.5, quality: 0.5, speed: 0.5, creativity: 0.5, reliability: 0.5 },
            emotional_state: EmotionalState::calm(),
            performance: PerformanceHistory { tasks_completed: 0, avg_response_time_ms: 0.0, accuracy: 1.0, memory_usage_mb: 0.0 },
            created_at: 0, last_used: 0, total_tasks: 10, success_rate, session_id: None,
        }
    }

    #[test]
    fn test_select_agent_empty() {
        let orch = AgentOrchestrator::new();
        let task = AgentTask { id: "t1".into(), description: "test".into(), context: String::new(), required_capability: None };
        assert!(orch.select_agent(&[], &task).is_none());
    }

    #[test]
    fn test_select_agent_prefers_higher_success_rate() {
        let orch = AgentOrchestrator::new();
        let agents = vec![
            make_test_agent("a1", 0.5),
            make_test_agent("a2", 0.9),
        ];
        let task = AgentTask { id: "t1".into(), description: "test".into(), context: String::new(), required_capability: None };
        let selected = orch.select_agent(&agents, &task);
        assert_eq!(selected.unwrap().name, "a2");
    }

    #[test]
    fn test_evaluate_result() {
        let orch = AgentOrchestrator::new();
        let r1 = AgentResult { agent_id: "a1".into(), output: "ok".into(), emotion: EmotionalState::calm(), confidence: 0.9, tokens_used: 100 };
        let r2 = AgentResult { agent_id: "a1".into(), output: "maybe".into(), emotion: EmotionalState::calm(), confidence: 0.6, tokens_used: 100 };
        let r3 = AgentResult { agent_id: "a1".into(), output: "bad".into(), emotion: EmotionalState::calm(), confidence: 0.3, tokens_used: 100 };
        assert_eq!(orch.evaluate_result(&r1), "complete");
        assert_eq!(orch.evaluate_result(&r2), "review");
        assert_eq!(orch.evaluate_result(&r3), "reassign");
    }
}
