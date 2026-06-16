//! No-op shims for the napi attributes when not compiling for Node.
//!
//! When the `napi` feature is active on Linux, these would be replaced by
//! real napi-rs macro-generated exports. For all other targets / configurations
//! they are plain Rust functions callable from tests and non-Node consumers.

use crate::convert::json_to_ffi;
use crate::registry::FfiCommandRegistry;
use crate::types::{FfiError, FfiResult, FfiValue};

/// Return the crate version as an FfiValue.
pub fn unify_version_ffi() -> FfiResult<FfiValue> {
    let registry = FfiCommandRegistry::new().with_builtins();
    registry.call("version", FfiValue::Null)
}

/// Dispatch a named command with a JSON-encoded input, returning a JSON string.
///
/// The `input_json` is parsed as a raw JSON value and converted to `FfiValue`
/// via `json_to_ffi` so that standard JSON types (null, numbers, strings, etc.)
/// map naturally to their `FfiValue` equivalents.
pub fn unify_dispatch_ffi(name: String, input_json: String) -> FfiResult<String> {
    let raw: serde_json::Value = serde_json::from_str(&input_json)
        .map_err(|e| FfiError::new("PARSE_ERROR", e.to_string()))?;
    let input = json_to_ffi(&raw);
    let registry = FfiCommandRegistry::new().with_builtins();
    let output = registry.call(&name, input)?;
    Ok(output.to_json())
}

/// List all registered commands as an FfiValue array of strings.
pub fn unify_list_commands_ffi() -> FfiResult<FfiValue> {
    let registry = FfiCommandRegistry::new().with_builtins();
    let names: Vec<FfiValue> = registry
        .list()
        .into_iter()
        .map(|s| FfiValue::Str(s.to_string()))
        .collect();
    Ok(FfiValue::Array(names))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unify_version_ffi_returns_ok() {
        let result = unify_version_ffi();
        assert!(result.is_ok());
        let v = result.unwrap();
        assert!(v.as_str().is_some(), "version should be a string");
    }

    #[test]
    fn test_unify_dispatch_ffi_echo_returns_input() {
        // Pass a JSON string value — echo returns it unchanged, serialized back as JSON
        let input_json = r#""hello world""#.to_string();
        let result = unify_dispatch_ffi("echo".to_string(), input_json);
        assert!(result.is_ok(), "echo dispatch failed: {:?}", result.err());
        let output = result.unwrap();
        // Output is JSON-encoded FfiValue::Str, so should contain "hello world"
        assert!(
            output.contains("hello world"),
            "echo should return the input: got {}",
            output
        );
    }

    #[test]
    fn test_unify_dispatch_ffi_ping() {
        let result = unify_dispatch_ffi("ping".to_string(), "null".to_string());
        assert!(result.is_ok(), "ping dispatch failed: {:?}", result.err());
        assert!(result.unwrap().contains("pong"));
    }

    #[test]
    fn test_unify_dispatch_ffi_unknown_command() {
        let result = unify_dispatch_ffi("no_such_cmd".to_string(), "null".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_unify_dispatch_ffi_invalid_json() {
        let result = unify_dispatch_ffi("echo".to_string(), "{{{".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_unify_list_commands_ffi_returns_array() {
        let result = unify_list_commands_ffi();
        assert!(result.is_ok());
        let v = result.unwrap();
        assert!(matches!(v, FfiValue::Array(_)));
        if let FfiValue::Array(arr) = v {
            // Should include at least the 3 builtins
            assert!(arr.len() >= 3);
        }
    }
}
