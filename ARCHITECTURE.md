# UMBRA — Architecture Document

**Version:** 0.1.0
**Last Updated:** 2026-06-09

---

## Evolution History

### Phase 1: Python + TypeScript (Original)
- **Python FastAPI server** (`server.py`) acting as voice frontend (JARVIS) with WebSocket realtime voice I/O, intent classification, desktop integration (screen/mail/calendar), browser control via Playwright
- **TypeScript vanilla frontend** (`settings.ts`, `orb.ts`, `ws.ts`) — Vue.js + Vite web UI for voice frontend
- **Rust backend** — Axum-based API server on port 8484, MateriaCore engine, SecurityGate, IronClaw guard, Synapsis memory, WASM sandbox, JEPA inference engine, audio synthesis
- **Communication:** HTTP REST + WebSocket between Python JARVIS frontend and Rust backend
- **Build:** npm for frontend, cargo for Rust, Python venv for JARVIS

### Phase 2: Angular + Rust Migration
- Python `server.py` functionality ported to Rust (`src/frontend/`) — voice websocket handler, TTS synthesis, speech correction, static file serving
- **Angular 19 frontend** replacing vanilla TypeScript/Vue.js — Clean Architecture layering (domain/data/use-cases/presentation)
- **Electron desktop wrapper** (`electron/main.js`) — tray icon, backend lifecycle management, frameless transparent window
- **Vault system** (AES-256-GCM) — encrypted credential storage at `~/.umbra/vault.enc` with PBKDF2 key derivation (600K iterations), auto-lock mechanism
- **Dual-backend support** — Angular frontend talks to Rust OR Python backend via configurable `UMBRA_API_URL`

### Phase 3: Clean Architecture + Desktop App (Current)
- **Rust backend restructured** into Clean Architecture layers:
  - `domain/` — pure models (`ApiProvider`, `VaultEntry`, `TrainingExample`, `SystemConfig`), ports (`VaultRepository`, `ProviderRepository`, `SettingsRepository`), domain errors
  - `application/` — use cases (`providers/configure`, `vault/unlock`, `settings/save_voice`, `training/trigger`)
  - `infrastructure/` — repository implementations (`EncryptedVaultRepository`, `TomlProviderRepository`), HTTP clients (Ollama, HuggingFace, TTS), persistence (vault encryption with AES-256-GCM + PBKDF2)
  - `api/` — Axum router, auth middleware, route handlers grouped by domain
- **Angular frontend** follows Clean Architecture:
  - `domain/` — models, enums, interfaces (repository, use case contracts)
  - `data/` — DTOs, repository implementations (HTTP calls to Rust backend)
  - `use-cases/` — business logic orchestrators for providers, vault, settings, training
  - `presentation/` — pages (Monitor, Chat, Settings), components (Orb, HUD, StatusBar, WindowControls), shared directives/pipes
  - `core/` — services (API, Auth, WebSocket, Backend, i18n), guards, interceptors
- **Desktop focus** — Electron primary distribution target; egui native GUI also supported
- **Background processing** — `JobQueue` (background tasks: training, browser collection, backup, cleanup), `RateLimiter` (token bucket), `TtlCache` (response caching for providers)
- **Supply chain** — `cargo-deny` + `cargo-vet` configured, 0 advisories, 2 ignored (fxhash, paste — both transitive via wasmtime/eframe)
- **pnpm + fnm** — migrated from npm, workspace configured in `pnpm-workspace.yaml`
- **Synapsis memory** — using external `synapsis` crate (path dep) for persistent memory with embeddings

---

## Current Architecture

### Layers (Rust Backend)

