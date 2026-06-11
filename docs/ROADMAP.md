# Roadmap — Umbra

## Fase 0: Fundación (Actual)
- [x] Arquitectura documentada
- [x] Diagramas Draw.io (5)
- [x] Especificaciones de componentes
- [ ] Workspace Rust (`cargo init --lib`)
- [ ] CI/CD privado local

## Fase 1: Engine Core (Semanas 1-4)
- [ ] Migrar MATERIA de prusia-core a umbra/src/engine/
- [ ] Integrar Synapsis como dependencia
- [ ] JEPA engine funcional
- [ ] Multi-backend router operativo
- [ ] Scheduler con detección hardware
- [ ] Pruebas unitarias del engine

## Fase 2: Seguridad (Semanas 5-8)
- [ ] Integrar pqc-crypto como módulo de seguridad
- [ ] Runtime enforcer funcional (<100ms)
- [ ] Zero-trust gate operativo
- [ ] AntiBrick protection
- [ ] Auditoría WORM con hash chaining
- [ ] WASM sandbox operativo
- [ ] Pruebas de penetración

## Fase 3: Aprendizaje (Semanas 9-12)
- [ ] Port del Hermes agent loop a Rust
- [ ] Sistema de tool calling
- [ ] Skill manager con auto-descubrimiento
- [ ] Training pipeline JEPA
- [ ] Messaging gateway (Telegram)

## Fase 4: MT4 Bridge (Semanas 13-16)
- [ ] C ABI FFI implementation
- [ ] Signal pipeline
- [ ] Order executor con risk check
- [ ] Strategy sandbox
- [ ] Backtesting integrado
- [ ] Paper trading funcional

## Fase 5: Infraestructura (Semanas 17-20)
- [ ] OpenVentus hardening scripts
- [ ] Ghost monitor + file watcher
- [ ] Backup engine cifrado
- [ ] Sora-Ghost anonimato opcional
- [ ] Mesh networking P2P

## Fase 6: Producción (Semanas 21-24)
- [ ] Live trading (riesgo controlado)
- [ ] Monitoreo continuo
- [ ] Optimización de rendimiento
- [ ] Documentación de operación
- [ ] Despliegue en servidor dedicado
