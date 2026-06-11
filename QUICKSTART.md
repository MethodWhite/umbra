# UMBRA — Quick Start

**Version:** 0.1.0
**Last Updated:** 2026-06-09

---

## Prerequisites

- **Rust** (latest stable) — install via `rustup`
- **pnpm** — already configured in workspace (see `frontend/pnpm-workspace.yaml`)
- **Node.js via fnm** — already configured (Angular 19 requires Node 18+)
- **Synapsis** — local crate at `../synapsis/` (path dependency in `Cargo.toml`)
- **Python 3.11+** — only needed if running JARVIS voice frontend (`../jarvis/`)

---

## Build & Run

### Desktop App (Recommended — Electron)

```bash
cd /mnt/external/projects/umbra

# 1. Build the Rust backend
cargo build --release

# 2. Build the Angular frontend
cd frontend && pnpm exec ng build --configuration production && cd ..

# 3. Run as Electron desktop app
cd electron && pnpm start
```

Electron will:
- Auto-start the Rust backend (`--api-port 8484 --no-frontend`)
- Load the Angular frontend from `frontend/dist/browser/`
- Show system tray with Start/Stop/Status controls
- Activate voice on `Cmd+Shift+U`

### Web App (Development Mode)

**Terminal 1 — Backend:**
```bash
cd /mnt/external/projects/umbra
cargo build --release
./target/release/umbra --api-port 8484
```

**Terminal 2 — Frontend dev server:**
```bash
cd /mnt/external/projects/umbra/frontend
pnpm exec ng serve
```

Open `http://localhost:4200` in your browser. The Angular dev server proxies API calls to the Rust backend on `:8484`.

### Web App (Production Mode, Rust-served)

Build the Angular frontend and let Rust serve it:

```bash
cd /mnt/external/projects/umbra
cargo build --release
cd frontend && pnpm exec ng build --configuration production && cd ..
./target/release/umbra
```

- Backend API: `http://localhost:8484`
- Frontend UI: `http://localhost:8340`

### CLI

```bash
# Build first
cargo build --release

# Start backend daemon
./target/release/umbra start

# Launch egui desktop GUI
./target/release/umbra gui

# Check running status
./target/release/umbra status

# Stop backend
./target/release/umbra stop

# Custom port
./target/release/umbra --api-port 9000 --frontend-port 9001

# Enable TLS (auto self-signed cert)
./target/release/umbra --ssl

# Backend only (no frontend server)
./target/release/umbra --api-port 8484 --no-frontend
```

---

## Configuration

### Config File: `~/.umbra/config.toml`

```toml
[api]
backend_port = 8484
frontend_port = 8340
backend_host = "127.0.0.1"
frontend_host = "127.0.0.1"

[audio]
fish_api_url = "https://api.fish.audio/v1/tts"
default_voice_id = "612b878b113047d9a770c069c8b4fdfe"

[ollama]
base_url = "http://localhost:11434"

[paths]
models_dir = "/mnt/external/projects/umbra/models"
subagents_dir = "/mnt/external/projects/umbra/sub_agents"
jarvis_dir = "/mnt/external/projects/jarvis"
logs_dir = "/mnt/external/projects/umbra/logs"

[training]
auto_train_interval_mins = 30
max_examples = 1000
jepa_epochs = 50

[security]
auth_dir = "~/.umbra"
env_file_perms = "0600"
```

### Vault (Encrypted Credentials)

```bash
# Vault is auto-created on first unlock
# Unlock via UI: Settings → Vault → Unlock with passphrase
# Or via API:
curl -X POST http://localhost:8340/api/vault/unlock \
  -H "Content-Type: application/json" \
  -d '{"passphrase": "your-passphrase"}'

# Lock:
curl -X POST http://localhost:8340/api/vault/lock

# Check status:
curl http://localhost:8340/api/vault/status

# Set a key:
curl -X POST http://localhost:8340/api/vault/key \
  -H "Content-Type: application/json" \
  -d '{"provider_id": "openai", "api_key": "sk-..."}'

# Migration from legacy:
curl -X POST http://localhost:8340/api/vault/migrate
```

### Data Locations

| Path | Purpose |
|------|---------|
| `~/.umbra/config.toml` | Backend configuration |
| `~/.umbra/vault.enc` | Encrypted API key storage (AES-256-GCM) |
| `~/.umbra/vault.lock` | Vault lock state |
| `~/.umbra/auth_token` | API authentication token (auto-generated) |
| `~/.umbra/customization.json` | User preferences (encrypted) |
| `~/.umbra/backend.log` | Backend runtime logs |
| `~/.umbra/tls/` | TLS cert + key (auto-generated) |
| `/mnt/external/projects/umbra/models/` | Local model files |
| `/mnt/external/projects/umbra/sub_agents/` | Sub-agent definitions (.materia) |

---

## System Modes

Three security modes configurable via UI or API:

| Mode | TLS | Vault Required | Auto-Lock | Use Case |
|------|-----|----------------|-----------|----------|
| **Secure** | Yes | Yes | 5 min | Production, sensitive data |
| **Balanced** (default) | Optional | No | 15 min | General use |
| **Unrestricted** | No | No | Never | Local development |