```
┌──────────────────────────────────────────────────────────────┐
│  API LAYER (api/)                                            │
│  ┌───────────┐ ┌───────────┐ ┌────────────┐ ┌────────────┐ │
│  │ server.rs │ │ auth.rs   │ │ routes/    │ │ middleware │ │
│  │ (Axum)    │ │ (session) │ │ (8 files)  │ │ (auth)     │ │
│  └─────┬─────┘ └───────────┘ └──────┬─────┘ └────────────┘ │
├────────┴────────────────────────────┴────────────────────────┤
│  APPLICATION LAYER (application/)                            │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────────────┐  │
│  │providers/│ │ vault/   │ │settings/ │ │ training/     │  │
│  │configure │ │unlock    │ │get_voice │ │trigger        │  │
│  │test_conn │ │lock      │ │save_voice│ │get_stats      │  │
│  │get_status│ │migrate   │ │get_status│ │               │  │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └──────┬────────┘  │
├───────┴──────────────┴────────────┴────────────────┴────────┤
│  DOMAIN LAYER (domain/)                                      │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐                    │
│  │ models/  │ │ ports/   │ │ errors/  │                    │
│  │Provider  │ │VaultRepo │ │AppError  │                    │
│  │VaultEntry│ │ProvRepo  │ │(axum     │                    │
│  │Training  │ │Settings  │ │ IntoResp)│                    │
│  │SystemCfg │ │          │ │          │                    │
│  └────┬─────┘ └────┬─────┘ └──────────┘                    │
├───────┴────────────┴────────────────────────────────────────┤
│  INFRASTRUCTURE LAYER (infrastructure/)                      │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐                    │
│  │repos/    │ │ http/    │ │persist/  │                    │
│  │vault_repo│ │ollama_cli│ │vault_    │                    │
│  │prov_repo │ │hf_client │ │encryption│                    │
│  │set_repo  │ │tts_client│ │          │                    │
│  │train_repo│ │          │ │          │                    │
│  └──────────┘ └──────────┘ └──────────┘                    │
├──────────────────────────────────────────────────────────────┤
│  CROSS-CUTTING (engine/ + security/ + learning/ + ...)       │
│  MateriaCore │ SecurityGate │ IronClaw │ AgentEngine         │
│  JEPA │ HDT │ HSAQ │ SNN │ WASM │ Router │ Scheduler         │
│  RuntimeEnforce │ PQC │ ZT Gate │ Audit │ AntiBrick          │
│  SubAgents │ Audio │ Persona │ Memory (Synapsis)             │
│  JobQueue │ RateLimiter │ TtlCache │ Debugger                │
└──────────────────────────────────────────────────────────────┘
```

#### Layer Responsibilities
- **Domain** — Pure Rust structs/enums/traits with zero dependencies. Models (`ApiProvider`, `VaultContents`, `TrainingExample`, `VoiceSettings`, `Preferences`), ports (trait interfaces for repositories), domain errors (mapped to HTTP responses via axum `IntoResponse`)
- **Application** — Use case orchestration. Each module is a thin wrapper that coordinates domain logic with infrastructure. Examples: `vault::unlock` decrypts vault and manages passphrase lifecycle; `providers::configure` saves provider config to TOML
- **Infrastructure** — Concrete implementations of domain ports. `EncryptedVaultRepository` (AES-256-GCM file I/O), `TomlProviderRepository` (TOML config read/write), HTTP clients (Ollama, HuggingFace, TTS). Persistence module handles vault key derivation (PBKDF2-SHA256, 600K iterations) and encryption/decryption.
- **API** — Axum router with two separate servers (backend on `:8484`, frontend+API on `:8340`). Routes are grouped: `provider_routes`, `vault_routes`, `settings_routes`, `training_routes`, `setup_routes`, `discovery_routes`, `browser_routes`, `security_routes`. Middleware handles auth via `x-umbra-key` header. Frontend routes serve Angular static files.
- **Engine** — `MateriaCore` containing JEPA inference engine, multi-backend model router (llama.cpp/Ollama/Cloud), Synapsis memory, adaptive hardware scheduler, WASM sandbox, SNN classifier, HDT hyperdimensional routing, HSAQ compression, model manager, hardware monitor.
- **Security** — `SecurityGate` wrapping RuntimeEnforcer (tool call validation <100ms), PQC crypto engine (Kyber/Dilithium), Zero-Trust Gate, AntiBrick (destructive operation prevention), AuditWorm (hash-chained immutable logging).
- **Learning** — `AgentEngine` containing UmbraAgent (plan→act→observe→learn cycle), SkillManager, TrainerEngine (JEPA training pipeline), MessagingGateway (Telegram/Discord/Signal/Email).
- **Infra** — `Infrastructure` containing OpenVentus (Linux hardening), GhostMonitor (watchdog/anomaly detection), BackupEngine (encrypted backup).

### Layers (Angular Frontend)

