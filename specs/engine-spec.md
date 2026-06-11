# Especificación de Componentes — Umbra

## 1. Engine Core

### JEPA Engine (`umbra/src/engine/jepa.rs`)
- Inferencia token-free basada en Joint Embedding Predictive Architecture
- 80% weight en el razonamiento principal
- Context aggregation mediante attention mechanism
- Hardware-bound `.materia` micro-modules

### Multi-Backend Router (`umbra/src/engine/router.rs`)
- Prioridad: llama.cpp (GPU) → Ollama (CPU/GPU) → Cloud (fallback)
- Detección automática de modelos disponibles
- Ranking por VRAM disponible y cuantización

### Synapsis Memory (`umbra/src/engine/memory.rs`)
- Memoria persistente con embeddings semánticos (fastembed)
- Background indexer cada 60 segundos
- PQC encryption en reposo
- Sesiones con aislamiento criptográfico

### Scheduler (`umbra/src/engine/scheduler.rs`)
- Detección de VRAM (NVIDIA/AMD)
- Selección de cuantización: Q4KM → Q5KM → Q8_0 → F16
- Escalado de context window según RAM disponible
- Arbitraje de recursos entre módulos sensoriales

### WASM Runner (`umbra/src/engine/wasm.rs`)
- Sandbox basado en wasmer
- Ejecución aislada de skills generadas por Hermes
- API controlada por capability-based permissions
- Heartbeat y timeout por skill

### SNN Classifier (`umbra/src/engine/snn.rs`)
- LIF neurons para clasificación ultrarrápida de intenciones
- Microsegundos de latencia
- Ideal para routing temprano de consultas

---

## 2. Security

### Runtime Enforcer (`umbra/src/security/enforcer.rs`)
- Valida toda tool call en <100ms
- 4 verificaciones: identidad, permiso, riesgo, contexto
- Bloqueo inmediato si alguna falla

### PQC Crypto (`umbra/src/security/pqc.rs`)
- Implementación from-scratch (del proyecto pqc-crypto)
- Kyber-512 para KEM
- Dilithium-4 para firmas digitales
- ChaCha20-Poly1305 + AES-256-GCM para cifrado simétrico

### Zero-Trust Gate (`umbra/src/security/zt_gate.rs`)
- Análisis estático y dinámico de comandos
- Clasificación por riesgo: Safe → Low → Medium → High → Critical → Blocked
- Integración con AntiBrick para prevención de daño

### AntiBrick (`umbra/src/security/antibrick.rs`)
- Detecta operaciones destructivas (dd, mkfs, fdisk, flash)
- Análisis de contexto del comando
- Sugerencia de alternativas seguras vía LLM

### Audit WORM (`umbra/src/security/audit.rs`)
- Log inmutable con encadenamiento hash SHA3-256
- Almacenamiento WORM (Write Once Read Many)
- Exportación forense
- Alertas en tiempo real

---

## 3. Learning

### Agent Loop (`umbra/src/learning/agent_loop.rs`)
- Ciclo: Plan → Think (CoT) → Act → Observe → Learn
- Máximo 8 iteraciones
- Tool calling con parsing automático
- Eventos streaming a TUI

### Skill Manager (`umbra/src/learning/skills.rs`)
- Creación dinámica de skills basada en experiencia
- Versionado y descubrimiento automático
- Formato `.materia` para micro-módulos
- Catálogo con búsqueda semántica

### Training Pipeline (`umbra/src/learning/trainer.rs`)
- Fine-tuning continuo del modelo base
- Auto-etiquetado de experiencias exitosas
- JEPA trainer con datos de trading real
- Validación cruzada temporal

### Multi-Platform Messaging (`umbra/src/learning/messaging.rs`)
- Gateway para Telegram, Discord, Signal, Email
- Cifrado PQC en tránsito
- Cola de mensajes con prioridad
- Reconexión automática

---

## 4. MT4 Bridge

### MQL5 FFI (`umbra/src/bridge/ffi.rs`)
- Interfaz C ABI para MQL5
- Funciones: umbra_init, umbra_analyze, umbra_execute, umbra_shutdown
- Paso de parámetros JSON serializados
- Callbacks para eventos de MT4

### Signal Pipeline (`umbra/src/bridge/signals.rs`)
- Conversión de señales de trading a órdenes MT4
- Risk check con reglas AntiBrick
- Firma PQC (Dilithium-4) de cada orden

### Order Executor (`umbra/src/bridge/executor.rs`)
- Ejecución sandboxeada de órdenes
- Límite de pérdida configurable
- Confirmación bidireccional

### Strategy Sandbox (`umbra/src/bridge/sandbox.rs`)
- Aislamiento de estrategia: sin acceso a saldo real
- Simulación de ejecución con datos históricos
- Validación de consistencia

---

## 5. Infrastructure

### OpenVentus Hardening (`umbra/src/infra/hardening.rs`)
- 10 capas de hardening (kernel, SSH, UFW, Fail2Ban, AIDE, AppArmor, auditd)
- Verificación de integridad de archivos críticos
- Escaneo de puertos y procesos

### Ghost Monitor (`umbra/src/infra/ghost.rs`)
- Watchdog de procesos del sistema
- Detección de anomalías en tiempo real
- File watcher con notificaciones

### Backup Engine (`umbra/src/infra/backup.rs`)
- Respaldo cifrado (AES-256-GCM + Kyber-512)
- Automatización programada
- Verificación de integridad post-backup
- Rotación y limpieza
