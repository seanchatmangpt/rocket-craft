//! Process mining over event logs — DFG discovery, Petri nets, conformance checking.
//!
//! This crate provides:
//! - Flat event logs with case/activity/timestamp structure
//! - Direct-Follows Graph (DFG) discovery
//! - Petri net representation and token-game simulation
//! - Conformance checking (fitness/precision) against a reference Petri net
//! - Lifecycle state tracking for rocket build pipeline operations
//!
//! # Example
//! ```
//! use unify_pm::{EventLog, DirectFollowsGraph, rocket_pipeline_petri_net, ConformanceChecker};
//!
//! let log = unify_pm::rocket_build_event_log();
//! let dfg = DirectFollowsGraph::discover(&log);
//! let net = rocket_pipeline_petri_net();
//! let checker = ConformanceChecker::new(&net);
//! let result = checker.check_log(&log);
//! assert!(result.fitness > 0.0);
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────
// Errors
// ──────────────────────────────────────────────

/// Errors that can occur during process mining operations.
#[derive(Debug, thiserror::Error)]
pub enum PmError {
    /// The specified transition is not enabled in the current marking.
    #[error("Transition not enabled: {0}")]
    NotEnabled(String),
    /// The specified transition id does not exist in the net.
    #[error("Transition not found: {0}")]
    NotFound(String),
    /// Invalid lifecycle transition for the current phase.
    #[error("Invalid lifecycle transition from phase {from:?} on event {event}")]
    InvalidTransition { from: String, event: String },
}

// ──────────────────────────────────────────────
// Event Log
// ──────────────────────────────────────────────

/// A single event in an event log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// The case identifier (e.g., project name "Brm").
    pub case_id: String,
    /// The activity name (e.g., "build", "audit", "package").
    pub activity: String,
    /// The ISO 8601 timestamp of the event.
    pub timestamp: String,
    /// Arbitrary key-value attributes attached to this event.
    pub attributes: HashMap<String, String>,
}

impl Event {
    /// Create a new event with no extra attributes.
    pub fn new(case_id: &str, activity: &str, timestamp: &str) -> Self {
        Self {
            case_id: case_id.to_string(),
            activity: activity.to_string(),
            timestamp: timestamp.to_string(),
            attributes: HashMap::new(),
        }
    }

    /// Create a new event with additional attributes.
    pub fn with_attrs(
        case_id: &str,
        activity: &str,
        timestamp: &str,
        attrs: Vec<(&str, &str)>,
    ) -> Self {
        let mut e = Self::new(case_id, activity, timestamp);
        for (k, v) in attrs {
            e.attributes.insert(k.to_string(), v.to_string());
        }
        e
    }
}

/// A flat event log: an ordered sequence of events.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventLog {
    /// All events in arrival order.
    pub events: Vec<Event>,
}

impl EventLog {
    /// Create an empty event log.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a new event with no extra attributes.
    pub fn push(&mut self, case_id: &str, activity: &str, timestamp: &str) {
        self.events.push(Event::new(case_id, activity, timestamp));
    }

    /// Append a new event with additional key-value attributes.
    pub fn push_with_attrs(
        &mut self,
        case_id: &str,
        activity: &str,
        timestamp: &str,
        attrs: Vec<(&str, &str)>,
    ) {
        self.events
            .push(Event::with_attrs(case_id, activity, timestamp, attrs));
    }

    /// Return the unique case identifiers present in the log, in order of first appearance.
    pub fn cases(&self) -> Vec<&str> {
        let mut seen: Vec<&str> = Vec::new();
        for e in &self.events {
            let id: &str = &e.case_id;
            if !seen.contains(&id) {
                seen.push(id);
            }
        }
        seen
    }

    /// Return the unique activity names present in the log, in order of first appearance.
    pub fn activities(&self) -> Vec<&str> {
        let mut seen: Vec<&str> = Vec::new();
        for e in &self.events {
            let act: &str = &e.activity;
            if !seen.contains(&act) {
                seen.push(act);
            }
        }
        seen
    }

    /// Return all events belonging to `case_id` in their original order.
    pub fn events_for_case(&self, case_id: &str) -> Vec<&Event> {
        self.events
            .iter()
            .filter(|e| e.case_id == case_id)
            .collect()
    }

    /// Serialize the log to a JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("EventLog serialization should never fail")
    }

    /// Deserialize the log from a JSON string.
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

// ──────────────────────────────────────────────
// Direct-Follows Graph
// ──────────────────────────────────────────────

/// A directly-follows graph: how often activity B directly follows activity A
/// within the same case.
#[derive(Debug, Clone, Default)]
pub struct DirectFollowsGraph {
    /// Edge counts: (from_activity, to_activity) → frequency.
    pub edges: HashMap<(String, String), usize>,
    /// How often each activity starts a case.
    pub start_activities: HashMap<String, usize>,
    /// How often each activity ends a case.
    pub end_activities: HashMap<String, usize>,
}