```
┌────────────────────────────────────────────────────────────┐
│  PRESENTATION LAYER (presentation/)                         │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐ │
│  │ pages/   │ │components│ │ shared/  │ │              │ │
│  │ Monitor  │ │ Orb      │ │ ripple   │ │              │ │
│  │ Chat     │ │ HUD      │ │ translate│ │              │ │
│  │ Settings │ │ StatusBar│ │          │ │              │ │
│  │  (5 sec) │ │ WinCtrls │ │          │ │              │ │
│  └────┬─────┘ └────┬─────┘ └──────────┘ └──────────────┘ │
├───────┴─────────────┴──────────────────────────────────────┤
│  USE-CASES LAYER (use-cases/)                              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐ │
│  │providers/│ │ vault/   │ │settings/ │ │ training/    │ │
│  │get/test  │ │unlock/   │ │save-voice│ │get-stats     │ │
│  │configure │ │lock/     │ │save-prefs│ │trigger       │ │
│  │          │ │migrate   │ │get-status│ │              │ │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └──────┬───────┘ │
├───────┴──────────────┴────────────┴────────────────┴───────┤
│  DATA LAYER (data/)                                         │
│  ┌─────────────────┐ ┌────────────────────────────────┐    │
│  │ repositories/   │ │ dto/                           │    │
│  │ provider.repo   │ │ provider.dto.ts                │    │
│  │ vault.repo      │ │ vault.dto.ts                   │    │
│  │ settings.repo   │ │ settings.dto.ts                │    │
│  │ training.repo   │ │                                │    │
│  │ hardware.repo   │ │                                │    │
│  └────────┬────────┘ └────────────────────────────────┘    │
├───────────┴─────────────────────────────────────────────────┤
│  DOMAIN LAYER (domain/)                                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐                    │
│  │ models/  │ │ enums/   │ │ interfaces/                   │
│  │ provider │ │ system-  │ │ use-case.                     │
│  │ hardware │ │ mode     │ │ interface                     │
│  │ training │ │ provider-│ │ repository.                   │
│  │ agent-   │ │ type     │ │ interface                     │
│  │ state    │ │ agent-   │ │                               │
│  │ user     │ │ state    │ │                               │
│  └──────────┘ └──────────┘ └──────────┘                    │
├─────────────────────────────────────────────────────────────┤
│  CORE LAYER (core/)                                         │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐  │
│  │services/ │ │auth      │ │intercepts│ │ guards/      │  │
│  │api/auth  │ │backend   │ │          │ │              │  │
│  │websocket │ │i18n      │ │auth int. │ │auth.guard    │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

#### Layer Responsibilities
- **Domain** — Pure TypeScript models, enums (`SystemMode`, `ProviderType`, `AgentState`), interfaces (`UseCase`, `Repository`)
- **Data** — DTOs matching API payloads, repository implementations that make HTTP calls to the Rust backend
- **Use Cases** — Business logic orchestrators that call repositories and transform data for presentation
- **Presentation** — Angular components organized by feature (pages) and shared (components/directives/pipes)
- **Core** — Singleton services (API client, auth, WebSocket, backend status, i18n), HTTP interceptors (auth token injection), route guards

### Desktop Application

```
┌────────────────────────────────────────────────────┐
│  ELECTRON SHELL (electron/main.js)                  │
│                                                     │
│  ┌─────────────────┐  ┌────────────────────────┐   │
│  │  BrowserWindow   │  │  System Tray           │   │
│  │  - frameless     │  │  - Start/Stop Backend  │   │
│  │  - transparent   │  │  - Status Indicator    │   │
│  │  - 1400x900      │  │  - Quit                │   │
│  │  - loads Angular │  │                        │   │
│  │    dist/browser  │  │                        │   │
│  └────────┬─────────┘  └────────────────────────┘   │
│           │                                          │
│  ┌────────┴─────────────────────────────────────┐   │
│  │  Backend Process Management                   │   │
│  │  - Spawns `umbra --api-port 8484 --no-frontend│   │
│  │  - Health check polling (1s interval)         │   │
│  │  - Auto-start on window creation              │   │
│  │  - Kill on quit                               │   │
│  └────────────────────────────────────────────────┘   │
│                                                     │
│  IPC: window-minimize/maximize/close/boot-complete   │
│  Shortcut: Cmd+Shift+U → activate voice              │
└────────────────────────────────────────────────────┘
         │
         │ spawns + monitors
         ▼
┌────────────────────────────────────────────────────┐
│  RUST BACKEND (target/release/umbra)                │
│  - API server on :8484                              │
│  - Serves frontend static files on :8340            │
│  - Engine, security, memory, audio, sub-agents      │
│  - Optional TLS (auto self-signed cert)             │
└────────────────────────────────────────────────────┘
         ▲
         │ HTTP/WS
┌────────────────────────────────────────────────────┐
│  ANGULAR FRONTEND ( Electron loads dist/browser )   │
│  - Talks to Rust backend API at localhost:8484      │
│  - Voice WebSocket at ws://localhost:8340/ws/voice  │
│  - Monitor, Chat, Settings pages                    │
└────────────────────────────────────────────────────┘
```

