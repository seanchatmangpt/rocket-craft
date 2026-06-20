//! Crypto / keystore management commands

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde_json::Value;

fn do_crypto_generate() -> Result<Value> {
    rocket_sdk::crypto::generate_all_keystores()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    Ok(serde_json::json!({"status": "ok"}))
}

fn do_crypto_status() -> Result<Value> {
    rocket_sdk::crypto::check_status()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{}", e)))?;
    Ok(serde_json::json!({"status": "ok"}))
}

/// Generate all missing Android keystores
#[verb("generate", "crypto")]
fn generate_crypto() -> Result<Value> {
    do_crypto_generate()
}

/// Check status of Android keystores
#[verb("status", "crypto")]
fn status_crypto() -> Result<Value> {
    do_crypto_status()
}
