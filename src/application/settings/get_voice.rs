use crate::domain::ports::SettingsRepository;

pub struct GetVoiceUseCase<R: SettingsRepository> {
    settings_repo: R,
}

impl<R: SettingsRepository> GetVoiceUseCase<R> {
    pub fn new(settings_repo: R) -> Self {
        Self { settings_repo }
    }

    pub fn execute(&self) -> serde_json::Value {
        let config = self.settings_repo.load_config();
        let voice = config.get("voice").and_then(|v| v.as_table());
        serde_json::json!({
            "stt_language": voice.and_then(|v| v.get("stt_language")).and_then(|v| v.as_str()).unwrap_or("en-US"),
            "tts_engine": voice.and_then(|v| v.get("tts_engine")).and_then(|v| v.as_str()).unwrap_or("fish"),
            "wake_word": voice.and_then(|v| v.get("wake_word")).and_then(|v| v.as_str()).unwrap_or("umbra"),
            "voice_feedback": voice.and_then(|v| v.get("voice_feedback")).and_then(|v| v.as_bool()).unwrap_or(true),
        })
    }
}