impl DirectFollowsGraph {
    /// Discover a DFG from an event log.
    ///
    /// Events within each case are assumed to be in the order they appear
    /// in `log.events`.
    pub fn discover(log: &EventLog) -> Self {
        let mut dfg = Self::default();

        for case_id in log.cases() {
            let case_events = log.events_for_case(case_id);
            if case_events.is_empty() {
                continue;
            }

            // First event → start activity
            let first_act = case_events[0].activity.clone();
            *dfg.start_activities.entry(first_act).or_insert(0) += 1;

            // Last event → end activity
            let last_act = case_events[case_events.len() - 1].activity.clone();
            *dfg.end_activities.entry(last_act).or_insert(0) += 1;

            // Consecutive pairs → edges
            for window in case_events.windows(2) {
                let from = window[0].activity.clone();
                let to = window[1].activity.clone();
                *dfg.edges.entry((from, to)).or_insert(0) += 1;
            }
        }

        dfg
    }

    /// Return the frequency of the edge (from, to), or 0 if it doesn't exist.
    pub fn edge_count(&self, from: &str, to: &str) -> usize {
        *self
            .edges
            .get(&(from.to_string(), to.to_string()))
            .unwrap_or(&0)
    }

    /// Return the most frequent path through the DFG using a greedy walk
    /// starting from the most common start activity.
    ///
    /// The walk terminates when no outgoing edge exists or the same activity
    /// would be visited twice (to avoid cycles).
    pub fn most_frequent_path(&self) -> Vec<String> {
        // Pick the start activity with the highest count
        let start = self
            .start_activities
            .iter()
            .max_by_key(|(_, &c)| c)
            .map(|(act, _)| act.clone());

        let Some(start) = start else {
            return Vec::new();
        };

        let mut path = vec![start.clone()];
        let mut current = start;

        loop {
            // Find the highest-count outgoing edge from current
            let next = self
                .edges
                .iter()
                .filter(|((from, to), _)| from == &current && !path.contains(to))
                .max_by_key(|(_, &c)| c)
                .map(|((_, to), _)| to.clone());

            match next {
                Some(n) => {
                    path.push(n.clone());
                    current = n;
                }
                None => break,
            }
        }

        path
    }

    /// Render the DFG as a GraphViz DOT string.
    pub fn to_dot(&self) -> String {
        let mut out = String::from("digraph DFG {\n  rankdir=LR;\n");

        // Collect all node names
        let mut nodes: Vec<String> = Vec::new();
        for act in self
            .start_activities
            .keys()
            .chain(self.end_activities.keys())
        {
            if !nodes.contains(act) {
                nodes.push(act.clone());
            }
        }
        for (from, to) in self.edges.keys() {
            if !nodes.contains(from) {
                nodes.push(from.clone());
            }
            if !nodes.contains(to) {
                nodes.push(to.clone());
            }
        }

        for node in &nodes {
            out.push_str(&format!("  \"{}\" [shape=box];\n", node));
        }

        // Start / end markers
        out.push_str(
            "  \"__start__\" [shape=circle, label=\"\", style=filled, fillcolor=black];\n",
        );
        out.push_str(
            "  \"__end__\" [shape=doublecircle, label=\"\", style=filled, fillcolor=black];\n",
        );

        for (act, count) in &self.start_activities {
            out.push_str(&format!(
                "  \"__start__\" -> \"{}\" [label=\"{}\"];\n",
                act, count
            ));
        }
        for (act, count) in &self.end_activities {
            out.push_str(&format!(
                "  \"{}\" -> \"__end__\" [label=\"{}\"];\n",
                act, count
            ));
        }

        let mut sorted_edges: Vec<_> = self.edges.iter().collect();
        sorted_edges.sort_by_key(|((f, t), _)| (f.clone(), t.clone()));
        for ((from, to), count) in sorted_edges {
            out.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
                from, to, count
            ));
        }

        out.push_str("}\n");
        out
    }

    /// Serialize the DFG to a JSON string.
    pub fn to_json(&self) -> String {
        // Edges need string keys for JSON
        let edges_str: HashMap<String, usize> = self
            .edges
            .iter()
            .map(|((f, t), c)| (format!("{} -> {}", f, t), *c))
            .collect();

        serde_json::json!({
            "edges": edges_str,
            "start_activities": self.start_activities,
            "end_activities": self.end_activities,
        })
        .to_string()
    }
}

// ──────────────────────────────────────────────
// Petri Net
// ──────────────────────────────────────────────

/// A place in a Petri net.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Place {
    /// Unique identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Initial token count (stored in the place definition for convenience;
    /// the authoritative token count lives in the marking map).
    pub tokens: usize,
}

/// A transition in a Petri net.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    /// Unique identifier.
    pub id: String,
    /// Optional label (None = silent / tau transition).
    pub label: Option<String>,
    /// Identifiers of the input places.
    pub input_places: Vec<String>,
    /// Identifiers of the output places.
    pub output_places: Vec<String>,
}

impl Transition {
    /// Return true if this is a silent (tau) transition.
    pub fn is_silent(&self) -> bool {
        self.label.is_none()
    }
}

