#[cfg(test)]
mod capability_tests {
    use crate::capability::{Capability, CapabilitySet};

    #[test]
    fn empty_has_no_capabilities() {
        let cs = CapabilitySet::empty();
        assert_eq!(cs.len(), 0);
        assert!(!cs.has(&Capability::Hover));
    }

    #[test]
    fn grant_adds_capability() {
        let mut cs = CapabilitySet::empty();
        cs.grant(Capability::Hover, b"hover-data");
        assert!(cs.has(&Capability::Hover));
        assert_eq!(cs.len(), 1);
    }

    #[test]
    fn has_returns_false_for_ungranted() {
        let cs = CapabilitySet::empty();
        assert!(!cs.has(&Capability::Completion));
    }

    #[test]
    fn revoke_removes_capability() {
        let mut cs = CapabilitySet::empty();
        cs.grant(Capability::Completion, b"data");
        cs.revoke(&Capability::Completion);
        assert!(!cs.has(&Capability::Completion));
        assert_eq!(cs.len(), 0);
    }

    #[test]
    fn verify_all_returns_true_after_granting() {
        let mut cs = CapabilitySet::empty();
        cs.grant(Capability::Diagnostics, b"diag-data");
        cs.grant(Capability::Hover, b"hover-data");
        assert!(cs.verify_all());
    }

    #[test]
    fn receipt_for_returns_receipt_after_grant() {
        let mut cs = CapabilitySet::empty();
        cs.grant(Capability::Definition, b"def-data");
        let r = cs.receipt_for(&Capability::Definition);
        assert!(r.is_some());
        assert_eq!(r.unwrap().key, "Definition");
    }

    #[test]
    fn custom_capability_is_supported() {
        let mut cs = CapabilitySet::empty();
        let cap = Capability::Custom("my-ext".to_owned());
        cs.grant(cap.clone(), b"custom");
        assert!(cs.has(&cap));
    }
}

#[cfg(test)]
mod diagnostic_tests {
    use crate::diagnostic::{Diagnostic, DiagnosticSet, DiagnosticSeverity, Position, Range};

    fn make_diag(severity: DiagnosticSeverity, msg: &str) -> Diagnostic {
        Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 5,
                },
            },
            severity,
            message: msg.to_owned(),
            code: None,
            source: None,
        }
    }

    #[test]
    fn add_and_get_diagnostics() {
        let mut ds = DiagnosticSet::new();
        ds.add(
            "file:///a.rs".to_owned(),
            make_diag(DiagnosticSeverity::Error, "oops"),
        );
        let diags = ds.get("file:///a.rs");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].message, "oops");
    }

    #[test]
    fn error_count_counts_errors_only() {
        let mut ds = DiagnosticSet::new();
        ds.add(
            "file:///a.rs".to_owned(),
            make_diag(DiagnosticSeverity::Error, "e1"),
        );
        ds.add(
            "file:///a.rs".to_owned(),
            make_diag(DiagnosticSeverity::Warning, "w1"),
        );
        ds.add(
            "file:///b.rs".to_owned(),
            make_diag(DiagnosticSeverity::Error, "e2"),
        );
        assert_eq!(ds.error_count(), 2);
    }

    #[test]
    fn clear_removes_diagnostics_for_uri() {
        let mut ds = DiagnosticSet::new();
        ds.add(
            "file:///a.rs".to_owned(),
            make_diag(DiagnosticSeverity::Error, "e1"),
        );
        ds.clear("file:///a.rs");
        assert_eq!(ds.get("file:///a.rs").len(), 0);
        assert_eq!(ds.uri_count(), 0);
    }

    #[test]
    fn merge_combines_sets() {
        let mut ds1 = DiagnosticSet::new();
        ds1.add(
            "file:///a.rs".to_owned(),
            make_diag(DiagnosticSeverity::Error, "e1"),
        );

        let mut ds2 = DiagnosticSet::new();
        ds2.add(
            "file:///a.rs".to_owned(),
            make_diag(DiagnosticSeverity::Warning, "w1"),
        );
        ds2.add(
            "file:///b.rs".to_owned(),
            make_diag(DiagnosticSeverity::Error, "e2"),
        );

        ds1.merge(ds2);
        assert_eq!(ds1.get("file:///a.rs").len(), 2);
        assert_eq!(ds1.uri_count(), 2);
        assert_eq!(ds1.error_count(), 2);
    }
}

#[cfg(test)]
mod gate_tests {
    use crate::gate::{AndonGate, AndonState};

    #[test]
    fn new_gate_is_open() {
        let gate = AndonGate::new();
        assert!(gate.is_open());
        assert_eq!(gate.state(), &AndonState::Open);
    }

    #[test]
    fn raise_changes_state_to_raised() {
        let mut gate = AndonGate::new();
        gate.raise("conformance check failed");
        assert!(!gate.is_open());
        assert_eq!(
            gate.state(),
            &AndonState::Raised("conformance check failed".to_owned())
        );
    }

    #[test]
    fn lower_resets_to_open() {
        let mut gate = AndonGate::new();
        gate.raise("reason");
        gate.lower();
        assert!(gate.is_open());
    }

