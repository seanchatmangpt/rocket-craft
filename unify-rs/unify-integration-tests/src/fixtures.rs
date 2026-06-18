//! Shared test fixtures and in-crate type definitions.
//!
//! Several upstream crates (`unify-ocel`, `unify-pm`, `unify-admission`) are
//! currently placeholder implementations.  This module provides the concrete types and helpers that
//! the integration tests exercise.

use serde::{Deserialize, Serialize};
use unify_rdf::store::TripleStore;
use unify_rdf::triple::Triple;
use unify_receipts::receipt::Receipt;

// ============================================================================
// Receipt chain (abstraction over Vec<Receipt>)
// ============================================================================

/// An ordered chain of [`Receipt`]s.  New receipts are appended at the tail;
/// `head()` returns a reference to the most recent one.
#[derive(Debug, Clone, Default)]
pub struct ReceiptChain {
    receipts: Vec<Receipt>,
}

impl ReceiptChain {
    pub fn new() -> Self {
        ReceiptChain::default()
    }

    pub fn append(&mut self, receipt: Receipt) {
        self.receipts.push(receipt);
    }

    pub fn receipts(&self) -> &[Receipt] {
        &self.receipts
    }

    pub fn head(&self) -> Option<&Receipt> {
        self.receipts.last()
    }

    pub fn len(&self) -> usize {
        self.receipts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.receipts.is_empty()
    }
}

// ============================================================================
// OCEL (Object-Centric Event Log) types
// ============================================================================

/// A single object in an OCEL log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OcelObject {
    pub id: String,
    pub object_type: String,
}

/// A single event in an OCEL log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OcelEvent {
    pub id: String,
    pub event_type: String,
    pub related_object_ids: Vec<String>,
    pub timestamp: u64,
}

/// An OCEL log containing objects and events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelLog {
    pub objects: Vec<OcelObject>,
    pub events: Vec<OcelEvent>,
}

/// Validate an [`OcelLog`] and return violation messages for any dangling
/// object references (events that reference object IDs not present in the log).
pub fn validate_ocel(log: &OcelLog) -> Vec<String> {
    let known_ids: std::collections::HashSet<&str> =
        log.objects.iter().map(|o| o.id.as_str()).collect();
    let mut violations = Vec::new();
    for event in &log.events {
        for ref_id in &event.related_object_ids {
            if !known_ids.contains(ref_id.as_str()) {
                violations.push(format!(
                    "Event '{}' references unknown object '{}'",
                    event.id, ref_id
                ));
            }
        }
    }
    violations
}

// ============================================================================
// Process-mining / event log types
// ============================================================================

/// A single activity event within a process trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    pub timestamp: u64,
}

/// A sequence of events belonging to a single case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub case_id: String,
    pub events: Vec<Event>,
}

/// A flat event log containing multiple traces (one per process case).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLog {
    pub traces: Vec<Trace>,
}

/// Bridge an [`EventLog`] to an [`OcelLog`].
///
/// Each trace becomes one OCEL object; each event within a trace becomes one
/// OCEL event whose `related_object_ids` contains the parent trace's case ID.
pub fn event_log_to_ocel(log: &EventLog) -> OcelLog {
    let objects: Vec<OcelObject> = log
        .traces
        .iter()
        .map(|t| OcelObject {
            id: t.case_id.clone(),
            object_type: "Case".into(),
        })
        .collect();

    let mut events = Vec::new();
    for trace in &log.traces {
        for (i, event) in trace.events.iter().enumerate() {
            events.push(OcelEvent {
                id: format!("{}-{}", trace.case_id, i),
                event_type: event.name.clone(),
                related_object_ids: vec![trace.case_id.clone()],
                timestamp: event.timestamp,
            });
        }
    }

    OcelLog { objects, events }
}

// ============================================================================
// Admission gate
// ============================================================================

/// A boolean gate used for admission control.
#[derive(Debug, Clone)]
pub struct AdmissionGate {
    pub name: String,
    open: bool,
}

impl AdmissionGate {
    /// Create a new gate in the **open** state.
    pub fn open(name: impl Into<String>) -> Self {
        AdmissionGate {
            name: name.into(),
            open: true,
        }
    }

    /// Returns `true` when the gate is open (allows entry).
    pub fn check(&self) -> bool {
        self.open
    }

    /// Raise (close) the gate.
    pub fn raise(&mut self) {
        self.open = false;
    }

    /// Lower (re-open) the gate.
    pub fn lower(&mut self) {
        self.open = true;
    }
}

// ============================================================================
// Receipt envelope
// ============================================================================

