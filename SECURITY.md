# UMBRA Security Policy

## Supply Chain Security Practices

### Dependency Pinning
All direct dependencies are pinned to exact versions using `=x.y.z` syntax in `Cargo.toml`. This prevents supply chain attacks via dependency confusion or malicious patch releases.

### Automated Auditing
- **cargo-audit**: Scans `Cargo.lock` against the RustSec Advisory Database for known vulnerabilities
- **cargo-deny**: Enforces license compliance, bans duplicate crate versions, and validates crate sources
- **cargo-vet**: Maintains a supply-chain integrity audit of all dependencies

### Verification Commands
```bash
# Check for known vulnerabilities
cargo audit

# Full dependency policy check
cargo deny check

# Supply chain audit
cargo vet
```

## How to Verify the Build

```bash
# 1. Authenticate sources
cargo fetch

# 2. Run all security checks
cargo audit
cargo deny check
cargo vet

# 3. Build with locked dependencies
cargo build --locked --release

# 4. Verify checksums
sha256sum Cargo.lock
```

## Dependency Update Process

1. Run `cargo audit` to check for existing vulnerabilities
2. Update individual crates with `cargo update -p <crate>`
3. Pin the new exact version in `Cargo.toml`
4. Re-run `cargo audit && cargo deny check && cargo vet`
5. Update `supply-chain/config.toml` entries via `cargo vet`

## How to Report Vulnerabilities

For security vulnerabilities in UMBRA itself:
- **Email**: methodwhite@proton.me
- **Do NOT** open public GitHub issues for security vulnerabilities
- Expected response time: 72 hours

For vulnerabilities in dependencies:
- Check the RustSec Advisory Database: https://rustsec.org
- Update the affected dependency and re-pin

## External Network Calls

UMBRA makes the following external network connections:

| Service | Protocol | Port | Purpose | Data Sent |
|---------|----------|------|---------|-----------|
| Ollama API | HTTP/TCP | 11434 | Local LLM inference | Prompts, model config |
| OpenAI API | HTTPS/TCP | 443 | Cloud LLM fallback | Prompts, auth token |
| HuggingFace Hub | HTTPS/TCP | 443 | Model downloads | None (public models) |
| MT4 Terminal | TCP | 15555 | Trading bridge | Orders, market data |
| MT4 Terminal | TCP | 15556 | Market data stream | Price ticks |
| Browser-based auth | HTTP/TCP | 8340 | Frontend UI | Session data |
| mDNS discovery | UDP | 5353 | Service discovery | Hostname, service type |

### Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        UMBRA Host Machine                        │
│                                                                  │
│  ┌──────────┐    ┌──────────┐    ┌────────────┐                 │
│  │  Web UI  │◄──►│ API Svr  │◄──►│ Agent Loop │                 │
│  │ :8340    │    │ :8341    │    │            │                 │
│  └──────────┘    └──────────┘    └──────┬─────┘                 │
│                                         │                        │
│          ┌──────────────────────────────┼──────────────┐         │
│          │          Outbound            │  Inbound      │         │
│          │          ┌─► Ollama (:11434) │              │         │
│          │          ├─► OpenAI (443)    │  MT4 Bridge  │         │
│          │          ├─► HF Hub (443)    │  (:15555)    │         │
│          │          └─► mDNS (5353)     │  (:15556)    │         │
│          └──────────────────────────────┼──────────────┘         │
└─────────────────────────────────────────────────────────────────┘
```

### What Data Leaves the Machine

1. **LLM Prompts** → Ollama (local) or OpenAI (cloud if configured)
2. **Authentication Tokens** → OpenAI API (if cloud LLM enabled)
3. **Trading Signals** → MT4 terminal (local network only)
4. **mDNS Announcements** → Local network (service discovery)
5. **Browser Telemetry** → Web UI user (frontend only, no external)

No user data, API keys, or trading credentials are ever transmitted to third parties outside of the configured LLM providers (and only if explicitly enabled by the user in `providers.toml`).
