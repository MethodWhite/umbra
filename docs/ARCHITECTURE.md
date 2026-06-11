# UMBRA — Arquitectura del Sistema

**Versión:** 0.1.0 — Borrador Inicial
**Clasificación:** PRIVADO — Proyecto Oculto
**Fecha:** 2026-06-02

---

## 1. Visión General

**Umbra** es un sistema de agente AI autónomo para trading automatizado en MetaTrader 4. Combina tres tecnologías base en una arquitectura unificada:

| Componente | Origen | Función |
|---|---|---|
| **MATERIA** | Propietario (Rust) | Motor de inferencia JEPA, memoria Synapsis, ejecución WASM |
| **Hermes** | Nous Research (Python → Rust) | Loop de aprendizaje continuo, creación de skills, mensajería multiplataforma |
| **Thoth** | Aten Security | Seguridad runtime (<100ms), gobierno de credenciales, auditoría WORM |

### 1.1 Principios de Diseño

- **Local-first:** 100% operación offline, sin dependencia cloud
- **Seguridad por diseño:** PQC desde el silicio, zero-trust en cada llamada
- **Oculto:** Sin exposición pública, sin telemetría, sin firmas digitales identificables
- **Rendimiento:** Inferencia <500ms, enforcement <100ms, MT4 tick processing <1ms
- **Autónomo:** Auto-aprendizaje, auto-recuperación, auto-mejora

---

## 2. Arquitectura en Capas

```
┌──────────────────────────────────────────────────────────────────┐
│                     INTERFAZ EXTERNA                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────────┐  │
│  │   CLI    │  │   TUI    │  │  MT4 API │  │  Mesh Network  │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └───────┬────────┘  │
├───────┴──────────────┴──────────────┴──────────────────┴────────┤
│                       CAPA DE SEGURIDAD (THOTH)                 │
│  ┌─────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐ │
│  │ Runtime │  │ PQC      │  │ Zero-    │  │ Auditoría        │ │
│  │Enforce  │  │Crypto    │  │Trust Gate│  │ WORM             │ │
│  └────┬────┘  └────┬─────┘  └────┬─────┘  └────────┬─────────┘ │
├───────┴──────────────┴──────────────┴──────────────────┴────────┤
│                       CAPA DE CORE (MATERIA)                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────────┐  │
│  │ JEPA     │  │ Multi-   │  │ Synapsis │  │ WASM Sandbox   │  │
│  │Engine    │  │Backend   │  │Memory    │  │ Executor       │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └───────┬────────┘  │
├───────┴──────────────┴──────────────┴──────────────────┴────────┤
│                       CAPA DE APRENDIZAJE (HERMES)              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────────┐  │
│  │ Agent    │  │ Skill    │  │Training  │  │ Multi-Platform │  │
│  │Loop      │  │Manager   │  │Pipeline  │  │ Messaging      │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └───────┬────────┘  │
├───────┴──────────────┴──────────────┴──────────────────┴────────┤
│                   CAPA DE INFRAESTRUCTURA                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────────┐  │
│  │OpenVentus│  │Sora-Ghost│  │ Ghost    │  │ Backup/DR      │  │
│  │Hardening │  │Anonymity │  │Monitor   │  │ System         │  │
│  └──────────┘  └──────────┘  └──────────┘  └────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

### 2.1 Capa de Infraestructura

Provee la base del sistema operativo y red:

- **OpenVentus:** Hardening Linux (AppArmor, AIDE, UFW, Fail2Ban, auditd)
- **Sora-Ghost:** Anonimato multicapa (Tor, bridges PQC, firejail)
- **Ghost Monitor:** Watchdog de procesos, detección de anomalías
- **Backup/DR:** Respaldo cifrado del estado completo del sistema

### 2.2 Capa de Core (MATERIA)

El motor de inferencia y razonamiento:

- **JEPA Engine:** Arquitectura Joint Embedding Predictive Architecture — inferencia sin tokens, basada en embeddings
- **Multi-Backend:** Selección adaptativa del backend (llama.cpp → Ollama → Cloud)
- **Synapsis Memory:** Memoria persistente con embeddings semánticos y PQC
- **WASM Sandbox:** Ejecución segura de skills en contenedores WebAssembly

### 2.3 Capa de Aprendizaje (Hermes)

El loop autónomo de mejora continua:

- **Agent Loop:** Ciclo plan → act → observe → learn (8 iteraciones máximas)
- **Skill Manager:** Creación, versionado y descubrimiento automático de skills
- **Training Pipeline:** JEPA trainer, fine-tuning continuo, auto-etiquetado
- **Multi-Platform Messaging:** Gateway para Telegram, Discord, Signal, Email

### 2.4 Capa de Seguridad (Thoth)

Runtime enforcement y gobierno:

- **Runtime Enforce:** Validación de cada tool call en <100ms
- **PQC Crypto:** Kyber-512 (KEM) + Dilithium-4 (firmas) desde implementación propia
- **Zero-Trust Gate:** Verificación de cada comando, análisis de riesgo AntiBrick
- **Auditoría WORM:** Logs inmutables con encadenamiento hash, evidencia forense

### 2.5 Interfaz Externa

Puntos de entrada al sistema:

- **CLI:** Interfaz de línea de comandos (mw-cli adaptado)
- **TUI:** Terminal UI con ratatui/crossterm
- **MT4 API:** Bridge MQL5 ↔ Rust vía FFI/C ABI
- **Mesh Network:** Comunicación P2P cifrada entre nodos Umbra

---

## 3. Flujo de Datos Principal

```
Usuario/Terminal
    │
    ▼