/// A Petri net: places, transitions, and markings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetriNet {
    pub places: Vec<Place>,
    pub transitions: Vec<Transition>,
    /// Initial marking: place_id → token count.
    pub initial_marking: HashMap<String, usize>,
    /// Final (accepting) marking: place_id → token count.
    pub final_marking: HashMap<String, usize>,
}

impl Default for PetriNet {
    fn default() -> Self {
        Self::new()
    }
}

impl PetriNet {
    /// Create an empty Petri net.
    pub fn new() -> Self {
        Self {
            places: Vec::new(),
            transitions: Vec::new(),
            initial_marking: HashMap::new(),
            final_marking: HashMap::new(),
        }
    }

    /// Add a place with the given id and name (no tokens by default).
    pub fn add_place(&mut self, id: &str, name: &str) -> &mut Self {
        self.places.push(Place {
            id: id.to_string(),
            name: name.to_string(),
            tokens: 0,
        });
        self
    }

    /// Add a transition with an optional label and input/output arc lists.
    pub fn add_transition(
        &mut self,
        id: &str,
        label: Option<&str>,
        inputs: &[&str],
        outputs: &[&str],
    ) -> &mut Self {
        self.transitions.push(Transition {
            id: id.to_string(),
            label: label.map(|l| l.to_string()),
            input_places: inputs.iter().map(|s| s.to_string()).collect(),
            output_places: outputs.iter().map(|s| s.to_string()).collect(),
        });
        self
    }

    /// Mark a place as part of the initial marking.
    pub fn set_initial_marking(&mut self, place_id: &str, tokens: usize) -> &mut Self {
        self.initial_marking.insert(place_id.to_string(), tokens);
        self
    }

    /// Mark a place as part of the final (accepting) marking.
    pub fn set_final_marking(&mut self, place_id: &str, tokens: usize) -> &mut Self {
        self.final_marking.insert(place_id.to_string(), tokens);
        self
    }

    /// Return all transitions that are enabled in the given marking.
    ///
    /// A transition is enabled when every input place holds at least one token.
    pub fn enabled_transitions<'a>(
        &'a self,
        marking: &HashMap<String, usize>,
    ) -> Vec<&'a Transition> {
        self.transitions
            .iter()
            .filter(|t| {
                t.input_places
                    .iter()
                    .all(|p| marking.get(p).copied().unwrap_or(0) >= 1)
            })
            .collect()
    }

    /// Fire a transition: consume one token from each input place and
    /// produce one token in each output place.
    pub fn fire(
        &self,
        marking: &mut HashMap<String, usize>,
        transition_id: &str,
    ) -> Result<(), PmError> {
        let t = self
            .transitions
            .iter()
            .find(|t| t.id == transition_id)
            .ok_or_else(|| PmError::NotFound(transition_id.to_string()))?;

        // Check enabled
        for p in &t.input_places {
            if marking.get(p).copied().unwrap_or(0) < 1 {
                return Err(PmError::NotEnabled(transition_id.to_string()));
            }
        }

        // Consume tokens
        for p in &t.input_places {
            *marking.entry(p.clone()).or_insert(0) -= 1;
        }

        // Produce tokens
        for p in &t.output_places {
            *marking.entry(p.clone()).or_insert(0) += 1;
        }

        Ok(())
    }

    /// Serialize the Petri net to a JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("PetriNet serialization should never fail")
    }
}

/// Build the Petri net for the rocket build pipeline:
///
/// ```text
/// [start] -> (load_manifest) -> [p1] -> (doctor) -> [p2]
///         -> (audit) -> [p3] -> (build) -> [p4] -> (package) -> [end]
/// ```
pub fn rocket_pipeline_petri_net() -> PetriNet {
    let mut net = PetriNet::new();

    net.add_place("start", "Start")
        .add_place("p1", "After load_manifest")
        .add_place("p2", "After doctor")
        .add_place("p3", "After audit")
        .add_place("p4", "After build")
        .add_place("end", "End");

    net.add_transition(
        "t_load_manifest",
        Some("load_manifest"),
        &["start"],
        &["p1"],
    )
    .add_transition("t_doctor", Some("doctor_check"), &["p1"], &["p2"])
    .add_transition("t_audit", Some("audit"), &["p2"], &["p3"])
    .add_transition("t_build", Some("build"), &["p3"], &["p4"])
    .add_transition("t_package", Some("package"), &["p4"], &["end"]);

    net.set_initial_marking("start", 1)
        .set_final_marking("end", 1);

    net
}

// ──────────────────────────────────────────────
// Conformance Checking
// ──────────────────────────────────────────────

/// A single conformance violation: an activity that could not be replayed.
#[derive(Debug, Clone)]
pub struct ConformanceViolation {
    /// The case identifier.
    pub case_id: String,
    /// The activity that caused the violation.
    pub activity: String,
    /// Position within the trace (0-based).
    pub position: usize,
    /// Human-readable description of the violation.
    pub description: String,
}

