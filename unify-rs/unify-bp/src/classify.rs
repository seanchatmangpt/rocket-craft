//! Blueprint noun-verb commands compatible with unify-cli's Classify pattern.

use crate::codegen::{BlueprintCodegen, BlueprintSpec};
use crate::gate::BlueprintAdmissionGate;

/// Trait modelling a noun-verb command understood by unify-cli.
pub trait Classify {
    fn noun(&self) -> &str;
    fn verb(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
}

// ---------------------------------------------------------------------------
// BlueprintGenerateCmd
// ---------------------------------------------------------------------------

/// Generate a UE4 T3D Blueprint from a JSON-encoded [`BlueprintSpec`].
pub struct BlueprintGenerateCmd;

impl Classify for BlueprintGenerateCmd {
    fn noun(&self) -> &str { "blueprint" }
    fn verb(&self) -> &str { "generate" }
    fn description(&self) -> &str { "Generate a Blueprint from a JSON spec" }

    fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
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
    fn noun(&self) -> &str { "blueprint" }
    fn verb(&self) -> &str { "validate" }
    fn description(&self) -> &str { "Validate a Blueprint spec against admission rules" }

    fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
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
