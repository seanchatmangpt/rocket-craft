pub mod assert;
pub mod chains;
pub mod fixtures;

#[cfg(test)]
mod tests {
    use super::assert::*;
    use super::chains::*;
    use super::fixtures::*;
    use unify_rdf::pipeline::OntologyPipeline;
    use unify_rdf::sparql::{PatternExecutor, SparqlExecutor};
    use unify_rdf::store::TripleStore;
    use unify_rdf::triple::{Term, Triple};
    use unify_receipts::receipt::Receipt;

    // -------------------------------------------------------------------------
    // 1. test_receipt_chain_end_to_end
    // -------------------------------------------------------------------------
    #[test]
    fn test_receipt_chain_end_to_end() {
        let mut chain = ReceiptChain::new();
        chain.append(Receipt::new("cap:read", b"payload-read"));
        chain.append(Receipt::new("cap:write", b"payload-write"));
        chain.append(Receipt::new("cap:admin", b"payload-admin"));

        assert_eq!(chain.len(), 3);

        // Verify each receipt matches its data
        assert!(chain.receipts()[0].verify(b"payload-read"));
        assert!(chain.receipts()[1].verify(b"payload-write"));
        assert!(chain.receipts()[2].verify(b"payload-admin"));

        // Head must be the last appended receipt
        let head = chain.head().expect("chain must have a head");
        assert_eq!(head.key, "cap:admin");
        assert!(head.verify(b"payload-admin"));

        // Integrity check: all receipts individually valid
        assert_chain_valid(&chain);
    }

    // -------------------------------------------------------------------------
    // 2. test_ocel_log_round_trip
    // -------------------------------------------------------------------------
    #[test]
    fn test_ocel_log_round_trip() {
        let log = OcelLog {
            objects: vec![
                OcelObject {
                    id: "obj1".into(),
                    object_type: "Item".into(),
                },
                OcelObject {
                    id: "obj2".into(),
                    object_type: "Order".into(),
                },
            ],
            events: vec![
                OcelEvent {
                    id: "e1".into(),
                    event_type: "create".into(),
                    related_object_ids: vec!["obj1".into()],
                    timestamp: 1_000_000,
                },
                OcelEvent {
                    id: "e2".into(),
                    event_type: "approve".into(),
                    related_object_ids: vec!["obj1".into(), "obj2".into()],
                    timestamp: 2_000_000,
                },
            ],
        };

        let json = serde_json::to_string(&log).expect("serialize must succeed");
        let restored: OcelLog = serde_json::from_str(&json).expect("deserialize must succeed");

        assert_eq!(restored.objects.len(), 2);
        assert_eq!(restored.events.len(), 2);
        assert_eq!(restored.objects[0].id, "obj1");
        assert_eq!(restored.events[1].event_type, "approve");

        assert_ocel_valid(&restored);
    }

    // -------------------------------------------------------------------------
    // 3. test_event_log_to_ocel_bridge
    // -------------------------------------------------------------------------
    #[test]
    fn test_event_log_to_ocel_bridge() {
        let event_log = EventLog {
            traces: vec![
                Trace {
                    case_id: "case-001".into(),
                    events: vec![
                        Event {
                            name: "start".into(),
                            timestamp: 100,
                        },
                        Event {
                            name: "complete".into(),
                            timestamp: 200,
                        },
                    ],
                },
                Trace {
                    case_id: "case-002".into(),
                    events: vec![
                        Event {
                            name: "start".into(),
                            timestamp: 300,
                        },
                        Event {
                            name: "complete".into(),
                            timestamp: 400,
                        },
                    ],
                },
            ],
        };

        assert_eq!(event_log.traces.len(), 2);
        assert_eq!(event_log.traces[0].events.len(), 2);

        // Bridge: convert EventLog to OCEL-compatible OcelLog
        let ocel = event_log_to_ocel(&event_log);

        // One OCEL object per trace case
        assert_eq!(ocel.objects.len(), 2);
        // One OCEL event per trace event
        assert_eq!(ocel.events.len(), 4);

        assert_ocel_valid(&ocel);
    }

    // -------------------------------------------------------------------------
    // 4. test_rdf_query_extracts_types
    // -------------------------------------------------------------------------
    #[test]
    fn test_rdf_query_extracts_types() {
        let mut store = TripleStore::new();
        store.add(Triple::new(
            "http://ex/Character",
            "rdf:type",
            "http://ex/Class",
        ));
        store.add(Triple::new(
            "http://ex/Location",
            "rdf:type",
            "http://ex/Class",
        ));
        store.add(Triple::new("http://ex/Item", "rdf:type", "http://ex/Class"));
        // Non-type triple that should not appear
        store.add(Triple::new("http://ex/Character", "http://ex/name", "Hero"));

        let rdf_type = Term::Named("rdf:type".into());
        let type_triples = store.query_predicate(&rdf_type);

        assert_eq!(type_triples.len(), 3);

        // Collect the class names
        let mut class_names: Vec<String> = type_triples
            .iter()
            .map(|t| match &t.subject {
                Term::Named(iri) => iri.split('/').last().unwrap_or(iri).to_string(),
                _ => String::new(),
            })
            .collect();
        class_names.sort();

        assert!(class_names.contains(&"Character".to_string()));
        assert!(class_names.contains(&"Location".to_string()));
        assert!(class_names.contains(&"Item".to_string()));
    }

