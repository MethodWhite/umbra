use axum::{Json, extract::{Path, Query}};
use serde::Deserialize;

use crate::infrastructure::http::huggingface_client::HuggingFaceClient;

#[derive(Deserialize)]
pub struct DiscoverQuery {
    pub language: Option<String>,
}

pub async fn discover(
    Query(query): Query<DiscoverQuery>,
) -> Json<serde_json::Value> {
    let language = query.language.unwrap_or_else(|| "en-US".into());
    let lang_code = lang_to_code(&language);
    let client = HuggingFaceClient::new();

    let stt_query = if lang_code != "en" {
        format!("whisper-small-{}", lang_code)
    } else {
        "whisper-small".into()
    };

    let stt = client.search_models("automatic-speech-recognition", &stt_query, &language, 5).await
        .unwrap_or_default();
    let tts_query = if lang_code != "en" {
        format!("piper-tts-{}", lang_code)
    } else {
        "piper-tts".into()
    };
    let tts = client.search_models("text-to-speech", &tts_query, &language, 5).await
        .unwrap_or_default();
    let llm = client.search_models("text-generation", &format!("{}-small", lang_code), &language, 5).await
        .unwrap_or_default();

    let stt_ids: Vec<String> = stt.iter()
        .filter_map(|m| m.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .collect();
    let tts_ids: Vec<String> = tts.iter()
        .filter_map(|m| m.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .collect();
    let llm_ids: Vec<String> = llm.iter()
        .filter_map(|m| m.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .collect();

    let stt_default = vec!["openai/whisper-small".to_string()];
    let tts_default = vec!["rhasspy/piper-voices".to_string()];
    Json(serde_json::json!({
        "stt": if stt_ids.is_empty() { &stt_default } else { &stt_ids },
        "tts": if tts_ids.is_empty() { &tts_default } else { &tts_ids },
        "llm": llm_ids,
        "language": language,
        "total_found": stt.len() + tts.len() + llm.len(),
    }))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub query: Option<String>,
    pub language: Option<String>,
    pub limit: Option<usize>,
}

pub async fn search(
    Path(pipeline_tag): Path<String>,
    Query(query): Query<SearchQuery>,
) -> Json<serde_json::Value> {
    let q = query.query.unwrap_or_default();
    let language = query.language.unwrap_or_else(|| "".into());
    let limit = query.limit.unwrap_or(10);
    let client = HuggingFaceClient::new();
    let results = client.search_models(&pipeline_tag, &q, &language, limit).await
        .unwrap_or_default();

    Json(serde_json::json!({"models": results}))
}

const LANG_MAP: &[(&str, &str)] = &[
    ("en-US", "en"), ("en-GB", "en"), ("es-ES", "es"), ("es-MX", "es"),
    ("zh-CN", "zh"), ("zh-TW", "zh"), ("fr-FR", "fr"), ("de-DE", "de"),
    ("ja-JP", "ja"), ("ko-KR", "ko"), ("pt-BR", "pt"), ("ru-RU", "ru"),
    ("ar-SA", "ar"), ("it-IT", "it"), ("nl-NL", "nl"), ("pl-PL", "pl"),
    ("sv-SE", "sv"), ("tr-TR", "tr"), ("vi-VN", "vi"), ("th-TH", "th"),
    ("id-ID", "id"), ("ms-MY", "ms"), ("ro-RO", "ro"), ("uk-UA", "uk"),
    ("ca-ES", "ca"), ("fi-FI", "fi"), ("cs-CZ", "cs"), ("el-GR", "el"),
    ("he-IL", "he"), ("hi-IN", "hi"), ("hu-HU", "hu"), ("no-NO", "no"),
    ("sk-SK", "sk"), ("da-DK", "da"), ("bg-BG", "bg"), ("hr-HR", "hr"),
    ("lt-LT", "lt"), ("sl-SI", "sl"), ("et-EE", "et"), ("lv-LV", "lv"),
    ("sr-RS", "sr"), ("mk-MK", "mk"), ("sq-AL", "sq"),
];

fn lang_to_code(language: &str) -> &str {
    for (lang, code) in LANG_MAP {
        if *lang == language { return code; }
    }
    "en"
}
