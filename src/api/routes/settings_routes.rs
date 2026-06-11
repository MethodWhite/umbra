use axum::{Json, extract::State, response::IntoResponse, http::StatusCode};
use std::sync::Mutex;
use std::sync::LazyLock;

use crate::domain::models::{VoiceSettings, Preferences, CustomizationBody};

use crate::application::settings::{GetVoiceUseCase, SaveVoiceUseCase, GetSettingsStatusUseCase};
use crate::infrastructure::repositories::TomlSettingsRepository;
use crate::infrastructure::persistence::derive_customization_key;

use super::super::FrontendRouterState;

static PREFS: LazyLock<Mutex<Preferences>> = LazyLock::new(|| Mutex::new(Preferences::default()));

pub async fn get_voice() -> Json<serde_json::Value> {
    let repo = TomlSettingsRepository::new();
    let use_case = GetVoiceUseCase::new(repo);
    Json(use_case.execute())
}

pub async fn set_voice(
    Json(body): Json<VoiceSettings>,
) -> impl IntoResponse {
    let repo = TomlSettingsRepository::new();
    let use_case = SaveVoiceUseCase::new(repo);
    match use_case.execute(body) {
        Ok(val) => Json(val).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e.to_string()}))).into_response(),
    }
}

pub async fn get_prefs() -> Json<Preferences> {
    let prefs = PREFS.lock().unwrap().clone();
    Json(prefs)
}

#[derive(serde::Deserialize)]
pub struct PrefsBody {
    pub user_name: Option<String>,
    pub honorific: Option<String>,
    pub calendar_accounts: Option<Vec<String>>,
}

pub async fn set_prefs(
    Json(body): Json<PrefsBody>,
) -> Json<serde_json::Value> {
    let mut p = PREFS.lock().unwrap().clone();
    if let Some(name) = body.user_name { p.user_name = name; }
    if let Some(h) = body.honorific { p.honorific = h; }
    if let Some(cal) = body.calendar_accounts { p.calendar_accounts = cal; }
    *PREFS.lock().unwrap() = p;
    Json(serde_json::json!({"success": true}))
}

pub async fn status(
    State(state): State<FrontendRouterState>,
) -> Json<serde_json::Value> {
    let use_case = GetSettingsStatusUseCase::new();
    let mut vault = state.vault.lock().await;
    let result = use_case.execute(&mut *vault, &state.backend_url, state.start_time).await;
    Json(result)
}

// Customization

use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use rand::Rng;

static DEFAULTS: &[(&str, &str)] = &[
    ("name", "UMBRA"),
    ("greeting", "sir"),
    ("wake_word", "umbra"),
    ("tts_engine", "fish"),
    ("tts_voice", "default"),
    ("stt_language", "en-US"),
    ("theme", "dark"),
    ("primary_color", "#7c3aed"),
    ("accent_color", "#06b6d4"),
    ("icon_style", "default"),
    ("response_style", "concise"),
    ("persona", "professional"),
];

fn customization_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    std::path::PathBuf::from(&home).join(".umbra/customization.enc")
}

fn load_customization_inner() -> serde_json::Value {
    let path = customization_path();
    if !path.exists() {
        return serde_json::json!({
            "name": "UMBRA",
            "greeting": "sir",
            "wake_word": "umbra",
            "tts_engine": "fish",
            "tts_voice": "default",
            "stt_language": "en-US",
            "theme": "dark",
            "primary_color": "#7c3aed",
            "accent_color": "#06b6d4",
            "icon_style": "default",
            "response_style": "concise",
            "persona": "professional",
        });
    }
    let data = match std::fs::read(&path) {
        Ok(d) => d,
        Err(_) => return serde_json::json!(DEFAULTS),
    };
    if data.len() < 28 { return serde_json::json!(DEFAULTS); }
    let key = derive_customization_key();
    let aes_key = Key::<Aes256Gcm>::from_slice(&key);
    let cipher = Aes256Gcm::new(aes_key);
    let nonce = Nonce::from_slice(&data[..12]);
    let ct = &data[12..];
    match cipher.decrypt(nonce, ct) {
        Ok(plain) => {
            let mut defaults: serde_json::Value = serde_json::json!(DEFAULTS);
            if let Ok(custom) = serde_json::from_slice::<serde_json::Value>(&plain) {
                if let Some(obj) = custom.as_object() {
                    if let Some(def_obj) = defaults.as_object_mut() {
                        for (k, v) in obj {
                            def_obj.insert(k.clone(), v.clone());
                        }
                    }
                }
            }
            defaults
        }
        Err(_) => serde_json::json!(DEFAULTS),
    }
}

pub async fn get_customization() -> Json<serde_json::Value> {
    Json(load_customization_inner())
}

pub async fn set_customization(
    Json(body): Json<CustomizationBody>,
) -> Json<serde_json::Value> {
    let mut current = load_customization_inner();
    if let Some(obj) = current.as_object_mut() {
        if let Some(v) = body.name { obj.insert("name".into(), serde_json::Value::String(v)); }
        if let Some(v) = body.greeting { obj.insert("greeting".into(), serde_json::Value::String(v)); }
        if let Some(v) = body.wake_word { obj.insert("wake_word".into(), serde_json::Value::String(v)); }
        if let Some(v) = body.tts_engine { obj.insert("tts_engine".into(), serde_json::Value::String(v)); }
        if let Some(v) = body.tts_voice { obj.insert("tts_voice".into(), serde_json::Value::String(v)); }
        if let Some(v) = body.stt_language { obj.insert("stt_language".into(), serde_json::Value::String(v)); }
        if let Some(v) = body.theme { obj.insert("theme".into(), serde_json::Value::String(v)); }
        if let Some(v) = body.primary_color { obj.insert("primary_color".into(), serde_json::Value::String(v)); }
        if let Some(v) = body.accent_color { obj.insert("accent_color".into(), serde_json::Value::String(v)); }
        if let Some(v) = body.icon_style { obj.insert("icon_style".into(), serde_json::Value::String(v)); }
        if let Some(v) = body.response_style { obj.insert("response_style".into(), serde_json::Value::String(v)); }
        if let Some(v) = body.persona { obj.insert("persona".into(), serde_json::Value::String(v)); }
    }

    let json = serde_json::to_vec(&current).unwrap_or_default();
    let key = derive_customization_key();
    let aes_key = Key::<Aes256Gcm>::from_slice(&key);
    let cipher = Aes256Gcm::new(aes_key);
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher.encrypt(nonce, json.as_slice()).unwrap_or_default();

    let mut out = Vec::with_capacity(12 + ct.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);

    let path = customization_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    use std::os::unix::fs::PermissionsExt;
    std::fs::write(&path, &out).ok();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).ok();

    Json(serde_json::json!({"success": true, "customization": current}))
}
