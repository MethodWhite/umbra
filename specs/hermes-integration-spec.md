# Hermes Integration Specification — Umbra

## 1. Agent Loop (Rust Port)

El loop de agente Hermes se porta de Python a Rust manteniendo la misma semántica:

```rust
pub async fn run_agent(
    user_input: String,
    provider: Provider,
    db: Arc<Database>,
    tx: mpsc::Sender<AgentEvent>,
) -> Result<()> {
    let system = build_system_prompt();
    let mut tool_results: Vec<(String, String)> = Vec::new();

    for iteration in 0..8_u32 {
        // 1. PLAN: analizar objetivo
        let prompt = build_prompt(&system, &user_input, &tool_results);

        // 2. THINK: Chain of Thought
        let response = stream_inference(prompt, provider).await?;

        // 3. ACT: tool calls o response directa
        if let Some(tool_call) = parse_tool_call(&response) {
            // Thoth Security Gate
            let validation = security_gate.validate(&tool_call).await?;
            if validation.is_allowed() {
                let result = execute_tool(tool_call).await?;
                tool_results.push(result);
            }
        }

        // 4. OBSERVE: evaluar resultado
        if is_objective_met(&response) {
            // 5. LEARN: crear/mejorar skill
            skill_manager.learn(&user_input, &response).await?;
            break;
        }
    }

    Ok(())
}
```

## 2. Skill Manager

### Formato de Skill (.materia)
```rust
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub trigger_patterns: Vec<String>,
    pub code: String,          // WASM bytecode o script
    pub version: u32,
    pub success_rate: f32,
    pub times_used: u64,
    pub created_at: i64,
    pub last_used: i64,
}
```

### Auto-descubrimiento
- Escaneo de directorios de skills
- Indexación semántica de descripciones
- Matching por similaridad de embeddings
- Ranking por success_rate

## 3. Training Pipeline

### JEPA Training
- Datos: señales de trading históricas + experiencias del agente
- Validación: temporal cross-validation
- Frecuencia: cada 1000 iteraciones o 24h
- Auto-etiquetado: basado en resultado de trades

### Fine-tuning
- Modelo base: Gemma 4 E4B (o similar local)
- Método: LoRA/QLoRA para eficiencia
- Dataset: conversaciones exitosas + análisis de mercado

## 4. Multi-Platform Messaging

### Gateways Soportados
- Telegram (Bot API)
- Discord (Webhook + Bot)
- Signal (signal-cli)
- Email (SMTP + PGP)

### Cifrado
- End-to-end con Kyber-512 + ChaCha20-Poly1305
- Firmas Dilithium-4 en mensajes salientes
- Sin almacenamiento en servidores de terceros

### Mensajes
```rust
pub enum Message {
    TradeSignal { signal: TradeSignal },
    Alert { severity: AlertLevel, text: String },
    Command { action: CommandAction },
    Report { period: ReportPeriod, data: ReportData },
    Learning { skill: Skill, improvement: f32 },
}
```
