# Security Specification — Umbra

## 1. Principios de Seguridad

1. **Zero-Trust:** Ninguna entidad es confiable por defecto
2. **Defense in Depth:** Múltiples capas de seguridad independientes
3. **Least Privilege:** Cada componente solo tiene los permisos mínimos
4. **Fail Secure:** Si algo falla, que falle en estado seguro
5. **Quantum-Resistant:** Todo cifrado debe resistir ataques cuánticos

## 2. PQC Implementación

### Kyber-512 (KEM)
- Tamaño de clave pública: 800 bytes
- Tamaño de clave secreta: 1632 bytes
- Tamaño de ciphertext: 768 bytes
- Secreto compartido: 32 bytes
- Seguridad: NIST Level 1 (~AES-128)

### Dilithium-4 (Firmas)
- Tamaño de clave pública: 1312 bytes
- Tamaño de clave secreta: 2528 bytes
- Tamaño de firma: 2420 bytes
- Seguridad: NIST Level 3 (~AES-256)

### ChaCha20-Poly1305
- Cifrado simétrico principal
- AEAD (Autenticación + Cifrado)
- Nonce de 12 bytes

### SHA3-256/512
- Hash principal del sistema
- Encadenamiento de auditoría

## 3. Runtime Enforcement Flow

```
Tool Call Request
    ↓
┌─────────────────────┐
│ 1. Identity Check   │ ← TPM attestation + MFA token
│    ¿Quién llama?    │
└─────────┬───────────┘
          ↓ (passed)
┌─────────────────────┐
│ 2. Permission Check │ ← Capability matrix
│    ¿Tiene permiso?  │
└─────────┬───────────┘
          ↓ (passed)
┌─────────────────────┐
│ 3. Risk Analysis    │ ← AntiBrick engine
│    ¿Es destructivo? │
└─────────┬───────────┘
          ↓ (passed)
┌─────────────────────┐
│ 4. Context Check    │ ← Session coherence
│    ¿Es coherente?   │
└─────────┬───────────┘
          ↓ (passed)
    Tool Executes
```

Tiempo objetivo por verificación: <25ms
Tiempo total: <100ms

## 4. WASM Sandbox Security

- **Memoria:** Aislamiento total por instancia wasmer
- **Filesystem:** Sin acceso por defecto (capability-based)
- **CPU:** Límite por quantum scheduler
- **Red:** Sin acceso a network sockets
- **API:** Solo funciones explícitamente concedidas
- **Time:** Timeout configurable por skill

## 5. Auditoría WORM

### Formato del Log
```json
{
  "entry_id": "hex(sha3-256(prev_hash + timestamp + data))",
  "prev_hash": "sha3-256(entry_anterior)",
  "timestamp": 1234567890,
  "level": "INFO|WARN|ERROR|ALERT",
  "source": "engine|security|learning|bridge",
  "data": { ... },
  "signature": "dilithium4(entry_id + timestamp + data)"
}
```

### Almacenamiento
- Write Once Read Many (WORM)
- Rotación diaria con compresión y cifrado
- Retención mínima: 90 días
- Exportación forense en formato estándar

## 6. MT4 Trading Security

- **Order Signing:** Cada orden firmada con Dilithium-4
- **Loss Limit:** Límite de pérdida configurable (AntiBrick)
- **Strategy Isolation:** Sandbox sin acceso a saldo real
- **Audit Trail:** Registro completo de cada orden
- **Kill Switch:** Parada de emergencia de todas las operaciones
