# UMBRA + Synapsis — Pendientes

## Synapsis (proyecto completo en ../synapsis/)
### Core (synapsis-core) — LISTO
- [x] Token-efficient storage: `summary`, `importance`, `token_count`, `access_count`
- [x] `Observation::efficient_content(max_tokens)` — retorna summary si cabe en budget
- [x] `SearchParams::with_max_tokens()` — token budget en búsquedas
- [x] `Database::retain(max_tokens)` — evicción LRU + baja importancia
- [x] `MemoryStats` con total_entries, total_tokens, avg_importance, unique_sessions
- [x] `compute_importance()` + `decay_importance()` — scoring automático
- [x] SQLite real: observaciones con índices en importance y created_at
- [x] 11 tests con métricas (token budget, importance ordering, retain eviction)

### Wrapper (synapsis) — ROTO (65 errores de compilación)
- [ ] `domain/crypto` — fix `CryptoProvider` trait vs struct (cambiado en core)
- [ ] `app_core/updater.rs` — import de `Database` (ok, funciona)
- [ ] `presentation/mcp/` — adaptar a nuevos tipos de core
- [ ] `bin/synapsis-cli.rs` — adaptar a `Observation` con nuevos campos
- [ ] `infrastructure/` — re-exportar nuevos tipos correctamente
- [ ] `application/memory/` — adaptar a `StoragePort` + `MemoryPort` traits
- [ ] Correr tests del wrapper

## Umbra (./)
### Urgente (rompen build)
- [ ] Nada — 0 errores, 0 warnings, 17 tests pass

### Funcionalidad
- [ ] **STT real**: conectar whisper.cpp al voice input del desktop
- [ ] **Market data**: conectar broker API al panel de trading
- [ ] **Responsive UI**: manejar resize de ventana correctamente
- [ ] **Voice clone**: implementar `CloneVoiceUseCase`
- [ ] **Agent orchestration**: lógica real de agentes (no mock)

### Deuda técnica
- [ ] `desktop/mod.rs` — 1467 líneas, necesita partitioning
- [ ] `infra/` (old) vs `infrastructure/` — merge o eliminar duplicado
- [ ] Más tests: apuntar a >40% cobertura (actual: 5%)
- [ ] Auditoría de seguridad: 2 vulns en openjarvis-tools

### Métricas deseadas (Synapsis core)
- [ ] Token budget: debe limitar resultados a ≤ max_tokens ✅ (test_token_budget_enforced)
- [ ] Importance ordering: resultados ordenados por importancia ✅
- [ ] Retain eviction: elimina entradas de baja prioridad ✅
- [ ] Summary efficiency: resumen usa ≤ tokens que contenido completo ✅

## Cómo continuar
1. `cd ../synapsis && cargo build --release` para ver errores del wrapper
2. Arreglar uno por uno los 65 errores de tipo en el wrapper
3. `cd umbra && cargo build --release --bin umbra-gui` para verificar desktop
4. `cd umbra && cargo build --release --bin umbra --features server` para verificar server
