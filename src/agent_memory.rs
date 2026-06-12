// Zone 1 — Desktop/UI
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentParams {
    pub id: String,
    pub name: String,
    pub agent_type: AgentType,
    pub capabilities: AgentCapabilities,
    pub emotional_state: EmotionalState,
    pub performance: PerformanceHistory,
    pub created_at: u64,
    pub last_used: u64,
    pub total_tasks: u64,
    pub success_rate: f32,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentType { LLM, JEPA, SNN, SSM, Audio, Vision }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    pub analysis: f32,
    pub quality: f32,
    pub speed: f32,
    pub creativity: f32,
    pub reliability: f32,
}

impl AgentCapabilities {
    pub fn overall_score(&self) -> f32 {
        (self.analysis + self.quality + self.speed + self.creativity + self.reliability) / 5.0
    }
}

// ── Plutchik Wheel of Emotions ──────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PrimaryEmotion {
    Joy, Trust, Fear, Surprise, Sadness, Disgust, Anger, Anticipation,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EmotionIntensity {
    Low, Medium, High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmotionalState {
    pub primary: PrimaryEmotion,
    pub intensity: EmotionIntensity,
    pub secondary: Option<PrimaryEmotion>,
    pub valence: f32,
    pub arousal: f32,
    pub label: String,
}

impl EmotionalState {
    // ── Backward-compatible constructors ────────────────────────────────────
    pub fn calm() -> Self {
        Self {
            primary: PrimaryEmotion::Trust,
            intensity: EmotionIntensity::Low,
            secondary: None,
            valence: 0.3,
            arousal: 0.15,
            label: "Calm".into(),
        }
    }

    pub fn happy() -> Self {
        Self {
            primary: PrimaryEmotion::Joy,
            intensity: EmotionIntensity::Medium,
            secondary: None,
            valence: 0.8,
            arousal: 0.6,
            label: "Happy".into(),
        }
    }

    pub fn analytical() -> Self {
        Self {
            primary: PrimaryEmotion::Trust,
            intensity: EmotionIntensity::Medium,
            secondary: Some(PrimaryEmotion::Anticipation),
            valence: 0.2,
            arousal: 0.5,
            label: "Analytical".into(),
        }
    }

    pub fn curious() -> Self {
        Self {
            primary: PrimaryEmotion::Anticipation,
            intensity: EmotionIntensity::Medium,
            secondary: Some(PrimaryEmotion::Trust),
            valence: 0.5,
            arousal: 0.7,
            label: "Curious".into(),
        }
    }

    // ── Backward-compatible accessors ──────────────────────────────────────

    pub fn hue(&self) -> f32 {
        self.color_hue()
    }

    pub fn saturation(&self) -> f32 {
        match self.intensity {
            EmotionIntensity::Low => 0.3,
            EmotionIntensity::Medium => 0.6,
            EmotionIntensity::High => 1.0,
        }
    }

    pub fn intensity(&self) -> f32 {
        self.arousal
    }

    // ── Cognitive self-regulation (CBT for AIs) ───────────────────────────

    pub fn is_hot(&self) -> bool {
        matches!(self.primary, PrimaryEmotion::Anger | PrimaryEmotion::Fear | PrimaryEmotion::Surprise)
            && self.intensity == EmotionIntensity::High
    }

    pub fn is_cold(&self) -> bool {
        matches!(self.primary, PrimaryEmotion::Joy | PrimaryEmotion::Trust | PrimaryEmotion::Anticipation)
            && self.intensity != EmotionIntensity::High
    }

    pub fn cool_down(&mut self) {
        match self.intensity {
            EmotionIntensity::High => self.intensity = EmotionIntensity::Medium,
            EmotionIntensity::Medium => self.intensity = EmotionIntensity::Low,
            EmotionIntensity::Low => {
                self.primary = PrimaryEmotion::Joy;
                self.intensity = EmotionIntensity::Low;
            }
        }
        self.valence = self.valence * 0.8 + 0.2;
        self.arousal *= 0.7;
    }

    // ── Color helpers ──────────────────────────────────────────────────────

    pub fn color_hue(&self) -> f32 {
        use PrimaryEmotion::*;
        let base = match self.primary {
            Joy => 0.12,
            Trust => 0.30,
            Fear => 0.20,
            Surprise => 0.50,
            Sadness => 0.70,
            Disgust => 0.80,
            Anger => 0.00,
            Anticipation => 0.08,
        };
        if let Some(ref sec) = self.secondary {
            let sec_hue = match sec {
                Joy => 0.12,
                Trust => 0.30,
                Fear => 0.20,
                Surprise => 0.50,
                Sadness => 0.70,
                Disgust => 0.80,
                Anger => 0.00,
                Anticipation => 0.08,
            };
            (base + sec_hue) / 2.0
        } else {
            base
        }
    }
}

impl std::fmt::Display for EmotionalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

// ── Performance History ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceHistory {
    pub tasks_completed: u64,
    pub avg_response_time_ms: f64,
    pub accuracy: f32,
    pub memory_usage_mb: f64,
}

