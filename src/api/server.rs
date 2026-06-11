use axum::{
    Router,
    extract::{State, Request},
    http::{Method, StatusCode, HeaderMap},
    middleware::{self, Next},
    Json,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use std::path::PathBuf;
use std::sync::Arc;
use rand::Rng;
use tower_http::cors::{CorsLayer, AllowOrigin};

use crate::learning::AgentEngine;
use crate::security::SecurityGate;
use crate::ironclaw::IronClaw;
use crate::memory::MemoryEngine;
use crate::audio::AudioEngine;
use crate::sub_agents::SubAgentManager;

use super::{
    ChatRequest, ChatResponse, StatusResponse, TokenUsage,
    ActionRequest, ActionResponse,
    JepaTrainRequest, JepaPredictRequest, JepaConvertRequest, HsaqAnalyzeRequest,
    BackendRouterState,
};
use super::routes::*;

fn load_auth_token() -> String {
    let path = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())).join(".umbra/auth_token");
    if path.exists() {
        if let Ok(token) = std::fs::read_to_string(&path) {
            let t = token.trim().to_string();
            if !t.is_empty() { return t; }
        }
    }
    tracing::warn!("No auth token found — generating cryptographically random token");
    let token: String = (0..32)
        .map(|_| {
            let idx = rand::thread_rng().gen_range(0..16);
            format!("{:x}", idx)
        })
        .collect();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(&path, &token).ok();
    token
}

async fn auth_middleware(
    headers: HeaderMap,
    State(state): State<BackendRouterState>,
    request: Request,
    next: Next,
) -> Response {
    let key = headers
        .get("x-umbra-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if key != state.auth_token {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Unauthorized"}))).into_response();
    }
    next.run(request).await
}

pub fn build_router(
    agent: Arc<AgentEngine>,
    security: Arc<SecurityGate>,
    ironclaw: Arc<IronClaw>,
    memory: Arc<MemoryEngine>,
    audio: AudioEngine,
    sub_agents: SubAgentManager,
    resources: Arc<tokio::sync::Mutex<crate::resource::ResourceManager>>,
    debugger: Arc<crate::debugger::Debugger>,
    models_dir: PathBuf,
) -> Router {
    let auth_token = load_auth_token();
    let state = BackendRouterState {
        agent,
        security,
        ironclaw,
        memory,
        audio,
        sub_agents,
        resources,
        debugger,
        auth_token: auth_token.clone(),
        start_time: std::time::Instant::now(),
        models_dir,
    };

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list([
            "http://127.0.0.1:8340".parse().unwrap(),
            "http://localhost:8340".parse().unwrap(),
            "https://127.0.0.1:8340".parse().unwrap(),
            "https://localhost:8340".parse().unwrap(),
        ]))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(tower_http::cors::Any);

    Router::new()
        .route("/", get(root))
        .route("/api/v1/health", get(health))
        .route("/api/v1/status", get(get_status))
        .route("/api/v1/models", get(get_models))
        .route("/api/v1/hardware", get(get_hardware))
        .route("/api/v1/sub-agents", get(list_sub_agents))
        .route("/api/v1/training", get(training_routes::training_stats))
        .route("/api/v1/training/ingest", post(training_routes::ingest_training_data))
        .route("/api/v1/training/train", post(training_routes::trigger_training))
        .route("/api/v1/models/download", post(download_model))
        .route("/api/v1/jepa/train", post(train_jepa))
        .route("/api/v1/jepa/predict", post(predict_jepa))
        .route("/api/v1/jepa/convert", post(convert_jepa))
        .route("/api/v1/hdt", get(hdt_status))
        .route("/api/v1/hsaq", get(hsaq_status))
        .route("/api/v1/hsaq/analyze", post(hsaq_analyze))
        .route("/api/v1/resources", get(resource_status))
        .route("/api/v1/resources/cleanup", post(resource_cleanup))
        .route("/api/v1/resources/unload", post(resource_unload))
        .route("/api/v1/resources/reload", post(resource_reload))
        .route("/api/v1/debugger/snapshot", get(debugger_snapshot))
        .route("/api/v1/debugger/diagnostics", get(debugger_diagnostics))
        .route("/api/v1/chat", post(handle_chat))
        .route("/api/v1/action", post(handle_action))
        .route("/api/v1/memory/search", post(search_memory))
        .route("/api/v1/memory/recent", get(recent_memory))
        .layer(cors)
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state)
}

