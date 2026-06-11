# UMBRA

**Version:** 0.1.0
**License:** MIT
**Author:** methodwhite

UMBRA is an open-source AI agent system with multi-agent orchestration, post-quantum security architecture, real-time 3D HUD interface, cognitive emotional system, encrypted vault, and MT5 trading integration. Built in Rust with a native egui desktop GUI and optional Electron+Angular frontend.

## Quick Start

```bash
git clone <repo> && cd umbra
cargo build --release
./target/release/umbra start         # Start backend on :8484
./target/release/umbra gui           # Launch native desktop GUI
```

Full guide: [QUICKSTART.md](QUICKSTART.md)

### Requirements
- **Rust** (latest stable)
- **Synapsis** at `../synapsis/` (path dependency)
- **Node 18+** with **pnpm** (optional, for Angular frontend)
- **Ollama** (optional, for local LLM inference)

## Features

- **Multi-Agent Orchestration** — Hermes agent loop with plan→act→observe→learn cycle, sub-agent management, skill system
- **23 API Providers** — OpenAI, Anthropic, Google, DeepSeek, Qwen, Ollama, llama.cpp, OpenCode Go, and 15 more
- **Encrypted Vault** — AES-256-GCM + PBKDF2 (600K iterations) credential storage with auto-lock
- **3D HUD Interface** — egui native desktop GUI with 500-particle Fibonacci sphere, emotional color system
- **Cognitive Emotional System** — 80+ emotional states (Plutchik's Wheel), CBT-inspired cooling, agent personality/gender
- **MT5 Trading Bridge** — C ABI FFI, signal pipeline, order executor, strategy sandbox, professional trading UI
- **Post-Quantum Security** — Kyber-512/Dilithium-4 foundations, IronClaw validation, Zero-Trust Gate, AuditWorm logging
- **Model Compression** — HSAQ mixed-precision quantization (~3.3x compression, 1.5–6x speedup)
- **Local-First** — Full offline functionality, optional cloud providers, no telemetry
- **Supply Chain Security** — cargo-deny, cargo-vet, exact version pinning, patched transitive deps

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                       UMBRA SYSTEM                           │
│  ┌──────────┐    ┌──────────┐    ┌───────────────────────┐  │
│  │  Desktop  │    │  Web UI  │    │  Voice Frontend       │  │
│  │  GUI      │    │  (Angular)│   │  JARVIS (Python)      │  │
│  │  (egui)   │    │  :4200   │    │  FastAPI :8340        │  │
│  └─────┬─────┘   └────┬─────┘   └──────────┬────────────┘  │
│        └──────────────┼─────────────────────┘               │
│                       ▼  HTTP/WS                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Rust Backend (Axum :8484)               │  │
│  │  ┌────────┐ ┌──────────┐ ┌────────┐ ┌───────────┐  │  │
│  │  │ Agent  │ │ Ironclaw │ │Memory  │ │ SubAgents │  │  │
│  │  │ Engine │ │ Security │ │Synapsis│ │ Orchestr. │  │  │
│  │  └────────┘ └──────────┘ └────────┘ └───────────┘  │  │
│  │  ┌────────┐ ┌──────────┐ ┌────────┐ ┌───────────┐  │  │
│  │  │ LLM    │ │ Embed    │ │Audio   │ │ Resource  │  │  │
│  │  │ Router │ │ fastembed│ │Engine  │ │ Manager   │  │  │
│  │  └────────┘ └──────────┘ └────────┘ └───────────┘  │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ PQC-Crypto│  │ openjarvis│  │ Ollama   │  │ Fish.Audio│ │
│  │ Kyber/Dil │  │ (WASM)   │  │ Local LLM│  │ TTS       │ │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │
└──────────────────────────────────────────────────────────────┘
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed layer documentation.

## Documentation

| Document | Description |
|----------|-------------|
| [PAPER.md](PAPER.md) | Professional technical paper covering all system aspects |
| [SPRINTS.md](SPRINTS.md) | Complete development sprint history |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Detailed architecture and design decisions |
| [QUICKSTART.md](QUICKSTART.md) | Setup, build, configuration, API reference |
| [SECURITY.md](SECURITY.md) | Security policy and supply chain practices |
| [CHANGELOG.md](CHANGELOG.md) | Detailed changelog by date |
| [TODO.md](TODO.md) | Current issues and roadmap |

## Screenshots

*(Screenshots to be added)*

## Tech Stack

| Layer | Technology |
|-------|-----------|
| **Language** | Rust (edition 2021) |
| **HTTP Server** | Axum 0.8, Tokio, Hyper |
| **Desktop GUI** | egui 0.29 / eframe |
| **Web Frontend** | Angular 19, Three.js, RxJS |
| **Desktop Shell** | Electron (optional) |
| **Encryption** | AES-256-GCM, PBKDF2-SHA256 |
| **PQC** | Kyber-512, Dilithium-4 (definitions) |
| **Memory** | Synapsis (path dep), AgentMemory |
| **TTS** | Fish.Audio API, Piper, NVIDIA Riva |
| **Trading** | MT4/MT5 bridge via C ABI FFI |
| **Sandbox** | wasmtime (WASM) |
| **Auth** | Token-based, session cookies, WebSocket auth |
| **Build** | cargo, pnpm, fnm |
| **Security** | cargo-deny, cargo-vet |

## Project Structure

```
umbra/
├── src/                    # Rust backend
│   ├── main.rs, lib.rs     # Entry points
│   ├── api/                # Axum HTTP/WS server
│   ├── engine/             # JEPA, HSAQ, SNN, HDT, WASM, Router
│   ├── application/        # Clean Architecture use cases
│   ├── domain/             # Models, ports, errors
│   ├── infrastructure/     # Repositories, HTTP clients, persistence
│   ├── security/           # PQC, enforcer, zt_gate, antibrick, audit
│   ├── learning/           # Agent loop, skills, trainer, messaging
│   ├── bridge/             # MT5 trading bridge
│   ├── desktop/            # egui native GUI
│   ├── audio/              # TTS engine
│   ├── ironclaw/           # Action validation
│   └── agents/             # Agent personality, memory
├── frontend/               # Angular 19 web UI
├── electron/               # Electron desktop shell
├── docs/                   # Documentation
├── specs/                  # Component specifications
├── diagrams/               # Architecture diagrams
├── supply-chain/           # cargo-vet audits
└── models/                 # Local model storage
```

## License

MIT — See [LICENSE](LICENSE) for details.
