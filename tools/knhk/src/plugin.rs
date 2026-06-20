use crate::receipt::Receipt;
use crate::{Law, LawError};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use wasmer::{Instance, Module, Store, Value, imports};

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
        let instance = Instance::new(&mut store, &module, &import_object).with_context(|| {
            format!("Failed to instantiate WASM module: {}", wasm_path.display())
        })?;

        let name = wasm_path
            .file_stem()
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
            Err(_) => {
                return Err(LawError {
                    law_name: self.name.clone(),
                    message: "WASM module does not export a 'validate' function".to_string(),
                });
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::receipt::Receipt;

    fn make_receipt(name: &str, passed: bool) -> Receipt {
        Receipt::new("test.wasm".into(), name, passed, if passed { "ok" } else { "fail" })
    }

    #[test]
    fn new_host_has_no_receipts() {
        let host = PluginHost::new();
        assert!(host.receipts().is_empty());
    }

    #[test]
    fn default_host_has_no_receipts() {
        let host = PluginHost::default();
        assert!(host.receipts().is_empty());
    }

    #[test]
    fn record_receipt_appends() {
        let mut host = PluginHost::new();
        host.record_receipt(make_receipt("LawA", true));
        host.record_receipt(make_receipt("LawB", false));
        assert_eq!(host.receipts().len(), 2);
        assert_eq!(host.receipts()[0].law_name, "LawA");
        assert_eq!(host.receipts()[1].law_name, "LawB");
    }

    #[test]
    fn receipts_preserves_pass_fail() {
        let mut host = PluginHost::new();
        host.record_receipt(make_receipt("Pass", true));
        host.record_receipt(make_receipt("Fail", false));
        assert!(host.receipts()[0].passed);
        assert!(!host.receipts()[1].passed);
    }

    #[test]
    fn load_law_returns_error_for_missing_file() {
        let mut host = PluginHost::new();
        let result = host.load_law(Path::new("/nonexistent/path/law.wasm"));
        assert!(result.is_err());
        let err = result.err().unwrap();
        let msg = format!("{err:#}");
        assert!(msg.contains("law.wasm"), "error should mention the path, got: {msg}");
    }

    #[test]
    fn load_law_returns_error_for_invalid_wasm() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), b"this is not wasm").unwrap();
        let mut host = PluginHost::new();
        let result = host.load_law(tmp.path());
        assert!(result.is_err());
    }
}