The egui desktop GUI (`src/desktop/mod.rs`) is an alternative native interface built with `eframe`/`egui` that communicates with the same backend over HTTP. It provides tabs for Monitor, Providers, Models, Agents, Training, and Debugger.

### Data Flow

```
User speaks
    │
    ▼
┌────────────────────────────────────────────────────────┐
│  ANGULAR FRONTEND / ELECTRON                          │
│  - Speech → WebSocket to Rust frontend server (:8340) │
│  - Voice activation via Cmd+Shift+U or wake word      │
└──────────────────────┬─────────────────────────────────┘
                       │ WS /ws/voice (transcript JSON)
                       ▼
┌────────────────────────────────────────────────────────┐
│  RUST FRONTEND SERVER (src/frontend/mod.rs)             │
│  - Auth check (auth_token from ~/.umbra/auth_token)    │
│  - Speech correction regex replacements                 │
│  - Forwards to backend via HTTP POST /api/v1/chat      │
│  - Synthesizes response via Fish.Audio or NVIDIA Riva  │
│  - Returns audio bytes (base64) + text over WebSocket  │
└──────────────────────┬─────────────────────────────────┘
                       │ HTTP POST /api/v1/chat
                       ▼
┌────────────────────────────────────────────────────────┐
│  RUST BACKEND API (api/server.rs :8484)                 │
│  - Auth middleware (x-umbra-key header check)          │
│  - Routes to AgentEngine::run()                        │
│  - validates via SecurityGate                          │
│  - enforces limits via IronClaw                        │
│  - logs via Debugger                                   │
└──────────────────────┬─────────────────────────────────┘
                       │
                       ▼
┌────────────────────────────────────────────────────────┐
│  AGENT ENGINE (learning/agent_loop.rs)                  │
│  1. Load context from Synapsis memory                  │
│  2. Infer via MateriaCore (JEPA + backend router)      │
│  3. Execute tool calls (validated by SecurityGate)     │
│  4. Store conversation in memory                       │
│  5. Optionally trigger training                         │
└──────────────────────┬─────────────────────────────────┘
                       │ response JSON
                       ▼
┌────────────────────────────────────────────────────────┐
│  RESPONSE CHAIN                                        │
│  Backend → Frontend Server → TTS synthesis → WebSocket │
│  → Angular displays text + plays audio                 │
└────────────────────────────────────────────────────────┘
```

---

## Key Design Decisions

### Why Rust?
- **Performance** — JEPA inference, SNN classification, HDT routing all benefit from zero-cost abstractions
- **Memory safety** — WASM sandbox, audio pipelines, security-sensitive operations
- **Concurrency** — Tokio async for HTTP servers, WebSocket connections, background jobs
- **FFI** — MT4 bridge via C ABI needs no-GC language
- **Supply chain** — cargo-deny/vet for dependency auditing

### Why Angular?
- **Clean Architecture by convention** — the Angular CLI encourages modular, layered code
- **Reactive streams** — RxJS for WebSocket voice streaming, real-time monitoring
- **Type safety** — TypeScript interfaces match domain models
- **Mature ecosystem** — Electron integration, build tooling, testing

### Why AES-256-GCM?
- **Authenticated encryption** — provides both confidentiality and integrity verification
- **Hardware acceleration** — AES-NI instructions on modern CPUs
- **Nonce-based** — deterministic encryption with random 12-byte nonce per operation
- **PBKDF2 with 600K iterations** — key derivation resistant to GPU/ASIC brute force

### Why Vault (not .env)?
- **Encrypted at rest** — API keys encrypted on disk, only decrypted in memory when unlocked
- **Auto-lock** — configurable timeout (default 15 min) auto-locks vault after inactivity
- **Migration path** — automatic migration from legacy `.env` and `providers.toml`
- **Multi-provider** — supports 23 API providers with individual key management
- **Auth binding** — vault key derived from both passphrase AND `~/.umbra/auth_token`

### Why Synapsis?
- **Persistent memory** — session-based observations with semantic embeddings
- **PQC integration** — memory encryption with post-quantum cryptography
- **Proven in trading context** — built for algorithmic trading agent memory

### Why Electron + egui?
- **Electron** — mature desktop shell, system tray, auto-updater, cross-platform packaging
- **egui** — instant-load native GUI for debugging/monitoring without browser overhead
- **Both** — Electron is primary distribution; egui is lightweight diagnostic fallback

---

## Project Structure

