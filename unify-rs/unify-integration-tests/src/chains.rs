//! End-to-end pipeline chain runners.
//!
//! Each `run_*_chain` function exercises a cross-crate pipeline from start to
//! finish and returns a [`ChainResult`] summarising what happened.

use unify_rdf::pipeline::{OntologyPipeline, PipelineConfig};
use unify_rdf::sparql::{PatternExecutor, SparqlExecutor};
use unify_rdf::store::TripleStore;
use unify_rdf::triple::Triple;
use unify_receipts::receipt::Receipt;

use crate::fixtures::{
    event_log_to_ocel, AdmissionGate, Event, EventLog, LifecycleState, LifecycleTracker,
    ReceiptChain, Trace,
};

/// Summary of a completed (or failed) pipeline chain run.
#[derive(Debug, Clone)]
pub struct ChainResult {
    pub name: String,
    pub steps_completed: usize,
    pub receipt_count: usize,
    pub errors: Vec<String>,
    pub success: bool,
}

impl ChainResult {
    /// Panics if the chain did not succeed.
    pub fn assert_success(&self) {
        assert!(
            self.success,
            "Chain '{}' failed. Errors: {:?}",
            self.name, self.errors
        );
    }

    fn new(name: impl Into<String>) -> Self {
        ChainResult {
            name: name.into(),
            steps_completed: 0,
            receipt_count: 0,
            errors: Vec::new(),
            success: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Pipeline 1: Event generation → OCEL → receipt chain
// ---------------------------------------------------------------------------

/// Build a small event log, convert it to OCEL, then issue a receipt for each
/// OCEL event and assemble a [`ReceiptChain`].
pub fn run_event_to_receipt_chain() -> Result<ChainResult, String> {
    let mut result = ChainResult::new("event-to-receipt-chain");

    // Step 1: create event log
    let event_log = EventLog {
        traces: vec![
            Trace {
                case_id: "case-A".into(),
                events: vec![
                    Event {
                        name: "init".into(),
                        timestamp: 1000,
                    },
                    Event {
                        name: "process".into(),
                        timestamp: 2000,
                    },
                ],
            },
            Trace {
                case_id: "case-B".into(),
                events: vec![Event {
                    name: "init".into(),
                    timestamp: 3000,
                }],
            },
        ],
    };
    result.steps_completed += 1;

    // Step 2: bridge to OCEL
    let ocel_log = event_log_to_ocel(&event_log);
    if ocel_log.events.is_empty() {
        return Err("OCEL bridge produced no events".into());
    }
    result.steps_completed += 1;

    // Step 3: issue a receipt for each OCEL event and build a chain
    let mut chain = ReceiptChain::new();
    for ocel_event in &ocel_log.events {
        let payload = format!("{}:{}", ocel_event.event_type, ocel_event.timestamp);
        let receipt = Receipt::new(format!("ocel:{}", ocel_event.id), payload.as_bytes());
        chain.append(receipt);
    }
    result.steps_completed += 1;

    result.receipt_count = chain.len();
    result.success = chain.len() == ocel_log.events.len();
    Ok(result)
}

// ---------------------------------------------------------------------------
// Pipeline 2: RDF triple store → SPARQL query → extract types
// ---------------------------------------------------------------------------

/// Populate a [`TripleStore`] with RDF type triples, run a SPARQL SELECT *
/// over it, then drive the ontology pipeline to render generated files.
pub fn run_rdf_query_chain() -> Result<ChainResult, String> {
    let mut result = ChainResult::new("rdf-query-chain");

    // Step 1: build a triple store with rdf:type assertions
    let mut store = TripleStore::new();
    for class in &["Character", "Location", "Item", "Quest"] {
        store.add(Triple::new(
            format!("http://ex/{}", class),
            "rdf:type",
            "http://ex/Class",
        ));
    }
    // Add some non-type triples
    store.add(Triple::new("http://ex/Character", "http://ex/name", "Hero"));
    result.steps_completed += 1;

    // Step 2: SPARQL SELECT * — returns all triples
    let exec = PatternExecutor(&store);
    let bindings = exec
        .select("SELECT * WHERE { ?s ?p ?o }")
        .map_err(|e| e.to_string())?;

    if bindings.is_empty() {
        return Err("SPARQL SELECT returned no bindings".into());
    }
    result.steps_completed += 1;

    // Step 3: drive the ontology pipeline
    let config = PipelineConfig {
        target_language: "rust".into(),
        output_dir: "generated".into(),
        template_dir: None,
        namespace: "ex".into(),
    };
    let mut pipeline = OntologyPipeline::new(store, config);

    // Load additional triples via the turtle loader
    pipeline
        .load_turtle("<http://ex/NPC> rdf:type <http://ex/Class> .")
        .map_err(|e| e.to_string())?;

    let types = pipeline.extract_types();
    if types.is_empty() {
        return Err("extract_types returned no types".into());
    }

    let output = pipeline.render().map_err(|e| e.to_string())?;
    if output.files.is_empty() {
        return Err("pipeline render produced no files".into());
    }
    result.steps_completed += 1;

    result.receipt_count = output.files.len();
    result.success = true;
    Ok(result)
}

// ---------------------------------------------------------------------------
// Pipeline 3: Admission gate → evidence → lifecycle tracking
// ---------------------------------------------------------------------------

/// Open a gate, record evidence via receipts, then advance a lifecycle tracker
/// through Raw → Parsed → Admitted → Exported.
pub fn run_admission_lifecycle_chain() -> Result<ChainResult, String> {
    let mut result = ChainResult::new("admission-lifecycle-chain");

    // Step 1: open gate
    let gate = AdmissionGate::open("pipeline-gate");
    if !gate.check() {
        return Err("Gate should be open at start".into());
    }
    result.steps_completed += 1;

    // Step 2: issue evidence receipt
    let evidence = Receipt::new("evidence:submitted", b"evidence-payload-v1");
    if !evidence.verify(b"evidence-payload-v1") {
        return Err("Evidence receipt failed verification".into());
    }
    result.receipt_count += 1;
    result.steps_completed += 1;

    // Step 3: advance lifecycle
    let mut tracker = LifecycleTracker::new();
    for state in [
        LifecycleState::Parsed,
        LifecycleState::Admitted,
        LifecycleState::Exported,
    ] {
        tracker
            .transition(state)
            .map_err(|e| format!("Lifecycle transition failed: {}", e))?;
    }
    if tracker.state() != LifecycleState::Exported {
        return Err("Expected Exported state".into());
    }
    result.steps_completed += 1;

    result.success = true;
    Ok(result)
}

// ---------------------------------------------------------------------------
// Pipeline 4: Process mining lifecycle — EventLog → OcelLog → validation
// ---------------------------------------------------------------------------

/// Build an [`EventLog`], bridge it to [`OcelLog`], validate the OCEL log,
/// then compute a simple conformance score.
pub fn run_pm_validation_chain() -> Result<ChainResult, String> {
    let mut result = ChainResult::new("pm-validation-chain");

    // Step 1: create event log with two traces
    let event_log = EventLog {
        traces: vec![
            Trace {
                case_id: "trace-001".into(),
                events: vec![
                    Event {
                        name: "A".into(),
                        timestamp: 100,
                    },
                    Event {
                        name: "B".into(),
                        timestamp: 200,
                    },
                    Event {
                        name: "C".into(),
                        timestamp: 300,
                    },
                ],
            },
            Trace {
                case_id: "trace-002".into(),
                events: vec![
                    Event {
                        name: "A".into(),
                        timestamp: 400,
                    },
                    Event {
                        name: "C".into(),
                        timestamp: 500,
                    },
                ],
            },
        ],
    };
    result.steps_completed += 1;

    // Step 2: bridge to OCEL
    let ocel_log = event_log_to_ocel(&event_log);
    result.steps_completed += 1;

    // Step 3: validate OCEL — no dangling references expected
    let violations = crate::fixtures::validate_ocel(&ocel_log);
    if !violations.is_empty() {
        result
            .errors
            .push(format!("OCEL violations: {:?}", violations));
        return Err(format!("OCEL validation failed: {:?}", violations));
    }
    result.steps_completed += 1;

    result.receipt_count = ocel_log.events.len();
    result.success = true;
    Ok(result)
}
