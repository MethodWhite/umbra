use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::sync::mpsc;
use crate::engine::MateriaCore;
use crate::engine::SynapsisMemory;
use crate::security::SecurityGate;
use crate::persona::JarvisPersona;
use crate::memory::MemoryEngine;
use synapsis::ObservationType;

#[derive(Debug, Clone)]
pub enum AgentEvent {
    Token(String),
    Status(String),
    ToolCall { name: String, args: String },
    ToolResult { name: String, output: String, success: bool },
    Done,
    Error(String),
}

#[derive(Clone)]
pub struct UmbraAgent {
    pub engine: Arc<MateriaCore>,
    pub security: Arc<SecurityGate>,
    pub persona: JarvisPersona,
    pub memory: Arc<MemoryEngine>,
    pub synapsis: Arc<SynapsisMemory>,
    pub max_iterations: u32,
}

impl UmbraAgent {
    pub fn new(engine: Arc<MateriaCore>, security: Arc<SecurityGate>, persona: JarvisPersona) -> Self {
        Self {
            engine,
            security,
            persona,
            memory: Arc::new(MemoryEngine::new()),
            synapsis: Arc::new(SynapsisMemory::new()),
            max_iterations: 8,
        }
    }

    pub fn with_memory(mut self, memory: Arc<MemoryEngine>) -> Self {
        self.memory = memory;
        self
    }

    pub async fn run(
        &self,
        input: String,
        tx: mpsc::Sender<AgentEvent>,
    ) -> Result<String> {
        let system = self.persona.system_prompt();
        let mut tool_results: Vec<(String, String)> = Vec::new();
        let mut final_response = String::new();

        let _ = tx.send(AgentEvent::Status(
            format!("[Umbra] {} — Analizando solicitud", self.persona.greeting)
        )).await;

        let memory_context = self.memory.get_context(&input).await.unwrap_or_default();

        let synapsis_context = self.synapsis.recall(&input).await.unwrap_or_default();
        let combined_context = if !synapsis_context.is_empty() {
            format!("{}\n## Synapsis Memory\n{}\n", memory_context, synapsis_context.join("\n"))
        } else {
            memory_context.clone()
        };

        for iteration in 0..self.max_iterations {
            let _ = tx.send(AgentEvent::Status(
                format!("[Umbra] Iteración {}/{}", iteration + 1, self.max_iterations)
            )).await;

            // 1. PLAN: construir prompt con contexto, memoria y herramientas disponibles
            let prompt = self.build_prompt(&system, &input, &tool_results, &combined_context);

            // 2. THINK: Chain of Thought via MATERIA/JEPA
            let _ = tx.send(AgentEvent::Status("[Umbra] Procesando...".into())).await;

            let backend = self.engine.scheduler.select_primary(&self.engine.router);
            let result = self.engine.jepa.infer(prompt, backend).await
                .map_err(|e| anyhow!("[Umbra] Error de inferencia: {}", e))?;

            final_response = result.text.clone();

            let _ = tx.send(AgentEvent::Token(result.text.clone())).await;

            // 3. ACT: detectar y ejecutar tool calls (con validación Thoth)
            if let Some(tool_call) = self.parse_tool_call(&result.text) {
                let _ = tx.send(AgentEvent::ToolCall {
                    name: tool_call.name.clone(),
                    args: tool_call.args.clone(),
                }).await;

                let args = vec![tool_call.args.clone()];
                match self.security.validate_tool_call(&tool_call.name, &args).await {
                    Ok(true) => {
                        let output = self.execute_tool(&tool_call.name, &tool_call.args).await;
                        let _ = tx.send(AgentEvent::ToolResult {
                            name: tool_call.name,
                            output: output.clone(),
                            success: true,
                        }).await;
                        tool_results.push((tool_call.args, output));
                    }
                    Ok(false) | Err(_) => {
                        let _ = tx.send(AgentEvent::Status(
                            "[Thoth] ⛔ Tool call bloqueada por seguridad".into()
                        )).await;
                    }
                }
            }

            // 4. OBSERVE: el objetivo está completo?
            if self.is_objective_complete(&result.text) {
                let _ = tx.send(AgentEvent::Status("[Umbra] ✅ Objetivo completado".into())).await;
                // 5. LEARN: registrar experiencia en Synapsis
                self.learn(&input, &final_response).await?;
                self.synapsis.save(&input, &final_response).await?;
                break;
            }
        }

        let _ = tx.send(AgentEvent::Done).await;
        Ok(final_response)
    }

    fn build_prompt(&self, system: &str, input: &str, tool_results: &[(String, String)], memory_context: &str) -> String {
        let config = self.engine.scheduler.select_config(&self.engine.router);
        let mut prompt = format!(
            "{}\n\n## Contexto del Sistema\n\
            Modelo principal: {}\n\
            Modelo secundario: {}\n\
            VRAM: {} MB | RAM: {} MB\n\
            Cuantización: {}\n\n",
            system,
            config.primary.name(),
            config.secondary.name(),
            self.engine.scheduler.vram_mb,
            self.engine.scheduler.ram_mb,
            self.engine.scheduler.quant_level,
        );

        if !memory_context.is_empty() {
            prompt.push_str(memory_context);
            prompt.push_str("\n\n");
        }

        prompt.push_str(&format!("## Usuario\n{}\n\n", input));

        if !tool_results.is_empty() {
            prompt.push_str("## Resultados de herramientas ejecutadas\n");
            for (i, (args, result)) in tool_results.iter().enumerate() {
                prompt.push_str(&format!("[{i}] Args: {args}\n    Resultado: {result}\n\n"));
            }
        }

        prompt.push_str("## Respuesta\n");
        prompt
    }

    fn parse_tool_call(&self, response: &str) -> Option<ToolCall> {
        let start = response.find("<tool>")?;
        let after = &response[start + 6..];
        let end = after.find("</tool>")?;
        let json = after[..end].trim();
        serde_json::from_str::<ToolCall>(json).ok()
    }

    async fn execute_tool(&self, name: &str, args: &str) -> String {
        match name {
            "analizar_mercado" => {
                format!("[Simulación] Análisis de mercado para '{}' completado", args)
            }
            "ejecutar_orden" => {
                format!("[Simulación] Orden '{}' registrada para revisión", args)
            }
            "investigar" => {
                format!("[Simulación] Investigación sobre '{}' completada", args)
            }
            _ => {
                format!("[Umbra] Tool '{}' ejecutada (args: {})", name, args)
            }
        }
    }

    fn is_objective_complete(&self, response: &str) -> bool {
        !response.contains("<tool>")
    }

    async fn learn(&self, input: &str, response: &str) -> Result<()> {
        let title = if input.len() > 60 {
            format!("{}...", &input[..60])
        } else {
            input.to_string()
        };

        let content = format!(
            "Pregunta: {}\nRespuesta: {}",
            input,
            if response.len() > 500 { &response[..500] } else { response }
        );

        match self.memory.add_observation(
            ObservationType::Learning,
            title,
            content,
        ).await {
            Ok(_) => {
                tracing::debug!("[Synapsis] Experiencia registrada en memoria persistente");
            }
            Err(e) => {
                tracing::warn!("[Synapsis] Error guardando memoria: {}", e);
            }
        }

        Ok(())
    }
}

#[derive(Debug, serde::Deserialize)]
struct ToolCall {
    name: String,
    args: String,
}
