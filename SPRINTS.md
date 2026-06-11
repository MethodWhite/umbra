# UMBRA Development Sprints

**Version:** 0.2.0
**Last Updated:** 2026-06-10

---

## Sprint 1: Foundation (June 2-3)

### Goals
Establish the core architecture and initial Rust backend with all foundational engine components.

### Deliverables

**Core Architecture**
- Project skeleton with `Cargo.toml`, workspace configuration
- Clean Architecture layering design — Domain, Application, Infrastructure, API
- Dual-server architecture concept (backend `:8484` + frontend `:8340`)

**MateriaCore Engine**
- `engine/jepa.rs` — Joint Embedding Predictive Architecture inference
- `engine/router.rs` — Multi-backend model router (llama.cpp/Ollama/Cloud)
- `engine/scheduler.rs` — Adaptive hardware scheduler for backend selection
- `engine/wasm.rs` — WASM sandbox runner via wasmtime
- `engine/snn.rs` — Spiking Neural Network classifier
- `engine/hdt_router.rs` — Hyperdimensional computing router
- `engine/hsaq.rs` — Hierarchical Stochastic Adaptive Quantization compression
- `engine/safety.rs` — Hardware monitor (temperature, VRAM, throttling)

**AgentEngine**
- `learning/agent_loop.rs` — UmbraAgent with plan→act→observe→learn cycle
- `learning/skills.rs` — SkillManager with skill creation, versioning, discovery
- `learning/trainer.rs` — TrainerEngine for JEPA training pipeline
- `learning/messaging.rs` — MessagingGateway (Telegram/Discord/Signal/Email)

**CLI & Server**
- `main.rs` — clap-based CLI with `umbra start / stop / status / gui`
- TLS support with auto self-signed cert generation (rcgen)
- MemoryEngine with Synapsis observation/session management
- AudioEngine with Fish.Audio TTS integration
- SubAgentManager with `.materia` file format
- JarvisPersona (voice, proactive engine)
- JarvisManager (Python FastAPI lifecycle management)
- Infrastructure module — OpenVentus hardening, GhostMonitor, BackupEngine
- Debugger with background severity reporting
- 23 API provider definitions
- Initial documentation and Draw.io diagrams

**Files Created:**
- `src/main.rs`, `src/lib.rs`, `src/config.rs`
- `src/engine/` — 9 files (jepa, router, scheduler, wasm, snn, hdt, hsaq, safety, models)
- `src/learning/` — 5 files (agent_loop, skills, trainer, messaging, trainer)
- `src/security/`, `src/audio/`, `src/bridge/`, `src/ironclaw/`
- `src/sub_agents/`, `src/persona/`, `src/jarvis/`, `src/infra/`
- `docs/ARCHITECTURE.md`, `docs/COMPONENTS.md`, `docs/ROADMAP.md`
- `diagrams/` — 5 Draw.io files
- `specs/` — 3 spec files

---

## Sprint 2: Vault & Security (June 3)

### Goals
Implement encrypted credential storage and harden the security layer.

### Deliverables

**Vault Encryption**
- AES-256-GCM encrypted vault at `~/.umbra/vault.enc`
- PBKDF2-SHA256 key derivation with 600,000 iterations
- Auth-token-bound key derivation (combines file token + user passphrase)
- Random 16-byte salt + 12-byte nonce per encryption operation
- Ciphertext format: `[salt][nonce][encrypted payload + GCM tag]`
- Auto-lock mechanism with configurable timeout (default 15 min)
- Migration from legacy `.env` and `providers.toml` files

**IronClaw Safety Constraints**
- `rate_limiter.rs` — token bucket algorithm, 30 actions/60s
- Input/output length limits (32K/8K chars)
- Blocked commands list (`rm`, `sudo`, `dd`, `mkfs`, `shutdown`)
- Max trading positions (5), max daily loss (5%), max iterations (12)

**SecurityGate**
- `RuntimeEnforcer` — tool call validation (<100ms)
- `CryptoEngine` — PQC key management
- `ZeroTrustGate` — per-command risk analysis
- `AntiBrick` — destructive operation prevention
- `AuditWorm` — hash-chained immutable logging

**PQC Cryptography**
- Kyber-512 KEM key exchange definitions
- Dilithium-4 digital signature definitions
- Memory-locked secret keys via `mlock()`

**API & Network**
- CORS configuration for localhost origins
- TLS mode with auto self-signed certs
- Security headers (X-Content-Type-Options, X-Frame-Options, CSP)

