#[cfg(test)]
mod tests {
    use crate::{
        sections::{LspConfig, TestConfig, WorkspaceConfig},
        ConfigFormat, ConfigLoader, ConfigMerge, ManifestValidator, UnifyManifest,
    };

    // 1. default_for creates valid manifest with name set
    #[test]
    fn test_default_for_sets_name() {
        let m = UnifyManifest::default_for("my-project");
        assert_eq!(m.name, "my-project");
        assert_eq!(m.version, "0.1.0");
    }

    // 2. to_toml/from_toml round-trip
    #[test]
    fn test_toml_round_trip() {
        let original = UnifyManifest::default_for("toml-test");
        let toml_str = original.to_toml().expect("serialize to toml");
        let parsed = UnifyManifest::from_toml(&toml_str).expect("parse from toml");
        assert_eq!(parsed.name, "toml-test");
        assert_eq!(parsed.version, "0.1.0");
    }

    // 3. to_json/from_json round-trip
    #[test]
    fn test_json_round_trip() {
        let original = UnifyManifest::default_for("json-test");
        let json_str = original.to_json().expect("serialize to json");
        let parsed = UnifyManifest::from_json(&json_str).expect("parse from json");
        assert_eq!(parsed.name, "json-test");
        assert_eq!(parsed.version, "0.1.0");
    }

    // 4. merge: override's non-None fields win
    #[test]
    fn test_merge_override_non_none_fields_win() {
        let base = UnifyManifest::default_for("base-project");
        let mut overrides = UnifyManifest::default_for("override-project");
        overrides.test = Some(TestConfig {
            style: "chicago".to_string(),
            coverage_threshold: 0.8,
            snapshot_dir: "snapshots".to_string(),
            golden_dir: "golden".to_string(),
        });
        let merged = UnifyManifest::merge(base, overrides);
        assert_eq!(merged.name, "override-project");
        assert!(merged.test.is_some());
        assert_eq!(merged.test.unwrap().style, "chicago");
    }

    // 5. merge: base fields kept when override is None
    #[test]
    fn test_merge_base_fields_kept_when_override_none() {
        let mut base = UnifyManifest::default_for("base-project");
        base.test = Some(TestConfig {
            style: "london".to_string(),
            coverage_threshold: 0.9,
            snapshot_dir: "snap".to_string(),
            golden_dir: "gold".to_string(),
        });
        let overrides = UnifyManifest::default_for(""); // empty name, no test
        let merged = UnifyManifest::merge(base, overrides);
        // name: override is empty so base wins
        assert_eq!(merged.name, "base-project");
        // test: override is None so base wins
        assert!(merged.test.is_some());
        assert_eq!(merged.test.unwrap().style, "london");
    }

    // 6. ManifestValidator: is_valid true for default manifest with proper name/version
    #[test]
    fn test_validator_valid_for_default_manifest() {
        let m = UnifyManifest::default_for("my-project");
        assert!(ManifestValidator::is_valid(&m));
    }

    // 7. ManifestValidator: returns violation for empty name
    #[test]
    fn test_validator_violation_for_empty_name() {
        let m = UnifyManifest {
            name: "".to_string(),
            version: "0.1.0".to_string(),
            ..Default::default()
        };
        let violations = ManifestValidator::validate(&m);
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.field == "name"));
    }

    // 8. ManifestValidator: returns violation for coverage > 1.0
    #[test]
    fn test_validator_violation_for_coverage_out_of_range() {
        let m = UnifyManifest {
            name: "proj".to_string(),
            version: "0.1.0".to_string(),
            test: Some(TestConfig {
                style: "chicago".to_string(),
                coverage_threshold: 1.5,
                snapshot_dir: String::new(),
                golden_dir: String::new(),
            }),
            ..Default::default()
        };
        let violations = ManifestValidator::validate(&m);
        assert!(violations
            .iter()
            .any(|v| v.field == "test.coverage_threshold"));
    }

    // 9. ConfigLoader::from_str parses toml correctly
    #[test]
    fn test_loader_from_str_toml() {
        let toml = r#"
name = "toml-loader"
version = "1.0.1"

[workspace]
root = "/workspace"
members = []
default_target = "debug"
"#;
        let m = ConfigLoader::from_str(toml, ConfigFormat::Toml).expect("parse toml");
        assert_eq!(m.name, "toml-loader");
        assert_eq!(m.version, "1.0.1");
    }

    // 10. ConfigLoader::from_str parses json correctly
    #[test]
    fn test_loader_from_str_json() {
        let json = r#"{
  "name": "json-loader",
  "version": "2.0.0",
  "workspace": {
    "root": "/ws",
    "members": [],
    "default_target": "release"
  }
}"#;
        let m = ConfigLoader::from_str(json, ConfigFormat::Json).expect("parse json");
        assert_eq!(m.name, "json-loader");
        assert_eq!(m.version, "2.0.0");
    }

    // 11. ConfigFormat: all variants covered
    #[test]
    fn test_config_format_variants() {
        // Verify both variants exist and can be cloned/copied
        let _toml = ConfigFormat::Toml;
        let _json = ConfigFormat::Json;
        let toml2 = ConfigFormat::Toml;
        let json2 = ConfigFormat::Json;
        // use them so the compiler doesn't optimize them away
        let _ = (toml2, json2);
    }

    // 12. ConfigMerge::merge: workspace.root from override wins
    #[test]
    fn test_config_merge_workspace_root_override_wins() {
        let mut base = UnifyManifest::default_for("base");
        base.workspace = WorkspaceConfig {
            root: "/base/root".to_string(),
            members: vec!["a".to_string()],
            default_target: "debug".to_string(),
        };
        let mut overrides = UnifyManifest::default_for("override");
        overrides.workspace = WorkspaceConfig {
            root: "/override/root".to_string(),
            members: vec![],
            default_target: String::new(),
        };
        let merged = ConfigMerge::merge(base, overrides);
        assert_eq!(merged.workspace.root, "/override/root");
        // members: override is empty so base wins
        assert_eq!(merged.workspace.members, vec!["a".to_string()]);
    }

    // 13. ManifestValidator: violation for bad version
    #[test]
    fn test_validator_violation_for_bad_version() {
        let m = UnifyManifest {
            name: "proj".to_string(),
            version: "not-a-version".to_string(),
            ..Default::default()
        };
        let violations = ManifestValidator::validate(&m);
        assert!(violations.iter().any(|v| v.field == "version"));
    }

    // 14. ManifestValidator: violation for lsp threshold out of range
    #[test]
    fn test_validator_lsp_threshold_out_of_range() {
        let m = UnifyManifest {
            name: "proj".to_string(),
            version: "0.1.0".to_string(),
            lsp: Some(LspConfig {
                servers: vec![],
                conformance_threshold: -0.1,
                andon_enabled: false,
            }),
            ..Default::default()
        };
        let violations = ManifestValidator::validate(&m);
        assert!(violations
            .iter()
            .any(|v| v.field == "lsp.conformance_threshold"));
    }
}