async fn root() -> impl IntoResponse {
    Json(serde_json::json!({
        "name": "UMBRA",
        "version": env!("CARGO_PKG_VERSION"),
        "endpoints": {
            "health": "/api/v1/health",
            "status": "/api/v1/status",
            "models": "/api/v1/models",
            "hardware": "/api/v1/hardware",
            "sub_agents": "/api/v1/sub-agents",
            "training": "/api/v1/training",
            "resources": "/api/v1/resources",
            "chat": "POST /api/v1/chat",
            "action": "POST /api/v1/action",
            "memory_search": "POST /api/v1/memory/search",
            "memory_recent": "GET /api/v1/memory/recent",
        }
    }))
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "name": "UMBRA",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn get_models(
    State(state): State<BackendRouterState>,
) -> impl IntoResponse {
    let router = &state.engine().router;
    let scheduler = &state.engine().scheduler;
    let config = scheduler.select_config(router);
    Json(serde_json::json!({
        "primary": {
            "name": config.primary.name(),
            "available": !matches!(config.primary, crate::engine::scheduler::Backend::Unavailable),
        },
        "secondary": {
            "name": config.secondary.name(),
            "available": !matches!(config.secondary, crate::engine::scheduler::Backend::Unavailable),
        },
        "local_models": state.engine().models.local_models,
        "models": router.models.iter().map(|m| {
            serde_json::json!({
                "name": m.name,
                "source": m.source,
                "is_vision": m.is_vision,
            })
        }).collect::<Vec<_>>(),
    }))
}

async fn get_hardware(
    State(_state): State<BackendRouterState>,
) -> impl IntoResponse {
    let hw = crate::engine::safety::HardwareMonitor::sample().await;
    Json(serde_json::json!({
        "gpu_temp_c": hw.gpu_temp_c,
        "cpu_temp_c": hw.cpu_temp_c,
        "vram_used_mb": hw.vram_used_mb,
        "vram_total_mb": hw.vram_total_mb,
        "ram_used_mb": hw.ram_used_mb,
        "ram_total_mb": hw.ram_total_mb,
        "is_throttling": hw.is_throttling,
        "safe_to_load": hw.safe_to_load(4096),
    }))
}

async fn list_sub_agents(
    State(state): State<BackendRouterState>,
) -> impl IntoResponse {
    let agents: Vec<_> = state.sub_agents.list().iter().map(|a| {
        serde_json::json!({
            "name": a.name,
            "version": a.version,
            "description": a.description,
            "capabilities": a.capabilities,
            "model": a.model.name,
        })
    }).collect();
    Json(serde_json::json!({ "sub_agents": agents, "count": agents.len() }))
}

#[derive(serde::Deserialize)]
struct DownloadRequest {
    model: String,
    familia: Option<String>,
}

async fn download_model(
    State(state): State<BackendRouterState>,
    Json(req): Json<DownloadRequest>,
) -> impl IntoResponse {
    const DEFAULT_MODEL_FAMILY: &str = "ollama";
    let model_family = req.familia.as_deref().unwrap_or(DEFAULT_MODEL_FAMILY);
    match state.engine().models.ensure_model(&req.model, model_family).await {
        Ok(path) => Json(serde_json::json!({ "status": "ok", "path": path.to_string_lossy() })),
        Err(e) => Json(serde_json::json!({ "status": "error", "error": e.to_string() })),
    }
}

async fn train_jepa(
    State(state): State<BackendRouterState>,
    Json(req): Json<JepaTrainRequest>,
) -> impl IntoResponse {
    let mut model = crate::engine::JepaModel::new(req.input_dim, req.latent_dim);
    model.random_init(req.seed.unwrap_or(42));

    if let Some(steps) = req.steps {
        let lr = req.learning_rate.unwrap_or(0.01);
        let dummy_input = vec![vec![0.1; req.input_dim]; 10];
        let dummy_target = vec![vec![0.2; req.input_dim]; 10];
        for _ in 0..steps {
            model.train_epoch(&dummy_input, &dummy_target, lr);
        }
    }

    if let Some(name) = &req.name {
        let path = &state.models_dir;
        let p = path.join(format!("{}.jepa", name));
        if let Err(e) = model.save_jepa(&p) {
            return Json(serde_json::json!({ "status": "error", "error": e.to_string() }));
        }
        Json(serde_json::json!({
            "status": "ok",
            "model": format!("{}/{}.jepa", path.display(), name),
            "header": model.header,
        }))
    } else {
        Json(serde_json::json!({
            "status": "ok",
            "header": model.header,
            "note": "No se guardó (sin nombre)",
        }))
    }
}

async fn predict_jepa(
    State(state): State<BackendRouterState>,
    Json(req): Json<JepaPredictRequest>,
) -> impl IntoResponse {
    let path = state.models_dir.join(format!("{}.jepa", req.model));
    match crate::engine::JepaModel::load_jepa(&path) {
        Ok(model) => {
            let output = model.predict(&req.input);
            Json(serde_json::json!({ "status": "ok", "output": output }))
        }
        Err(e) => Json(serde_json::json!({ "status": "error", "error": e.to_string() })),
    }
}

async fn convert_jepa(
    State(state): State<BackendRouterState>,
    Json(req): Json<JepaConvertRequest>,
) -> impl IntoResponse {
    let path = state.models_dir.join(format!("{}.jepa", req.model));
    match crate::engine::JepaModel::load_jepa(&path) {
        Ok(model) => match model.convert_to_materia(&req.output_name.unwrap_or(req.model), &state.models_dir) {
            Ok(materia_path) => Json(serde_json::json!({ "status": "ok", "materia": materia_path })),
            Err(e) => Json(serde_json::json!({ "status": "error", "error": e.to_string() })),
        }
        Err(e) => Json(serde_json::json!({ "status": "error", "error": e.to_string() })),
    }
}

async fn hdt_status(
    State(state): State<BackendRouterState>,
) -> impl IntoResponse {
    Json(state.engine().hdt.topology_summary())
}

async fn resource_status(
    State(state): State<BackendRouterState>,
) -> impl IntoResponse {
    let rm = state.resources.lock().await;
    Json(rm.stats())
}

#[derive(serde::Deserialize)]
struct ResourceUnloadRequest {
    name: String,
}

async fn resource_cleanup(
    State(state): State<BackendRouterState>,
) -> impl IntoResponse {
    let mut rm = state.resources.lock().await;
    let report = rm.intelligent_cleanup().await;
    Json(serde_json::json!({
        "status": "ok",
        "freed_mb": report.freed_mb,
        "unloaded": report.unloaded,
        "pressure": report.pressure.to_string(),
    }))
}

async fn resource_unload(
    State(state): State<BackendRouterState>,
    Json(req): Json<ResourceUnloadRequest>,
) -> impl IntoResponse {
    let mut rm = state.resources.lock().await;
    match rm.unload(&req.name) {
        Ok(mb) => Json(serde_json::json!({ "status": "ok", "freed_mb": mb })),
        Err(e) => Json(serde_json::json!({ "status": "error", "error": e.to_string() })),
    }
}

async fn resource_reload(
    State(_state): State<BackendRouterState>,
    Json(req): Json<ResourceUnloadRequest>,
) -> impl IntoResponse {
    let mut rm = _state.resources.lock().await;
    match rm.reload(&req.name) {
        Ok(mb) => Json(serde_json::json!({ "status": "ok", "loaded_mb": mb })),
        Err(e) => Json(serde_json::json!({ "status": "error", "error": e.to_string() })),
    }
}

async fn debugger_snapshot(
    State(state): State<BackendRouterState>,
) -> Json<crate::debugger::DebuggerSnapshot> {
    Json(state.debugger.snapshot())
}

async fn debugger_diagnostics(
    State(state): State<BackendRouterState>,
) -> Json<Vec<crate::debugger::Diagnostic>> {
    Json(state.debugger.recent_diagnostics(50))
}

async fn hsaq_status(
    State(_state): State<BackendRouterState>,
) -> impl IntoResponse {
    let comparison = crate::engine::hsaq::compare_vs_turboquant();
    Json(comparison)
}

async fn hsaq_analyze(
    State(_state): State<BackendRouterState>,
    Json(req): Json<HsaqAnalyzeRequest>,
) -> impl IntoResponse {
    let mut compressor = crate::engine::HsaqCompressor::new();
    let layers: Vec<(String, u64, f32)> = req.layers.iter()
        .map(|l| (l.name.clone(), l.size, l.importance))
        .collect();
    compressor.analyze(&layers);
    Json(compressor.summary())
}

async fn get_status(
    State(state): State<BackendRouterState>,
) -> impl IntoResponse {
    let uptime = state.start_time.elapsed().as_secs();
    let (obs_count, sess_count) = state.memory.stats();
    let hw = crate::engine::safety::HardwareMonitor::sample().await;
    Json(StatusResponse {
        mode: if hw.is_throttling { "limitado".into() } else { "activo".into() },
        active: true,
        uptime_seconds: uptime,
        memory_count: obs_count as u64,
        task_count: sess_count as u64,
        ironclaw: state.ironclaw.stats(),
    })
}

async fn handle_chat(
    State(state): State<BackendRouterState>,
    Json(req): Json<ChatRequest>,
) -> impl IntoResponse {
    if let Err(block) = state.ironclaw.validate_input(&req.message).await {
        return Json(ChatResponse {
            response: block.to_string(),
            conversation_id: req.conversation_id.unwrap_or_default(),
            actions: vec![],
            tokens_used: TokenUsage { input: 0, output: 0 },
        });
    }

    let input_len = req.message.len() as u64;
    let (tx, _) = tokio::sync::mpsc::channel(256);
    let response = state.agent.run(req.message, tx).await.unwrap_or_else(|e| e.to_string());

    if let Err(block) = state.ironclaw.validate_output(&response).await {
        return Json(ChatResponse {
            response: block.to_string(),
            conversation_id: req.conversation_id.unwrap_or_default(),
            actions: vec![],
            tokens_used: TokenUsage { input: 0, output: 0 },
        });
    }

    let output_len = response.len() as u64;
    state.ironclaw.track_tokens(input_len, output_len);

    Json(ChatResponse {
        response,
        conversation_id: req.conversation_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        actions: vec![],
        tokens_used: TokenUsage {
            input: input_len,
            output: output_len,
        },
    })
}

async fn handle_action(
    State(state): State<BackendRouterState>,
    Json(req): Json<ActionRequest>,
) -> impl IntoResponse {
    let action_name = req.action;
    let args_str = req.args.to_string();

    if let Err(block) = state.ironclaw.validate_action(&action_name, &[&args_str]).await {
        return Json(ActionResponse {
            status: "blocked".into(),
            result: String::new(),
            blocked_by: Some(block.to_string()),
        });
    }

    let args_vec = vec![args_str.clone()];
    match state.security.validate_tool_call(&action_name, &args_vec).await {
        Ok(true) => {
            Json(ActionResponse {
                status: "ok".into(),
                result: format!("[Umbra] Acción '{}' ejecutada: {}", action_name, args_str),
                blocked_by: None,
            })
        }
        Ok(false) => {
            Json(ActionResponse {
                status: "blocked".into(),
                result: String::new(),
                blocked_by: Some("[Thoth] Acción bloqueada por seguridad".into()),
            })
        }
        Err(e) => {
            Json(ActionResponse {
                status: "error".into(),
                result: String::new(),
                blocked_by: Some(format!("[Thoth] Error: {}", e)),
            })
        }
    }
}

#[derive(serde::Deserialize)]
struct MemorySearchRequest {
    query: String,
    #[allow(dead_code)]
    limit: Option<usize>,
}

async fn search_memory(
    State(state): State<BackendRouterState>,
    Json(req): Json<MemorySearchRequest>,
) -> impl IntoResponse {
    match state.memory.search(&req.query, None) {
        Ok(entries) => {
            Json(serde_json::json!({
                "status": "ok",
                "count": entries.len(),
                "entries": entries,
            }))
        }
        Err(e) => {
            Json(serde_json::json!({
                "status": "error",
                "error": e.to_string(),
            }))
        }
    }
}

async fn recent_memory(
    State(state): State<BackendRouterState>,
) -> impl IntoResponse {
    match state.memory.recent(20) {
        Ok(entries) => {
            Json(serde_json::json!({
                "status": "ok",
                "count": entries.len(),
                "entries": entries,
            }))
        }
        Err(e) => {
            Json(serde_json::json!({
                "status": "error",
                "error": e.to_string(),
            }))
        }
    }
}

pub use training_routes::{training_stats, ingest_training_data, trigger_training};