/// Wraps a [`Receipt`] with an additional service-routing tag.
#[derive(Debug, Clone)]
pub struct ReceiptEnvelope {
    pub receipt: Receipt,
    pub service_tag: String,
}

impl ReceiptEnvelope {
    pub fn wrap(receipt: Receipt, service_tag: impl Into<String>) -> Self {
        ReceiptEnvelope {
            receipt,
            service_tag: service_tag.into(),
        }
    }

    /// Create a new envelope re-tagged for a different service (same receipt).
    pub fn map_service(&self, new_tag: impl Into<String>) -> ReceiptEnvelope {
        ReceiptEnvelope {
            receipt: self.receipt.clone(),
            service_tag: new_tag.into(),
        }
    }

    /// Derive a new receipt covering `data` under the same key as the wrapped
    /// receipt.
    pub fn derive_receipt(&self, data: &[u8]) -> Receipt {
        Receipt::new(self.receipt.key.clone(), data)
    }
}

// ============================================================================
// Receipt builder
// ============================================================================

/// Fluent builder for [`Receipt`].
pub struct ReceiptBuilder {
    key: String,
    data: Vec<u8>,
}

impl ReceiptBuilder {
    pub fn new(key: impl Into<String>) -> Self {
        ReceiptBuilder {
            key: key.into(),
            data: Vec::new(),
        }
    }

    pub fn with_data(mut self, data: &[u8]) -> Self {
        self.data = data.to_vec();
        self
    }

    pub fn build(self) -> Receipt {
        Receipt::new(self.key, &self.data)
    }
}

// ============================================================================
// Lifecycle tracker
// ============================================================================

/// The ordered states a document or artefact may pass through.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleState {
    Raw,
    Parsed,
    Admitted,
    Exported,
}

impl LifecycleState {
    fn rank(&self) -> u8 {
        match self {
            LifecycleState::Raw => 0,
            LifecycleState::Parsed => 1,
            LifecycleState::Admitted => 2,
            LifecycleState::Exported => 3,
        }
    }
}

impl std::fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LifecycleState::Raw => "Raw",
            LifecycleState::Parsed => "Parsed",
            LifecycleState::Admitted => "Admitted",
            LifecycleState::Exported => "Exported",
        };
        write!(f, "{}", s)
    }
}

/// Tracks an artefact through its lifecycle states, enforcing monotone
/// forward-only transitions.
#[derive(Debug, Clone)]
pub struct LifecycleTracker {
    history: Vec<LifecycleState>,
}

impl LifecycleTracker {
    pub fn new() -> Self {
        LifecycleTracker {
            history: vec![LifecycleState::Raw],
        }
    }

    pub fn state(&self) -> LifecycleState {
        // SAFETY: history always has at least one element (Raw)
        self.history.last().unwrap().clone()
    }

    /// Attempt to transition to `next`.  Returns `Err` if `next` is not
    /// exactly one step ahead of the current state.
    pub fn transition(&mut self, next: LifecycleState) -> Result<(), String> {
        let current_rank = self.state().rank();
        let next_rank = next.rank();
        if next_rank != current_rank + 1 {
            return Err(format!(
                "Invalid transition: {:?} → {:?} (ranks {} → {})",
                self.state(),
                next,
                current_rank,
                next_rank
            ));
        }
        self.history.push(next);
        Ok(())
    }

    pub fn history(&self) -> &[LifecycleState] {
        &self.history
    }
}

// ============================================================================
// Petri net
// ============================================================================

/// Index of a place in a [`PetriNet`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaceId(usize);

/// Index of a transition in a [`PetriNet`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionId(usize);

/// A simple directed Petri net.
#[derive(Debug, Clone, Default)]
pub struct PetriNet {
    places: Vec<String>,
    transitions: Vec<String>,
    /// Arcs from places to transitions.
    pt_arcs: Vec<(PlaceId, TransitionId)>,
    /// Arcs from transitions to places.
    tp_arcs: Vec<(TransitionId, PlaceId)>,
    source: Option<PlaceId>,
    sink: Option<PlaceId>,
}

impl PetriNet {
    pub fn new() -> Self {
        PetriNet::default()
    }

    pub fn add_place(&mut self, name: impl Into<String>) -> PlaceId {
        let id = PlaceId(self.places.len());
        self.places.push(name.into());
        id
    }

    pub fn add_transition(&mut self, name: impl Into<String>) -> TransitionId {
        let id = TransitionId(self.transitions.len());
        self.transitions.push(name.into());
        id
    }

    pub fn connect_place_to_transition(&mut self, p: PlaceId, t: TransitionId) {
        self.pt_arcs.push((p, t));
    }