**Files Created:**
- `src/infrastructure/persistence/vault_encryption.rs`
- `src/infrastructure/repositories/vault_repo.rs`
- `src/rate_limiter.rs`
- `src/security/pqc.rs`, `src/security/enforcer.rs`
- `src/security/zt_gate.rs`, `src/security/antibrick.rs`
- `src/security/audit.rs`

---

## Sprint 3: Provider System (June 4)

### Goals
Build the multi-provider LLM routing system with 23 API providers and OpenCode Go integration.

### Deliverables

**Provider Registry**
- `src/providers/mod.rs` — ProviderRegistry with dynamic loading from TOML
- Chat completion API supporting OpenAI-compatible, Anthropic, and Google formats
- Streaming and non-streaming completion support
- Dynamic API key resolution from vault or environment variables

**OpenCode Go Provider**
- Built-in model list with OpenCode Go API
- OpenCode-compatible provider registration

**API Routes**
- Provider configuration CRUD (create, read, update, delete)
- Connection testing for individual and bulk providers
- Provider status and health checking

**Multi-Language i18n**
- Translation pipe infrastructure in Angular frontend
- i18n service with language switching

**Files Created:**
- `src/providers/mod.rs`
- `src/api/routes/provider_routes.rs`
- `src/application/providers/configure.rs`
- `src/application/providers/test_connection.rs`
- `src/application/providers/get_status.rs`
- `src/infrastructure/repositories/provider_repo.rs`

---

## Sprint 4: Angular Frontend + Electron (June 5-8)

### Goals
Replace the Vue.js frontend with Angular 19 featuring Clean Architecture, and wrap it in an Electron desktop shell.

### Deliverables

**Angular 19 Frontend**
- Clean Architecture directory structure:
  - `domain/` — models, enums (SystemMode, ProviderType, AgentState), interfaces
  - `data/` — DTOs, repository implementations (HTTP calls to Rust backend)
  - `use-cases/` — business logic orchestrators for providers, vault, settings, training
  - `presentation/` — pages (Monitor, Chat, Settings), components (Orb, HUD, StatusBar, WindowControls)
  - `core/` — services (API, Auth, WebSocket, Backend, i18n), guards, interceptors
- Three.js Orb component for animated 3D visualization
- HUD component with voice visualization overlay
- Settings page with 5 sections: System, Voice, Preferences, Providers, Vault, Customization
- Monitor page for system status, hardware metrics, agent state
- Chat page for text conversation with agent
- pnpm migration from npm, fnm for Node.js version management

**Electron Desktop Shell**
- `electron/main.js` — main process with:
  - Frameless transparent window (1400x900)
  - Custom title bar with window controls (minimize, maximize, close)
  - System tray with Start/Stop/Status menu
  - Backend process spawn + health check polling (1s interval)
  - IPC for window controls
  - Global shortcut `Cmd+Shift+U` for voice activation
- `electron/preload.js` — secure context bridge
- `electron-builder` configuration for cross-platform packaging (AppImage, deb, dmg, nsis)

**Rust Frontend Server**
- Ported Python FastAPI features to Rust (`src/frontend/mod.rs`):
  - Voice WebSocket handler (`/ws/voice`) with auth, transcription, TTS
  - TTS synthesis via Fish.Audio API and NVIDIA Riva
  - Speech correction (regex: "cloud code" → "Claude Code", etc.)
  - Markdown stripping for TTS output
  - Static file serving with SPA fallback
  - Security headers middleware

**Files Created:**
- `frontend/` — Full Angular 19 project (~50+ files)
- `electron/main.js`, `electron/preload.js`
- `src/frontend/mod.rs`

---

## Sprint 5: Egui Desktop Native (June 9)

### Goals
Port the GUI from Electron/Angular to a native egui desktop application for instant-load and lower resource usage.

### Deliverables

**Native egui GUI**
- `src/desktop/mod.rs` — egui/eframe-based desktop application
- Tabs: HUD, Trading, Conversations
- Sidebar with settings navigation
- 3D sphere with 500 particles using Fibonacci sphere distribution
- Emotional color system — sphere color dynamically maps to agent emotion
- Particle interaction with hover/selection and analysis panel

**GUI Features**
- Window drag/resize with custom title bar
- Button hover effects (purple highlight)
- Text contrast improvements across 15+ color values
- Scrollable sidebar with sub-tab navigation
- Real-time clock display
- Logo texture loading and display

**Critical Bug Fixes**
- Fixed `unwrap()` on NaN in sphere sort (`unwrap_or(Ordering::Equal)`)
- Fixed redundant `is_ok()` + `unwrap()` pattern (`if let Ok(pos)`)
- Fixed sphere click closing inner panel elements (hover mode when selected)

**Files Created:**
- `src/desktop/mod.rs` — 1303 lines of egui UI code
- `src/sphere.rs` — 3D sphere renderer

