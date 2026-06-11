use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Arc;
use axum::{
    Router, Json,
    extract::{State, Request},
    http::{HeaderValue, Method, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response, Html},
    routing::{get, post},
};
use tokio::sync::Mutex;
use tower_http::cors::{CorsLayer, AllowOrigin};

use crate::api::routes::*;
use crate::api::FrontendRouterState;
use crate::domain::ports::VaultRepository;
use crate::infrastructure::repositories::EncryptedVaultRepository;

#[derive(Clone)]
pub struct FrontendState {
    pub vault: Arc<Mutex<EncryptedVaultRepository>>,
    pub auth_token: String,
    pub backend_url: String,
    pub start_time: std::time::Instant,
    pub frontend_dir: PathBuf,
}

impl FrontendState {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        let auth_path = PathBuf::from(&home).join(".umbra/auth_token");
        let auth_token = if auth_path.exists() {
            std::fs::read_to_string(&auth_path).unwrap_or_default().trim().to_string()
        } else {
            use std::fmt::Write;
            let mut token = String::with_capacity(64);
            let since_epoch = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            for byte in since_epoch.as_nanos().to_ne_bytes() {
                write!(&mut token, "{:02x}", byte).ok();
            }
            if let Some(parent) = auth_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(&auth_path, &token).ok();
            let _ = std::fs::set_permissions(&auth_path, std::fs::Permissions::from_mode(0o600));
            token
        };

        let backend_url = std::env::var("UMBRA_API_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8484".into());

        Self {
            vault: Arc::new(Mutex::new(EncryptedVaultRepository::new())),
            auth_token,
            backend_url,
            start_time: std::time::Instant::now(),
            frontend_dir: PathBuf::from(
                std::env::var("UMBRA_FRONTEND_DIR")
                    .unwrap_or_else(|_| {
                        let cwd = std::env::current_dir().unwrap_or_default();
                        let candidates = [
                            cwd.join("frontend/dist"),
                            cwd.join("../frontend/dist"),
                            dirs::home_dir().map(|d| d.join("frontend/dist")).unwrap_or_default(),
                        ];
                        for c in &candidates {
                            if c.join("index.html").exists() {
                                return c.to_string_lossy().to_string();
                            }
                        }
                        "frontend/dist".into()
                    }),
            ),
        }
    }
}

pub fn build_frontend_router(state: FrontendState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(
            ["http://127.0.0.1:8340", "http://localhost:8340",
             "http://127.0.0.1:5173", "http://localhost:5173",
             "https://127.0.0.1:8340", "https://localhost:8340"]
                .iter().filter_map(|s| s.parse::<HeaderValue>().ok()),
        ))
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers(tower_http::cors::Any);

    let router_state = FrontendRouterState {
        vault: state.vault.clone(),
        auth_token: state.auth_token.clone(),
        backend_url: state.backend_url.clone(),
        start_time: state.start_time,
        frontend_dir: state.frontend_dir.clone(),
    };

    let public = Router::new()
        .route("/api/health", get(crate::api::auth::health))
        .route("/api/auth/session", get(crate::api::auth::session))
        .route("/api/setup/status", get(setup_routes::status))
        .route("/api/providers", get(provider_routes::list))
        .route("/api/providers/{id}", get(provider_routes::get))
        .route("/api/settings/voice", get(settings_routes::get_voice))
        .route("/api/customization", get(settings_routes::get_customization))
        .route("/api/models/discover", get(discovery_routes::discover))
        .route("/api/models/discover/{pipeline_tag}", get(discovery_routes::search))
        .route("/api/providers/config/status", get(provider_routes::config_status))
        .route("/", get(serve_index))
        .route("/{*path}", get(serve_static));

    let protected = Router::new()
        .route("/api/settings/voice", post(settings_routes::set_voice))
        .route("/api/settings/preferences", get(settings_routes::get_prefs).post(settings_routes::set_prefs))
        .route("/api/settings/status", get(settings_routes::status))
        .route("/api/customization", post(settings_routes::set_customization))
        .route("/api/providers/configure", post(provider_routes::configure))
        .route("/api/providers/test", post(provider_routes::test))
        .route("/api/providers/test-all", post(provider_routes::test_all))
        .route("/api/vault/status", get(vault_routes::status))
        .route("/api/vault/unlock", post(vault_routes::unlock))
        .route("/api/vault/lock", post(vault_routes::lock))
        .route("/api/vault/keys", get(vault_routes::list_keys))
        .route("/api/vault/key", post(vault_routes::set_key))
        .route("/api/vault/key/{provider_id}", get(vault_routes::get_key).delete(vault_routes::delete_key))
        .route("/api/vault/migrate", post(vault_routes::migrate))
        .route("/api/vault/auto-lock", post(vault_routes::auto_lock))
        .route("/api/setup/mode", post(setup_routes::set_mode))
        .route("/api/security/check", get(security_routes::security_check))
        .route("/api/browser/search-and-train", post(browser_routes::search_and_train))
        .route("/api/browser/visit", post(browser_routes::visit))
        .route("/api/browser/status", get(browser_routes::status))
        .route("/api/browser/collect", post(browser_routes::collect))
        .route("/api/browser/settings", get(browser_routes::settings).post(browser_routes::settings_update))
        .route("/api/tts-test", get(tts_test))
        .route("/ws/voice", get(voice_ws))
        .layer(middleware::from_fn_with_state(router_state.clone(), crate::api::middleware::auth::frontend_auth_middleware));

    public
        .merge(protected)
        .with_state(router_state)
        .layer(axum::middleware::from_fn(security_headers_middleware))
        .layer(cors)
}

