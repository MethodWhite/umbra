# Umbra Project — Comprehensive QA Audit Report

**Project:** Umbra v0.2.0
**Audit Date:** 2026-06-10
**Audited Files:** `src/desktop/mod.rs`, `src/sphere.rs`, `src/agent_memory.rs`, `src/agent_personality.rs`, `src/cache.rs`, `src/config.rs`, `src/debugger.rs`, `src/frontend/mod.rs`, `src/job_queue.rs`, `src/lib.rs`, `src/main.rs`, `src/providers/mod.rs`, `src/rate_limiter.rs`, `src/vault.rs`, `Cargo.toml`
**Lines Audited:** ~4,200+ Rust source lines across 15+ files

---

## Overall Code Quality Score: **4 / 10**

**Summary:** The project has ambitious scope (AI agents, trading, emotional modeling, 3D visualization) but suffers from severe structural issues. The desktop UI is a single 1,400-line file with a 1,000-line `update` function. There are zero unit or integration tests in the main `umbra` crate. Security-sensitive data (API keys, broker passwords) is stored in plaintext in-memory. The vault abstraction is a no-op stub. Critical patterns (HSV→RGB conversion) are duplicated. The project cannot currently compile due to a missing external dependency (`synapsis-core`).

---

## Test Coverage Assessment

| Area | Coverage |
|---|---|
| **umbra crate (Rust)** | **0%** — Zero `#[test]` functions found in `src/`. |
| **openjarvis crate (Rust)** | Moderate — ~500 `#[test]` annotations in sub-crates. |
| **openjarvis (Python)** | Extensive Python test suite. |
| **Desktop module** | **0%** — No tests for UI logic, state management, or rendering. |
| **Sphere renderer** | **0%** — No tests for particle math, rotation, or projection. |
| **Agent memory/personality** | **0%** — No tests for serialization, emotional regulation, or CRUD. |

**Conclusion:** The core `umbra` Rust library has zero automated tests. All existing tests live in the `openjarvis` sub-crate and are Python-based. This is a critical gap for a financial-trading AI agent system.

---

## Issues by Severity

### P0 — Critical (blocks compilation or correct operation)

| # | File | Line | Description | Suggested Fix |
|---|---|---|---|---|
| 1 | `Cargo.toml` | 88 | Path dep `synapsis = { path = "../synapsis" }` points to `/mnt/external/projects/synapsis` which depends on `synapsis-core` that does not exist at `../synapsis-core`. **Project cannot compile.** | Add `synapsis-core` as a workspace member or publish both crates; update path to point to correct location, or use git dependency. |
| 2 | `src/desktop/mod.rs` | 181–1192 | **Monstrous `update()` function: ~1,011 lines.** Violates single-responsibility principle. Contains all UI rendering, event handling, shortcut processing, data generation, and state mutation. Impossible to test, debug, or reason about. | Split into separate methods per panel/concern: `render_hud()`, `render_trading()`, `render_conversations()`, `render_sidebar()`, `handle_shortcuts()`, `handle_emotion_transitions()`. Each method should be <100 lines. |

### P1 — High (security, data integrity, major reliability)

| # | File | Line | Description | Suggested Fix |
|---|---|---|---|---|
| 3 | `src/desktop/mod.rs` | 155, 170 | **Plaintext API keys and broker passwords in `App` struct.** `ProviderEntry.key` and `broker_password` are `String` fields readable via any memory dump or debugger. Vault is not used for these. | Use the vault abstraction for all secrets; zeroize on drop; never keep plaintext keys in long-lived UI structs. |
| 4 | `src/frontend/mod.rs` | 228–229 | **Hardcoded Fish Audio voice ID:** `612b878b113047d9a770c069c8b4fdfe` baked into production code as fallback. | Move to config file or user settings; never hardcode provider-specific IDs in source. |
| 5 | `src/vault.rs` | 48–53 | **`VaultReader::get_key()` always returns `None`.** The entire vault abstraction is a no-op stub. No API key is ever retrievable. | Implement actual decryption of vault.enc, or remove the dead abstraction entirely. |
| 6 | `src/vault.rs` | 55–63 | **`check_key_available()` makes a blocking HTTP call to `127.0.0.1:8340`** on every key check with no timeout handling. | Use async reqwest; add timeout; cache results; handle server-offline gracefully. |
| 7 | `src/desktop/mod.rs` | 1111 | **`broker_password` field is `String`** (plaintext) in a struct that is `Clone` and `Debug` (via derive on `App` not shown, but `Default` is derived). | Mark as `#[cfg_attr(not(test))]` sensitive; implement `Drop` to zeroize; do not derive `Clone`/`Debug` on `App`. |
| 8 | `src/config.rs` | 75–79 | **Hardcoded absolute paths in default config:** `/mnt/external/projects/umbra/models`, `/mnt/external/projects/jarvis`, etc. These are development-machine-specific and will break on any other system. | Default to relative paths from `$HOME/.umbra/` or use XDG directory specifications. |
| 9 | `src/agent_memory.rs` | 510–524 | **`load_from_disk()` / `save_to_disk()` hold `Mutex` lock across blocking I/O.** If called from async context, this can stall the runtime. | Use `tokio::sync::Mutex` and async I/O (`tokio::fs`), or scope the lock to not span the I/O call. |
| 10 | `src/rate_limiter.rs` | 20–21 | **`check_action()` is `async` but uses `std::sync::Mutex`.** Asynchronous function calling `lock().unwrap()` can cause deadlocks with tokio if the lock is held across an await point (currently not, but fragily). | Use `tokio::sync::Mutex` or refactor to sync-only usage. |
| 11 | `src/sphere.rs::render()` + `src/agent_memory.rs::hsv_to_rgb()` | 111–124 vs 718–735 | **HSV→RGB conversion logic is copy-pasted identically** in two different files with identical implementation. | Extract to a shared utility function (e.g. `crate::util::hsv_to_rgb`). |