---

## Sprint 6: Cognitive & Emotional System (June 9-10)

### Goals
Implement a rich emotional model for AI agents with cognitive behavioral therapy principles.

### Deliverables

**Plutchik Wheel of Emotions**
- 8 primary emotions: Joy, Trust, Fear, Surprise, Sadness, Disgust, Anger, Anticipation
- 3 intensity levels: Low, Medium, High
- Secondary emotion combinations producing 80+ distinct states
- Named emotional states: Calm, Happy, Focused, Analytical, Creative, Excited, Sad, Depressed, Angry, Anxious, Fearful, Tired, Curious, Surprised, Ashamed, Flow, Intuitive, more

**Emotional State Model**
- Valence (-1.0 to 1.0) — negative to positive
- Arousal (0.0 to 1.0) — calm to excited
- Color mapping — hue derived from primary emotion, saturation from intensity
- Voice tone mapping — TTS parameters modulated by emotion

**Cognitive Behavioral Therapy for AIs**
- `CognitiveBehavior` struct with frustration tracking
- Cooling system — frustration decays over time
- Emotional spiral prevention — negative emotions trigger self-correction
- Performance history tracking with emotional context

**Agent Personality**
- `AiGender` enum: Male, Female, Androgynous, Neutral
- Gender is chosen once and immutable thereafter
- Communication styles per gender (analytical, intuitive, adaptive, objective)
- Personality traits: confidence, pragmatism, creativity (0.0–1.0)
- TTS voice assignment based on gender
- Gender-aware greetings and interaction patterns

**Files Created/Modified:**
- `src/agent_memory.rs` — 736 lines (EmotionalState, AgentMemory, CognitiveBehavior)
- `src/agent_personality.rs` — 149 lines (AgentPersonality, AiGender)

---

## Sprint 7: Trading + MT5 Integration (June 10)

### Goals
Build a professional-grade trading UI and complete the MT5 bridge integration.

### Deliverables

**Trading Panel Redesign**
- Complete trading interface with three views:
  - **Watchlist** — configurable symbol sets (Forex, Crypto, Commodities, Indices)
  - **Chart** — visual price display with selectable chart types and timeframes
  - **Order Entry** — symbol, volume, buy/sell with confirmation
- 5 asset class filters: All, Forex, Crypto, Commodities, Indices
- Symbol sets per filter (EURUSD, BTCUSD, XAUUSD, SP500, etc.)
- Balance/equity/margin display in real-time

**Simulated Account System**
- Paper trading environment with simulated balance
- Order execution with position tracking
- P&L calculation and display

**Strategy System**
- 144K Method — scalping with 144-period MA + stochastic
- 3.4 Unification — multi-timeframe confluence strategy
- Manual mode — direct trading with AI guidance
- Strategy sandbox for isolated testing

**Broker Research**
- IC Markets integration assessment
- Pepperstone integration assessment
- MT4/MT5 bridge protocol finalization

**Files Modified:**
- `src/desktop/mod.rs` — trading UI components
- `src/bridge/` — MT5 bridge modules (ffi, signals, executor, sandbox, types)

---

## Sprint 8: Sound + Voice + Logo (June 10)

### Goals
Integrate text-to-speech with emotional voice modulation, finalize branding with logo system installation.

### Deliverables

**TTS Integration**
- Fish.Audio API TTS integration with multiple voice options
- Local Piper TTS detection and fallback
- NVIDIA Riva TTS support
- Voice tone modulation based on emotional state
  - Calm → soft, measured delivery
  - Excited → faster, higher pitch
  - Sad → slower, lower pitch
  - Angry → clipped, emphatic
- Markdown stripping for clean TTS output
- Speech correction regex system

**User Gender Awareness**
- Voice engine adapts to user gender
- Personalization settings include gender preference
- Communication style adjustment

**Logo Design**
- SVG logo (`logo.svg`)
- Multi-resolution PNG icons (16x16 through 512x512)
- `.iconset` for macOS
- System-wide installation via `install.sh` to `~/.local/share/icons/hicolor/`
- `.desktop` file creation for application menu integration
- AppImage build script (`build-appimage.sh`)

**Files Created:**
- `logo.svg`, `logo_*.png`, `logo_full_*.png`
- `logo.iconset/`
- `install.sh`, `uninstall.sh`
- `build-appimage.sh`

---

## Sprint 9: Final Polish & Documentation (June 10)

### Goals
Fix remaining UI bugs, optimize performance, audit security, and create comprehensive documentation.

### Deliverables