    #[test]
    fn check_returns_ok_when_open() {
        let gate = AndonGate::new();
        assert!(gate.check().is_ok());
    }

    #[test]
    fn check_returns_err_when_raised() {
        let mut gate = AndonGate::new();
        gate.raise("bad conformance");
        let err = gate.check().unwrap_err();
        assert_eq!(err, "bad conformance");
    }

    #[test]
    fn history_records_state_transitions() {
        let mut gate = AndonGate::new();
        gate.raise("r1");
        gate.lower();
        gate.raise("r2");
        assert_eq!(gate.event_count(), 3);
        assert_eq!(gate.history().len(), 3);
        assert!(matches!(gate.history()[0].0, AndonState::Raised(_)));
        assert_eq!(gate.history()[1].0, AndonState::Open);
        assert!(matches!(gate.history()[2].0, AndonState::Raised(_)));
    }
}

#[cfg(test)]
mod compositor_tests {
    use crate::compositor::{CompositorState, ServerEntry};
    use crate::diagnostic::{Diagnostic, DiagnosticSet, DiagnosticSeverity, Position, Range};

    fn server(name: &str, lang: &str) -> ServerEntry {
        ServerEntry {
            name: name.to_owned(),
            language: lang.to_owned(),
            weight: 1.0,
        }
    }

    fn error_set(uri: &str, msg: &str) -> DiagnosticSet {
        let mut ds = DiagnosticSet::new();
        ds.add(
            uri.to_owned(),
            Diagnostic {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 1,
                    },
                },
                severity: DiagnosticSeverity::Error,
                message: msg.to_owned(),
                code: None,
                source: None,
            },
        );
        ds
    }

    #[test]
    fn add_server_reflects_in_health() {
        let mut cs = CompositorState::new();
        assert_eq!(cs.health().server_count, 0);
        cs.add_server(server("rust-analyzer", "rust"));
        assert_eq!(cs.health().server_count, 1);
        cs.add_server(server("clangd", "cpp"));
        assert_eq!(cs.health().server_count, 2);
    }

    #[test]
    fn raise_andon_makes_gate_closed_in_health() {
        let mut cs = CompositorState::new();
        assert!(cs.health().gate_open);
        cs.raise_andon("test failure");
        let h = cs.health();
        assert!(!h.gate_open);
        assert!(!h.healthy);
    }

    #[test]
    fn healthy_requires_no_errors_and_open_gate() {
        let mut cs = CompositorState::new();
        // Initially healthy
        assert!(cs.health().healthy);
        // Add an error
        cs.merge_diagnostics(error_set("file:///x.rs", "err"));
        assert!(!cs.health().healthy);
    }

    #[test]
    fn lower_andon_restores_gate_open() {
        let mut cs = CompositorState::new();
        cs.raise_andon("r");
        cs.lower_andon();
        assert!(cs.health().gate_open);
    }
}

#[cfg(test)]
mod conformance_tests {
    use crate::conformance::ConformanceScore;

    #[test]
    fn f_measure_is_harmonic_mean_of_fitness_and_precision() {
        let s = ConformanceScore::new(0.8, 0.6, 1.0, 1.0);
        // harmonic mean: 2 * 0.8 * 0.6 / (0.8 + 0.6) = 0.96 / 1.4
        let expected = 2.0 * 0.8 * 0.6 / (0.8 + 0.6);
        let diff = (s.f_measure() - expected).abs();
        assert!(
            diff < 1e-10,
            "f_measure={} expected={}",
            s.f_measure(),
            expected
        );
    }

    #[test]
    fn is_above_threshold_with_threshold_0_8() {
        let high = ConformanceScore::new(0.9, 0.9, 1.0, 1.0);
        assert!(high.is_above_threshold(0.8));

        let low = ConformanceScore::new(0.5, 0.5, 1.0, 1.0);
        assert!(!low.is_above_threshold(0.8));
    }

    #[test]
    fn delta_computes_differences() {
        let a = ConformanceScore::new(0.9, 0.8, 0.7, 0.6);
        let b = ConformanceScore::new(0.5, 0.4, 0.3, 0.2);
        let d = a.delta(&b);
        let diff_f = (d.fitness_delta - 0.4).abs();
        let diff_p = (d.precision_delta - 0.4).abs();
        let diff_g = (d.generalization_delta - 0.4).abs();
        let diff_s = (d.simplicity_delta - 0.4).abs();
        assert!(diff_f < 1e-10);
        assert!(diff_p < 1e-10);
        assert!(diff_g < 1e-10);
        assert!(diff_s < 1e-10);
    }

    #[test]
    fn perfect_f_measure_is_one() {
        let p = ConformanceScore::perfect();
        let diff = (p.f_measure() - 1.0).abs();
        assert!(diff < 1e-10, "f_measure of perfect should be 1.0");
    }

    #[test]
    fn zero_f_measure_is_zero() {
        let z = ConformanceScore::zero();
        let diff = (z.f_measure() - 0.0).abs();
        assert!(diff < 1e-10, "f_measure of zero should be 0.0");
    }
}