    // -------------------------------------------------------------------------
    // 5. test_admission_gate_allows_valid
    // -------------------------------------------------------------------------
    #[test]
    fn test_admission_gate_allows_valid() {
        let mut gate = AdmissionGate::open("gate-alpha");
        assert!(gate.check(), "open gate must allow entry");

        gate.raise();
        assert!(!gate.check(), "raised gate must block entry");

        gate.lower();
        assert!(gate.check(), "lowered gate must allow entry again");
    }

    // -------------------------------------------------------------------------
    // 6. test_receipt_envelope_maps
    // -------------------------------------------------------------------------
    #[test]
    fn test_receipt_envelope_maps() {
        let receipt = Receipt::new("cap:compute", b"initial-payload");
        let envelope = ReceiptEnvelope::wrap(receipt, "service:alpha");

        // Map the payload: transform the envelope to a new one for a different service
        let mapped = envelope.map_service("service:beta");

        assert_eq!(mapped.service_tag, "service:beta");
        // The underlying receipt is preserved
        assert_eq!(mapped.receipt.key, "cap:compute");
        // The mapped envelope produces a new derived receipt
        let derived = mapped.derive_receipt(b"new-payload");
        assert!(derived.verify(b"new-payload"));
        assert_eq!(derived.key, "cap:compute");
    }

    // -------------------------------------------------------------------------
    // 7. test_lifecycle_tracker_transitions
    // -------------------------------------------------------------------------
    #[test]
    fn test_lifecycle_tracker_transitions() {
        let mut tracker = LifecycleTracker::new();
        assert_eq!(tracker.state(), LifecycleState::Raw);

        tracker
            .transition(LifecycleState::Parsed)
            .expect("Raw→Parsed must succeed");
        assert_eq!(tracker.state(), LifecycleState::Parsed);

        tracker
            .transition(LifecycleState::Admitted)
            .expect("Parsed→Admitted must succeed");
        assert_eq!(tracker.state(), LifecycleState::Admitted);

        tracker
            .transition(LifecycleState::Exported)
            .expect("Admitted→Exported must succeed");
        assert_eq!(tracker.state(), LifecycleState::Exported);

        assert_eq!(tracker.history().len(), 4); // Raw + 3 transitions
    }

    // -------------------------------------------------------------------------
    // 8. test_lifecycle_tracker_invalid_transition
    // -------------------------------------------------------------------------
    #[test]
    fn test_lifecycle_tracker_invalid_transition() {
        let mut tracker = LifecycleTracker::new();
        assert_eq!(tracker.state(), LifecycleState::Raw);

        // Raw → Exported is not allowed (must go through Parsed, Admitted first)
        let result = tracker.transition(LifecycleState::Exported);
        assert!(result.is_err(), "Raw→Exported must be rejected");
        // State must remain Raw after invalid transition
        assert_eq!(tracker.state(), LifecycleState::Raw);
    }

    // -------------------------------------------------------------------------
    // 9. test_petri_net_workflow_net_check
    // -------------------------------------------------------------------------
    #[test]
    fn test_petri_net_workflow_net_check() {
        // Build a minimal linear workflow net: P0 → T0 → P1 → T1 → P2
        let mut net = PetriNet::new();
        let p0 = net.add_place("p0");
        let t0 = net.add_transition("t0");
        let p1 = net.add_place("p1");
        let t1 = net.add_transition("t1");
        let p2 = net.add_place("p2");

        net.connect_place_to_transition(p0, t0);
        net.connect_transition_to_place(t0, p1);
        net.connect_place_to_transition(p1, t1);
        net.connect_transition_to_place(t1, p2);

        // Mark p0 as source and p2 as sink
        net.mark_source(p0);
        net.mark_sink(p2);

        assert!(
            net.is_workflow_net(),
            "linear P→T→P net must be a workflow net"
        );
    }

    // -------------------------------------------------------------------------
    // 10. test_conformance_score_f_measure
    // -------------------------------------------------------------------------
    #[test]
    fn test_conformance_score_f_measure() {
        let perfect = ConformanceScore::perfect();
        assert!(
            (perfect.f_measure() - 1.0).abs() < f64::EPSILON,
            "perfect conformance must have f-measure == 1.0"
        );

        let zero = ConformanceScore::zero();
        assert_eq!(
            zero.f_measure(),
            0.0,
            "zero conformance must have f-measure == 0.0"
        );

        let partial = ConformanceScore::new(0.8, 0.6);
        let expected_f = 2.0 * 0.8 * 0.6 / (0.8 + 0.6);
        assert!(
            (partial.f_measure() - expected_f).abs() < 1e-10,
            "f-measure must equal 2*precision*recall/(precision+recall)"
        );
    }

