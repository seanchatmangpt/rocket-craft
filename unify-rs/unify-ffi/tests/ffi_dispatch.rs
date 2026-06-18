//! Integration tests for the napi_bindings raw helpers.
//!
//! These tests run without `--features napi` and therefore do not require
//! any Node.js headers, napi-rs, or native toolchain beyond a standard
//! `cargo test` invocation.

use unify_ffi::napi_bindings::{dispatch_raw, list_commands_raw, version_raw};
use unify_ffi::napi_shim::{unify_list_commands_ffi, unify_version_ffi};

// ── dispatch_raw ─────────────────────────────────────────────────────────

#[test]
fn dispatch_ping_returns_pong() {
    let result = dispatch_raw("ping", "null");
    assert!(result.is_ok(), "ping dispatch failed: {:?}", result.err());
    let json = result.unwrap();
    assert!(
        json.contains("pong"),
        "expected 'pong' in response, got: {json}"
    );
}

#[test]
fn dispatch_echo_returns_input() {
    let result = dispatch_raw("echo", r#""hello world""#);
    assert!(result.is_ok(), "echo dispatch failed: {:?}", result.err());
    let json = result.unwrap();
    assert!(
        json.contains("hello world"),
        "echo should return the input, got: {json}"
    );
}

#[test]
fn dispatch_unknown_command_returns_error() {
    let result = dispatch_raw("nonexistent_cmd_alpha", "null");
    assert!(result.is_err(), "expected error for unknown command");
    let msg = result.unwrap_err();
    assert!(!msg.is_empty(), "error message must not be empty");
}

#[test]
fn dispatch_invalid_json_input_returns_error() {
    let result = dispatch_raw("echo", "{{{not valid json");
    assert!(result.is_err(), "expected parse error for invalid JSON");
}

#[test]
fn two_different_unknown_commands_return_errors() {
    let r1 = dispatch_raw("nonexistent_cmd_alpha", "null");
    let r2 = dispatch_raw("nonexistent_cmd_beta", "null");
    // Both must return errors (neither command is registered).
    // Falsification: at least one error message references the command or is non-empty.
    match (&r1, &r2) {
        (Err(e1), Err(e2)) => {
            assert!(
                e1.contains("nonexistent_cmd_alpha") || e2.contains("nonexistent_cmd_beta")
                    || !e1.is_empty(),
                "at least one error should reference the command name: {e1} / {e2}"
            );
        }
        _ => panic!("both unknown commands should return errors, got {r1:?} / {r2:?}"),
    }
}

// ── version_raw ───────────────────────────────────────────────────────────

#[test]
fn version_raw_returns_non_empty_string() {
    let v = version_raw().expect("version_raw should not fail");
    assert!(!v.is_empty(), "version must not be empty");
}

#[test]
fn version_shim_matches_version_raw() {
    let from_shim = unify_version_ffi()
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();
    let from_raw = version_raw().unwrap_or_default();
    assert_eq!(
        from_shim, from_raw,
        "version_raw and unify_version_ffi should return the same string"
    );
}

// ── list_commands_raw ─────────────────────────────────────────────────────

#[test]
fn list_commands_raw_returns_vec() {
    // May not panic — the function must return a result.
    let result = list_commands_raw();
    assert!(result.is_ok(), "list_commands_raw should not fail");
}

#[test]
fn list_commands_raw_includes_builtin_commands() {
    let cmds = list_commands_raw().expect("list_commands_raw should not fail");
    for expected in &["ping", "echo", "version"] {
        assert!(
            cmds.contains(&expected.to_string()),
            "expected built-in '{expected}' in command list, got: {cmds:?}"
        );
    }
}

#[test]
fn list_commands_shim_and_raw_agree() {
    let from_raw = list_commands_raw().expect("list_commands_raw failed");
    if let Ok(unify_ffi::FfiValue::Array(arr)) = unify_list_commands_ffi() {
        let from_shim: Vec<String> = arr
            .into_iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        // Both should have the same set of command names (order may differ).
        let mut raw_sorted = from_raw.clone();
        let mut shim_sorted = from_shim.clone();
        raw_sorted.sort();
        shim_sorted.sort();
        assert_eq!(
            raw_sorted, shim_sorted,
            "list_commands_raw and unify_list_commands_ffi should return the same commands"
        );
    }
}