pub use vault_routes::*;

// TTS handlers still in frontend module since they depend on FrontendState
use crate::infrastructure::http::tts_client::TtsClient;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use chrono::Timelike;
use regex::Regex;

static CORRECTIONS: &[(&str, &str)] = &[
    (r"(?i)\bcloud code\b", "Claude Code"),
    (r"(?i)\bclock code\b", "Claude Code"),
    (r"(?i)\bquad code\b", "Claude Code"),
    (r"(?i)\bclawed code\b", "Claude Code"),
    (r"(?i)\bclod code\b", "Claude Code"),
    (r"(?i)\bcloud\b", "Claude"),
    (r"(?i)\bquad\b", "Claude"),
    (r"(?i)\btravis\b", "UMBRA"),
    (r"(?i)\bjarves\b", "UMBRA"),
];

fn apply_speech_corrections(text: &str) -> String {
    let mut result = text.to_string();
    for (pattern, replacement) in CORRECTIONS {
        if let Ok(re) = Regex::new(pattern) {
            result = re.replace_all(&result, *replacement).to_string();
        }
    }
    result
}

fn strip_markdown_for_tts(text: &str) -> String {
    let mut result = text.to_string();
    if let Ok(re) = Regex::new(r"```[\s\S]*?```") {
        result = re.replace_all(&result, "").to_string();
    }
    result = result.replace('`', "");
    result = result.replace("**", "").replace("*", "");
    if let Ok(re) = Regex::new(r"(?m)^#{1,6}\s*") {
        result = re.replace_all(&result, "").to_string();
    }
    if let Ok(re) = Regex::new(r"\[([^\]]+)\]\([^\)]+\)") {
        result = re.replace_all(&result, "$1").to_string();
    }
    if let Ok(re) = Regex::new(r"(?m)^\s*[-*+]\s+") {
        result = re.replace_all(&result, "").to_string();
    }
    if let Ok(re) = Regex::new(r"(?m)^\s*\d+\.\s+") {
        result = re.replace_all(&result, "").to_string();
    }
    if let Ok(re) = Regex::new(r"\n{2,}") {
        result = re.replace_all(&result, ". ").to_string();
    }
    result = result.replace('\n', " ");
    if let Ok(re) = Regex::new(r"\s{2,}") {
        result = re.replace_all(&result, " ").to_string();
    }
    result.trim().trim_start_matches(',').trim_start_matches('—').trim_start_matches('-').to_string()
}

