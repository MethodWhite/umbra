use anyhow::Result;
use wasmtime::{Engine, Linker, Module, Store};

pub struct WasmSandbox {
    engine: Engine,
}

impl WasmSandbox {
    pub fn new() -> Self {
        Self {
            engine: Engine::default(),
        }
    }

    pub fn execute(&self, code: &[u8]) -> Result<String> {
        if code.is_empty() {
            anyhow::bail!("WASM: no module provided (empty bytecode)");
        }

        let module = Module::new(&self.engine, code)
            .map_err(|e| anyhow::anyhow!("WASM compilation failed: {}", e))?;

        tracing::info!(
            "WASM: compiled {} bytes, {} imports, {} exports",
            code.len(),
            module.imports().count(),
            module.exports().count()
        );

        let mut store = Store::new(&self.engine, ());
        let linker = Linker::new(&self.engine);
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| anyhow::anyhow!("WASM instantiation failed: {}", e))?;

        // Try to call main or _start export
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