```
/mnt/external/projects/umbra/
├── Cargo.toml              # Rust workspace manifest (single crate)
├── Cargo.lock              # Dependency lockfile
├── deny.toml               # cargo-deny configuration (advisories, licenses)
├── install.sh              # Build + install script (Rust + Python venv + launcher)
├── uninstall.sh            # Uninstallation script
├── build-appimage.sh       # AppImage builder for Linux distribution
├── README.md               # Project readme
├── SECURITY.md             # Security policy
│
├── src/                    # Rust backend source
│   ├── main.rs             # CLI entrypoint (clap), TLS setup, dual server startup
│   ├── lib.rs              # Library root: UmbraApp init, module declarations
│   ├── config.rs           # TOML config loader (~/.umbra/config.toml)
│   ├── rate_limiter.rs     # ActionRateLimiter (token bucket, 30 actions/60s)
│   ├── cache.rs            # TtlCache + ProviderCache (response caching)
│   ├── job_queue.rs        # Background job queue (training, browser, backup)
│   ├── vault.rs            # Legacy VaultReader (Python vault interop)
│   ├── resource.rs         # ResourceManager (model/audio/cache/sub-agent tracking)
│   ├── memory.rs           # MemoryEngine (Synapsis observation/session management)
│   ├── debugger.rs         # Debugger (background logging, severity reports)
│   │
│   ├── domain/             # Clean Architecture — Domain Layer
│   │   ├── mod.rs
│   │   ├── models/         # Pure domain models
│   │   │   ├── mod.rs
│   │   │   ├── provider.rs          # ApiProvider + ALL_PROVIDERS (23 providers)
│   │   │   ├── vault_entry.rs       # VaultContents, VaultKeyEntry, VaultStatus
│   │   │   ├── training_example.rs  # TrainingExample
│   │   │   └── system_config.rs     # VoiceSettings, Preferences, CustomizationBody
│   │   ├── ports/          # Repository trait interfaces
│   │   │   ├── mod.rs
│   │   │   ├── vault_port.rs       # VaultRepository trait
│   │   │   ├── provider_port.rs    # ProviderRepository trait
│   │   │   └── settings_port.rs    # SettingsRepository trait
│   │   └── errors/         # Domain errors mapped to HTTP responses
│   │       └── mod.rs     # AppError enum (NotFound, Unauthorized, VaultLocked, etc.)
│   │
│   ├── application/        # Clean Architecture — Application Layer
│   │   ├── mod.rs
│   │   ├── providers/      # Provider configuration use cases
│   │   │   ├── mod.rs
│   │   │   ├── configure.rs       # Configure provider settings
│   │   │   ├── test_connection.rs # Test provider API connection
│   │   │   └── get_status.rs      # Get provider configuration status
│   │   ├── vault/          # Vault management use cases
│   │   │   ├── mod.rs
│   │   │   ├── unlock.rs          # Unlock vault with passphrase
│   │   │   ├── lock.rs            # Lock vault
│   │   │   └── migrate.rs         # Migrate from .env/providers.toml
│   │   ├── settings/       # Settings management use cases
│   │   │   ├── mod.rs
│   │   │   ├── get_voice.rs       # Get voice settings
│   │   │   ├── save_voice.rs      # Save voice settings
│   │   │   └── get_status.rs      # Get system status
│   │   └── training/       # Training use cases
│   │       ├── mod.rs
│   │       ├── trigger.rs         # Trigger training
│   │       └── get_stats.rs       # Get training statistics
│   │
│   ├── infrastructure/     # Clean Architecture — Infrastructure Layer
│   │   ├── mod.rs
│   │   ├── repositories/   # Repository implementations
│   │   │   ├── mod.rs
│   │   │   ├── vault_repo.rs      # EncryptedVaultRepository (AES-256-GCM)
│   │   │   ├── provider_repo.rs   # TomlProviderRepository (TOML file)
│   │   │   ├── settings_repo.rs   # TomlSettingsRepository (TOML file)
│   │   │   └── training_repo.rs   # TrainingRepository (JSON file)
│   │   ├── http/           # HTTP client implementations
│   │   │   ├── mod.rs
│   │   │   ├── ollama_client.rs   # Ollama local LLM client
│   │   │   ├── huggingface_client.rs  # HuggingFace model discovery
│   │   │   └── tts_client.rs     # TTS synthesis clients (Fish.Audio, NVIDIA Riva)
│   │   └── persistence/    # Data persistence utilities
│   │       ├── mod.rs
│   │       └── vault_encryption.rs  # AES-256-GCM + PBKDF2 encryption/decryption
│   │
│   ├── api/                # HTTP API layer
│   │   ├── mod.rs          # Shared types (ChatRequest/Response, router state)
│   │   ├── server.rs       # Axum router builder, auth middleware, route mounting
│   │   ├── auth.rs         # Health check, session auth endpoints
│   │   ├── middleware/
│   │   │   └── auth.rs     # Frontend auth middleware (x-umbra-key validation)
│   │   └── routes/         # Route handlers
│   │       ├── mod.rs
│   │       ├── provider_routes.rs  # API provider CRUD
│   │       ├── vault_routes.rs     # Vault unlock/lock/key management
│   │       ├── settings_routes.rs  # Voice/preferences/customization settings
│   │       ├── training_routes.rs  # Training trigger/stats
│   │       ├── setup_routes.rs     # Setup status, system mode
│   │       ├── discovery_routes.rs # HuggingFace model discovery
│   │       ├── browser_routes.rs   # Browser automation (search, visit, collect)
│   │       └── security_routes.rs  # Security check endpoint
│   │
│   ├── frontend/           # Frontend server (serves Angular + voice WS)
│   │   └── mod.rs          # FrontendState, build_frontend_router, voice WS handler,
│   │                        # TTS synthesis, static file serving, speech corrections
│   │
│   ├── engine/             # Core engine (MateriaCore)
│   │   ├── mod.rs          # MateriaCore struct, inference methods
│   │   ├── jepa.rs         # JEPA inference engine (Joint Embedding Predictive Architecture)
│   │   ├── jepa_model/     # JEPA model implementation
│   │   │   └── mod.rs
│   │   ├── router.rs       # Multi-backend model router
│   │   ├── scheduler.rs    # Adaptive hardware scheduler (backend selection)
│   │   ├── memory.rs       # Synapsis memory integration
│   │   ├── wasm.rs         # WASM sandbox runner (wasmtime)
│   │   ├── snn.rs          # Spiking Neural Network classifier
│   │   ├── models/         # Model manager, local model scanning
│   │   │   └── mod.rs
│   │   ├── safety.rs       # Hardware monitor (temperature, VRAM, throttling)
│   │   ├── hdt_router.rs   # Hyperdimensional computing router
│   │   └── hsaq.rs         # HSAQ (Hierarchical Stochastic Adaptive Quantization)
│   │
│   ├── security/           # Security layer (SecurityGate)
│   │   ├── mod.rs          # SecurityGate struct
│   │   ├── enforcer.rs     # RuntimeEnforcer — tool call validation <100ms
│   │   ├── pqc.rs          # PQC crypto engine (Kyber-512 KEM, Dilithium-4)
│   │   ├── zt_gate.rs      # ZeroTrustGate — per-command risk analysis
│   │   ├── antibrick.rs    # AntiBrick — destructive operation prevention
│   │   └── audit.rs        # AuditWorm — hash-chained immutable logging
│   │
│   ├── learning/           # Agent learning (AgentEngine)
│   │   ├── mod.rs          # AgentEngine struct
│   │   ├── agent_loop.rs   # UmbraAgent — plan→act→observe→learn cycle
│   │   ├── skills.rs       # SkillManager — skill creation, versioning, discovery
│   │   ├── trainer.rs      # TrainerEngine — JEPA training pipeline
│   │   └── messaging.rs    # MessagingGateway — Telegram/Discord/Signal/Email
│   │
│   ├── providers/          # LLM provider integration
│   │   └── mod.rs          # ModelProvider, ProviderRegistry, chat/completions API
│   │
│   ├── bridge/             # MT4 trading bridge
│   │   ├── mod.rs
│   │   ├── ffi.rs          # C ABI FFI for MQL5
│   │   ├── signals.rs      # Signal pipeline
│   │   ├── executor.rs     # Order executor
│   │   ├── sandbox.rs      # Strategy sandbox
│   │   └── types.rs        # Shared types
│   │
│   ├── audio/              # Audio engine (TTS)
│   │   └── mod.rs          # AudioEngine, local Whisper/Piper, Fish.Audio API
│   │
│   ├── ironclaw/           # IronClaw safety constraints
│   │   └── mod.rs          # Action validation, rate limiting, output constraints
│   │
│   ├── sub_agents/         # Sub-agent orchestration
│   │   └── mod.rs          # SubAgentManager, MateriaSubAgent definitions
│   │
│   ├── persona/            # Agent persona
│   │   ├── mod.rs          # JarvisPersona — identity, traits, system prompt
│   │   ├── voice.rs        # VoiceEngine — voice characteristics
│   │   └── proactive.rs    # ProactiveEngine — proactive monitoring
│   │
│   ├── jarvis/             # JARVIS Python frontend bridge
│   │   ├── mod.rs          # JarvisManager — process lifecycle management
│   │   └── bridge.rs       # JarvisApi — HTTP bridge to Python FastAPI
│   │
│   ├── infra/              # Infrastructure utilities
│   │   ├── mod.rs          # Infrastructure struct
│   │   ├── hardening.rs    # OpenVentus — Linux hardening (AppArmor, UFW, auditd)
│   │   ├── ghost.rs        # GhostMonitor — watchdog, anomaly detection
│   │   └── backup.rs       # BackupEngine — encrypted backup/DR
│   │
│   └── desktop/            # egui native desktop GUI
│       └── mod.rs          # Tabs: Monitor, Providers, Models, Agents, Training, Debugger
│
├── frontend/               # Angular frontend
│   ├── angular.json        # Angular workspace config
│   ├── tsconfig.json       # TypeScript config
│   ├── package.json        # Angular 19 + Three.js dependencies
│   ├── pnpm-lock.yaml
│   ├── pnpm-workspace.yaml # pnpm config (hoisted, allow-builds)
│   ├── dist/               # Build output
│   └── src/
│       ├── index.html      # Entry HTML
│       ├── main.ts         # Angular bootstrap
│       ├── styles.scss     # Global styles
│       └── app/
│           ├── app.component.ts        # Root component
│           ├── app.config.ts           # Angular app config
│           ├── app.routes.ts           # Route definitions
│           ├── core/                   # Core layer
│           │   ├── services/           # API, Auth, WebSocket, Backend, i18n
│           │   ├── guards/             # Route guards
│           │   └── interceptors/       # HTTP interceptors (auth)
│           ├── domain/                 # Domain layer
│           │   ├── models/             # Provider, Hardware, Training, Agent-State, User
│           │   ├── enums/              # SystemMode, ProviderType, AgentState
│           │   └── interfaces/         # UseCase, Repository interfaces
│           ├── data/                   # Data layer
│           │   ├── repositories/       # Provider, Vault, Settings, Training, Hardware
│           │   └── dto/                # Provider, Vault, Settings DTOs
│           ├── use-cases/              # Use case layer
│           │   ├── providers/          # GetProviders, TestProvider, ConfigureProvider
│           │   ├── vault/              # Unlock, Lock, Migrate
│           │   ├── settings/           # SaveVoice, SavePreferences, GetSystemStatus
│           │   └── training/           # GetTrainingStats, TriggerTraining
│           └── presentation/           # Presentation layer
│               ├── pages/              # Monitor, Chat, Settings (5 sections)
│               ├── components/         # Orb, HUD, StatusBar, WindowControls
│               └── shared/             # Directives (ripple), Pipes (translate)
│
├── electron/               # Electron desktop shell
│   ├── package.json        # Electron + electron-builder deps
│   ├── main.js             # Main process: window, tray, backend lifecycle, IPC
│   ├── preload.js          # Context bridge for IPC
│   └── pnpm-lock.yaml
│
├── docs/                   # Documentation
│   ├── ARCHITECTURE.md     # (old) Initial architecture document
│   ├── COMPONENTS.md       # Component catalog
│   └── ROADMAP.md          # Phase roadmap
│
├── diagrams/               # Architecture diagrams (Draw.io)
│   ├── umbra-architecture.drawio
│   ├── umbra-data-flow.drawio
│   ├── umbra-security-layers.drawio
│   ├── umbra-mt4-bridge.drawio
│   └── umbra-agent-loop.drawio
│
├── specs/                  # Component specifications
│   ├── engine-spec.md
│   ├── security-spec.md
│   ├── mt4-bridge-spec.md
│   └── hermes-integration-spec.md
│
├── models/                 # Local model storage
├── sub_agents/             # Sub-agent definition files (.materia)
├── scripts/                # Utility scripts
├── logs/                   # Backend logs
├── supply-chain/           # cargo-vet supply chain audits
│   ├── config.toml
│   ├── audits.toml
│   └── imports.lock
├── dist-electron/          # Electron build output
└── logo.svg                # Application logo
```

