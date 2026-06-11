# Umbra Product Review

## Business Value Assessment

### Highest Value Features
1. **Multi-Provider LLM Router (23 providers)** — The core value prop. User brings their own API keys, routes through one interface. Essential for cost optimization and redundancy.
2. **Local-First Architecture** — Works fully offline with Ollama/llama.cpp. Strong differentiator for privacy-conscious traders.
3. **Encrypted Vault (AES-256-GCM + PBKDF2)** — Required for broker credential security. Makes trading integration viable.
4. **IronClaw Safety Constraints** — Max positions, daily loss limits, blocked commands. Critical for any autonomous trading agent.
5. **Clean Architecture (47 modules)** — Domain/application/infrastructure layers with trait-based ports. Enables testability and swapping implementations.

### What's Missing for MVP
- **Real trading execution** — MT5 bridge is defined but uses simulated balance/equity. No actual broker connection works.
- **Working STT** — Speech-to-text is listed as a port but has no real implementation.
- **Agent actually uses an LLM** — `generate_ai_response()` returns hardcoded strings; no real AI inference happens in the GUI.
- **Tests** — 0% test coverage in the core `umbra` crate. Cannot ship with confidence.
- **Vault actually works** — `VaultReader::get_key()` always returns `None`. API keys are stored in plaintext.
- **Compilation** — Blocked by missing `synapsis-core` dependency.

### Can Be Deferred
- Voice cloning, emotional AI, 3D particle sphere (visual candy)
- WASM sandbox (advanced threat model, overkill for MVP)
- Post-quantum crypto definitions (Kyber/Dilithium are spec-only; no real exchange)
- HSAQ compression engine (useful but not core)
- HuggingFace model downloader (nice-to-have)

## Feature Gap Analysis

| Feature | Current State | Industry Standard | Priority |
|---|---|---|---|
| LLM Chat | Mock responses only | Real streaming inference | P0 |
| Broker Integration | Simulated paper trading | Live API/FIX connection | P0 |
| Credential Security | Plaintext in struct | Vault/HSM-backed | P0 |
| Test Coverage | 0% (umbra crate) | >80% | P0 |
| Agent Autonomy | Stub loop | Production agents execute real tasks | P0 |
| Build Reliability | Broken (missing dep) | Green CI | P0 |
| Voice TTS | Fish/Piper stubs | Working TTS with fallback | P1 |
| Multi-Agent Orchestration | 4 agents listed, none functional | Real sub-agent delegation | P1 |
| Real-time Data | Fake hardcoded prices | Live market data feed | P1 |
| Emotional AI | 80+ states defined | Basic sentiment analysis | P2 |
| Post-Quantum Crypto | Spec-only | N/A for MVP | P3 |
| 3D Sphere Visualization | 500 particles | N/A for utility | P3 |

## Market Positioning

### Compared to Competitors

| Aspect | Umbra | AutoGPT | MetaGPT | AgentGPT |
|---|---|---|---|---|
| Language | Rust | Python | Python | TypeScript |
| Architecture | Clean Architecture | Monolithic | Role-based | SaaS |
| Trading | MT5 bridge (planned) | None | None | None |
| Security | PQC, vault, IronClaw | Minimal | None | None |
| Local-First | Yes (Ollama/llama.cpp) | Partial | No | No |
| GUI | egui native + Angular | Web only | CLI | Web only |
| Emotional AI | Plutchik wheel | None | None | None |
| Test Coverage | 0% (core) | Moderate | Low | Low |
| Maturity | Pre-alpha | Production | Production | Production |

### Unique Selling Points
1. **Rust-based AI agent** — Memory-safe, no GC, instant startup. No other major agent framework uses Rust.
2. **Trading-first design** — MT4/MT5 bridge, position management, risk controls. Only Umbra targets this niche.
3. **Security layers** — IronClaw, RuntimeEnforcer, ZeroTrustGate, AntiBrick, AuditWorm. Over-engineered but defensible for a trading agent.
4. **Post-quantum cryptography** — First-mover positioning in AI agent security.
5. **Local + cloud hybrid** — Switch between Ollama and OpenAI with a config change.

