# UMBRA — Pendientes

## Estado actual (todo compila, 28 tests)
- `umbra` desktop: 0 errors, 2 warnings, 17 tests ✓
- `umbra` server: `cargo build --release --bin umbra --features server` ✓
- `synapsis-core`: 0 errors, 13 warnings, 11 tests ✓ (token-efficient memory)
- `synapsis` wrapper: NO compila (usar `synapsis-core` directamente)

## Urgente
- [ ] **synapsis wrapper**: arreglar los ~42 errores restantes (la UI de opencode separada para esto)
- [ ] **Warnings en synapsis-core**: 13 warnings (unused imports). Limpiar.

## Funcionalidad
- [ ] **STT real**: conectar whisper.cpp al voice input del desktop
- [ ] **Market data**: conectar broker API al panel de trading
- [ ] **Responsive UI**: manejar resize de ventana correctamente
- [ ] **Voice clone**: implementar `CloneVoiceUseCase`
- [ ] **Agent orchestration**: lógica real de agentes

## Deuda técnica
- [ ] `desktop/mod.rs` — 1467 líneas, partitioning
- [ ] `infra/` vs `infrastructure/` — merge
- [ ] Más tests (>40% cobertura)
- [ ] 2 vulns en openjarvis-tools

## Cómo compilar
```bash
# Desktop (rápido, no necesita synapsis wrapper)
cd umbra && cargo build --release --bin umbra-gui

# Server (necesita synapsis wrapper compilando)
cd umbra && cargo build --release --bin umbra --features server
```
