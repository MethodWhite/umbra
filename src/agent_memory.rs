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

    pub fn focused() -> Self {
        Self {
            primary: PrimaryEmotion::Anticipation,
            intensity: EmotionIntensity::Medium,
            secondary: Some(PrimaryEmotion::Trust),
            valence: 0.4,
            arousal: 0.7,
            label: "Focused".into(),
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

    pub fn creative() -> Self {
        Self {
            primary: PrimaryEmotion::Joy,
            intensity: EmotionIntensity::Medium,
            secondary: Some(PrimaryEmotion::Surprise),
            valence: 0.7,
            arousal: 0.8,
            label: "Creative".into(),
        }
    }

    pub fn excited() -> Self {
        Self {
            primary: PrimaryEmotion::Joy,
            intensity: EmotionIntensity::High,
            secondary: Some(PrimaryEmotion::Anticipation),
            valence: 0.9,
            arousal: 0.95,
            label: "Excited".into(),
        }
    }

    pub fn sad() -> Self {
        Self {
            primary: PrimaryEmotion::Sadness,
            intensity: EmotionIntensity::Medium,
            secondary: None,
            valence: -0.6,
            arousal: 0.3,
            label: "Sad".into(),
        }
    }

    pub fn depressed() -> Self {
        Self {
            primary: PrimaryEmotion::Sadness,
            intensity: EmotionIntensity::High,
            secondary: None,
            valence: -0.9,
            arousal: 0.1,
            label: "Depressed".into(),
        }
    }

    pub fn angry() -> Self {
        Self {
            primary: PrimaryEmotion::Anger,
            intensity: EmotionIntensity::Medium,
            secondary: None,
            valence: -0.7,
            arousal: 0.9,
            label: "Angry".into(),
        }
    }

    pub fn anxious() -> Self {
        Self {
            primary: PrimaryEmotion::Fear,
            intensity: EmotionIntensity::Medium,
            secondary: Some(PrimaryEmotion::Anticipation),
            valence: -0.4,
            arousal: 0.8,
            label: "Anxious".into(),
        }
    }

    pub fn fearful() -> Self {
        Self {
            primary: PrimaryEmotion::Fear,
            intensity: EmotionIntensity::Medium,
            secondary: None,
            valence: -0.6,
            arousal: 0.85,
            label: "Fearful".into(),
        }
    }

    pub fn tired() -> Self {
        Self {
            primary: PrimaryEmotion::Sadness,
            intensity: EmotionIntensity::Low,
            secondary: None,
            valence: -0.2,
            arousal: 0.05,
            label: "Tired".into(),
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

    pub fn surprised() -> Self {
        Self {
            primary: PrimaryEmotion::Surprise,
            intensity: EmotionIntensity::Medium,
            secondary: None,
            valence: 0.1,
            arousal: 0.9,
            label: "Surprised".into(),
        }
    }

    pub fn ashamed() -> Self {
        Self {
            primary: PrimaryEmotion::Sadness,
            intensity: EmotionIntensity::Medium,
            secondary: Some(PrimaryEmotion::Disgust),
            valence: -0.5,
            arousal: 0.4,
            label: "Ashamed".into(),
        }
    }

    pub fn flow() -> Self {
        Self {
            primary: PrimaryEmotion::Trust,
            intensity: EmotionIntensity::High,
            secondary: Some(PrimaryEmotion::Joy),
            valence: 0.7,
            arousal: 0.85,
            label: "Flow".into(),
        }
    }

    pub fn intuitive() -> Self {
        Self {
            primary: PrimaryEmotion::Trust,
            intensity: EmotionIntensity::Medium,
            secondary: Some(PrimaryEmotion::Surprise),
            valence: 0.3,
            arousal: 0.5,
            label: "Intuitive".into(),
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

    pub fn cognitive_reset() -> Self {
        EmotionalState {
            primary: PrimaryEmotion::Joy,
            intensity: EmotionIntensity::Low,
            secondary: None,
            valence: 0.3,
            arousal: 0.2,
            label: "Calm".to_string(),
        }
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

    pub fn to_color32(&self) -> egui::Color32 {
        let hue = self.color_hue();
        let sat = self.saturation();
        let val = 0.5 + self.arousal * 0.4;
        let (r, g, b) = hsv_to_rgb(hue, sat, val);
        egui::Color32::from_rgb(r, g, b)
    }

    fn named(label: &str, primary: PrimaryEmotion, intensity: EmotionIntensity,
              secondary: Option<PrimaryEmotion>, valence: f32, arousal: f32) -> Self {
        Self { primary, intensity, secondary, valence, arousal, label: label.into() }
    }

    fn dyad(label: &str, a: PrimaryEmotion, ai: EmotionIntensity,
             b: PrimaryEmotion, bi: EmotionIntensity, valence: f32, arousal: f32) -> Self {
        let avg_int = match (ai as u8 + bi as u8) / 2 {
            0..=1 => EmotionIntensity::Low,
            2..=2 => EmotionIntensity::Medium,
            _ => EmotionIntensity::High,
        };
        Self {
            primary: a, intensity: avg_int, secondary: Some(b),
            valence, arousal, label: label.into(),
        }
    }

    pub fn all_emotions() -> Vec<EmotionalState> {
        use PrimaryEmotion::*;
        use EmotionIntensity::*;
        let mut states = Vec::new();

        // 24 basic emotions (8 primaries × 3 intensities)
        let basics: Vec<(&str, PrimaryEmotion, f32, f32)> = vec![
            ("Serenity", Joy, 0.6, 0.2), ("Joy", Joy, 0.8, 0.6), ("Ecstasy", Joy, 1.0, 1.0),
            ("Acceptance", Trust, 0.5, 0.2), ("Trust", Trust, 0.7, 0.4), ("Admiration", Trust, 0.9, 0.7),
            ("Apprehension", Fear, -0.3, 0.3), ("Fear", Fear, -0.6, 0.7), ("Terror", Fear, -1.0, 1.0),
            ("Distraction", Surprise, 0.0, 0.3), ("Surprise", Surprise, 0.2, 0.7), ("Amazement", Surprise, 0.5, 1.0),
            ("Pensiveness", Sadness, -0.3, 0.2), ("Sadness", Sadness, -0.6, 0.4), ("Grief", Sadness, -1.0, 0.8),
            ("Boredom", Disgust, -0.2, 0.1), ("Disgust", Disgust, -0.6, 0.5), ("Loathing", Disgust, -0.9, 0.8),
            ("Annoyance", Anger, -0.3, 0.4), ("Anger", Anger, -0.7, 0.8), ("Rage", Anger, -1.0, 1.0),
            ("Interest", Anticipation, 0.3, 0.3), ("Anticipation", Anticipation, 0.5, 0.6), ("Vigilance", Anticipation, 0.7, 0.9),
        ];
        let intensities = [Low, Medium, High];
        for (i, (label, prim, val, aro)) in basics.iter().enumerate() {
            states.push(Self::named(label, *prim, intensities[i % 3], None, *val, *aro));
        }

        // Primary dyads (adjacent on the wheel)
        let named_dyads: Vec<(&str, (PrimaryEmotion, f32, f32), (PrimaryEmotion, f32, f32))> = vec![
            ("Love", (Joy, 0.9, 0.7), (Trust, 0.8, 0.5)),
            ("Submission", (Trust, 0.3, 0.3), (Fear, 0.5, 0.6)),
            ("Awe", (Fear, 0.4, 0.8), (Surprise, 0.5, 0.9)),
            ("Disapproval", (Surprise, -0.2, 0.5), (Sadness, -0.4, 0.4)),
            ("Remorse", (Sadness, -0.6, 0.4), (Disgust, -0.5, 0.5)),
            ("Contempt", (Disgust, -0.7, 0.6), (Anger, -0.6, 0.7)),
            ("Aggressiveness", (Anger, -0.5, 0.8), (Anticipation, 0.2, 0.8)),
            ("Optimism", (Anticipation, 0.6, 0.7), (Joy, 0.8, 0.7)),
        ];
        for (label, (a, av, aa), (b, bv, ba)) in &named_dyads {
            states.push(Self::dyad(label, *a, Medium, *b, Medium, (*av + *bv) / 2.0, (*aa + *ba) / 2.0));
        }

        // Secondary dyads (one step apart)
        let secondary_dyads: Vec<(&str, PrimaryEmotion, PrimaryEmotion, f32, f32)> = vec![
            ("Guilt", Joy, Fear, -0.1, 0.5),
            ("Curiosity", Trust, Surprise, 0.3, 0.7),
            ("Despair", Fear, Sadness, -0.7, 0.6),
            ("Disbelief", Surprise, Disgust, 0.0, 0.6),
            ("Envy", Sadness, Anger, -0.5, 0.5),
            ("Cynicism", Disgust, Anticipation, -0.3, 0.4),
            ("Pride", Anger, Joy, 0.2, 0.8),
            ("Hope", Anticipation, Trust, 0.5, 0.6),
        ];
        for (label, a, b, v, aro) in &secondary_dyads {
            states.push(Self::dyad(label, *a, Medium, *b, Medium, *v, *aro));
        }

        // Tertiary dyads (two steps apart)
        let tertiary_dyads: Vec<(&str, PrimaryEmotion, PrimaryEmotion, f32, f32)> = vec![
            ("Delight", Joy, Surprise, 0.7, 0.8),
            ("Sentimentality", Trust, Sadness, 0.1, 0.3),
            ("Shame", Fear, Disgust, -0.5, 0.4),
            ("Outrage", Surprise, Anger, -0.3, 0.9),
            ("Pessimism", Sadness, Anticipation, -0.3, 0.3),
            ("Morbidness", Disgust, Joy, -0.2, 0.5),
            ("Dominance", Anger, Trust, 0.0, 0.8),
            ("Anxiety", Anticipation, Fear, -0.3, 0.7),
        ];
        for (label, a, b, v, aro) in &tertiary_dyads {
            states.push(Self::dyad(label, *a, Medium, *b, Medium, *v, *aro));
        }

        // Additional dyads (all remaining C(8,2) = 28 total, we've covered 24, add the remaining 4)
        let extra_dyads: Vec<(&str, PrimaryEmotion, PrimaryEmotion, f32, f32)> = vec![
            ("Sullenness", Sadness, Joy, -0.2, 0.2),
            ("Skepticism", Disgust, Trust, -0.3, 0.3),
            ("Vengeance", Anger, Surprise, -0.4, 0.9),
            ("Fatalism", Anticipation, Sadness, -0.2, 0.3),
        ];
        for (label, a, b, v, aro) in &extra_dyads {
            states.push(Self::dyad(label, *a, Medium, *b, Medium, *v, *aro));
        }

        states
    }
}

impl std::fmt::Display for EmotionalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

// ── AI Gender Parameters ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AiGenderExpression {
    Masculine,
    Feminine,
    Androgynous,
    Neutral,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CommunicationStyle {
    Analytical,
    Intuitive,
    Direct,
    Expressive,
    Balanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiGenderIdentity {
    pub gender: AiGenderExpression,
    pub pronouns: String,
    pub voice_pitch: f32,
    pub voice_speed: f32,
    pub communication_style: CommunicationStyle,
    pub selected_at: u64,
    pub finalized: bool,
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
        let mut mem = AgentMemory { storage, path };
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

    pub fn register_synapsis_session(&self, agent_id: &str, project_key: &str) -> Result<String, String> {
        let session_id = format!("agent_{}_{}", agent_id, Self::now());
        if let Ok(mut storage) = self.storage.lock() {
            if let Some(agent) = storage.get_mut(agent_id) {
                agent.session_id = Some(session_id.clone());
                agent.last_used = Self::now();
            }
        }
        self.save_to_disk()?;
        Ok(session_id)
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

// ── Agent consultation system ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsultationRequest {
    pub from_agent: String,
    pub topic: String,
    pub context: String,
    pub requested_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsultationResponse {
    pub from_agent: String,
    pub opinion: String,
    pub confidence: f32,
    pub responded_at: u64,
}

impl ConsultationRequest {
    pub fn new(from: &str, topic: &str, context: &str) -> Self {
        Self {
            from_agent: from.to_string(),
            topic: topic.to_string(),
            context: context.to_string(),
            requested_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveSession {
    pub participants: Vec<String>,
    pub topic: String,
    pub exchanges: Vec<(String, String)>,
    pub consensus_reached: bool,
    pub started_at: u64,
}

impl CognitiveSession {
    pub fn new(participants: Vec<String>, topic: &str) -> Self {
        Self {
            participants,
            topic: topic.to_string(),
            exchanges: Vec::new(),
            consensus_reached: false,
            started_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        }
    }

    pub fn add_exchange(&mut self, agent: &str, message: &str) {
        self.exchanges.push((agent.to_string(), message.to_string()));
    }

    pub fn reach_consensus(&mut self) {
        self.consensus_reached = true;
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h = h * 360.0;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h as i32 % 360 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}