### Target Audience
- **Quantitative traders / retail algo traders** — Core audience. Need automated trading with guardrails.
- **Privacy-conscious developers** — Want local-first AI without telemetry.
- **Rust enthusiasts** — Technical audience who appreciate systems-level performance.
- **NOT ready for non-technical users** — Requires `cargo build`, missing `synapsis` dep, no packaging.

## Innovation Opportunities

### High-Impact
1. **Voice-first AI agent** — "Umbra, what's my P&L?" / "Umbra, open a 0.5 BTCUSD buy." Differentiator: most agents are chat-only. Voice + trading = immediate value.
2. **Emotional AI for trading discipline** — Detect fear/greed patterns in user input; warn before irrational trades. Unique angle: AI as trading psychologist.
3. **Real-time market data with local LLM analysis** — Streaming prices fed to local LLM with structured prompts. "BTC just broke resistance on 4H — recommend?"
4. **On-device fine-tuning for trading** — Fine-tune small models on user's trading history and win/loss patterns.

### Medium-Impact
5. **Integrated trading journal** — Auto-log every trade with AI commentary, emotional state, market conditions. Build a data moat.
6. **Sub-agent marketplace** — `.materia` file format as a plugin system; community-contributed trading strategies.
7. **Multi-broker aggregation** — Execute across IC Markets, Pepperstone, FTX (if it returns) through one interface.

### Low-Impact (Defer)
8. **3D sphere visualization** — Looks cool, zero utility for traders. Defer to v2.
9. **80+ emotional states** — Over-engineered. 5 basic states (calm, focused, excited, cautious, stressed) would suffice for a trading agent.
10. **Post-quantum key exchange** — No quantum computer will break ECDSA in the next 5 years for retail trading.

## Roadmap Recommendations

### Next 3 Sprints

#### Sprint A: Ship the Core (Fix Broken Stuff)
1. **Fix synapsis-core dependency** — Compilation is priority 0.
2. **Make vault actually decrypt** — Implement `VaultReader::get_key()`. No more `None` returns.
3. **Replace mock AI responses with real LLM inference** — Connect conversation to a provider (Ollama first, for local dev).
4. **Add tests** — At minimum: config loading, vault encryption/decryption, emotional state transitions, sphere particle math.

#### Sprint B: Trading MVP (Get Real)
5. **Connect to a real broker** — IC Markets or Pepperstone via MT5 bridge. Paper trading on a real connection.
6. **Live market data** — Replace fake prices with WebSocket feed (e.g., Polygon.io, Finnhub, or broker-native feed).
7. **Implement real stop-loss / take-profit** — Orders go through with actual risk parameters.
8. **IronClaw risk enforcement on real orders** — Validate max loss, max positions before execution.

#### Sprint C: Voice + Polish (Make It Usable)
9. **Voice input (STT)** — Implement whisper locally. Voice commands: "open buy 0.1 EURUSD".
10. **Voice output (TTS)** — Fish Audio or Piper with emotional tone modulation. "Your stop-loss was hit on BTCUSD, loss $45."
11. **Keyboard shortcuts overlay** — Hit `?` to show available shortcuts.
12. **Split `update()` into sub-methods** — Monstrosity is blocking all further UI work.

### Build vs Buy/Integrate
| Decision | Recommendation | Rationale |
|---|---|---|
| Market data | **Buy** (Polygon.io/Finnhub) | Building a data feed aggregator is not core IP |
| STT | **Integrate** (whisper.cpp) | Mature open-source, Rust bindings exist, local-first |
| TTS | **Integrate** (Piper/espeak) | Already partially done; complete the integration |
| Broker API | **Build** (MT5 bridge) | Core to product; no off-shelf solution for MT5 + Rust |
| Emotional AI | **Build** (simplified) | Differentiator, but trim from 80 states to 5-10 |
| WASM sandbox | **Buy** (wasmtime) | Already integrated; just configure properly |
| PQC | **Buy** (liboqs Rust bindings) | Don't implement Kyber/Dilithium from scratch |