/// Summary result of conformance checking.
#[derive(Debug, Clone)]
pub struct ConformanceResult {
    /// Fraction of activities in the log that could be replayed in the model (0.0–1.0).
    pub fitness: f64,
    /// Precision estimate (0.0–1.0); a simple approximation based on
    /// enabled-transitions ratio.
    pub precision: f64,
    /// True when fitness ≥ 0.8.
    pub is_fitting: bool,
    /// All individual violations encountered.
    pub violations: Vec<ConformanceViolation>,
}

/// Token-replay based conformance checker.
pub struct ConformanceChecker<'a> {
    /// Reference Petri net.
    pub net: &'a PetriNet,
}

impl<'a> ConformanceChecker<'a> {
    /// Create a new checker for the given Petri net.
    pub fn new(net: &'a PetriNet) -> Self {
        Self { net }
    }

    /// Check all cases in the log and return an aggregated result.
    pub fn check_log(&self, log: &EventLog) -> ConformanceResult {
        let mut all_violations: Vec<ConformanceViolation> = Vec::new();
        let mut total_events = 0usize;
        let mut total_fit = 0usize;
        let mut precision_sum = 0.0f64;
        let mut case_count = 0usize;

        for case_id in log.cases() {
            let activities: Vec<&str> = log
                .events_for_case(case_id)
                .iter()
                .map(|e| e.activity.as_str())
                .collect();

            let result = self.check_trace(case_id, &activities);
            let n = activities.len();
            let fit_events = (result.fitness * n as f64).round() as usize;
            total_events += n;
            total_fit += fit_events;
            all_violations.extend(result.violations);
            precision_sum += result.precision;
            case_count += 1;
        }

        let fitness = if total_events == 0 {
            1.0
        } else {
            total_fit as f64 / total_events as f64
        };
        let precision = if case_count == 0 {
            1.0
        } else {
            precision_sum / case_count as f64
        };

        ConformanceResult {
            fitness,
            precision,
            is_fitting: fitness >= 0.8,
            violations: all_violations,
        }
    }

    /// Perform token-replay for a single trace and return a per-trace result.
    pub fn check_trace(&self, case_id: &str, activities: &[&str]) -> ConformanceResult {
        let mut marking = self.net.initial_marking.clone();
        let mut violations: Vec<ConformanceViolation> = Vec::new();
        let total = activities.len();
        let mut fit_count = 0usize;
        let mut precision_sum = 0.0f64;
        let mut steps = 0usize;

        for (pos, &activity) in activities.iter().enumerate() {
            // Try to fire any transition whose label matches the activity
            let matched = self
                .net
                .transitions
                .iter()
                .find(|t| t.label.as_deref() == Some(activity));

            match matched {
                Some(t) => {
                    // Count enabled transitions before firing (for precision)
                    let enabled_before = self.net.enabled_transitions(&marking).len();

                    if enabled_before > 0 {
                        precision_sum += 1.0 / enabled_before as f64;
                    }
                    steps += 1;

                    let tid = t.id.clone();
                    match self.net.fire(&mut marking, &tid) {
                        Ok(()) => {
                            fit_count += 1;
                        }
                        Err(_) => {
                            // Transition exists but is not enabled – produce missing tokens
                            // (synchronous move: count as partial fit, force-fire)
                            violations.push(ConformanceViolation {
                                case_id: case_id.to_string(),
                                activity: activity.to_string(),
                                position: pos,
                                description: format!(
                                    "Transition '{}' not enabled; missing tokens in input places",
                                    activity
                                ),
                            });
                            // Force the marking to allow the transition (add missing tokens)
                            for p in &t.input_places {
                                let count = marking.entry(p.clone()).or_insert(0);
                                if *count == 0 {
                                    *count = 1;
                                }
                            }
                            // Now fire it
                            let _ = self.net.fire(&mut marking, &tid);
                            // Partial fit: 0.5 credit
                            fit_count = fit_count.saturating_add(0);
                        }
                    }
                }
                None => {
                    // Activity not in the model at all
                    violations.push(ConformanceViolation {
                        case_id: case_id.to_string(),
                        activity: activity.to_string(),
                        position: pos,
                        description: format!(
                            "Activity '{}' not found in the reference model",
                            activity
                        ),
                    });
                    steps += 1;
                }
            }
        }

        let fitness = if total == 0 {
            1.0
        } else {
            fit_count as f64 / total as f64
        };

        let precision = if steps == 0 {
            1.0
        } else {
            (precision_sum / steps as f64).min(1.0)
        };

        ConformanceResult {
            fitness,
            precision,
            is_fitting: fitness >= 0.8,
            violations,
        }
    }
}

// ──────────────────────────────────────────────
// Lifecycle Tracker
// ──────────────────────────────────────────────

/// The current phase of a rocket build pipeline case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelinePhase {
    /// No activity has started yet.
    Idle,
    /// `load_manifest` has completed.
    ManifestLoaded,
    /// `doctor_check` has completed.
    Validated,
    /// `build` has completed.
    Built,
    /// `package` has completed.
    Packaged,
    /// A failure occurred; the string holds the reason.
    Failed(String),
}

