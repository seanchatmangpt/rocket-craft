//! # Unify Blueprint (unify-bp) Crate
//!
//! Provides bridges between the unify schema/TDD ecosystems and the blueprint core representation.
//! Includes admissions, classification, code generation, PWA template exporting, and cryptographic receipt chain.

pub mod classify;
pub mod codegen;
pub mod gate;
pub mod ocel;
pub mod pwa_export;
pub mod receipt;
pub use pwa_export::{BlueprintPwaExporter, BlueprintPwaMetadata, PwaBundle};

#[cfg(test)]
mod tests {
    use crate::classify::{BlueprintGenerateCmd, BlueprintValidateCmd, Classify};
    use crate::codegen::{BlueprintCodegen, BlueprintSpec, VarSpec};
    use crate::gate::BlueprintAdmissionGate;
    use crate::ocel::BlueprintOcelBridge;
    use crate::receipt::BlueprintReceiptChain;
    use blueprint_core::Blueprint;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn valid_bp() -> Blueprint {
        Blueprint::new("TestActor", "Actor")
    }

    fn empty_name_bp() -> Blueprint {
        Blueprint::new("", "Actor")
    }

    fn simple_spec() -> BlueprintSpec {
        BlueprintSpec {
            name: "MyActor".to_string(),
            parent_class: "Actor".to_string(),
            events: vec!["begin_play".to_string()],
            variables: vec![VarSpec {
                name: "Health".to_string(),
                var_type: "int".to_string(),
                default: "100".to_string(),
            }],
            print_on_begin: Some("Hello!".to_string()),
        }
    }

    // -----------------------------------------------------------------------
    // BlueprintAdmissionGate tests
    // -----------------------------------------------------------------------

    #[test]
    fn gate_new_is_open() {
        let gate = BlueprintAdmissionGate::new();
        assert!(gate.is_open());
    }

    #[test]
    fn gate_admit_passes_on_valid_blueprint() {
        let mut gate = BlueprintAdmissionGate::new();
        let bp = valid_bp();
        assert!(gate.admit(&bp).is_ok());
        assert!(gate.is_open());
    }

    #[test]
    fn gate_admit_fails_on_empty_name() {
        let mut gate = BlueprintAdmissionGate::new();
        let bp = empty_name_bp();
        let result = gate.admit(&bp);
        assert!(result.is_err());
        assert!(!gate.is_open());
        let violations = result.unwrap_err();
        assert!(!violations.is_empty());
    }

    #[test]
    fn gate_validate_returns_ok_for_valid() {
        let bp = valid_bp();
        assert!(BlueprintAdmissionGate::validate(&bp).is_ok());
    }

    #[test]
    fn gate_validate_returns_err_for_invalid() {
        let bp = empty_name_bp();
        assert!(BlueprintAdmissionGate::validate(&bp).is_err());
    }

    // -----------------------------------------------------------------------
    // BlueprintReceiptChain tests
    // -----------------------------------------------------------------------

    #[test]
    fn receipt_chain_new_is_empty() {
        let chain = BlueprintReceiptChain::new();
        assert_eq!(chain.chain().len(), 0);
        assert!(!chain.is_valid());
    }

    #[test]
    fn receipt_chain_record_generation_appends() {
        let mut chain = BlueprintReceiptChain::new();
        let bp = valid_bp();
        let receipt = chain.record_generation(&bp);
        assert!(receipt.key.contains("blueprint.generate"));
        assert_eq!(chain.chain().len(), 1);
    }

    #[test]
    fn receipt_chain_record_validation_appends_with_label() {
        let mut chain = BlueprintReceiptChain::new();
        let bp = valid_bp();
        let receipt = chain.record_validation(&bp, true);
        assert!(receipt.key.contains("validation"));
        assert_eq!(chain.chain().len(), 1);
    }

    #[test]
    fn receipt_chain_is_valid_after_records() {
        let mut chain = BlueprintReceiptChain::new();
        let bp = valid_bp();
        chain.record_generation(&bp);
        assert!(chain.is_valid());
    }

    // -----------------------------------------------------------------------
    // BlueprintCodegen tests
    // -----------------------------------------------------------------------

    #[test]
    fn codegen_from_spec_creates_blueprint_with_correct_name() {
        let spec = simple_spec();
        let bp = BlueprintCodegen::from_spec(&spec);
        assert_eq!(bp.name, "MyActor");
    }

    #[test]
    fn codegen_to_t3d_produces_nonempty_string() {
        let spec = simple_spec();
        let t3d = BlueprintCodegen::to_t3d(&spec);
        assert!(!t3d.is_empty());
        assert!(t3d.contains("Begin Object"));
    }

    #[test]
    fn codegen_to_json_produces_valid_json() {
        let spec = simple_spec();
        let json = BlueprintCodegen::to_json(&spec).expect("JSON serialization failed");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("not valid JSON");
        assert_eq!(parsed["name"], "MyActor");
    }

    // -----------------------------------------------------------------------
    // BlueprintOcelBridge tests
    // -----------------------------------------------------------------------

    #[test]
    fn ocel_bridge_record_generation_increases_event_count() {
        let mut bridge = BlueprintOcelBridge::new();
        let bp = valid_bp();
        assert_eq!(bridge.event_count(), 0);
        bridge.record_generation(&bp);
        assert_eq!(bridge.event_count(), 1);
    }

    #[test]
    fn ocel_bridge_to_json_produces_valid_json() {
        let mut bridge = BlueprintOcelBridge::new();
        let bp = valid_bp();
        bridge.record_generation(&bp);
        bridge.record_validation(&bp, 0);
        let json = bridge.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("not valid JSON");
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 2);
    }

    // -----------------------------------------------------------------------
    // BlueprintGenerateCmd / BlueprintValidateCmd tests
    // -----------------------------------------------------------------------

    #[test]
    fn generate_cmd_noun_verb_description() {
        let cmd = BlueprintGenerateCmd;
        assert_eq!(cmd.noun(), "blueprint");
        assert_eq!(cmd.verb(), "generate");
        assert_eq!(cmd.description(), "Generate a Blueprint from a JSON spec");
    }

    #[test]
    fn validate_cmd_noun_verb() {
        let cmd = BlueprintValidateCmd;
        assert_eq!(cmd.noun(), "blueprint");
        assert_eq!(cmd.verb(), "validate");
    }

    #[test]
    fn generate_cmd_execute_returns_t3d() {
        let cmd = BlueprintGenerateCmd;
        let input = serde_json::json!({
            "name": "CmdActor",
            "parent_class": "Actor",
            "events": ["begin_play"],
            "variables": [],
            "print_on_begin": null
        });
        let result = cmd.execute(input).expect("execute failed");
        assert!(result["t3d"].is_string());
        assert!(!result["t3d"].as_str().unwrap().is_empty());
    }

    #[test]
    fn validate_cmd_execute_returns_valid_for_good_spec() {
        let cmd = BlueprintValidateCmd;
        let input = serde_json::json!({
            "name": "GoodActor",
            "parent_class": "Actor",
            "events": [],
            "variables": [],
            "print_on_begin": null
        });
        let result = cmd.execute(input).expect("execute failed");
        assert_eq!(result["valid"], true);
    }
}
