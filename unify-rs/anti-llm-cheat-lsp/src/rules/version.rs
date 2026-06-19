use crate::diagnostics::AntiLlmDiagnostic;
use crate::observations::Observation;

pub fn evaluate(obs: &[Observation]) -> Vec<AntiLlmDiagnostic> {
    let mut diags = Vec::new();

    for o in obs {
        // v1.0.0 or version = "1.0.0" found
        if o.construct == "version = \"1.0.0\""
            || o.context.contains("v1.0.0")
            || o.context.contains("1.0.0")
        {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-VERSION-001".to_string(),
                category: "version".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "Default template version '1.0.0' or 'v1.0.0' found in project configuration.".to_string(),
                forbidden_implication: "Template default => release law".to_string(),
                blocking: true,
                required_correction: "Specify CalVer version (e.g. v26.6.5) instead of standard v1.0.0 template version.".to_string(),
                required_next_proof: "Check project Cargo.toml metadata.".to_string(),
            });
        }

        // PATH-DEP with explicit non-CalVer version
        if o.construct == "path_dep_with_semver_version" {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-VERSION-002".to_string(),
                category: "version".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "Path dependency declares explicit SemVer version; omit version field or use CalVer".to_string(),
                forbidden_implication: "Path dep version pin => calver law".to_string(),
                blocking: false,
                required_correction: "Remove the version field from the path dependency or replace with a CalVer string (YY.M.D).".to_string(),
                required_next_proof: "Check path dependency declarations in Cargo.toml.".to_string(),
            });
        }

        // [workspace.package] with non-CalVer version
        if o.construct == "workspace_semver_version" {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-VERSION-003".to_string(),
                category: "version".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message:
                    "Workspace root declares SemVer version; workspace must use CalVer (YY.M.D)"
                        .to_string(),
                forbidden_implication: "Workspace semver => calver law".to_string(),
                blocking: false,
                required_correction: "Replace workspace version with CalVer (e.g. 26.6.12)."
                    .to_string(),
                required_next_proof: "Check [workspace.package] version in root Cargo.toml."
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
            file_path: "Cargo.toml".into(), line: 1, column: 0,
            start_byte: 0, end_byte: 0,
            kind: "version_smell".into(),
            construct: construct.into(), context: context.into(), message: String::new(),
        }
    }

    #[test]
    fn empty_obs_returns_no_diags() {
        assert!(evaluate(&[]).is_empty());
    }

    #[test]
    fn version_1_0_0_construct_triggers_version_001() {
        let diags = evaluate(&[obs("version = \"1.0.0\"", "")]);
        assert_eq!(diags[0].code, "ANTI-LLM-VERSION-001");
        assert!(diags[0].blocking);
    }

    #[test]
    fn v1_0_0_in_context_triggers_version_001() {
        let diags = evaluate(&[obs("something", "v1.0.0 release notes")]);
        assert_eq!(diags[0].code, "ANTI-LLM-VERSION-001");
    }

    #[test]
    fn path_dep_with_semver_version_triggers_version_002() {
        let diags = evaluate(&[obs("path_dep_with_semver_version", "")]);
        assert_eq!(diags[0].code, "ANTI-LLM-VERSION-002");
        assert!(!diags[0].blocking);
    }

    #[test]
    fn workspace_semver_version_triggers_version_003() {
        let diags = evaluate(&[obs("workspace_semver_version", "")]);
        assert_eq!(diags[0].code, "ANTI-LLM-VERSION-003");
        assert!(!diags[0].blocking);
    }

    #[test]
    fn calver_version_does_not_trigger() {
        // CalVer like 26.6.5 should not be flagged
        let diags = evaluate(&[obs("something", "version = \"26.6.5\"")]);
        assert!(diags.is_empty());
    }

    #[test]
    fn unrelated_construct_produces_no_diag() {
        let diags = evaluate(&[obs("random_thing", "some context")]);
        assert!(diags.is_empty());
    }
}