---

## Configuration

| File | Path | Purpose |
|------|------|---------|
| `config.toml` | `~/.umbra/config.toml` | API ports, audio settings, Ollama URL, paths (models, sub-agents, JARVIS, logs), training parameters, security settings |
| `vault.enc` | `~/.umbra/vault.enc` | AES-256-GCM encrypted credential store — API keys for 23 providers |
| `vault.lock` | `~/.umbra/vault.lock` | Lock state file ("locked" or "unlocked:{timestamp}") |
| `auth_token` | `~/.umbra/auth_token` | Auto-generated 64-char hex token for API authentication |
| `customization.json` | `~/.umbra/customization.json` | Encrypted user preferences (name, greeting, theme, voice, persona) |
| `backend.log` | `~/.umbra/backend.log` | Backend runtime logs |
| `providers.toml` | `~/.umbra/providers.toml` | Legacy provider config (migrated to vault) |

### Environment Variables

| Variable | Purpose |
|----------|---------|
| `RUST_LOG` | Tracing/logging level filter (e.g., `info`, `debug`) |
| `UMBRA_API_URL` | Backend URL for frontend server (default: `http://127.0.0.1:8484`) |
| `UMBRA_FRONTEND_DIR` | Override frontend static files directory |
| `FISH_API_KEY` | Fish.Audio TTS API key |
| `FISH_VOICE_ID` | Fish.Audio voice ID |
| `NVIDIA_API_KEY` | NVIDIA Riva TTS API key |