async fn synthesize_speech(text: &str, engine: &str, state: &FrontendState) -> Result<Vec<u8>, String> {
    let client = TtsClient::new();
    match engine {
        "nvidia-riva" => {
            let key = {
                let mut vault = state.vault.lock().await;
                vault.get_key("nvidia-riva").or_else(|| std::env::var("NVIDIA_API_KEY").ok())
            };
            match key {
                Some(k) => client.synthesize_nvidia(text, &k, "magpie-tts-multilingual").await
                    .map_err(|e| e.to_string()),
                None => Err("NVIDIA_API_KEY not configured".into()),
            }
        }
        _ => {
            let key = {
                let mut vault = state.vault.lock().await;
                vault.get_key("fish_audio").or_else(|| std::env::var("FISH_API_KEY").ok())
            };
            match key {
                Some(k) => {
                    let vid = std::env::var("FISH_VOICE_ID").ok()
                        .unwrap_or_else(|| "612b878b113047d9a770c069c8b4fdfe".into());
                    client.synthesize_fish(text, &k, &vid).await
                        .map_err(|e| e.to_string())
                }
                None => Err("FISH_API_KEY not configured".into()),
            }
        }
    }
}

async fn ask_umbra(message: &str, backend_url: &str) -> serde_json::Value {
    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "message": message,
        "conversation_id": null,
        "context": {},
    });
    match client
        .post(format!("{}/api/v1/chat", backend_url))
        .json(&payload)
        .send()
        .await
    {
        Ok(resp) if resp.status() == 200 => {
            resp.json::<serde_json::Value>().await.unwrap_or_else(|_| {
                serde_json::json!({"response": "I encountered a system error, sir."})
            })
        }
        Ok(resp) => {
            serde_json::json!({"response": format!("I'm having trouble reaching my core systems, sir. (HTTP {})", resp.status())})
        }
        Err(_) => {
            serde_json::json!({"response": "My core systems are offline, sir. Starting them up now. Please try again in a moment."})
        }
    }
}

async fn send_msg(socket: &mut WebSocket, value: serde_json::Value) {
    if let Ok(text) = serde_json::to_string(&value) {
        let _ = socket.send(Message::Text(text.into())).await;
    }
}

async fn handle_voice_socket(mut socket: WebSocket, state: FrontendState) {
    let auth_token = state.auth_token.clone();
    let backend_url = state.backend_url.clone();

    let auth_result = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            match socket.recv().await {
                Some(Ok(Message::Text(raw))) => {
                    if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&raw) {
                        if msg.get("type").and_then(|v| v.as_str()) == Some("auth") {
                            let token = msg.get("token").and_then(|v| v.as_str()).unwrap_or("");
                            return token == auth_token;
                        }
                    }
                }
                Some(Ok(Message::Close(_))) | None => return false,
                _ => continue,
            }
        }
    }).await;

    let authenticated = auth_result.unwrap_or(false);
    if !authenticated {
        let _ = socket.send(Message::Close(None)).await;
        return;
    }

    let hour = chrono::Local::now().hour();
    let greeting = if hour < 12 {
        "Good morning, sir."
    } else if hour < 17 {
        "Good afternoon, sir."
    } else {
        "Good evening, sir."
    };

    send_msg(&mut socket, serde_json::json!({"type": "status", "state": "speaking"})).await;
    if let Ok(audio) = synthesize_speech(greeting, "fish", &state).await {
        send_msg(&mut socket, serde_json::json!({
            "type": "audio",
            "data": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &audio),
            "text": greeting,
        })).await;
    }
    send_msg(&mut socket, serde_json::json!({"type": "status", "state": "idle"})).await;

    loop {
        let msg = match socket.recv().await {
            Some(Ok(Message::Text(raw))) => raw,
            Some(Ok(Message::Close(_))) | None => break,
            _ => continue,
        };

        let parsed: serde_json::Value = match serde_json::from_str(&msg) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if parsed.get("type").and_then(|v| v.as_str()) != Some("transcript") {
            continue;
        }
        if !parsed.get("isFinal").and_then(|v| v.as_bool()).unwrap_or(false) {
            continue;
        }

        let user_text = parsed.get("text").and_then(|v| v.as_str()).unwrap_or("");
        let user_text = apply_speech_corrections(user_text.trim());
        if user_text.is_empty() {
            continue;
        }

        send_msg(&mut socket, serde_json::json!({"type": "status", "state": "thinking"})).await;

        let umbra_resp = ask_umbra(&user_text, &backend_url).await;
        let response_text = umbra_resp.get("response")
            .and_then(|v| v.as_str())
            .unwrap_or("Understood, sir.")
            .to_string();

        let tts_text = strip_markdown_for_tts(&response_text);

        send_msg(&mut socket, serde_json::json!({"type": "status", "state": "speaking"})).await;
        match synthesize_speech(&tts_text, "fish", &state).await {
            Ok(audio) => {
                let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &audio);
                send_msg(&mut socket, serde_json::json!({
                    "type": "audio",
                    "data": b64,
                    "text": response_text,
                })).await;
            }
            Err(_) => {
                send_msg(&mut socket, serde_json::json!({
                    "type": "text",
                    "text": response_text,
                })).await;
            }
        }
        send_msg(&mut socket, serde_json::json!({"type": "status", "state": "idle"})).await;
    }
}

