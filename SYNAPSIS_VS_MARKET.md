# Synapsis vs Competidores — Análisis Comparativo

## Competidores principales

| Proyecto | Stars | Lenguaje | Enfoque |
|----------|-------|----------|---------|
| **Honcho** | 5089 | Python | Memoria para agentes stateful |
| **OpenMemory** | 4225 | TypeScript | Memoria persistente local multi-backend |
| **Supermemory MCP** | 1699 | TypeScript | MCP universal para mem entre LLMs |
| **chromem-go** | 972 | Go | Vector DB embebible zero-deps |
| **TrueMemory** | 279 | Python | Captura y recall automático 100% local |

## Donde Synapsis SUPERA

1. **Eficiencia de tokens** — Summary automático, `max_tokens` budget, `importance` scoring. Nadie más lo hace.
2. **Contexto emocional** — Almacena `EmotionalState` + personalidad del agente en cada entrada. Búsqueda por similitud emocional (30% del score).
3. **Rendimiento nativo** — Rust puro, sin Python/Node. SQLite con WAL. 12MB binary vs >100MB de los otros.
4. **Arquitectura limpia** — Domain/Application/Infrastructure con traits. Feature-gating desktop vs server.
5. **Zero-deps runtime** — No necesita Python, Node, Docker. Solo el binary.
6. **Auto-evicción inteligente** — `retain(max_tokens)` elimina entradas de baja prioridad + LRU.

## Donde Synapsis está DETRÁS

| Feature | Competidor | Synapsis |
|---------|-----------|----------|
| **Búsqueda semántica** | OpenMemory (embeddings vectoriales) | ❌ Solo FTS textual |
| **Knowledge Graph** | Honcho (relaciones entre entidades) | ❌ Sin graph |
| **MCP Server** | Supermemory MCP (plug-and-play) | ⚠️ Comentado, roto |
| **Multi-backend** | OpenMemory (PostgreSQL, SQLite, etc.) | ❌ Solo SQLite |
| **Chunking pipeline** | TrueMemory (auto-chunk + resumen) | ⚠️ Summary manual |
| **Extracción de entidades** | Honcho (NER automático) | ❌ No existe |
| **Plugins/API** | Todos tienen npm/pip | ❌ Solo Rust crate |
| **Sync en tiempo real** | Honcho (WebSocket multi-agent) | ❌ Sin sync |
| **Cross-session merge** | OpenMemory (fusión automática) | ❌ Sesiones aisladas |

## Recomendación

**Prioridad alta:**
1. **MCP Server** — arreglar `synapsis-mcp` para que sea usable desde opencode
2. **Búsqueda semántica** — integrar embeddings vía Ollama (`nomic-embed-text`) o similar
3. **Chunking automático** — dividir contenido largo en chunks con resumen

**Prioridad media:**
4. **Knowledge Graph** — extraer entidades y relaciones de las observaciones
5. **Multi-backend** — soporte PostgreSQL además de SQLite

**Prioridad baja:**
6. **Plugins** — exponer API pública para extensiones
7. **Sync** — WebSocket para multi-agente en tiempo real