---

## Security Model

### Authentication
- **API auth:** `x-umbra-key` header with token from `~/.umbra/auth_token` (auto-generated at startup)
- **WebSocket auth:** Token sent as `{"type": "auth", "token": "..."}` message on connection
- **Session auth:** Cookie-based session via `/api/auth/session`
- **Auth token generation:** Nanotimestamp hashed to 64-char hex; file permissions set to `0600`

### Vault Encryption
- **Algorithm:** AES-256-GCM (authenticated encryption)
- **Key derivation:** PBKDF2-SHA256 with 600,000 iterations
- **Key binding:** Derived from `auth_token:passphrase` combo (token file + user passphrase)
- **Salt:** Random 16 bytes per encryption operation (stored in ciphertext header)
- **Nonce:** Random 12 bytes per operation
- **Ciphertext format:** `[16-byte salt][12-byte nonce][encrypted payload + GCM tag]`
- **Auto-lock:** Configurable timeout (default 15 min); auto-locks on inactivity

### TLS
- **Mode:** Optional (`--ssl` flag)
- **Cert:** Auto-generated self-signed cert at `~/.umbra/tls/cert.pem` (rcgen)
- **Cipher:** rustls with modern TLS 1.3
- **Port:** Both backend (`:8484`) and frontend (`:8340`) can serve TLS