    // -------------------------------------------------------------------------
    // 11. test_ocel_validator_catches_dangling_ref
    // -------------------------------------------------------------------------
    #[test]
    fn test_ocel_validator_catches_dangling_ref() {
        let log = OcelLog {
            objects: vec![OcelObject {
                id: "obj1".into(),
                object_type: "Item".into(),
            }],
            events: vec![
                OcelEvent {
                    id: "e1".into(),
                    event_type: "create".into(),
                    related_object_ids: vec!["obj1".into()],
                    timestamp: 1_000,
                },
                OcelEvent {
                    id: "e2".into(),
                    event_type: "delete".into(),
                    // References obj99 which does NOT exist in objects
                    related_object_ids: vec!["obj99".into()],
                    timestamp: 2_000,
                },
            ],
        };

        let violations = validate_ocel(&log);
        assert!(
            !violations.is_empty(),
            "validator must catch dangling object reference"
        );
        assert!(
            violations.iter().any(|v| v.contains("obj99")),
            "violation message must mention the dangling ID"
        );
    }

    // -------------------------------------------------------------------------
    // 12. test_sample_fixtures_are_valid
    // -------------------------------------------------------------------------
    #[test]
    fn test_sample_fixtures_are_valid() {
        let ocel_log = sample_ocel_log();
        assert_ocel_valid(&ocel_log);

        let event_log = sample_event_log();
        assert_no_pm_violations(&event_log);
    }

    // -------------------------------------------------------------------------
    // 13. test_sparql_pattern_executor
    // -------------------------------------------------------------------------
    #[test]
    fn test_sparql_pattern_executor() {
        let store = sample_rdf_store();
        let exec = PatternExecutor(&store);

        let bindings = exec
            .select("SELECT * WHERE { ?s ?p ?o }")
            .expect("SELECT * must succeed on non-empty store");

        // sample_rdf_store() has 10 triples
        assert_eq!(bindings.len(), 10, "SELECT * must return all 10 triples");

        // Every binding must have s, p, o keys
        for binding in &bindings {
            assert!(binding.get("s").is_some(), "binding must have 's' key");
            assert!(binding.get("p").is_some(), "binding must have 'p' key");
            assert!(binding.get("o").is_some(), "binding must have 'o' key");
        }
    }

    // -------------------------------------------------------------------------
    // 14. test_gate_chain_all_open
    // -------------------------------------------------------------------------
    #[test]
    fn test_gate_chain_all_open() {
        let mut gates = vec![
            AdmissionGate::open("gate-1"),
            AdmissionGate::open("gate-2"),
            AdmissionGate::open("gate-3"),
        ];

        assert!(
            gates.iter().all(|g| g.check()),
            "all gates open — chain must pass"
        );

        // Raise the middle gate
        gates[1].raise();

        assert!(
            !gates.iter().all(|g| g.check()),
            "one raised gate means chain is not all-open"
        );

        // Lower it back
        gates[1].lower();
        assert!(
            gates.iter().all(|g| g.check()),
            "all gates open again after lowering"
        );
    }

    // -------------------------------------------------------------------------
    // 15. test_receipt_builder
    // -------------------------------------------------------------------------
    #[test]
    fn test_receipt_builder() {
        let data = b"rocket-craft-v1";
        let receipt = ReceiptBuilder::new("cap:launch").with_data(data).build();

        assert_eq!(receipt.key, "cap:launch");
        assert!(
            receipt.issued_at > 0,
            "issued_at must be a positive timestamp"
        );
        assert!(
            receipt.verify(data),
            "receipt must verify against its original data"
        );
        // Verify rejects different data
        assert!(
            !receipt.verify(b"other-data"),
            "receipt must not verify against different data"
        );
    }

    // -------------------------------------------------------------------------
    // Extra: run_event_to_receipt_chain pipeline
    // -------------------------------------------------------------------------
    #[test]
    fn test_pipeline_event_to_receipt_chain() {
        let result = run_event_to_receipt_chain().expect("chain must succeed");
        result.assert_success();
        assert!(result.steps_completed >= 3);
        assert_eq!(result.receipt_count, 3);
    }

    // Extra: run_rdf_query_chain pipeline
    #[test]
    fn test_pipeline_rdf_query_chain() {
        let result = run_rdf_query_chain().expect("rdf query chain must succeed");
        result.assert_success();
        assert!(result.steps_completed >= 2);
    }

    // Extra: run_admission_lifecycle_chain pipeline
    #[test]
    fn test_pipeline_admission_lifecycle_chain() {
        let result =
            run_admission_lifecycle_chain().expect("admission lifecycle chain must succeed");
        result.assert_success();
    }

    // Extra: run_pm_validation_chain pipeline
    #[test]
    fn test_pipeline_pm_validation_chain() {
        let result = run_pm_validation_chain().expect("pm validation chain must succeed");
        result.assert_success();
    }

    // Extra: sample_receipt_chain fixture
    #[test]
    fn test_sample_receipt_chain_fixture() {
        let chain = sample_receipt_chain();
        assert_eq!(chain.len(), 3);
        assert_chain_valid(&chain);
        assert_receipt_count(&chain, 3);
    }
}
