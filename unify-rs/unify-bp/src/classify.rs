//! Blueprint noun-verb commands compatible with unify-cli's Classify pattern.

use crate::codegen::{BlueprintCodegen, BlueprintSpec};
use crate::gate::BlueprintAdmissionGate;

/// Trait modelling a noun-verb command understood by unify-cli.
pub trait Classify {
    fn noun(&self) -> &str;
    fn verb(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(
        &self,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
}

// ---------------------------------------------------------------------------
// BlueprintGenerateCmd
// ---------------------------------------------------------------------------

/// Generate a UE4 T3D Blueprint from a JSON-encoded [`BlueprintSpec`].
pub struct BlueprintGenerateCmd;

impl Classify for BlueprintGenerateCmd {
    fn noun(&self) -> &str {
        "blueprint"
    }
    fn verb(&self) -> &str {
        "generate"
    }
    fn description(&self) -> &str {
        "Generate a Blueprint from a JSON spec"
    }

    fn execute(
        &self,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let spec: BlueprintSpec = serde_json::from_value(input)?;
        let t3d = BlueprintCodegen::to_t3d(&spec);
        Ok(serde_json::json!({ "t3d": t3d }))
    }
}

// ---------------------------------------------------------------------------
// BlueprintValidateCmd
// ---------------------------------------------------------------------------

/// Validate a Blueprint spec and report admission violations.
pub struct BlueprintValidateCmd;

impl Classify for BlueprintValidateCmd {
    fn noun(&self) -> &str {
        "blueprint"
    }
    fn verb(&self) -> &str {
        "validate"
    }
    fn description(&self) -> &str {
        "Validate a Blueprint spec against admission rules"
    }

    fn execute(
        &self,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let spec: BlueprintSpec = serde_json::from_value(input)?;
        let bp = BlueprintCodegen::from_spec(&spec);
        match BlueprintAdmissionGate::validate(&bp) {
            Ok(_) => Ok(serde_json::json!({ "valid": true, "violations": [] })),
            Err(violations) => Ok(serde_json::json!({
                "valid": false,
                "violations": violations,
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid_spec_json() -> serde_json::Value {
        json!({
            "name": "Hero",
            "parent_class": "Actor",
            "events": ["begin_play"],
            "variables": [],
            "print_on_begin": null
        })
    }

    // ── BlueprintGenerateCmd ──────────────────────────────────────────────────

    #[test]
    fn generate_noun_and_verb() {
        let cmd = BlueprintGenerateCmd;
        assert_eq!(cmd.noun(), "blueprint");
        assert_eq!(cmd.verb(), "generate");
    }

    #[test]
    fn generate_execute_returns_t3d_key() {
        let cmd = BlueprintGenerateCmd;
        let result = cmd.execute(valid_spec_json()).unwrap();
        assert!(result["t3d"].is_string());
    }

    #[test]
    fn generate_execute_errors_on_invalid_input() {
        let cmd = BlueprintGenerateCmd;
        assert!(cmd.execute(json!({"bad": "data"})).is_err());
    }

    // ── BlueprintValidateCmd ──────────────────────────────────────────────────

    #[test]
    fn validate_noun_and_verb() {
        let cmd = BlueprintValidateCmd;
        assert_eq!(cmd.noun(), "blueprint");
        assert_eq!(cmd.verb(), "validate");
    }

    #[test]
    fn validate_execute_passes_on_valid_spec() {
        let cmd = BlueprintValidateCmd;
        let result = cmd.execute(valid_spec_json()).unwrap();
        assert_eq!(result["valid"], true);
    }

    #[test]
    fn validate_execute_fails_on_empty_name() {
        let cmd = BlueprintValidateCmd;
        let input = json!({
            "name": "",
            "parent_class": "Actor",
            "events": [],
            "variables": [],
            "print_on_begin": null
        });
        let result = cmd.execute(input).unwrap();
        assert_eq!(result["valid"], false);
        assert!(result["violations"].as_array().unwrap().len() > 0);
    }
}