### P2 — Medium (design issues, dead code, missing docs, maintainability)

| # | File | Line | Description | Suggested Fix |
|---|---|---|---|---|
| 12 | `src/sphere.rs` | 46 | **`SphereParticle.vx`, `vy`, `vz` are always initialized to `0.0` and never mutated.** Unused dead weight in every particle. | Remove unused velocity fields from `SphereParticle`. |
| 13 | `src/sphere.rs` | 48 | **`SphereParticle.hue` is computed but never read** in `render()` — only `mood_hue` from the parameter is used. | Remove the per-particle `hue` field or use it in rendering. |
| 14 | `src/sphere.rs` | 24–30 | **`AgentSphere.hue` field is set (line 86) but never read** in the `render()` method. Agent color is determined by `active` boolean only. | Remove unused `hue` field or use for color computation. |
| 15 | `src/desktop/mod.rs` | 206 | **`_dim_bg` variable is assigned but never used.** Suppressed with underscore prefix but still computed every frame. | Remove dead variable and its computation. |
| 16 | `src/desktop/mod.rs` | 288 | **`_top_bar` variable unused** — TopBottomPanel result discarded. | Remove `let _top_bar =` binding. |
| 17 | `src/desktop/mod.rs` | 6 | **Unused import: `use crate::agent_memory::{AgentMemory, ...}`** — `AgentMemory` is never directly referenced in this file (only via `CognitiveBehavior`, `EmotionalState`). Wait — actually check: line 97 uses `self.agent_memory = AgentMemory::new()`. OK, `AgentMemory` is used. | N/A — not actually unused. See next line. |
| 18 | `src/desktop/mod.rs` | 6 | **`CognitiveBehavior` is imported but only used implicitly via `self.cognitive`** which is type-inferred. Consider explicit types for clarity. | Use explicit type annotation or keep import for clarity. |
| 19 | `src/desktop/mod.rs` | N/A | **All `View`, `State`, `Message`, `Conversation`, `ShortcutEntry`, `ProviderEntry`, `AgentEntry` structs lack doc comments.** Public APIs are undocumented. | Add `///` doc comments explaining each type's purpose and invariants. |
| 20 | `src/sphere.rs` | 1 | **Module-level documentation missing.** No explanation of what `SphereRenderer` is, coordinate system, or usage. | Add `//!` documentation block explaining the 3D sphere rendering system. |
| 21 | `src/desktop/mod.rs` | 1194–1365 | **`impl App` block methods lack documentation.** `send_hud_message`, `send_conv_message`, `send_trading_message`, `generate_ai_response`, `detect_local_models`, etc. have no doc comments. | Document each method's purpose, side effects, and error conditions. |
| 22 | `src/providers/mod.rs` | 114–116 | **`DEFAULT_API_TYPE` is `"openai"` but `OPENCODE_GO_ID` is `"opencode-go"`** — inconsistent naming convention. | Use consistent kebab-case or snake_case for identifiers. |
| 23 | `src/desktop/mod.rs` | 80–86 | **`SYMBOL_SETS` is defined but never referenced** in the code; trading symbols are hardcoded as `prices` vector on line 452 instead. | Either use `SYMBOL_SETS` or remove dead constant. |
| 24 | `src/desktop/mod.rs` | 452–458 | **Hardcoded price data** for five symbols replicated as a literal `vec!`. Prices are stale and will never update. | Remove fake prices; connect to real data feed or at least generate them procedurally. |
| 25 | `src/config.rs` | 99 | **Hardcoded `auth_dir: PathBuf::from("~/.umbra")`** — the `~` is NOT expanded by Rust; this creates a literal directory named `~` in CWD. | Use `std::env::var("HOME")` or the `dirs` crate (already in dependencies!) to get home directory. |
| 26 | `src/config.rs` | 58–59 | **`UmbraConfig::load()` silently swallows parse errors** via `unwrap_or_default()`. A malformed config.toml will silently use defaults with no warning to the user. | Log a warning and return a `Result` so the caller can inform the user. |
| 27 | `src/desktop/mod.rs` | 1195–1210 | **`load_logo_texture()` silently ignores all errors** (file not found, corrupt image, etc.). | Log errors at `tracing::warn!` for debugging. |
| 28 | `src/desktop/mod.rs` | 1244–1258 | **`detect_local_models()` uses `reqwest::blocking`** in what may be called from an async context or during startup. Blocking I/O in async context can stall the runtime. | Make async or use `tokio::task::spawn_blocking`. |
| 29 | `Cargo.toml` | 18–88 | **All dependencies pinned to exact patch versions** (e.g., `=0.12.28`). No flexibility for security patches. | Use caret requirements (`0.12` or `1.52`) and rely on `Cargo.lock` for reproducibility. |
| 30 | `Cargo.toml` | 94–96 | **Features `full`, `minimal` are empty** — no conditional compilation features defined. | Either remove or add meaningful feature gates for optional functionality. |
| 31 | `Cargo.toml` | 95–96 | **`paste` crate patched to a custom fork** at `https://github.com/MethodWhite/paste`. This is a supply-chain risk and is opaque. | Document why the fork is needed, or upstream the change. |
| 32 | `src/frontend/mod.rs` | 67 | **Hardcoded developer home path** `/home/methodwhite/frontend/dist` in production source code. | Remove personal paths; rely on environment variables or build-time configuration. |