### Supply Chain
- **cargo-deny** (`deny.toml`) — blocks yanked crates, enforces license allowlist (MIT, Apache-2.0, BSD, etc.)
- **cargo-vet** (`supply-chain/`) — audited dependency chain with criteria `safe-to-deploy`
- **Patch:** `paste` crate patched from GitHub fork `MethodWhite/paste` (tagged `v1.0.15-umbra`)

### Runtime Security
- **IronClaw** — action rate limiting, input/output length constraints, blocked commands (`rm`, `sudo`, `dd`, etc.), max trading positions, max daily loss
- **SecurityGate** — 4-verification chain: identity → permission → risk → context, all <100ms
- **RuntimeEnforcer** — tool call validation against capability rules
- **AntiBrick** — prevents destructive operations (financial and system)
- **AuditWorm** — immutable log with hash chaining for forensic evidence

---

## Current Limitations

1. **Single-crate Rust project** — not yet split into workspace; monolithic compile times
2. **Python JARVIS bridge** — still references `../jarvis/` Python server; migration incomplete
3. **No test suite** — zero Rust or Angular tests in the codebase
4. **Synapsis path dependency** — external crate at `../synapsis` not published; requires local clone
5. **Electron packaging incomplete** — `dist-electron/` exists but no verified build pipeline
6. **MT4 bridge untested** — FFI, signal pipeline, strategy sandbox defined but likely not wired
7. **TLS auto-cert** — self-signed certs trigger browser warnings; no Let's Encrypt integration
8. **No CI/CD** — no GitHub Actions or other CI pipeline
9. **Vault migration** — migration from `.env`/`providers.toml` implemented but may miss edge cases
10. **JEPA engine** — pure Rust implementation; no GPU acceleration beyond CPU inferencing
11. **No backup/restore** — BackupEngine exists but no automated restore or disaster recovery
12. **Angular i18n** — translation pipe exists but only English content; no i18n files
13. **No schema validation** — config.toml and providers.toml parsed without schema enforcement
14. **egui desktop** — functional but basic; no system tray, no global shortcuts, no packaging
15. **Supply chain** — 2 ignored advisories (RUSTSEC-2025-0057, RUSTSEC-2024-0436) for transitive deps