    pub fn connect_transition_to_place(&mut self, t: TransitionId, p: PlaceId) {
        self.tp_arcs.push((t, p));
    }

    pub fn mark_source(&mut self, p: PlaceId) {
        self.source = Some(p);
    }

    pub fn mark_sink(&mut self, p: PlaceId) {
        self.sink = Some(p);
    }

    /// A net is a workflow net when it has exactly one source and one sink
    /// place (as declared via `mark_source`/`mark_sink`).
    pub fn is_workflow_net(&self) -> bool {
        self.source.is_some() && self.sink.is_some()
    }
}

// ============================================================================
// Conformance score
// ============================================================================

/// Precision / recall conformance score with F-measure helper.
#[derive(Debug, Clone)]
pub struct ConformanceScore {
    pub precision: f64,
    pub recall: f64,
}

impl ConformanceScore {
    pub fn new(precision: f64, recall: f64) -> Self {
        ConformanceScore { precision, recall }
    }

    pub fn perfect() -> Self {
        ConformanceScore {
            precision: 1.0,
            recall: 1.0,
        }
    }

    pub fn zero() -> Self {
        ConformanceScore {
            precision: 0.0,
            recall: 0.0,
        }
    }

    /// F₁ measure (harmonic mean of precision and recall).
    /// Returns 0.0 when both are 0.0 to avoid division by zero.
    pub fn f_measure(&self) -> f64 {
        let denom = self.precision + self.recall;
        if denom == 0.0 {
            return 0.0;
        }
        2.0 * self.precision * self.recall / denom
    }
}

// ============================================================================
// Pre-built fixture functions
// ============================================================================

/// Returns a pre-built [`ReceiptChain`] with 3 receipts.
pub fn sample_receipt_chain() -> ReceiptChain {
    let mut chain = ReceiptChain::new();
    chain.append(Receipt::new("cap:read", b"read-payload"));
    chain.append(Receipt::new("cap:write", b"write-payload"));
    chain.append(Receipt::new("cap:exec", b"exec-payload"));
    chain
}

/// Returns a pre-built [`OcelLog`] with 2 objects and 3 events.
pub fn sample_ocel_log() -> OcelLog {
    OcelLog {
        objects: vec![
            OcelObject {
                id: "obj-a".into(),
                object_type: "Item".into(),
            },
            OcelObject {
                id: "obj-b".into(),
                object_type: "Order".into(),
            },
        ],
        events: vec![
            OcelEvent {
                id: "ev-1".into(),
                event_type: "create".into(),
                related_object_ids: vec!["obj-a".into()],
                timestamp: 1_000,
            },
            OcelEvent {
                id: "ev-2".into(),
                event_type: "link".into(),
                related_object_ids: vec!["obj-a".into(), "obj-b".into()],
                timestamp: 2_000,
            },
            OcelEvent {
                id: "ev-3".into(),
                event_type: "complete".into(),
                related_object_ids: vec!["obj-b".into()],
                timestamp: 3_000,
            },
        ],
    }
}

/// Returns a pre-built [`EventLog`] with 2 traces and 4 total events.
pub fn sample_event_log() -> EventLog {
    EventLog {
        traces: vec![
            Trace {
                case_id: "case-001".into(),
                events: vec![
                    Event {
                        name: "start".into(),
                        timestamp: 100,
                    },
                    Event {
                        name: "process".into(),
                        timestamp: 200,
                    },
                    Event {
                        name: "complete".into(),
                        timestamp: 300,
                    },
                ],
            },
            Trace {
                case_id: "case-002".into(),
                events: vec![
                    Event {
                        name: "start".into(),
                        timestamp: 400,
                    },
                    Event {
                        name: "complete".into(),
                        timestamp: 500,
                    },
                ],
            },
        ],
    }
}

/// Returns a [`TripleStore`] with 10 RDF triples about a "Character" class.
pub fn sample_rdf_store() -> TripleStore {
    let mut store = TripleStore::new();

    // 5 rdf:type triples for different individuals
    for name in &["Hero", "Villain", "NPC", "Merchant", "Guard"] {
        store.add(Triple::new(
            format!("http://ex/{}", name),
            "rdf:type",
            "http://ex/Character",
        ));
    }

    // 5 attribute triples
    store.add(Triple::new("http://ex/Hero", "http://ex/level", "10"));
    store.add(Triple::new("http://ex/Villain", "http://ex/level", "20"));
    store.add(Triple::new("http://ex/NPC", "http://ex/level", "1"));
    store.add(Triple::new("http://ex/Merchant", "http://ex/gold", "500"));
    store.add(Triple::new("http://ex/Guard", "http://ex/rank", "sergeant"));

    store
}