impl PipelinePhase {
    /// Human-readable name of the phase.
    pub fn name(&self) -> &str {
        match self {
            Self::Idle => "Idle",
            Self::ManifestLoaded => "ManifestLoaded",
            Self::Validated => "Validated",
            Self::Built => "Built",
            Self::Packaged => "Packaged",
            Self::Failed(_) => "Failed",
        }
    }
}

/// Tracks the lifecycle of a single rocket build pipeline case.
pub struct LifecycleTracker {
    /// The case identifier.
    pub case_id: String,
    /// Current phase.
    pub phase: PipelinePhase,
    /// History: (timestamp, new_phase).
    pub history: Vec<(String, PipelinePhase)>,
}

impl LifecycleTracker {
    /// Create a new tracker in the `Idle` phase.
    pub fn new(case_id: &str) -> Self {
        Self {
            case_id: case_id.to_string(),
            phase: PipelinePhase::Idle,
            history: Vec::new(),
        }
    }

    /// Attempt to advance the lifecycle by processing the given event name.
    ///
    /// Valid transitions:
    /// - Idle → ManifestLoaded  on `load_manifest`
    /// - ManifestLoaded → Validated  on `doctor_check` or `audit`
    /// - Validated → Built  on `build`
    /// - Built → Packaged  on `package`
    /// - Any → Failed  on `fail` or `error`
    pub fn transition(&mut self, event: &str, timestamp: &str) -> Result<(), PmError> {
        let next = match (&self.phase, event) {
            (PipelinePhase::Idle, "load_manifest") => PipelinePhase::ManifestLoaded,
            (PipelinePhase::ManifestLoaded, "doctor_check" | "audit") => PipelinePhase::Validated,
            (PipelinePhase::Validated, "build") => PipelinePhase::Built,
            (PipelinePhase::Built, "package") => PipelinePhase::Packaged,
            (_, "fail" | "error") => PipelinePhase::Failed(event.to_string()),
            (phase, ev) => {
                return Err(PmError::InvalidTransition {
                    from: phase.name().to_string(),
                    event: ev.to_string(),
                });
            }
        };

        self.history.push((timestamp.to_string(), next.clone()));
        self.phase = next;
        Ok(())
    }

    /// Return a reference to the current phase.
    pub fn current_phase(&self) -> &PipelinePhase {
        &self.phase
    }

    /// Return true if the case is in a terminal state (Packaged or Failed).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.phase,
            PipelinePhase::Packaged | PipelinePhase::Failed(_)
        )
    }

    /// Convert the history into an `EventLog` with one event per phase transition.
    pub fn to_event_log(&self) -> EventLog {
        let mut log = EventLog::new();
        for (ts, phase) in &self.history {
            log.push(&self.case_id, phase.name(), ts);
        }
        log
    }
}

// ──────────────────────────────────────────────
// Example Logs
// ──────────────────────────────────────────────

/// Generate a realistic rocket build pipeline event log with five cases.
///
/// Cases: `SurvivalGame`, `Brm`, `ShooterGame`, `FullSpectrum`, `RealisticRendering`.
/// Activities per case: `load_manifest` → `doctor_check` → `audit` → `build` → `package`.
pub fn rocket_build_event_log() -> EventLog {
    let mut log = EventLog::new();

    let cases = [
        ("SurvivalGame", "2024-01-10"),
        ("Brm", "2024-01-11"),
        ("ShooterGame", "2024-01-12"),
        ("FullSpectrum", "2024-01-13"),
        ("RealisticRendering", "2024-01-14"),
    ];

    let activities = [
        ("load_manifest", "T09:00:00Z"),
        ("doctor_check", "T09:05:00Z"),
        ("audit", "T09:10:00Z"),
        ("build", "T09:30:00Z"),
        ("package", "T10:00:00Z"),
    ];

    for (case, date) in &cases {
        for (act, time) in &activities {
            log.push(case, act, &format!("{}{}", date, time));
        }
    }

    log
}

/// Generate a blueprint authoring event log.
///
/// Activities: `draft` → `validate` → `serialize` → `paste` → `test`.
/// Cases: `BlueprintA`, `BlueprintB`, `BlueprintC`.
pub fn blueprint_authoring_event_log() -> EventLog {
    let mut log = EventLog::new();

    let cases = [
        ("BlueprintA", "2024-02-01"),
        ("BlueprintB", "2024-02-02"),
        ("BlueprintC", "2024-02-03"),
    ];

    let activities = [
        ("draft", "T08:00:00Z"),
        ("validate", "T08:15:00Z"),
        ("serialize", "T08:30:00Z"),
        ("paste", "T08:45:00Z"),
        ("test", "T09:00:00Z"),
    ];

    for (case, date) in &cases {
        for (act, time) in &activities {
            log.push(case, act, &format!("{}{}", date, time));
        }
    }

    log
}

