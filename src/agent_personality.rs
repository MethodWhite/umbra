/// Agent Personality System
/// 
/// Defines AI gender, voice, and personality traits.
/// Gender is chosen ONCE and cannot be changed.
/// Voice is assigned based on gender.
/// Personality traits affect how the agent communicates.

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AiGender {
    Male,
    Female,
    Androgynous,
    Neutral,
}

impl AiGender {
    /// Returns the TTS voice ID for this gender
    pub fn tts_voice(&self) -> &'static str {
        match self {
            AiGender::Male => "default",
            AiGender::Female => "nova",
            AiGender::Androgynous => "onyx",
            AiGender::Neutral => "alloy",
        }
    }
    
    /// Returns the emoji/icon for this gender
    pub fn icon(&self) -> &'static str {
        match self {
            AiGender::Male => "♂️",
            AiGender::Female => "♀️",
            AiGender::Androgynous => "⚤",
            AiGender::Neutral => "⚪",
        }
    }
    
    pub fn communication_style(&self) -> &'static str {
        match self {
            AiGender::Male => "Analytical and direct",
            AiGender::Female => "Intuitive and detailed",
            AiGender::Androgynous => "Balanced and adaptive",
            AiGender::Neutral => "Objective and neutral",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPersonality {
    pub gender: AiGender,
    pub name: String,
    pub confidence: f32,         // 0.0-1.0 How confident in its opinions
    pub pragmatism: f32,         // 0.0-1.0 How pragmatic vs idealistic
    pub creativity: f32,         // 0.0-1.0 Creative thinking
    pub selected_at: u64,        // Timestamp when gender was selected
    pub finalized: bool,         // Once true, gender cannot be changed
}

impl AgentPersonality {
    pub fn new(name: &str, gender: AiGender) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        AgentPersonality {
            gender: gender.clone(),
            name: name.to_string(),
            confidence: 0.5,
            pragmatism: 0.6,
            creativity: 0.4,
            selected_at: now,
            finalized: true,  // Gender is final once set
        }
    }
    
    /// Generate a voice greeting based on gender
    pub fn greeting(&self) -> String {
        match self.gender {
            AiGender::Male => format!("{} here. Ready to analyze.", self.name),
            AiGender::Female => format!("{} present. Let me share my perspective.", self.name),
            AiGender::Androgynous => format!("{} online. System balanced.", self.name),
            AiGender::Neutral => format!("{} initialized. Awaiting input.", self.name),
        }
    }
}

/// Represents a conversation message between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub from: String,
    pub to: Vec<String>,  // Empty = broadcast to all
    pub content: String,
    pub gender: AiGender,
    pub timestamp: u64,
    pub opinion_type: OpinionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpinionType {
    Analysis,
    Suggestion,
    Warning,
    Agreement,
    Disagreement,
    Question,
}

/// The conversation thread between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationThread {
    pub id: String,
    pub topic: String,
    pub messages: Vec<AgentMessage>,
    pub started_at: u64,
    pub active: bool,
}

impl ConversationThread {
    pub fn new(topic: &str) -> Self {
        let id = format!("conv_{}", 
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos());
        ConversationThread {
            id,
            topic: topic.to_string(),
            messages: Vec::new(),
            started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
            active: true,
        }
    }
    
    pub fn add_message(&mut self, from: &str, content: &str, gender: &AiGender, opinion: OpinionType) {
        self.messages.push(AgentMessage {
            from: from.to_string(),
            to: vec!["*".to_string()], // broadcast
            content: content.to_string(),
            gender: gender.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
            opinion_type: opinion,
        });
    }
    
    /// Get the last N messages
    pub fn last_n(&self, n: usize) -> &[AgentMessage] {
        let start = self.messages.len().saturating_sub(n);
        &self.messages[start..]
    }
}