```bash
# Set mode via API:
curl -X POST http://localhost:8340/api/setup/mode \
  -H "Content-Type: application/json" \
  -d '{"mode": "secure"}'
```

---

## Providers

23 API providers supported across 3 categories:

### Western Cloud
| Provider | ID | Auth |
|----------|----|------|
| OpenAI | `openai` | `OPENAI_API_KEY` |
| Anthropic | `anthropic` | `ANTHROPIC_API_KEY` |
| Google Gemini | `google` | `GEMINI_API_KEY` |
| Mistral AI | `mistral` | `MISTRAL_API_KEY` |
| Groq | `groq` | `GROQ_API_KEY` |
| Together AI | `together` | `TOGETHER_API_KEY` |
| Cohere | `cohere` | `COHERE_API_KEY` |
| Perplexity | `perplexity` | `PERPLEXITY_API_KEY` |
| NVIDIA Riva TTS | `nvidia-riva` | `NVIDIA_API_KEY` |

### Chinese Cloud
| Provider | ID | Auth |
|----------|----|------|
| DeepSeek | `deepseek` | `DEEPSEEK_API_KEY` |
| Qwen (Alibaba) | `qwen` | `QWEN_API_KEY` |
| Baidu ERNIE | `baidu` | `BAIDU_API_KEY` |
| Zhipu GLM | `zhipu` | `ZHIPU_API_KEY` |
| Moonshot AI | `moonshot` | `MOONSHOT_API_KEY` |
| 01.AI Yi | `yi` | `YI_API_KEY` |
| StepFun | `stepfun` | `STEPFUN_API_KEY` |
| MiniMax | `minimax` | `MINIMAX_API_KEY` |

### Local
| Provider | ID | URL |
|----------|----|-----|
| Ollama | `ollama` | `http://localhost:11434/v1` |
| llama.cpp | `llamacpp` | `http://localhost:8080/v1` |

### Subscription
| Provider | ID | URL |
|----------|----|-----|
| OpenCode Go | `opencode-go` | `https://opencode.ai/zen/go/v1` |

---

## API Endpoints

### Backend (`:8484`)

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/chat` | Send message to agent |
| POST | `/api/v1/command` | Execute command action |
| GET | `/api/v1/status` | Agent status and health |
| GET | `/api/v1/health` | Backend health check |
| WS | `/api/v1/ws` | WebSocket for streaming responses |
| POST | `/api/v1/memory/search` | Search agent memory |
| POST | `/api/v1/memory/store` | Store a memory entry |
| GET | `/api/v1/subagents` | List running sub-agents |
| POST | `/api/v1/subagents/spawn` | Spawn a new sub-agent |
| GET | `/api/v1/config` | Get current configuration |

### Frontend (`:8340`)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Frontend health check |
| GET | `/api/auth/session` | Create auth session |
| GET | `/api/setup/status` | Setup/wizard status |
| GET | `/api/providers` | List all providers |
| GET | `/api/providers/{id}` | Get provider details |
| POST | `/api/providers/configure` | Configure provider |
| POST | `/api/providers/test` | Test provider connection |
| POST | `/api/providers/test-all` | Test all providers |
| GET | `/api/providers/config/status` | Config status |
| GET | `/api/vault/status` | Vault status |
| POST | `/api/vault/unlock` | Unlock vault |
| POST | `/api/vault/lock` | Lock vault |
| GET | `/api/vault/keys` | List vault keys |
| POST | `/api/vault/key` | Set API key |
| GET/DELETE | `/api/vault/key/{provider_id}` | Get/delete key |
| POST | `/api/vault/migrate` | Migrate from env/providers.toml |
| POST | `/api/vault/auto-lock` | Set auto-lock minutes |
| GET | `/api/settings/voice` | Get voice settings |
| POST | `/api/settings/voice` | Set voice settings |
| GET/POST | `/api/settings/preferences` | Get/set preferences |
| GET | `/api/settings/status` | System status |
| GET/POST | `/api/customization` | Get/set customization |
| POST | `/api/setup/mode` | Set system mode |
| GET | `/api/security/check` | Security audit check |
| POST | `/api/browser/search-and-train` | Browser search + train |
| POST | `/api/browser/visit` | Visit URL |
| GET | `/api/browser/status` | Browser status |
| POST | `/api/browser/collect` | Collect URLs |
| GET/POST | `/api/browser/settings` | Browser settings |
| GET | `/api/models/discover` | Discover HuggingFace models |
| GET | `/api/models/discover/{tag}` | Search models by tag |
| GET | `/api/tts-test` | Test TTS synthesis |
| WS | `/ws/voice` | Voice WebSocket |

---

## Development

```bash
# Build backend only
cargo build

# Build with optimizations
cargo build --release

# Run tests (none exist yet)
cargo test

# Check advisories
cargo deny check advisories
cargo deny check licenses

# Check supply chain
cargo vet
```

### Common Issues

1. **"Synapsis not found"** — Ensure `../synapsis/` exists (path dependency in Cargo.toml)
2. **"Frontend not built"** — Run `cd frontend && pnpm exec ng build --configuration production`
3. **"Port 8484 in use"** — Kill existing process or use `--api-port` flag
4. **"Vault locked"** — Unlock via API or UI Settings → Vault
5. **"Auth unauthorized"** — Check `~/.umbra/auth_token` exists and matches between requests