**UI Polish**
- Window drag/resize functionality with custom title bar
- Button hover effects system-wide (`btn()`, `btn_rounded()`, `btn_fill()` helpers)
- Window state persistence infrastructure (save/load position + size)
- Improved text contrast (15+ color values brightened)
- Scrollable sidebar content areas
- Close button for sphere analysis panel

**Performance Optimization**
- Fixed `update_voice_tone()` calling every frame (now only on emotion change)
- Logo texture disk read optimization (cached after first load)
- Frame-based randomness patterns addressed
- Reduced unnecessary re-renders

**Security Audit**
- Fixed all `unwrap()` calls that could panic:
  - `frontend/mod.rs:44` — path parent unwrap → `if let Some`
  - `frontend/mod.rs:82-87` — URL parse unwrap → `filter_map`
- Removed `#[allow(dead_code)]` from all modules (renamed with `_` prefix)
- Removed unused imports (`Stroke` from sphere.rs)
- `cargo build --release` compiles with 0 errors

**Supply Chain Hardening**
- `cargo-deny` fully configured — blocks yanked crates, enforces license allowlist
- `cargo-vet` supply-chain audits for 3000+ dependencies
- `paste` crate patched via GitHub fork (v1.0.15-umbra)
- 0 active advisories, 2 ignorable transitive advisories

**Documentation**
- `PAPER.md` — Professional technical paper covering all system aspects
- `SPRINTS.md` — Complete sprint history
- `README.md` — Updated with full project overview, quick start, feature list
- `ARCHITECTURE.md` — Comprehensive architecture document
- `QUICKSTART.md` — Quick start guide
- `SECURITY.md` — Security policy

---

## Sprint 10: v0.2.0 Release — Provider UX & Local Detection (June 10)

### Goals
Fix critical provider UX issues: broken local model detection, missing API key status display, and version bump to 0.2.0.

### Deliverables

**Local Model Detection Fix**
- Fixed llama.cpp health endpoint from `/health` to `/v1/models`
- `detect_local_models()` now properly called during `App::default()` startup
- Ollama detects via `http://localhost:11434/api/tags`
- llama.cpp detects via `http://localhost:8080/v1/models`

**Provider Status UI**
- PROVIDERS tab now shows per-provider connection status:
  - Local providers (Ollama, llama.cpp): shows "auto-detected" when found
  - API providers (OpenAI, etc.): shows "key saved" when configured
  - Unconfigured providers show input field for API key entry

**Version Bump**
- Updated version from 0.1.0 → 0.2.0 across all files:
  - `Cargo.toml`, `PAPER.md`, `SPRINTS.md`
  - ABOUT tab and CODING tab version strings

**Files Modified:**
- `Cargo.toml` — version bump
- `src/desktop/mod.rs` — version strings, detect_local_models startup call, llama.cpp endpoint fix, provider status UI
- `PAPER.md` — version bump
- `SPRINTS.md` — version bump + Sprint 10 entry

---

## Sprint Summary

| Sprint | Dates | Focus | Files Changed |
|--------|-------|-------|---------------|
| 1 | June 2-3 | Foundation, Core Engine, Architecture | 50+ |
| 2 | June 3 | Vault Encryption, Security Layers | 15+ |
| 3 | June 4 | Provider System, OpenCode Go, i18n | 10+ |
| 4 | June 5-8 | Angular Frontend, Electron Shell | 60+ |
| 5 | June 9 | egui Desktop Native GUI | 3 |
| 6 | June 9-10 | Emotional & Cognitive Systems | 2 |
| 7 | June 10 | Trading UI, MT5 Integration | 6 |
| 8 | June 10 | TTS, Voice, Logo, Branding | 15+ |
| 9 | June 10 | Polish, Security, Documentation | 30+ |
| 10 | June 10 | v0.2.0 Release — Provider UX & Local Detection | 4 |

**Total Development Time:** 9 days (June 2-10, 2026)
**Total Lines of Rust:** ~15,000+
**Total Lines of TypeScript/Angular:** ~10,000+
**Total Files:** 200+

---

## Key Metrics

- **Rust Backend**: ~15K lines across 80+ files
- **Angular Frontend**: ~10K lines across 50+ files
- **API Providers**: 23 (9 Western, 8 Chinese, 2 Local, 1 Subscription, 3 TTS)
- **Emotions**: 80+ (8 primaries × 3 intensities + secondary combinations)
- **Trading Strategies**: 3 (144K Method, 3.4 Unification, Manual)
- **Security Layers**: 7 (IronClaw, SecurityGate, RuntimeEnforcer, ZeroTrust, AntiBrick, AuditWorm, PQC)
- **Desktop GUIs**: 2 (egui native, Electron + Angular)
- **Dependencies**: 3000+ (all audited via cargo-vet)
- **License**: MIT
