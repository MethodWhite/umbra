# Changelog

All notable changes to Umbra will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-06-12

### Added

- **Real AI inference** — Ollama client integration with chat completion, streaming, and model management (17 tests)
- **STT integration** — whisper.cpp speech-to-text with mic button in HUD
- **Voice clone** — audio analysis pipeline with agent orchestrator and scoring (24 tests)
- **Market data client** — real-time market data feeds with streaming support
- **synapsis-core integration** — token-efficient memory with importance scoring, summary budgets, and embedding-based search (11+13 tests)
- **Emotional memory system** — personality and emotion tagging with hybrid scoring algorithm
- **47 tests** (from 24) — coverage across memory, vault, config, AI client, and agent personality modules
- **Zone annotations** — responsive layout zones with dev-security CI
- **Supply chain security** — `cargo-deny` (yanked crate blocking, license allowlist), `cargo-vet` (3000+ dependency exemptions)
- **CI/CD pipeline** — automated builds with Dependabot for weekly Cargo updates

### Changed

- **Clean Architecture migration** — monolithic modules restructured into `domain/`, `application/`, `infrastructure/`, `api/` layers
- **Desktop panel partitioning** — `desktop/mod.rs` split into `helpers.rs`, `panels.rs`, `actions.rs` for maintainability
- **Server code feature-gated** — server-only code under `--features server`, AI client extracted, zero warnings across all targets
- **Vault encryption hardened** — PBKDF2 + AES-256-GCM with auth-token-bound key derivation
- **synapsis dependency** — migrated from monolithic `synapsis` wrapper to lightweight `synapsis-core` with compatibility stubs
- **Dependency cleanup** — removed legacy `openjarvis` crate; documented `infra` vs `infrastructure` distinction
- **Dead code removal** — eliminated unused imports, vars, consts across codebase

### Fixed

- STT microphone initialization, config path resolution, sub-agent restoration
- Emoji font loading with multi-path logo search
- Zero compiler warnings, zero errors — both `umbra` and `umbra-gui` binaries build clean
- Logo toggle, sphere size reset, window controls, panel redesign issues

## [0.2.0] - 2026-06-09

### Added

- **Clean Architecture** — domain (models, ports, errors), application (use case orchestrators), infrastructure (repositories, HTTP clients, persistence), API (Axum router, auth middleware)
- **RateLimiter** — token bucket algorithm, 30 actions per 60-second window
- **JobQueue** — tokio mpsc-based background processor (TrainModel, BrowserCollect, Backup, Cleanup)
- **TtlCache** — generic TTL cache with ProviderCache for model/status/connection caching
- **ResourceManager** — resource tracking for models, audio pipelines, cache, sub-agents
- **Angular 19 frontend** — Clean Architecture (domain/data/use-cases/presentation/core), Three.js Orb, HUD, Chat, Settings, Monitor pages
- **Electron shell** — frameless transparent window, system tray, backend lifecycle, IPC window controls, voice shortcut (Cmd+Shift+U)
- **Vault system** — EncryptedVaultRepository with AES-256-GCM, PBKDF2-SHA256 (600K iterations), auto-lock, migration from legacy files
- **Synapsis Memory Integration** — MemoryEngine wrapping Synapsis crate, session-based observation management with embeddings

### Changed

- Migrated from Vue.js + Vite to Angular 19 with `@angular/build`
- Frontend restructured from flat to Clean Architecture (domain/data/use-cases/presentation/core)
- Electron-builder configured for AppImage/deb (Linux), dmg (macOS), nsis (Windows)
- Node.js managed via fnm, npm replaced with pnpm

## [0.1.0] - 2026-06-02

### Added

- **MateriaCore engine** — JEPA, multi-backend model router (llama.cpp/Ollama/Cloud), adaptive hardware scheduler, WASM sandbox (wasmtime), SNN classifier, HDC router, HSAQ quantization, hardware safety monitor
- **AgentEngine** — UmbraAgent with plan→act→observe→learn cycle, SkillManager, TrainerEngine, MessagingGateway (Telegram/Discord/Signal/Email)
- **CLI** — `umbra start`/`stop`/`status` with `--api-port`, `--api-host`, `--no-frontend`, `--ssl` flags; TLS with auto self-signed certs
- **API routes** — Chat, command, status, health, memory, sub-agent, config endpoints
- **AudioEngine** — Fish.Audio TTS integration
- **SubAgentManager** — `.materia` file format for sub-agent definitions
- **JarvisPersona** — voice and proactive engines
- **JarvisManager** — Python FastAPI process lifecycle management
- **Infrastructure** — OpenVentus Linux hardening, GhostMonitor watchdog, BackupEngine encrypted backup
- **UmbraConfig** — TOML-based configuration
- **MemoryEngine** — Synapsis observation/session management
- **Debugger** — background severity reporting
- **23 API provider definitions**, documentation (ARCHITECTURE.md, COMPONENTS.md, ROADMAP.md), diagrams, component specs

[Unreleased]: https://github.com/methodwhite/umbra/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/methodwhite/umbra/releases/tag/v0.3.0
[0.2.0]: https://github.com/methodwhite/umbra/releases/tag/v0.2.0
[0.1.0]: https://github.com/methodwhite/umbra/releases/tag/v0.1.0