// ──────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── EventLog ─────────────────────────────

    #[test]
    fn test_event_log_push_and_events_for_case() {
        let mut log = EventLog::new();
        log.push("case1", "start", "2024-01-01T00:00:00Z");
        log.push("case1", "end", "2024-01-01T01:00:00Z");
        log.push("case2", "start", "2024-01-02T00:00:00Z");

        let c1 = log.events_for_case("case1");
        assert_eq!(c1.len(), 2);
        assert_eq!(c1[0].activity, "start");
        assert_eq!(c1[1].activity, "end");

        let c2 = log.events_for_case("case2");
        assert_eq!(c2.len(), 1);
    }

    #[test]
    fn test_event_log_cases_unique() {
        let mut log = EventLog::new();
        log.push("A", "x", "t1");
        log.push("B", "x", "t2");
        log.push("A", "y", "t3");
        let cases = log.cases();
        assert_eq!(cases, vec!["A", "B"]);
    }

    #[test]
    fn test_event_log_activities_unique() {
        let mut log = EventLog::new();
        log.push("c1", "build", "t1");
        log.push("c1", "test", "t2");
        log.push("c2", "build", "t3");
        let acts = log.activities();
        assert_eq!(acts, vec!["build", "test"]);
    }

    #[test]
    fn test_event_log_empty_case_returns_empty() {
        let log = EventLog::new();
        assert!(log.events_for_case("nonexistent").is_empty());
    }

    #[test]
    fn test_event_log_to_json_from_json_roundtrip() {
        let mut log = EventLog::new();
        log.push("proj", "build", "2024-01-01T00:00:00Z");
        log.push_with_attrs(
            "proj",
            "test",
            "2024-01-01T01:00:00Z",
            vec![("result", "pass")],
        );

        let json = log.to_json();
        let restored = EventLog::from_json(&json).expect("round-trip should succeed");
        assert_eq!(restored.events.len(), 2);
        assert_eq!(
            restored.events[1].attributes.get("result"),
            Some(&"pass".to_string())
        );
    }

    #[test]
    fn test_event_log_from_json_invalid() {
        assert!(EventLog::from_json("not valid json").is_err());
    }

    // ── DirectFollowsGraph ────────────────────

    #[test]
    fn test_dfg_discover_counts_correctly() {
        let mut log = EventLog::new();
        log.push("c1", "A", "t1");
        log.push("c1", "B", "t2");
        log.push("c1", "C", "t3");
        log.push("c2", "A", "t4");
        log.push("c2", "B", "t5");

        let dfg = DirectFollowsGraph::discover(&log);
        assert_eq!(dfg.edge_count("A", "B"), 2);
        assert_eq!(dfg.edge_count("B", "C"), 1);
        assert_eq!(dfg.edge_count("A", "C"), 0);
    }

    #[test]
    fn test_dfg_start_and_end_activities() {
        let mut log = EventLog::new();
        log.push("c1", "A", "t1");
        log.push("c1", "B", "t2");
        log.push("c2", "A", "t3");
        log.push("c2", "C", "t4");

        let dfg = DirectFollowsGraph::discover(&log);
        assert_eq!(*dfg.start_activities.get("A").unwrap(), 2);
        assert_eq!(*dfg.end_activities.get("B").unwrap(), 1);
        assert_eq!(*dfg.end_activities.get("C").unwrap(), 1);
    }

    #[test]
    fn test_dfg_edge_count_zero_for_nonexistent() {
        let dfg = DirectFollowsGraph::default();
        assert_eq!(dfg.edge_count("X", "Y"), 0);
    }

    #[test]
    fn test_dfg_most_frequent_path_nonempty() {
        let log = rocket_build_event_log();
        let dfg = DirectFollowsGraph::discover(&log);
        let path = dfg.most_frequent_path();
        assert!(!path.is_empty(), "most_frequent_path should be non-empty");
        assert_eq!(path[0], "load_manifest");
    }

    #[test]
    fn test_dfg_most_frequent_path_follows_pipeline_order() {
        let log = rocket_build_event_log();
        let dfg = DirectFollowsGraph::discover(&log);
        let path = dfg.most_frequent_path();
        // Should start with load_manifest and end with package
        assert_eq!(path.first().map(String::as_str), Some("load_manifest"));
        assert_eq!(path.last().map(String::as_str), Some("package"));
    }

    #[test]
    fn test_dfg_to_dot_contains_activity_names() {
        let log = rocket_build_event_log();
        let dfg = DirectFollowsGraph::discover(&log);
        let dot = dfg.to_dot();
        assert!(dot.contains("load_manifest"));
        assert!(dot.contains("build"));
        assert!(dot.contains("package"));
        assert!(dot.contains("digraph DFG"));
    }

    #[test]
    fn test_dfg_to_json_is_valid_json() {
        let log = rocket_build_event_log();
        let dfg = DirectFollowsGraph::discover(&log);
        let json = dfg.to_json();
        let val: serde_json::Value = serde_json::from_str(&json).expect("should be valid JSON");
        assert!(val["edges"].is_object());
        assert!(val["start_activities"].is_object());
    }

    // ── PetriNet ──────────────────────────────

    #[test]
    fn test_petrinet_add_place_and_transition() {
        let mut net = PetriNet::new();
        net.add_place("p0", "Start");
        net.add_place("p1", "End");
        net.add_transition("t0", Some("act"), &["p0"], &["p1"]);
        assert_eq!(net.places.len(), 2);
        assert_eq!(net.transitions.len(), 1);
        assert_eq!(net.transitions[0].label, Some("act".to_string()));
    }

    #[test]
    fn test_petrinet_enabled_with_initial_marking() {
        let mut net = PetriNet::new();
        net.add_place("p0", "Start")
            .add_place("p1", "End")
            .add_transition("t0", Some("act"), &["p0"], &["p1"])
            .set_initial_marking("p0", 1);

        let enabled = net.enabled_transitions(&net.initial_marking.clone());
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].id, "t0");
    }

    #[test]
    fn test_petrinet_fire_moves_tokens() {
        let mut net = PetriNet::new();
        net.add_place("p0", "Start")
            .add_place("p1", "End")
            .add_transition("t0", Some("act"), &["p0"], &["p1"])
            .set_initial_marking("p0", 1);

        let mut marking = net.initial_marking.clone();
        net.fire(&mut marking, "t0").expect("should fire");

        assert_eq!(marking.get("p0").copied().unwrap_or(0), 0);
        assert_eq!(marking.get("p1").copied().unwrap_or(0), 1);
    }

    #[test]
    fn test_petrinet_fire_returns_error_when_not_enabled() {
        let mut net = PetriNet::new();
        net.add_place("p0", "Start")
            .add_place("p1", "End")
            .add_transition("t0", Some("act"), &["p0"], &["p1"]);

        let mut marking: HashMap<String, usize> = HashMap::new(); // empty marking
        let result = net.fire(&mut marking, "t0");
        assert!(matches!(result, Err(PmError::NotEnabled(_))));
    }

    #[test]
    fn test_petrinet_fire_not_found() {
        let net = PetriNet::new();
        let mut marking = HashMap::new();
        let result = net.fire(&mut marking, "nonexistent");
        assert!(matches!(result, Err(PmError::NotFound(_))));
    }

    #[test]
    fn test_rocket_pipeline_petri_net_structure() {
        let net = rocket_pipeline_petri_net();
        let place_ids: Vec<&str> = net.places.iter().map(|p| p.id.as_str()).collect();
        assert!(place_ids.contains(&"start"), "should have start place");
        assert!(place_ids.contains(&"end"), "should have end place");
        assert_eq!(net.transitions.len(), 5);
        assert_eq!(net.initial_marking.get("start"), Some(&1));
        assert_eq!(net.final_marking.get("end"), Some(&1));
    }

    #[test]
    fn test_rocket_pipeline_petri_net_full_replay() {
        let net = rocket_pipeline_petri_net();
        let mut marking = net.initial_marking.clone();
        net.fire(&mut marking, "t_load_manifest").unwrap();
        net.fire(&mut marking, "t_doctor").unwrap();
        net.fire(&mut marking, "t_audit").unwrap();
        net.fire(&mut marking, "t_build").unwrap();
        net.fire(&mut marking, "t_package").unwrap();
        assert_eq!(marking.get("end").copied().unwrap_or(0), 1);
    }

    // ── ConformanceChecker ────────────────────

    #[test]
    fn test_conformance_trace_fully_conforming() {
        let net = rocket_pipeline_petri_net();
        let checker = ConformanceChecker::new(&net);
        let activities = &["load_manifest", "doctor_check", "audit", "build", "package"];
        let result = checker.check_trace("case1", activities);
        assert_eq!(
            result.fitness, 1.0,
            "fully conforming trace should have fitness 1.0"
        );
        assert!(result.is_fitting);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_conformance_trace_completely_wrong() {
        let net = rocket_pipeline_petri_net();
        let checker = ConformanceChecker::new(&net);
        // Activities not in the model at all
        let activities = &["fly", "jump", "swim", "run"];
        let result = checker.check_trace("case1", activities);
        assert!(
            result.fitness < 1.0,
            "wrong activities should yield fitness < 1.0"
        );
        assert_eq!(result.violations.len(), 4);
    }

    #[test]
    fn test_conformance_trace_partial_fit() {
        let net = rocket_pipeline_petri_net();
        let checker = ConformanceChecker::new(&net);
        // Missing audit step
        let activities = &["load_manifest", "doctor_check", "build", "package"];
        let result = checker.check_trace("case1", activities);
        // build is out-of-order (missing audit), so fitness should be < 1.0
        assert!(result.fitness < 1.0);
    }

    #[test]
    fn test_conformance_is_fitting_threshold() {
        let net = rocket_pipeline_petri_net();
        let checker = ConformanceChecker::new(&net);
        let activities = &["load_manifest", "doctor_check", "audit", "build", "package"];
        let result = checker.check_trace("x", activities);
        assert!(result.is_fitting);

        let bad = checker.check_trace("y", &["unknown_act"]);
        assert!(!bad.is_fitting);
    }

    #[test]
    fn test_conformance_check_log() {
        let net = rocket_pipeline_petri_net();
        let checker = ConformanceChecker::new(&net);
        let log = rocket_build_event_log();
        let result = checker.check_log(&log);
        assert!(result.fitness > 0.0);
    }

    // ── LifecycleTracker ──────────────────────

    #[test]
    fn test_lifecycle_tracker_initial_state() {
        let tracker = LifecycleTracker::new("proj");
        assert_eq!(*tracker.current_phase(), PipelinePhase::Idle);
        assert!(!tracker.is_terminal());
    }

    #[test]
    fn test_lifecycle_tracker_full_transition() {
        let mut t = LifecycleTracker::new("proj");
        t.transition("load_manifest", "t1").unwrap();
        assert_eq!(*t.current_phase(), PipelinePhase::ManifestLoaded);

        t.transition("doctor_check", "t2").unwrap();
        assert_eq!(*t.current_phase(), PipelinePhase::Validated);

        t.transition("build", "t3").unwrap();
        assert_eq!(*t.current_phase(), PipelinePhase::Built);

        t.transition("package", "t4").unwrap();
        assert_eq!(*t.current_phase(), PipelinePhase::Packaged);
        assert!(t.is_terminal());
    }

    #[test]
    fn test_lifecycle_tracker_audit_transition() {
        let mut t = LifecycleTracker::new("proj");
        t.transition("load_manifest", "t1").unwrap();
        // audit is also valid from ManifestLoaded
        t.transition("audit", "t2").unwrap();
        assert_eq!(*t.current_phase(), PipelinePhase::Validated);
    }

    #[test]
    fn test_lifecycle_tracker_invalid_transition() {
        let mut t = LifecycleTracker::new("proj");
        let err = t.transition("build", "t1"); // can't build from Idle
        assert!(err.is_err());
    }

    #[test]
    fn test_lifecycle_tracker_fail_transition() {
        let mut t = LifecycleTracker::new("proj");
        t.transition("load_manifest", "t1").unwrap();
        t.transition("fail", "t2").unwrap();
        assert!(matches!(t.current_phase(), PipelinePhase::Failed(_)));
        assert!(t.is_terminal());
    }

    #[test]
    fn test_lifecycle_tracker_is_terminal_when_packaged() {
        let mut t = LifecycleTracker::new("proj");
        t.transition("load_manifest", "t1").unwrap();
        t.transition("doctor_check", "t2").unwrap();
        t.transition("build", "t3").unwrap();
        t.transition("package", "t4").unwrap();
        assert!(t.is_terminal());
    }

    #[test]
    fn test_lifecycle_tracker_is_terminal_when_failed() {
        let mut t = LifecycleTracker::new("proj");
        t.transition("fail", "t1").unwrap();
        assert!(t.is_terminal());
    }

    #[test]
    fn test_lifecycle_tracker_to_event_log() {
        let mut t = LifecycleTracker::new("Brm");
        t.transition("load_manifest", "2024-01-01T09:00:00Z")
            .unwrap();
        t.transition("doctor_check", "2024-01-01T09:05:00Z")
            .unwrap();
        t.transition("build", "2024-01-01T09:30:00Z").unwrap();
        t.transition("package", "2024-01-01T10:00:00Z").unwrap();

        let log = t.to_event_log();
        assert_eq!(log.events.len(), 4);
        assert!(log.events.iter().all(|e| e.case_id == "Brm"));
    }

    // ── Example logs ──────────────────────────

    #[test]
    fn test_rocket_build_event_log_has_five_cases() {
        let log = rocket_build_event_log();
        let cases = log.cases();
        assert_eq!(cases.len(), 5);
        assert!(cases.contains(&"SurvivalGame"));
        assert!(cases.contains(&"Brm"));
        assert!(cases.contains(&"ShooterGame"));
        assert!(cases.contains(&"FullSpectrum"));
        assert!(cases.contains(&"RealisticRendering"));
    }

    #[test]
    fn test_rocket_build_event_log_activities() {
        let log = rocket_build_event_log();
        let acts = log.activities();
        assert!(acts.contains(&"load_manifest"));
        assert!(acts.contains(&"audit"));
        assert!(acts.contains(&"build"));
        assert!(acts.contains(&"package"));
    }

    #[test]
    fn test_blueprint_authoring_event_log_has_at_least_three_cases() {
        let log = blueprint_authoring_event_log();
        assert!(log.cases().len() >= 3);
    }

    #[test]
    fn test_blueprint_authoring_event_log_activities() {
        let log = blueprint_authoring_event_log();
        let acts = log.activities();
        assert!(acts.contains(&"draft"));
        assert!(acts.contains(&"validate"));
        assert!(acts.contains(&"test"));
    }

    #[test]
    fn test_silent_transition() {
        let mut net = PetriNet::new();
        net.add_place("p0", "Start")
            .add_place("p1", "End")
            .add_transition("tau", None, &["p0"], &["p1"])
            .set_initial_marking("p0", 1);

        assert!(net.transitions[0].is_silent());
        let enabled = net.enabled_transitions(&net.initial_marking.clone());
        assert_eq!(enabled.len(), 1);
    }
}
