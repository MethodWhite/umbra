# Changelog

All major changes to the Umbra project.

---

## 2026-06-09 — Clean Architecture + Desktop Focus

### Rust Backend — Clean Architecture Migration
- Restructured monolithic modules into Clean Architecture layers:
  - **domain/** — pure models (`ApiProvider`, `VaultEntry`, `TrainingExample`, `SystemConfig`), port traits (`VaultRepository`, `ProviderRepository`, `SettingsRepository`), domain errors (`AppError` with axum `IntoResponse`)
  - **application/** — use case orchestrators (`providers/configure`, `providers/test_connection`, `providers/get_status`; `vault/unlock`, `vault/lock`, `vault/migrate`; `settings/get_voice`, `settings/save_voice`, `settings/get_status`; `training/trigger`, `training/get_stats`)
  - **infrastructure/** — repository implementations (`EncryptedVaultRepository` with AES-256-GCM, `TomlProviderRepository`, `TomlSettingsRepository`, `TrainingRepository`), HTTP clients (`OllamaClient`, `HuggingFaceClient`, `TtsClient`), persistence (`vault_encryption.rs` — PBKDF2 + AES-256-GCM)
  - **api/** — Axum router with clean route separation (8 route modules), auth middleware, shared types

### Infrastructure Additions
- **RateLimiter** (`src/rate_limiter.rs`) — token bucket algorithm, 30 actions per 60-second window
- **JobQueue** (`src/job_queue.rs`) — tokio mpsc-based background job processor (TrainModel, BrowserCollect, Backup, Cleanup)
- **TtlCache** (`src/cache.rs`) — generic TTL-based cache with `ProviderCache` for model/status/connection caching
- **ResourceManager** (`src/resource.rs`) — resource tracking for models, audio pipelines, cache, sub-agents

### Frontend — Clean Architecture Migration
- Migrated from flat Angular structure to Clean Architecture:
  - **domain/** — `models/` (Provider, Hardware, Training, AgentState, User), `enums/` (SystemMode, ProviderType, AgentState), `interfaces/` (UseCase, Repository)
  - **data/** — `dto/` (Provider, Vault, Settings), `repositories/` (Provider, Vault, Settings, Training, Hardware)
  - **use-cases/** — `providers/` (GetProviders, TestProvider, ConfigureProvider), `vault/` (Unlock, Lock, Migrate), `settings/` (SaveVoice, SavePreferences, GetSystemStatus), `training/` (GetTrainingStats, TriggerTraining)
  - **presentation/** — `pages/` (Monitor, Chat, Settings with 5 sections), `components/` (Orb, HUD, StatusBar, WindowControls), `shared/` (ripple directive, translate pipe)
  - **core/** — `services/` (API, Auth, WebSocket, BackendStatus, i18n), `guards/` (AuthGuard), `interceptors/` (AuthInterceptor)

### Supply Chain Security
- Configured `cargo-deny` (`deny.toml`) — yanked crate blocking, license allowlist (MIT, Apache-2.0, BSD, etc.)
- Configured `cargo-vet` (`supply-chain/`) — 3000+ dependency exemptions audited as `safe-to-deploy`
- Patched `paste` crate via GitHub fork `MethodWhite/paste` (tag `v1.0.15-umbra`) to fix supply chain issues
- 0 active advisories; 2 ignored (RUSTSEC-2025-0057 fxhash, RUSTSEC-2024-0436 paste — both transitive via wasmtime/eframe)

### Build System
- Migrated from npm to **pnpm** — workspace configured at `frontend/pnpm-workspace.yaml`
- Node.js managed via **fnm**
- Added `pnpm exec ng build` as standard build command
- Electron builder configured for AppImage/deb (Linux), dmg (macOS), nsis (Windows)

### Desktop Focus
- **Electron** primary distribution target — `electron/main.js` with system tray, backend lifecycle, IPC window controls, voice shortcut (Cmd+Shift+U)
- **egui** GUI (`src/desktop/mod.rs`) — 6-tab native interface (Monitor, Providers, Models, Agents, Training, Debugger)
- Dual frontend serving: Electron loads Angular from disk, egui communicates via HTTP

### Vault System
- Implemented `EncryptedVaultRepository` — AES-256-GCM with PBKDF2-SHA256 (600K iterations)
- Key binding: derived from `auth_token:passphrase` combination
- Auto-lock with configurable timeout (default 15 min)
- Migration from legacy `.env` and `providers.toml` files
- Full CRUD API for provider keys

### Synapsis Memory Integration
- MemoryEngine wrapping Synapsis crate (`../synapsis` path dependency)
- Session-based observation management with embeddings
- Used by AgentEngine for conversational context

---

## 2026-06-08 — Angular Frontend + Electron

- Replaced Vue.js + Vite frontend with **Angular 19** using `@angular/build` application builder
- Created Clean Architecture directory structure (domain/data/use-cases/presentation/core)
- Implemented Three.js Orb component for animated 3D visualization
- Built HUD component with voice visualization overlay
- Created Settings page with 5 sections: System, Voice, Preferences, Providers, Vault, Customization
- Built Monitor page for system status, hardware metrics, agent state
- Built Chat page for text conversation with agent
- Created Electron shell (`electron/main.js`):
  - Frameless transparent window with custom title bar
  - System tray with Start/Stop/Status menu
  - Backend process spawn + health check polling
  - IPC for window controls (minimize, maximize, close)
  - Global shortcut `Cmd+Shift+U` for voice activation
- Created `preload.js` for secure context bridge
- Configured `electron-builder` for cross-platform packaging (AppImage, dmg, nsis)

---

## 2026-06-05 — Rust Frontend Migration

- Ported Python FastAPI server features to Rust:
  - Voice WebSocket handler (`/ws/voice`) with auth, transcription, TTS
  - TTS synthesis via Fish.Audio API and NVIDIA Riva
  - Speech correction (regex-based: "cloud code" → "Claude Code", etc.)
  - Markdown stripping for TTS output
  - Static file serving for Angular frontend
- Created `src/frontend/mod.rs` — 475 lines of voice frontend logic
- Dual-server architecture: backend `:8484` + frontend `:8340`
- Frontend router serves Angular SPA with SPA fallback (index.html for unknown routes)
- Security headers middleware (X-Content-Type-Options, X-Frame-Options, CSP)
- CORS configuration for localhost origins

---

## 2026-06-04 — Bridge + Provider System

- Implemented MT4 trading bridge:
  - `bridge/ffi.rs` — C ABI for MQL5 interop
  - `bridge/signals.rs` — signal pipeline
  - `bridge/executor.rs` — order execution
  - `bridge/sandbox.rs` — strategy isolation
- Built LLM provider system (`src/providers/mod.rs`):
  - `ProviderRegistry` with provider loading from TOML
  - Chat completion API with OpenAI-compatible, Anthropic, and Google formats
  - Dynamic API key resolution from vault or env vars
  - Streaming and non-streaming support
  - OpenCode Go provider with built-in model list

---

## 2026-06-03 — Vault + Security Hardening

- Implemented IronClaw safety constraints:
  - `rate_limiter.rs` — token bucket (30 actions/60s)
  - Input/output length limits (32K/8K chars)
  - Blocked commands list (`rm`, `sudo`, `dd`, `mkfs`, `shutdown`)
  - Max trading positions (5), max daily loss (5%), max iterations (12)
- Built SecurityGate (`src/security/mod.rs`):
  - `RuntimeEnforcer` — tool call validation
  - `CryptoEngine` — PQC key management
  - `ZeroTrustGate` — per-command risk analysis
  - `AntiBrick` — destructive operation prevention
  - `AuditWorm` — hash-chained immutable logging
- Implemented PQC cryptography (`security/pqc.rs`):
  - Kyber-512 KEM key exchange
  - Dilithium-4 digital signatures
- Created vault encryption module (`infrastructure/persistence/vault_encryption.rs`):
  - AES-256-GCM with PBKDF2 key derivation (600K iterations)
  - Auth-token-bound key derivation
  - Salt + nonce random per encryption

---

## 2026-06-02 — Initial Architecture Setup

- Created project skeleton with Cargo.toml
- Implemented MateriaCore engine:
  - `engine/jepa.rs` — Joint Embedding Predictive Architecture
  - `engine/router.rs` — multi-backend model router (llama.cpp/Ollama/Cloud)
  - `engine/scheduler.rs` — adaptive hardware scheduler
  - `engine/wasm.rs` — WASM sandbox runner (wasmtime)
  - `engine/snn.rs` — Spiking Neural Network classifier
  - `engine/hdt_router.rs` — hyperdimensional computing router
  - `engine/hsaq.rs` — Hierarchical Stochastic Adaptive Quantization
  - `engine/safety.rs` — hardware temperature/VRAM monitor
- Implemented AgentEngine (`learning/`):
  - `agent_loop.rs` — UmbraAgent with plan→act→observe→learn cycle
  - `skills.rs` — SkillManager for skill creation/versioning
  - `trainer.rs` — TrainerEngine for JEPA training
  - `messaging.rs` — MessagingGateway (Telegram/Discord/Signal/Email)
- Implemented CLI with clap (`main.rs`):
  - `umbra start` / `umbra stop` / `umbra status`
  - `--api-port`, `--api-host`, `--no-frontend`, `--ssl` flags
  - TLS support with auto self-signed cert generation
- Created dual-server startup (backend `:8484` + frontend `:8340`)
- Implemented API routes:
  - Chat, command, status, health endpoints
  - Memory search/store
  - Sub-agent list/spawn
  - Config endpoint
- Built AudioEngine (`audio/mod.rs`) with Fish.Audio TTS integration
- Created SubAgentManager (`sub_agents/mod.rs`) with `.materia` file format
- Implemented JarvisPersona (`persona/`) with voice and proactive engines
- Built JarvisManager (`jarvis/`) for Python FastAPI process lifecycle
- Created Infrastructure module (`infra/`):
  - `hardening.rs` — OpenVentus Linux hardening
  - `ghost.rs` — GhostMonitor watchdog
  - `backup.rs` — BackupEngine encrypted backup
- Created UmbraConfig with TOML loading (`config.rs`)
- Built MemoryEngine (`memory.rs`) with Synapsis observation/session management
- Implemented Debugger (`debugger.rs`) with background severity reporting
- Established 23 API provider definitions in `domain/models/provider.rs`
- Created documentation: `docs/ARCHITECTURE.md`, `docs/COMPONENTS.md`, `docs/ROADMAP.md`
- Created Draw.io diagrams in `diagrams/`
- Created component specs in `specs/`
