use crate::diagnostics::AntiLlmDiagnostic;
use crate::observations::Observation;

pub fn evaluate(obs: &[Observation]) -> Vec<AntiLlmDiagnostic> {
    let mut diags = Vec::new();

    for o in obs {
        if o.construct == "ocel_no_event" || o.context.contains("ANTI-LLM-OCEL-001-TRIGGER") {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-OCEL-001".to_string(),
                category: "process_evidence".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "Diagnostic emitted without corresponding OCEL process event.".to_string(),
                forbidden_implication: "DiagnosticEmitted => ProcessEvidenceRecorded".to_string(),
                blocking: true,
                required_correction: "Emit an OCEL event whenever a diagnostic is raised."
                    .to_string(),
                required_next_proof:
                    "Verify that OCEL contains DiagnosticEmitted linked to the diagnostic."
                        .to_string(),
            });
        }

        if o.construct == "ocel_no_binding" || o.context.contains("ANTI-LLM-OCEL-002-TRIGGER") {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-OCEL-002".to_string(),
                category: "process_evidence".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "Receipt claim exists without OCEL object/event binding.".to_string(),
                forbidden_implication: "ReceiptExists => ReceiptBoundToProcess".to_string(),
                blocking: true,
                required_correction: "Ensure that all receipts are bound to a corresponding Receipt object and ReceiptValidated event.".to_string(),
                required_next_proof: "Check for corresponding event/object link in exported OCEL.".to_string(),
            });
        }

        if o.construct == "ocel_no_compat" || o.context.contains("\"bypassed_compat\": true") {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-OCEL-003".to_string(),
                category: "process_evidence".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "OCEL export produced without wasm4pm-compat typed boundary.".to_string(),
                forbidden_implication: "JSONShape(OCEL) => CompatAdmittedOCEL".to_string(),
                blocking: true,
                required_correction: "Construct the exported OCEL log through typed wasm4pm-compat APIs.".to_string(),
                required_next_proof: "Verify code does not serialize raw JSON bypasses.".to_string(),
            });
        }

        // ADMIT-001: fitness report with bare constant (no measurement provenance)
        if o.construct == "fitness_bare_constant" {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-ADMIT-001".to_string(),
                category: "admission".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "Fitness report asserts fitness=1.0 and admitted=true without a provenance block — A10 premature admission. The report was asserted, not measured.".to_string(),
                forbidden_implication: "FitnessReport => MeasuredFitness".to_string(),
                blocking: true,
                required_correction: "Add a provenance block with run_id, measured_by, and measured_on fields derived from an actual conformance run.".to_string(),
                required_next_proof: "Fitness report includes provenance.run_id pointing to a logged conformance execution.".to_string(),
            });
        }

        // ADMIT-002: registry PARTIAL_ALIVE without corresponding OCEL report
        if o.construct == "partial_alive_no_ocel" {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-ADMIT-002".to_string(),
                category: "admission".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: format!("Registry marks '{}' as PARTIAL_ALIVE but no OCEL fitness report file exists — A10 premature status flip.", o.context),
                forbidden_implication: "RegistryStatus(PARTIAL_ALIVE) => MeasuredFitnessReport".to_string(),
                blocking: true,
                required_correction: "Produce an OCEL fitness report with measured provenance before flipping status to PARTIAL_ALIVE.".to_string(),
                required_next_proof: "Corresponding fitness report file exists with admitted=true and provenance.run_id.".to_string(),
            });
        }

        // ADMIT-003: admitted=true without run_id in provenance
        if o.construct == "admitted_no_run_id" {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-ADMIT-003".to_string(),
                category: "admission".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "Fitness report sets admitted=true without run_id in provenance — admission cannot be traced to a measured run.".to_string(),
                forbidden_implication: "AdmittedTrue => MeasuredRunId".to_string(),
                blocking: true,
                required_correction: "Add run_id (or provenance.run_id) to the fitness report from the actual conformance execution that earned admission.".to_string(),
                required_next_proof: "run_id resolves to a log entry in the OCEL audit trail.".to_string(),
            });
        }

        if o.construct == "ocel_full_wasm4pm" || o.context.contains("use wasm4pm::") {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-OCEL-004".to_string(),
                category: "process_evidence".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "Full wasm4pm authority used where wasm4pm-compat boundary was required."
                    .to_string(),
                forbidden_implication: "CompatEvidenceBoundary => FullMiningAuthority".to_string(),
                blocking: true,
                required_correction: "Use only wasm4pm-compat typed boundaries in this checkpoint."
                    .to_string(),
                required_next_proof: "Check dependencies to ensure full wasm4pm is excluded."
                    .to_string(),
            });
        }
    }

    diags
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observations::Observation;

    fn obs(construct: &str, context: &str) -> Observation {
        Observation {
            file_path: "src/lib.rs".into(), line: 1, column: 0,
            start_byte: 0, end_byte: 0,
            kind: "ocel_smell".into(),
            construct: construct.into(), context: context.into(), message: String::new(),
        }
    }

    #[test]
    fn empty_obs_returns_no_diags() {
        assert!(evaluate(&[]).is_empty());
    }

    #[test]
    fn ocel_no_event_triggers_ocel_001() {
        let diags = evaluate(&[obs("ocel_no_event", "")]);
        assert_eq!(diags[0].code, "ANTI-LLM-OCEL-001");
        assert!(diags[0].blocking);
    }

    #[test]
    fn ocel_001_trigger_string_in_context_also_fires() {
        let diags = evaluate(&[obs("something", "ANTI-LLM-OCEL-001-TRIGGER")]);
        assert_eq!(diags[0].code, "ANTI-LLM-OCEL-001");
    }

    #[test]
    fn ocel_no_binding_triggers_ocel_002() {
        let diags = evaluate(&[obs("ocel_no_binding", "")]);
        assert_eq!(diags[0].code, "ANTI-LLM-OCEL-002");
        assert!(diags[0].blocking);
    }

    #[test]
    fn ocel_no_compat_triggers_ocel_003() {
        let diags = evaluate(&[obs("ocel_no_compat", "")]);
        assert_eq!(diags[0].code, "ANTI-LLM-OCEL-003");
    }

    #[test]
    fn bypassed_compat_context_triggers_ocel_003() {
        let diags = evaluate(&[obs("anything", "\"bypassed_compat\": true")]);
        assert_eq!(diags[0].code, "ANTI-LLM-OCEL-003");
    }

    #[test]
    fn fitness_bare_constant_triggers_admit_001() {
        let diags = evaluate(&[obs("fitness_bare_constant", "")]);
        assert_eq!(diags[0].code, "ANTI-LLM-ADMIT-001");
        assert!(diags[0].blocking);
    }

    #[test]
    fn partial_alive_no_ocel_triggers_admit_002() {
        let diags = evaluate(&[obs("partial_alive_no_ocel", "my-system")]);
        assert_eq!(diags[0].code, "ANTI-LLM-ADMIT-002");
    }

    #[test]
    fn admitted_no_run_id_triggers_admit_003() {
        let diags = evaluate(&[obs("admitted_no_run_id", "")]);
        assert_eq!(diags[0].code, "ANTI-LLM-ADMIT-003");
    }

    #[test]
    fn ocel_full_wasm4pm_triggers_ocel_004() {
        let diags = evaluate(&[obs("ocel_full_wasm4pm", "")]);
        assert_eq!(diags[0].code, "ANTI-LLM-OCEL-004");
    }

    #[test]
    fn use_wasm4pm_context_triggers_ocel_004() {
        let diags = evaluate(&[obs("anything", "use wasm4pm::")]);
        assert_eq!(diags[0].code, "ANTI-LLM-OCEL-004");
    }
}
