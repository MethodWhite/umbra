# MT4 Bridge Specification — Umbra

## 1. C ABI Interface

```rust
// lib.rs — compiled as cdylib
#[no_mangle]
pub extern "C" fn umbra_init(config_json: *const c_char) -> i32 {
    // Inicializa el bridge, conecta con MT4
    // Retorna 0 = éxito, -1 = error
}

#[no_mangle]
pub extern "C" fn umbra_analyze(signal_json: *const c_char) -> *mut c_char {
    // Analiza señal de trading, retorna decisión
    // El caller debe liberar con umbra_free_string()
}

#[no_mangle]
pub extern "C" fn umbra_execute(order_json: *const c_char) -> i32 {
    // Ejecuta orden en MT4
    // Retorna 0 = éxito, código de error = fallo
}

#[no_mangle]
pub extern "C" fn umbra_shutdown() {
    // Cierre graceful del bridge
}

#[no_mangle]
pub extern "C" fn umbra_free_string(s: *mut c_char) {
    // Libera string retornado por umbra_analyze
}
```

## 2. Signal JSON Format (Umbra → MT4)

```json
{
  "signal_id": "sig_abc123",
  "timestamp": 1234567890,
  "symbol": "EURUSD",
  "action": "BUY|SELL|CLOSE|HOLD",
  "volume": 0.01,
  "confidence": 0.85,
  "reasoning": "Patrón detectado: doble suelo en H1 con RSI divergente",
  "risk_assessment": {
    "stop_loss": 1.1234,
    "take_profit": 1.1345,
    "max_risk_pct": 1.0
  },
  "signature": "dilithium4_firma_hex"
}
```

## 3. Order JSON Format (MT4 → Umbra)

```json
{
  "order_id": 12345678,
  "signal_id": "sig_abc123",
  "status": "EXECUTED|REJECTED|PENDING|CANCELLED",
  "execution_time_ms": 45,
  "filled_price": 1.1278,
  "filled_volume": 0.01,
  "error_code": 0,
  "error_message": "",
  "balance_impact": -1.23,
  "timestamp": 1234567895
}
```

## 4. MQL5 Expert Advisor Template

```mql5
// mql5_bridge.mq5
#import "umbra_bridge.dll"
  int umbra_init(string config);
  string umbra_analyze(string signal);
  int umbra_execute(string order);
  void umbra_shutdown();
#import

// EA principal
int OnInit() {
    return umbra_init("{\"mode\":\"paper\",\"max_risk\":1.0}");
}

void OnTick() {
    string signal = BuildSignal();
    string decision = umbra_analyze(signal);
    ProcessDecision(decision);
}
```

## 5. Security Requirements

- **PQC Signing:** Toda orden firmada con Dilithium-4 antes de enviar
- **Verificación:** Bridge verifica firma antes de ejecutar en MT4
- **Sandbox:** Modo paper trading como default
- **Kill Switch:** Comando de emergencia para cerrar todas las posiciones
- **Rate Limit:** Máximo N órdenes por minuto configurable