### P3 — Low (style, minor performance, code organization)

| # | File | Line | Description | Suggested Fix |
|---|---|---|---|---|
| 33 | `src/lib.rs` | 117 | **`pub mod sphere;` is declared after the `UmbraApp` struct** — inconsistent with all other `pub mod` declarations at the top of the file. | Move to line 24 with other module declarations. |
| 34 | `src/desktop/mod.rs` | 330 | **String allocation every frame for clock display.** `format!("{:02}:{:02}:{:02}", ...)` allocates on every UI frame (~60fps). | Use chrono formatting or pre-compute only when seconds change. |
| 35 | `src/desktop/mod.rs` | 330 | **`update_voice_tone()` called every frame** (line 192) but only needs to change when emotion changes. | Gate on `current_emo != last_emotion_str` (which is already tracked). |
| 36 | `src/desktop/mod.rs` | 963, 967, 970 | **Gender vectors re-allocated every frame** — `genders` vec (line 967) created on each render pass. | Define as a `const` outside the function. |
| 37 | `src/desktop/mod.rs` | 858, 874 | **Tab arrays re-created per frame** — `["SETTINGS", ...]` and `["VAULT", ...]` string slices allocated each frame. | Define as `const` or `static` arrays. |
| 38 | `src/sphere.rs` | 107–109 | **Magic numbers `0.3`, `0.2`, `0.15`** for rotation speeds. No named constants. | Define as `const ROT_Y_SPEED: f32 = 0.3;` etc. |
| 39 | `src/sphere.rs` | 154 | **Hardcoded FOV `300.0`** used in two places (render and agent render). | Define as `const FOV: f32 = 300.0;` and reuse. |
| 40 | `src/agent_personality.rs` | 1–6 | **Module doc comment is a design intent note, not API documentation.** Redundant and not useful to consumers. | Write proper API docs or remove. |
| 41 | `src/job_queue.rs` | 14 | **`handle: Arc<Mutex<Option<JoinHandle>>>`** — Overly complex. A single `tokio::sync::Notify` or `AbortHandle` would suffice for shutdown. | Use `tokio_util::sync::CancellationToken` or `JoinHandle` directly. |
| 42 | `src/desktop/mod.rs` | 1353–1363 | **`generate_ai_response()` returns hardcoded responses** — it's a stub that cycles through 7 pre-programmed messages. No actual AI inference. | Either integrate actual AI model or mark explicitly as `TODO` stub with documentation. |
| 43 | `src/desktop/mod.rs` | 1327–1347 | **`send_trading_message()` uses pattern matching on hardcoded keywords** to produce fake AI responses. This is a mock that will confuse users into thinking trading is functional. | Clearly label as simulation mode; separate fake responses from real integration. |
| 44 | `src/cache.rs` | 84 | **`unwrap_or_default()` on reqwest `Client::builder().build()`** — silently uses a default (potentially broken) HTTP client if builder fails. | Propagate error or handle explicitly. |

---

### Summary Statistics

| Metric | Value |
|---|---|
| Files audited | 15+ |
| Total issues found | 44 |
| P0 (critical) | 2 |
| P1 (high) | 9 |
| P2 (medium) | 21 |
| P3 (low) | 12 |
| Code quality score | 4/10 |
| Test coverage (umbra crate) | 0% |
| Functions > 100 lines | 1 (`update` at ~1,011 lines) |
| Dead code instances | 6 |
| Hardcoded values / paths | 9 |
| Race condition / sync risks | 4 |
| Security issues | 4 |
| Missing documentation | 8 |
| Duplicated code areas | 2 |

### Top 5 Priority Fixes

1. **Fix synapsis-core dependency** (P0) — Unblocks compilation.
2. **Refactor `update()` into focused sub-methods** (P0) — Unblocks testing and maintenance.
3. **Implement actual vault decryption** (P1) — API keys are entirely inaccessible.
4. **Add tests for core modules** (P0-effect) — Agent memory, sphere math, config loading.
5. **Fix hardcoded `~` path in config** (P2) — `~/.umbra` literal directory will silently break on most systems.
