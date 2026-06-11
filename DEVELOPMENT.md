# UMBRA — Development Methodology (Gentleman Programming)

## Zones System

Each module belongs to a zone. Zones define the security layer, evaluation criteria, and handoff protocol.

| Zone | Layer | Security | Evaluation |
|------|-------|----------|------------|
| **0** Config/Init | Safe | None | Compiles |
| **1** Desktop/UI | User | Type-check | No crashes |
| **2** Domain/Ports | Contract | Trait bounds | Interface stable |
| **3** Application | Logic | Input validation | Use cases pass |
| **4** Infrastructure | I/O | Rate-limit + Audit | Integration tests |
| **5** Bridge/FFI | External | Sandbox + Thoth | E2E tests |
| **6** Research/Stubs | Experimental | Feature-gated | Not compiled by default |

## Workflow Rules

1. **Know your zone**: Before editing, annotate `// Zone X` in the file.
2. **Crossing zones**: When code in zone N calls zone N+1, wrap in a use case or port interface. Never skip layers.
3. **Security at boundaries**:
   - Zone 3→4: validate all inputs
   - Zone 4→5: sandbox FFI calls, rate-limit
   - Zone 5→6: feature-gate behind `#[cfg(feature)]`
4. **Evaluation**: Each zone has a `health()` check:
   - Zone 0: `cargo check`
   - Zone 1: `cargo build --bin umbra-gui`
   - Zone 2-3: `cargo test --lib`
   - Zone 4: `cargo test --features integration`
   - Zone 5: Requires MT5 terminal
   - Zone 6: `cargo build --features server`

## File Annotation

Every file should have a zone comment at the top:

```rust
// Zone 1 — Desktop UI
```

```rust
// Zone 2 — Domain port trait
```

```rust
// Zone 4 — Infrastructure implementation (server-gated)
```

## Current Zone Map

| Zone | Files | Status |
|------|-------|--------|
| 0 | `config.rs`, `Cargo.toml`, `deny.toml` | ✅ |
| 1 | `desktop/mod.rs`, `sphere.rs`, `agent_memory.rs` | ⚠️ 1500-line file needs split |
| 2 | `domain/ports/*.rs`, `domain/models/*.rs` | ✅ |
| 3 | `application/*/` | ⚠️ Stubs (todo!()) |
| 4 | `infrastructure/http/*`, `infrastructure/repositories/*` | ⚠️ Server-gated |
| 5 | `bridge/ffi.rs` | ⚠️ Not compiled |
| 6 | `engine/`, `jarvis/`, `ironclaw/`, `persona/` | ⚠️ Behind `--features server` |

## Security Layers

```
User Input ──Zone 1──> Desktop UI
   │
   ├──Zone 3──> Application Use Case
   │                │
   │                ├──Zone 2──> Port Interface
   │                │                │
   │                │                ├──Zone 4──> Infrastructure Impl (rate-limited)
   │                │                │                │
   │                │                │                └──Zone 5──> FFI/Bridge (sandboxed)
   │                │                │
   │                │                ├──Zone 4──> Vault (AES-256-GCM, auto-lock)
   │                │                │
   │                │                └──Zone 6──> Research/Experimental
   │
   └──Zone G──> LLM Inference (Ollama/llama.cpp)
                    │
                    └──Zone 4──> RAG over Synapsis memory (token-budgeted)
```

### Note: `infra/` vs `infrastructure/`

These are distinct modules, NOT duplicates:

- **`infra/`** (Zone 6) — System-level infrastructure: hardening (`OpenVentus`), backup, filesystem, ghost monitoring. Server-gated, not compiled for desktop.
- **`infrastructure/`** (Zone 4) — Application-level infrastructure: HTTP clients (Ollama, STT, TTS), persistence (vault repos), TTS/STT adapters, security (IronClaw, Thoth). Server-gated behind `--features server`.

They serve different layers and should remain separate.

## Quick Evaluation

```bash
# Zone 0
cargo check

# Zone 1 (desktop)
cargo build --release --bin umbra-gui

# Zone 2-3 (domain + application)
cargo test --lib --release

# Zone 4 + 5 (server)
cargo build --release --bin umbra --features server

# Zone 6 (research modules)
cargo build --release --features server

# Full security
cargo deny check
```