┌─────────────────────────────────────────────────┐
│              THOTH SECURITY GATE                │
│  • Valida identidad (TPM/MFA)                   │
│  • Clasifica riesgo                             │
│  • Enruta a sandbox si es necesario             │
└─────────────────────┬───────────────────────────┘
                      │ (aprobado)
                      ▼
┌─────────────────────────────────────────────────┐
│           MATERIA CORE ENGINE                   │
│  • Carga contexto de Synapsis Memory            │
│  • JEPA procesa entrada (embedding)             │
│  • Scheduler selecciona backend                 │
│  • Inference genera respuesta                   │
│  • Guarda en Synapsis Memory                    │
└─────────────────────┬───────────────────────────┘
                      │ (respuesta + acciones)
                      ▼
┌─────────────────────────────────────────────────┐
│           HERMES LEARNING LOOP                  │
│  • Si hay acción → ejecuta tool call            │
│  • Evalúa resultado                             │
│  • Si mejora posible → crea skill               │
│  • Si error → retroalimenta training            │
│  • Registra en audit log                        │
└─────────────────────┬───────────────────────────┘
                      │ (si trading)
                      ▼
┌─────────────────────────────────────────────────┐
│              MT4 BRIDGE                         │
│  • Convierte señal → orden MQL5                 │
│  • Firma con PQC                                │
│  • Ejecuta en sandbox de estrategia             │
│  • Reporta resultado                            │
└─────────────────────────────────────────────────┘
```

---

## 4. Componentes del Sistema

### 4.1 MATERIA Core (`engine/`)

| Módulo | Archivo | Función |
|---|---|---|
| JEPA Engine | `engine/jepa.rs` | Inferencia token-free basada en embeddings |
| Multi-Backend Router | `engine/router.rs` | Selección adaptativa llama.cpp/Ollama/Cloud |
| Synapsis Memory | `engine/memory.rs` | Memoria persistente con embeddings semánticos |
| Scheduler | `engine/scheduler.rs` | Detección hardware, selección de cuantización |
| WASM Runner | `engine/wasm.rs` | Ejecución sandboxeada de skills WebAssembly |
| SNN Classifier | `engine/snn.rs` | Clasificación rápida de intenciones (spiking) |

### 4.2 Seguridad (`security/`)

| Módulo | Archivo | Función |
|---|---|---|
| Runtime Enforcer | `security/enforcer.rs` | Validación de tool calls en <100ms |
| PQC Crypto | `security/pqc.rs` | Kyber-512 KEM + Dilithium-4 firmas |
| Zero-Trust Gate | `security/zt_gate.rs` | Análisis de riesgo por comando |
| AntiBrick | `security/antibrick.rs` | Prevención de operaciones destructivas |
| Audit WORM | `security/audit.rs` | Log inmutable encadenado hash |

### 4.3 Aprendizaje (`learning/`)

| Módulo | Archivo | Función |
|---|---|---|
| Agent Loop | `learning/agent_loop.rs` | Ciclo plan→act→observe→learn |
| Skill Manager | `learning/skills.rs` | Creación y versionado de skills |
| Training Pipeline | `learning/trainer.rs` | Fine-tuning JEPA continuo |
| Messaging | `learning/messaging.rs` | Gateway Telegram/Discord/Signal/Email |

### 4.4 Bridge MT4 (`bridge/`)

| Módulo | Archivo | Función |
|---|---|---|
| MQL5 FFI | `bridge/ffi.rs` | Interfaz C ABI para MQL5 |
| Signal Pipeline | `bridge/signals.rs` | Conversión señal → orden MT4 |
| Order Executor | `bridge/executor.rs` | Ejecución sandboxeada de órdenes |
| Strategy Sandbox | `bridge/sandbox.rs` | Aislamiento de estrategias de trading |

### 4.5 Infraestructura (`infra/`)

| Módulo | Archivo | Función |
|---|---|---|
| OpenVentus | `infra/hardening.rs` | Configuración de hardening |
| Ghost Monitor | `infra/ghost.rs` | Watchdog y detección de anomalías |
| Backup Engine | `infra/backup.rs` | Respaldo cifrado automatizado |

---

## 5. Seguridad

### 5.1 Post-Quantum Cryptography

```
Umbra PQC Stack:
┌─────────────────────────────────────────┐
│  Aplicación                             │
├─────────────────────────────────────────┤
│  Dilithium-4 (Firmas digitales)         │
├─────────────────────────────────────────┤
│  Kyber-512 (Intercambio de claves KEM)  │
├─────────────────────────────────────────┤
│  SHA3-256/512 (Hash)                    │
├─────────────────────────────────────────┤
│  ChaCha20-Poly1305 (Cifrado simétrico)  │
├─────────────────────────────────────────┤
│  AES-256-GCM (Cifrado legacy fallback)  │
└─────────────────────────────────────────┘
```

### 5.2 Zero-Trust Runtime

Toda tool call pasa por 4 verificaciones:

1. **Identidad:** ¿Quién llama? (TPM attestation + MFA)
2. **Permiso:** ¿Tiene permiso para esto?
3. **Riesgo:** ¿Es destructivo? (AntiBrick analysis)
4. **Contexto:** ¿Es coherente con la sesión actual?

Tiempo objetivo: <100ms por verificación.

### 5.3 WASM Sandbox

Las skills de terceros (o generadas por Hermes) se ejecutan en WebAssembly con:

- Aislamiento de memoria (wasmer)
- Sin acceso al sistema de archivos (por defecto)
- Límite de CPU por skill (quantum scheduler)
- API controlada por capability-based permissions

---

## 6. MT4 Integration

### 6.1 Bridge Architecture

```
┌─────────────────────────┐     ┌──────────────────────────┐
│      Umbra Rust         │     │     MetaTrader 4         │
│                         │     │                          │
│  signal_pipeline.rs ────┼─FFI─┼→ mql5_bridge.mq5        │
│       │                 │     │         │                │
│       ▼                 │     │         ▼                │
│  order_executor.rs      │     │  Expert Advisor (.ex4)   │
│       │                 │     │         │                │
│       ▼                 │     │         ▼                │
│  strategy_sandbox.rs    │     │  MT4 Terminal            │
└─────────────────────────┘     └──────────────────────────┘
```

### 6.2 MQL5 ↔ Rust FFI

```c
// Interfaz C ABI desde Rust
extern "C" {
    fn umbra_init(config: *const c_char) -> i32;
    fn umbra_analyze(signal: *const c_char) -> *const c_char;
    fn umbra_execute(order: *const c_char) -> i32;
    fn umbra_shutdown();
}
```

### 6.3 Security en Trading

- Cada orden firmada con Dilithium-4 antes de enviar a MT4
- Límite de pérdida configurable (AntiBrick para trading)
- Sandbox de estrategia: la estrategia no accede al saldo real
- Audit trail completo de cada orden

---

## 7. Roadmap — Fases

### Fase 1: Fundación (Semanas 1-4)
- [x] Documentación de arquitectura completa
- [ ] Migrar MATERIA engine a Umbra (prusia-core/src/materia/)
- [ ] Integrar Synapsis como memoria base
- [ ] Adaptar pqc-crypto como módulo de seguridad
- [ ] Setup del workspace Rust + CI privado

### Fase 2: Seguridad (Semanas 5-8)
- [ ] Implementar Thoth Runtime Enforcer
- [ ] Zero-Trust Gate funcional
- [ ] Auditoría WORM con encadenamiento hash
- [ ] WASM Sandbox operativo
- [ ] Pruebas de penetración internas

### Fase 3: Aprendizaje (Semanas 9-12)
- [ ] Portar Hermes Agent Loop a Rust
- [ ] Skill Manager + descubrimiento automático
- [ ] Training pipeline JEPA continuo
- [ ] Sistema de retroalimentación

### Fase 4: MT4 Bridge (Semanas 13-16)
- [ ] Implementar C ABI FFI
- [ ] Signal pipeline funcional
- [ ] Strategy sandbox
- [ ] Backtesting integrado
- [ ] Paper trading

### Fase 5: Infraestructura (Semanas 17-20)
- [ ] OpenVentus hardening automatizado
- [ ] Sora-Ghost anonimato
- [ ] Ghost monitoring
- [ ] Backup/DR cifrado
- [ ] Mesh networking P2P

### Fase 6: Producción (Semanas 21-24)
- [ ] Pruebas de carga
- [ ] Auditoría de seguridad externa
- [ ] Documentación de operación
- [ ] Despliegue en servidor dedicado

---

## 8. Stack Tecnológico

| Capa | Tecnología | Versión |
|---|---|---|
| Lenguaje | Rust | 2021 edition |
| Async | Tokio | 1.x |
| Inference | llama.cpp, Ollama | — |
| Memory | Synapsis (SQLite + PQC) | 0.1 |
| PQC | pqc-crypto (from-scratch) | 0.2 |
| WASM | wasmer | 4.x |
| UI | ratatui + crossterm | 0.29/0.28 |
| CLI | clap | 4.x |
| Serialization | serde + bincode | — |
| Crypto | pqcrypto-kyber, pqcrypto-dilithium | 0.8 |
| Hash | SHA3, blake3 | — |
| MT4 Bridge | C ABI FFI | — |

---

## 9. Glosario

| Término | Definición |
|---|---|
| **JEPA** | Joint Embedding Predictive Architecture — inferencia sin tokens |
| **PQC** | Post-Quantum Cryptography — criptografía resistente a computación cuántica |
| **KEM** | Key Encapsulation Mechanism — intercambio de claves PQC |
| **WASM** | WebAssembly — formato de instrucciones binarias sandboxeable |
| **SNN** | Spiking Neural Network — red neuronal de pulsos, baja latencia |
| **WORM** | Write Once Read Many — almacenamiento de auditoría inmutable |
| **FFI** | Foreign Function Interface — interfaz para llamar Rust desde otros lenguajes |
| **ABI** | Application Binary Interface — convención de llamada binaria |
