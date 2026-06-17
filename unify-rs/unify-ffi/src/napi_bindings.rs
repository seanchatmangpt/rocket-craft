//! Real napi-rs bindings — enabled with `--features napi`.
//!
//! When compiled without the `napi` feature, only the `*_raw` helper
//! functions are available; these delegate directly to `napi_shim` and are
//! used by unit tests and other Rust callers.
//!
//! When compiled with `--features napi`, the `#[napi]` attribute functions
//! are also emitted and become part of the cdylib `.node` binary that
//! Node.js can `require()`.

#[cfg(feature = "napi")]
use napi::bindgen_prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;

use crate::napi_shim::{unify_dispatch_ffi, unify_list_commands_ffi, unify_version_ffi};
use crate::types::FfiValue;

// ── napi-exported functions (only present with --features napi) ───────────

/// Dispatch a named command with a JSON-encoded input string.
/// Returns the result serialised as a JSON string.
#[cfg(feature = "napi")]
#[napi]
pub fn dispatch(name: String, input: String) -> napi::Result<String> {
    unify_dispatch_ffi(name, input)
        .map_err(|e| napi::Error::new(napi::Status::GenericFailure, e.to_string()))
}

/// Return the crate version string (e.g. `"0.1.0"`).
#[cfg(feature = "napi")]
#[napi]
pub fn version() -> napi::Result<String> {
    match unify_version_ffi() {
        Ok(val) => Ok(ffi_value_to_string(val)),
        Err(e) => Err(napi::Error::new(napi::Status::GenericFailure, e.to_string())),
    }
}

/// List all registered command names.
#[cfg(feature = "napi")]
#[napi]
pub fn list_commands() -> napi::Result<Vec<String>> {
    match unify_list_commands_ffi() {
        Ok(FfiValue::Array(arr)) => {
            let names: Vec<String> = arr
                .into_iter()
                .filter_map(|v| match v {
                    FfiValue::Str(s) => Some(s),
                    _ => None,
                })
                .collect();
            Ok(names)
        }
        Ok(other) => Ok(vec![ffi_value_to_string(other)]),
        Err(e) => Err(napi::Error::new(napi::Status::GenericFailure, e.to_string())),
    }
}

// ── Helper: convert any FfiValue to its most natural string form ──────────

fn ffi_value_to_string(v: FfiValue) -> String {
    match v {
        FfiValue::Str(s) => s,
        FfiValue::Null => "null".to_string(),
        FfiValue::Bool(b) => b.to_string(),
        FfiValue::Int(i) => i.to_string(),
        FfiValue::Float(f) => f.to_string(),
        other => other.to_json(),
    }
}

// ── Plain-Rust helpers (always compiled, no napi dependency) ─────────────

/// Dispatch a command by name with a JSON-encoded input.
/// Returns the JSON-encoded output, or an error string.
///
/// Usable from Rust unit tests without any napi toolchain.
pub fn dispatch_raw(name: &str, input: &str) -> Result<String, String> {
    unify_dispatch_ffi(name.to_string(), input.to_string()).map_err(|e| e.to_string())
}

/// Return the version string without going through napi.
pub fn version_raw() -> Result<String, String> {
    unify_version_ffi()
        .map(ffi_value_to_string)
        .map_err(|e| e.to_string())
}

/// Return all registered command names without going through napi.
pub fn list_commands_raw() -> Result<Vec<String>, String> {
    match unify_list_commands_ffi() {
        Ok(FfiValue::Array(arr)) => {
            let names: Vec<String> = arr
                .into_iter()
                .filter_map(|v| match v {
                    FfiValue::Str(s) => Some(s),
                    _ => None,
                })
                .collect();
            Ok(names)
        }
        Ok(other) => Ok(vec![ffi_value_to_string(other)]),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_raw_ping_returns_pong() {
        let result = dispatch_raw("ping", "null");
        assert!(result.is_ok(), "ping failed: {:?}", result.err());
        assert!(result.unwrap().contains("pong"));
    }

    #[test]
    fn dispatch_raw_echo_returns_input() {
        let result = dispatch_raw("echo", r#""hello""#);
        assert!(result.is_ok(), "echo failed: {:?}", result.err());
        assert!(result.unwrap().contains("hello"));
    }

    #[test]
    fn dispatch_raw_unknown_command_is_error() {
        let result = dispatch_raw("nonexistent_command_xyz", "null");
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(!msg.is_empty());
    }

    #[test]
    fn version_raw_returns_non_empty_string() {
        let v = version_raw().expect("version_raw should not fail");
        assert!(!v.is_empty(), "version must not be empty");
    }

    #[test]
    fn list_commands_raw_includes_builtins() {
        let cmds = list_commands_raw().expect("list_commands_raw should not fail");
        assert!(cmds.contains(&"ping".to_string()));
        assert!(cmds.contains(&"echo".to_string()));
        assert!(cmds.contains(&"version".to_string()));
    }
}
