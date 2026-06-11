// Zone 6 — Research/Stubs (server-gated)
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MateriaSubAgent {
    pub name: String,
    pub version: String,
    pub description: String,
    pub model: MateriaModelRef,
    pub capabilities: Vec<String>,
    pub config: HashMap<String, String>,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MateriaModelRef {
    pub name: String,
    pub familia: String,
    pub quant: Option<String>,
    pub context: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MateriaFile {
    pub materia: String,
    pub version: String,
    pub agent: MateriaSubAgent,
}

#[derive(Clone)]
pub struct SubAgentManager {
    pub agents_dir: PathBuf,
    pub agents: HashMap<String, MateriaSubAgent>,
}

impl SubAgentManager {
    pub fn new(agents_dir: PathBuf) -> Self {
        Self {
            agents_dir,
            agents: HashMap::new(),
        }
    }

    pub async fn scan(&mut self) -> Result<()> {
        if !self.agents_dir.exists() {
            std::fs::create_dir_all(&self.agents_dir)?;
            self.create_default_agents().await?;
        }

        for entry in std::fs::read_dir(&self.agents_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "materia") {
                match self.load_file(&path).await {
                    Ok(agent) => {
                        let name = agent.agent.name.clone();
                        tracing::info!("Sub-agente .materia cargado: {}", name);
                        self.agents.insert(name, agent.agent);
                    }
                    Err(e) => {
                        tracing::warn!("Error cargando {}: {}", path.display(), e);
                    }
                }
            }
        }
        Ok(())
    }

    async fn load_file(&self, path: &Path) -> Result<MateriaFile> {
        let content = tokio::fs::read_to_string(path).await?;
        let materia: MateriaFile = toml::from_str(&content)
            .or_else(|_| serde_json::from_str(&content).map_err(|e| anyhow!("Parse error: {}", e)))?;

        if materia.materia != "umbra_sub_agent" {
            return Err(anyhow!("Formato .materia inválido: '{}'", materia.materia));
        }

        Ok(materia)
    }

    pub fn get(&self, name: &str) -> Option<&MateriaSubAgent> {
        self.agents.get(name)
    }

    pub fn list(&self) -> Vec<&MateriaSubAgent> {
        self.agents.values().collect()
    }

    pub fn agent_for_capability(&self, capability: &str) -> Option<&MateriaSubAgent> {
        self.agents.values()
            .find(|a| a.capabilities.iter().any(|c| c.contains(capability)))
    }

    async fn create_default_agents(&self) -> Result<()> {
        let defaults = vec![
            (
                "trader.materia",
                r#"
materia = "umbra_sub_agent"
version = "1.0"

[agent]
name = "trader"
version = "1.0.0"
description = "Sub-agente especializado en análisis y ejecución de trading algorítmico en MT4"
capabilities = ["analisis_tecnico", "ejecucion_mt4", "gestion_riesgo", "backtesting"]
tools = ["analizar_mercado", "ejecutar_orden", "calcular_riesgo", "optimizar_estrategia"]

[agent.model]
name = "qwen2.5:7b"
familia = "ollama"
quant = "Q4_K_M"
context = 8192

[agent.config]
max_positions = "5"
daily_loss_limit = "5.0"
leverage = "1:30"
paper_trading = "true"
"#.trim(),
            ),
            (
                "analyst.materia",
                r#"
materia = "umbra_sub_agent"
version = "1.0"

[agent]
name = "analyst"
version = "1.0.0"
description = "Sub-agente de análisis de mercado e investigación en tiempo real"
capabilities = ["investigacion", "analisis_noticias", "analisis_onchain", "reportes"]
tools = ["investigar", "analizar_sentimiento", "generar_reporte"]

[agent.model]
name = "llama3.2:3b"
familia = "ollama"
quant = "Q8_0"
context = 4096

[agent.config]
max_sources = "10"
update_interval = "300"
"#.trim(),
            ),
            (
                "voice.materia",
                r#"
materia = "umbra_sub_agent"
version = "1.0"

[agent]
name = "voice"
version = "1.0.0"
description = "Sub-agente ligero de procesamiento de voz y audio. Usa whisper.cpp tiny + piper TTS"
capabilities = ["stt", "tts", "procesamiento_audio"]
tools = ["transcribir", "sintetizar_voz", "detectar_idioma"]

[agent.model]
name = "whisper:tiny"
familia = "whisper"
quant = "Q8_0"
context = 2048

[agent.config]
whisper_model = "tiny"
piper_voice = "en_US-lessac-medium"
sample_rate = "16000"
"#.trim(),
            ),
            (
                "monitor.materia",
                r#"
materia = "umbra_sub_agent"
version = "1.0"

[agent]
name = "monitor"
version = "1.0.0"
description = "Sub-agente de monitoreo de hardware, sistema y seguridad"
capabilities = ["monitoreo_hardware", "seguridad", "alertas", "auto_mantenimiento"]
tools = ["check_hardware", "analizar_logs", "limpiar_cache", "reportar_estado"]

[agent.model]
name = "tinyllama:latest"
familia = "ollama"
quant = "Q4_K_M"
context = 2048

[agent.config]
temp_threshold = "85"
check_interval = "60"
auto_cleanup = "true"
"#.trim(),
            ),
        ];

        for (filename, content) in defaults {
            let path = self.agents_dir.join(filename);
            if !path.exists() {
                tokio::fs::write(&path, content).await?;
                tracing::info!("Sub-agente default creado: {}", filename);
            }
        }

        Ok(())
    }
}
