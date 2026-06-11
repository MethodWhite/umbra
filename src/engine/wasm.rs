// Zone 6 — Research/Stubs (server-gated)
use anyhow::Result;
use wasmtime::{Config, Engine, Linker, Module, Store};

const MAX_WASM_SIZE: usize = 1_048_576;

pub struct WasmSandbox {
    engine: Engine,
}

impl WasmSandbox {
    pub fn new() -> Self {
        let mut config = Config::default();
        config.wasm_reference_types(false);
        config.wasm_simd(false);
        config.wasm_bulk_memory(false);
        config.wasm_multi_value(false);
        let engine = Engine::new(&config).unwrap_or_else(|_| Engine::default());
        Self { engine }
    }

    pub fn execute(&self, code: &[u8]) -> Result<String> {
        if code.is_empty() {
            anyhow::bail!("WASM: no module provided (empty bytecode)");
        }
        if code.len() > MAX_WASM_SIZE {
            anyhow::bail!(
                "WASM: bytecode too large ({} bytes, max {})",
                code.len(),
                MAX_WASM_SIZE
            );
        }

        let module = Module::new(&self.engine, code)
            .map_err(|e| anyhow::anyhow!("WASM compilation failed: {}", e))?;

        tracing::info!(
            "WASM: compiled {} bytes, {} imports, {} exports",
            code.len(),
            module.imports().count(),
            module.exports().count()
        );

        if module.imports().count() > 0 {
            anyhow::bail!("WASM: module requires {} imports which are not allowed in sandbox", module.imports().count());
        }

        let mut store = Store::new(&self.engine, ());
        let linker = Linker::new(&self.engine);
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| anyhow::anyhow!("WASM instantiation failed: {}", e))?;

        for name in &["main", "_start"] {
            if let Ok(func) = instance.get_typed_func::<(), ()>(&mut store, name) {
                func.call(&mut store, ())
                    .map_err(|e| anyhow::anyhow!("WASM {}() failed: {}", name, e))?;
                tracing::info!("WASM: called {}() successfully", name);
                return Ok(format!("WASM function '{}' executed", name));
            }
        }

        Ok("WASM module instantiated (no entry point)".to_string())
    }
}
