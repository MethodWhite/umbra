// Zone 2 — Domain/Ports
pub struct ThreatAssessment {
    pub is_threat: bool,
    pub threat_level: ThreatLevel,
    pub description: String,
}

pub enum ThreatLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

pub trait CybersecurityPort: Send + Sync {
    fn analyze_command(&self, command: &str) -> Result<ThreatAssessment, String>;
    fn sanitize_input(&self, input: &str) -> Result<String, String>;
    fn scan_audio(&self, audio_data: &[u8]) -> Result<ThreatAssessment, String>;
}
