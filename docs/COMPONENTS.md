# Module Map — Umbra

## Árbol del Proyecto

```
umbra/
├── README.md
├── docs/
│   ├── ARCHITECTURE.md          ← Documento principal de arquitectura
│   ├── COMPONENTS.md            ← Catálogo de componentes
│   └── ROADMAP.md               ← Plan de fases
│
├── diagrams/
│   ├── umbra-architecture.drawio    ← Arquitectura en capas
│   ├── umbra-data-flow.drawio       ← Flujo de datos del sistema
│   ├── umbra-security-layers.drawio ← Arquitectura de seguridad
│   ├── umbra-mt4-bridge.drawio      ← Bridge MT4 detallado
│   └── umbra-agent-loop.drawio      ← Ciclo de agente Hermes
│
├── specs/
│   ├── engine-spec.md               ← Especificación del core MATERIA
│   ├── security-spec.md             ← Especificación de seguridad
│   ├── mt4-bridge-spec.md           ← Especificación del bridge MT4
│   └── hermes-integration-spec.md   ← Especificación de Hermes
│
└── src/
    ├── engine/
    │   ├── jepa.rs                  ← JEPA inference engine
    │   ├── router.rs                ← Multi-backend model router
    │   ├── memory.rs                ← Synapsis memory integration
    │   ├── scheduler.rs             ← Adaptive hardware scheduler
    │   ├── wasm.rs                  ← WASM sandbox runner
    │   ├── snn.rs                   ← Spiking Neural Network
    │   └── mod.rs
    │
    ├── security/
    │   ├── enforcer.rs              ← Runtime enforcement (<100ms)
    │   ├── pqc.rs                   ← PQC crypto (Kyber/Dilithium)
    │   ├── zt_gate.rs               ← Zero-trust command gate
    │   ├── antibrick.rs             ← Anti-brick protection
    │   ├── audit.rs                 ← WORM audit logging
    │   └── mod.rs
    │
    ├── learning/
    │   ├── agent_loop.rs            ← Hermes agent loop
    │   ├── skills.rs                ← Skill manager
    │   ├── trainer.rs               ← JEPA training pipeline
    │   ├── messaging.rs             ← Multi-platform messaging
    │   └── mod.rs
    │
    ├── bridge/
    │   ├── ffi.rs                   ← C ABI for MQL5
    │   ├── signals.rs               ← Signal pipeline
    │   ├── executor.rs              ← Order executor
    │   ├── sandbox.rs               ← Strategy sandbox
    │   └── mod.rs
    │
    ├── infra/
    │   ├── hardening.rs             ← OpenVentus integration
    │   ├── ghost.rs                 ← Ghost monitor
    │   ├── backup.rs                ← Backup engine
    │   └── mod.rs
    │
    ├── lib.rs
    └── main.rs
```

## Dependencias entre Módulos

```
main.rs
  └── lib.rs
        ├── engine/
        │   ├── jepa.rs ←── router.rs
        │   ├── router.rs ←── scheduler.rs
        │   ├── memory.rs ←── Synapsis (external crate)
        │   ├── wasm.rs ←── wasmer (external crate)
        │   └── snn.rs (independiente)
        │
        ├── security/
        │   ├── enforcer.rs ←── zt_gate.rs + pqc.rs + antibrick.rs
        │   ├── pqc.rs (independiente, from-scratch)
        │   ├── zt_gate.rs ←── antibrick.rs
        │   ├── antibrick.rs (independiente)
        │   └── audit.rs (independiente, SQLite)
        │
        ├── learning/
        │   ├── agent_loop.rs ←── engine/jepa.rs + security/enforcer.rs + bridge/signals.rs
        │   ├── skills.rs ←── engine/memory.rs (almacenamiento)
        │   ├── trainer.rs ←── engine/jepa.rs (fine-tuning)
        │   └── messaging.rs (independiente, async HTTP)
        │
        ├── bridge/
        │   ├── ffi.rs ←── C ABI (punto de entrada MT4)
        │   ├── signals.rs ←── security/pqc.rs (firmas)
        │   ├── executor.rs ←── signals.rs + security/antibrick.rs
        │   └── sandbox.rs (independiente, simulación)
        │
        └── infra/
            ├── hardening.rs (independiente, scripts bash)
            ├── ghost.rs ←── notify (file watcher)
            └── backup.rs ←── security/pqc.rs (cifrado)

Leyenda:
  ←── : depende de
  (independiente): sin dependencias del proyecto
```

## Configuración (`umbra.toml`)

```toml
[umbra]
name = "umbra"
version = "0.1.0"
mode = "development"  # development | paper | live

[engine]
backend_priority = ["llamacpp", "ollama", "cloud"]
context_size = 8192
temperature = 0.7
max_iterations = 8

[security]
runtime_enforce_ms = 100
pqc_algorithm = "kyber512"
audit_retention_days = 90
worm_storage = "/var/umbra/audit"

[bridge]
mode = "paper"  # paper | live
ffi_library = "umbra_bridge.dll"
max_orders_per_minute = 10
default_risk_percent = 1.0

[learning]
auto_skill_creation = true
training_interval_hours = 24
messaging_platforms = ["telegram", "email"]
```