// ── Agent Memory Store ──────────────────────────────────────────────────────

pub struct AgentMemory {
    storage: Mutex<HashMap<String, AgentParams>>,
    path: PathBuf,
}

impl AgentMemory {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        let path = PathBuf::from(home).join(".umbra/agent_memory.json");
        let storage = Mutex::new(HashMap::new());
        let mem = AgentMemory { storage, path };
        let _ = mem.load_from_disk();
        mem
    }

    fn now() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
    }

    fn load_from_disk(&self) -> Result<(), String> {
        if !self.path.exists() { return Ok(()); }
        let data = std::fs::read_to_string(&self.path).map_err(|e| e.to_string())?;
        let agents: HashMap<String, AgentParams> = serde_json::from_str(&data).map_err(|e| e.to_string())?;
        if let Ok(mut storage) = self.storage.lock() { *storage = agents; }
        Ok(())
    }

    fn save_to_disk(&self) -> Result<(), String> {
        let storage = self.storage.lock().map_err(|e| e.to_string())?;
        let data = serde_json::to_string_pretty(&*storage).map_err(|e| e.to_string())?;
        if let Some(parent) = self.path.parent() { std::fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
        std::fs::write(&self.path, &data).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn save(&self, mut agent: AgentParams) -> Result<(), String> {
        agent.last_used = Self::now();
        let id = agent.id.clone();
        if let Ok(mut storage) = self.storage.lock() { storage.insert(id, agent); }
        self.save_to_disk()
    }

    pub fn load(&self, id: &str) -> Option<AgentParams> {
        self.storage.lock().ok().and_then(|s| s.get(id).cloned())
    }

    pub fn find_best(&self, task_type: &str) -> Option<AgentParams> {
        let storage = self.storage.lock().ok()?;
        storage.values()
            .filter(|a| {
                match task_type {
                    "analysis" => a.capabilities.analysis > 0.6,
                    "creative" => a.capabilities.creativity > 0.6,
                    "speed" => a.capabilities.speed > 0.6,
                    _ => true,
                }
            })
            .max_by(|a, b| {
                let sa = a.capabilities.overall_score() * a.success_rate;
                let sb = b.capabilities.overall_score() * b.success_rate;
                sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }

    pub fn list_all(&self) -> Vec<AgentParams> {
        self.storage.lock().ok().map(|s| s.values().cloned().collect()).unwrap_or_default()
    }

    pub fn delete(&self, id: &str) -> Result<(), String> {
        if let Ok(mut storage) = self.storage.lock() { storage.remove(id); }
        self.save_to_disk()
    }

    pub fn update_emotional_state(&self, id: &str, state: EmotionalState) -> Result<(), String> {
        if let Ok(mut storage) = self.storage.lock() {
            if let Some(agent) = storage.get_mut(id) {
                agent.emotional_state = state;
                agent.last_used = Self::now();
            }
        }
        self.save_to_disk()
    }

    pub fn record_task_result(&self, id: &str, success: bool, response_time_ms: f64) -> Result<(), String> {
        if let Ok(mut storage) = self.storage.lock() {
            if let Some(agent) = storage.get_mut(id) {
                agent.total_tasks += 1;
                agent.performance.tasks_completed += 1;
                agent.performance.avg_response_time_ms =
                    (agent.performance.avg_response_time_ms * (agent.total_tasks - 1) as f64 + response_time_ms) / agent.total_tasks as f64;
                agent.performance.accuracy =
                    (agent.performance.accuracy * (agent.total_tasks - 1) as f32 + if success { 1.0 } else { 0.0 }) / agent.total_tasks as f32;
                agent.success_rate = agent.performance.accuracy;
                agent.last_used = Self::now();
            }
        }
        self.save_to_disk()
    }
}

// ── Cognitive self-regulation system ────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CognitiveBehavior {
    pub emotion_history: VecDeque<EmotionalState>,
    pub frustration_level: f32,
    pub cooling_cycles: u32,
    pub last_regulation: Instant,
    pub max_history: usize,
}

impl CognitiveBehavior {
    pub fn new() -> Self {
        Self {
            emotion_history: VecDeque::new(),
            frustration_level: 0.0,
            cooling_cycles: 0,
            last_regulation: Instant::now(),
            max_history: 50,
        }
    }

    pub fn regulate(&mut self, current_emotion: &EmotionalState) -> EmotionalState {
        self.emotion_history.push_back(current_emotion.clone());
        if self.emotion_history.len() > self.max_history {
            self.emotion_history.pop_front();
        }

        if current_emotion.is_hot() || self.frustration_level > 0.7 {
            let mut cooled = current_emotion.clone();
            cooled.cool_down();
            self.cooling_cycles += 1;
            self.frustration_level *= 0.5;
            self.last_regulation = Instant::now();
            return cooled;
        }
        current_emotion.clone()
    }

    pub fn record_failure(&mut self) {
        self.frustration_level = (self.frustration_level + 0.2).min(1.0);
    }

    pub fn record_success(&mut self) {
        self.frustration_level = (self.frustration_level - 0.2).max(0.0);
    }
}

impl Default for CognitiveBehavior {
    fn default() -> Self {
        Self::new()
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emotional_state_calm() {
        let state = EmotionalState::calm();
        assert_eq!(state.intensity, EmotionIntensity::Low);
    }

    #[test]
    fn test_emotional_state_happy() {
        let state = EmotionalState::happy();
        assert!(state.valence > 0.5);
    }

    #[test]
    fn test_cognitive_behavior_new() {
        let cb = CognitiveBehavior::new();
        assert_eq!(cb.frustration_level, 0.0);
        assert_eq!(cb.cooling_cycles, 0);
    }

    #[test]
    fn test_cognitive_behavior_record_failure() {
        let mut cb = CognitiveBehavior::new();
        cb.record_failure();
        assert!((cb.frustration_level - 0.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cognitive_behavior_record_success() {
        let mut cb = CognitiveBehavior::new();
        cb.frustration_level = 0.5;
        cb.record_success();
        assert!((cb.frustration_level - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn test_agent_memory_save_and_load() {
        let memory = AgentMemory::new();
        let agent = AgentParams {
            id: "test-1".into(),
            name: "test_agent".into(),
            agent_type: AgentType::LLM,
            capabilities: AgentCapabilities { analysis: 0.5, quality: 0.5, speed: 0.5, creativity: 0.5, reliability: 0.5 },
            emotional_state: EmotionalState::calm(),
            performance: PerformanceHistory { tasks_completed: 0, avg_response_time_ms: 0.0, accuracy: 1.0, memory_usage_mb: 0.0 },
            created_at: 0,
            last_used: 0,
            total_tasks: 0,
            success_rate: 1.0,
            session_id: None,
        };
        memory.save(agent.clone()).unwrap();
        let loaded = memory.load("test-1");
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().name, "test_agent");
    }

    #[test]
    fn test_agent_memory_load_not_found() {
        let memory = AgentMemory::new();
        assert!(memory.load("nonexistent").is_none());
    }

}


