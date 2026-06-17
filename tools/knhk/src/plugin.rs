use std::path::{Path, PathBuf};
use wasmer::{Instance, Module, Store, imports, Value};
use crate::{Law, LawError};
use anyhow::{Result, Context};
use std::sync::Mutex;
use crate::receipt::Receipt;

pub struct PluginHost {
    receipts: Vec<Receipt>,
}

impl Default for PluginHost {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginHost {
    pub fn new() -> Self {
        Self {
            receipts: Vec::new(),
        }
    }

    pub fn record_receipt(&mut self, receipt: Receipt) {
        self.receipts.push(receipt);
    }

    pub fn receipts(&self) -> &[Receipt] {
        &self.receipts
    }

    pub fn load_law(&mut self, wasm_path: &Path) -> Result<WasmLaw> {
        let mut store = Store::default();
        let wasm_bytes = std::fs::read(wasm_path)
            .with_context(|| format!("Failed to read WASM file: {}", wasm_path.display()))?;
        
        let module = Module::new(&store, wasm_bytes)
            .with_context(|| format!("Failed to compile WASM module: {}", wasm_path.display()))?;

        let import_object = imports! {};
        let instance = Instance::new(&mut store, &module, &import_object)
            .with_context(|| format!("Failed to instantiate WASM module: {}", wasm_path.display()))?;

        let name = wasm_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("UnknownWasmLaw")
            .to_string();

        Ok(WasmLaw {
            name,
            store: Mutex::new(store),
            instance,
            _path: wasm_path.to_path_buf(),
        })
    }

}

pub struct WasmLaw {
    name: String,
    store: Mutex<Store>,
    instance: Instance,
    _path: PathBuf,
}

impl Law for WasmLaw {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "WASM-based validation law"
    }

    fn validate(&self, _project_path: &Path) -> Result<(), LawError> {
        let validate_fn = match self.instance.exports.get_function("validate") {
            Ok(f) => f,
            Err(_) => return Err(LawError {
                law_name: self.name.clone(),
                message: "WASM module does not export a 'validate' function".to_string(),
            }),
        };

        let mut store = self.store.lock().unwrap();
        match validate_fn.call(&mut *store, &[]) {
            Ok(values) => {
                if let Some(Value::I32(result)) = values.first() {
                    if *result == 0 {
                        Ok(())
                    } else {
                        Err(LawError {
                            law_name: self.name.clone(),
                            message: format!("WASM validation failed with exit code: {}", result),
                        })
                    }
                } else {
                    Ok(()) // Assume success if no return value
                }
            }
            Err(e) => Err(LawError {
                law_name: self.name.clone(),
                message: format!("WASM execution error: {}", e),
            }),
        }
    }
}
