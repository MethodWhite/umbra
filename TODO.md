# UMBRA — Pendientes

## Estado actual
- `umbra` desktop: 0 errors, 0 warnings, 17 tests ✓
- `umbra` server: `--features server` ✓
- `synapsis-core`: 0 errors, 0 warnings, 11 tests ✓ (token-efficient)
- `synapsis` wrapper: ~42 errors (pausado)

## Pendientes
- [ ] **STT real**: conectar whisper.cpp al voice input del desktop
- [ ] **Market data**: broker API al panel de trading
- [ ] **Responsive UI**: resize + layout adaptativo
- [ ] **Voice clone**: implementar `CloneVoiceUseCase`
- [ ] **Agent orchestration**: lógica real de agentes
- [ ] `desktop/mod.rs` partitioning (1467→~500 líneas)
- [ ] `infra/` vs `infrastructure/` merge
- [ ] Más tests (>40%)
- [ ] 2 vulns openjarvis-tools

## Cómo compilar
```bash
# Desktop (rápido, no necesita synapsis wrapper)
cd umbra && cargo build --release --bin umbra-gui

# Server (necesita synapsis wrapper compilando)
cd umbra && cargo build --release --bin umbra --features server
```