pub async fn voice_ws(
    ws: WebSocketUpgrade,
    State(state): State<FrontendRouterState>,
) -> impl IntoResponse {
    let frontend_state = FrontendState {
        vault: state.vault.clone(),
        auth_token: state.auth_token.clone(),
        backend_url: state.backend_url.clone(),
        start_time: state.start_time,
        frontend_dir: state.frontend_dir.clone(),
    };
    ws.on_upgrade(move |socket| handle_voice_socket(socket, frontend_state))
}

pub async fn tts_test(
    State(state): State<FrontendRouterState>,
) -> Json<serde_json::Value> {
    let frontend_state = FrontendState {
        vault: state.vault.clone(),
        auth_token: state.auth_token.clone(),
        backend_url: state.backend_url.clone(),
        start_time: state.start_time,
        frontend_dir: state.frontend_dir.clone(),
    };
    match synthesize_speech("Testing audio, sir.", "fish", &frontend_state).await {
        Ok(audio) => {
            let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &audio);
            Json(serde_json::json!({"audio": b64}))
        }
        Err(e) => {
            Json(serde_json::json!({"audio": null, "error": e}))
        }
    }
}

async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let h = response.headers_mut();
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"));
    h.insert("X-Content-Type-Options", HeaderValue::from_static("nosniff"));
    h.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
    h.insert("X-XSS-Protection", HeaderValue::from_static("1; mode=block"));
    h.insert("Referrer-Policy", HeaderValue::from_static("no-referrer"));
    h.insert("Permissions-Policy", HeaderValue::from_static("camera=(), microphone=(self), geolocation=(), interest-cohort=()"));
    response
}

fn mime_type(path: &str) -> &'static str {
    if path.ends_with(".js") { return "application/javascript"; }
    if path.ends_with(".css") { return "text/css"; }
    if path.ends_with(".html") { return "text/html"; }
    if path.ends_with(".png") { return "image/png"; }
    if path.ends_with(".jpg") || path.ends_with(".jpeg") { return "image/jpeg"; }
    if path.ends_with(".svg") { return "image/svg+xml"; }
    if path.ends_with(".ico") { return "image/x-icon"; }
    if path.ends_with(".woff2") { return "font/woff2"; }
    if path.ends_with(".woff") { return "font/woff"; }
    if path.ends_with(".ttf") { return "font/ttf"; }
    if path.ends_with(".json") { return "application/json"; }
    if path.ends_with(".map") { return "application/json"; }
    "application/octet-stream"
}

async fn serve_index(
    State(state): State<FrontendRouterState>,
) -> Response {
    let index_path = state.frontend_dir.join("index.html");
    match tokio::fs::read_to_string(&index_path).await {
        Ok(html) => Html(html).into_response(),
        Err(_) => Html(
            "<h1>UMBRA</h1><p>Frontend not built. Run: cd frontend && npm install && npm run build</p>"
        ).into_response(),
    }
}

async fn serve_static(
    axum::extract::Path(path): axum::extract::Path<String>,
    State(state): State<FrontendRouterState>,
) -> Response {
    let file_path = state.frontend_dir.join(&path);

    if !file_path.starts_with(&state.frontend_dir) {
        return (StatusCode::FORBIDDEN, "Forbidden").into_response();
    }

    let content = match tokio::fs::read(&file_path).await {
        Ok(c) => c,
        Err(_) => {
            let index_path = state.frontend_dir.join("index.html");
            return match tokio::fs::read_to_string(&index_path).await {
                Ok(html) => Html(html).into_response(),
                Err(_) => (StatusCode::NOT_FOUND, "Not found").into_response(),
            };
        }
    };

    let mime = mime_type(&path);
    ([(header::CONTENT_TYPE, mime)], content).into_response()
}
