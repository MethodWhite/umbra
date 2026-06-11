// Zone 3 — Application
use crate::domain::models::VoiceSettings;
use crate::domain::ports::SettingsRepository;
use crate::domain::errors::AppError;

pub struct SaveVoiceUseCase<R: SettingsRepository> {
    settings_repo: R,
}

impl<R: SettingsRepository> SaveVoiceUseCase<R> {
    pub fn new(settings_repo: R) -> Self {
        Self { settings_repo }
    }

    pub fn execute(&self, body: VoiceSettings) -> Result<serde_json::Value, AppError> {
        let path = self.settings_repo.config_path();
        let mut config: toml::Value = if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            content.parse::<toml::Value>().unwrap_or(toml::Value::Table(Default::default()))
        } else {
            toml::Value::Table(Default::default())
        };

        let mut voice = toml::map::Map::new();
        voice.insert("stt_language".into(), toml::Value::String(body.stt_language.unwrap_or_else(|| "en-US".into())));
        voice.insert("tts_engine".into(), toml::Value::String(body.tts_engine.unwrap_or_else(|| "fish".into())));
        voice.insert("wake_word".into(), toml::Value::String(body.wake_word.unwrap_or_else(|| "umbra".into())));
        voice.insert("voice_feedback".into(), toml::Value::Boolean(body.voice_feedback.unwrap_or(true)));

        if let Some(table) = config.as_table_mut() {
            table.insert("voice".into(), toml::Value::Table(voice));
        }

        self.settings_repo.save_config(&config)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Save voice failed: {}", e)))?;

        Ok(serde_json::json!({"success": true}))
    }
}
