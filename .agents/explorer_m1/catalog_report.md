# Ecosystem Catalog and Analysis Report: Typestates, RulePackServer, and ggen

**Author**: Ecosystem Cataloger (explorer_m1)  
**Date**: 2026-06-22  
**Status**: PARTIAL_ALIVE candidate  
**Object under test**: Architectural Patterns and Code Abstractions in `rocket-craft`, `lsp-max`, and `praxis`  

---

## 1. Executive Summary

A detailed investigation of the `rocket-craft`, `lsp-max`, and `praxis` workspaces reveals three core architectural patterns designed to eliminate configuration drift, runtime state violations, and manual boilerplate. These patterns form the foundation of the post-Chatman ecosystem:

1. **Generative Typestates**: Used to enforce software phase transitions (such as data validation and asset admission) at compile time. By utilizing zero-sized marker structs (e.g., `Measured`, `Validated`, `Admitted`) and generic constraints, illegal transitions are rejected at compile time (no-op implementation blocks) rather than causing runtime failures. Furthermore, the **Witness Pattern** (via generic type markers) and **Seal Pattern** (via private fields) enforce constructor purity, preventing callers from bypassing validation rules.
2. **`RulePackServer`**: A bridge trait in `lsp-max` that absorbs the protocol boilerplate of `tower-lsp`. It provides default implementations for text document syncing, workspace indexing, and per-axis diagnostics. Key innovations include:
   - **`EvalBudget`**: Dynamic latency classification (Sync vs. Background) preventing main-thread editor latency spikes.
   - **`WorkspaceIndex` & `CrossFileRule`**: Workspace-wide cross-file diagnostic evaluation and conformance scoring without blocking document edits.
   - **Statistical Process Control (SPC) & Circuit Breakers**: Statistical monitoring of evaluation latency and loop protection.
3. **`ggen` Micro-Pipeline**: The code generation engine lowerer that translates RDF/TTL ontology models into compile-ready code artifacts ($A = \mu(O^*)$). It operates in discrete, sequential stages: Loading, Construct (Inference), Validation (SHACL & SPARQL ASK), Extract (SELECT), Template Lowering (Tera rendering with LLM-backed fallback), and Receipt Generation (BLAKE3 content addressing).

---

## 2. Rust Libraries Catalog

The following table catalogs the key Rust libraries, crates, and modules across the three investigated workspaces:

| Workspace | Crate / Path | Purpose & Key Modules | Core Patterns Observed |
| :--- | :--- | :--- | :--- |
| **rocket-craft** | `crates/mech_morphology_law` | Enforces bipedal mecha socket connections and metric bounds rules (`MeasuredMech`, `MorphologyLaw` trait). | Generative Typestates (`Machine<L, P>`), zero-sized marker phases. |
| **rocket-craft** | `crates/ggen-asset-lsp` | LSP server for asset linting (e.g. VIS200, USD300). Contains code actions parsing `# ggen-source` annotations. | `lsp-max` client adapter integration. |
| **rocket-craft** | `crates/rocket_preue4_verifier` | Handles pre-UE4 visual verification, scoring visual discrepancies, and checking assets. | Playwright integration, OCEL event logging. |
| **rocket-craft** | `asset-pipeline/pipeline-core` | Core models and types for mecha asset compilation. | Structural serializers, USD generation mappings. |
| **rocket-craft** | `blueprint-rs/blueprint-core` | Processes mecha blueprint files and query specifications. | Schema parsing, SPARQL bindings. |
| **rocket-craft** | `nexus-engine/crates/nexus-mecha` | Simulation physics, properties, and bipedal joints. | Typestates for state machine gameplay components. |
| **rocket-craft** | `gmf-ocel-adapter` | Formats mecha assembly activities into OCEL 2.0. | OCEL logging, JSON serialization. |
| **lsp-max** | `lsp-max-protocol` | Custom protocol specifications (`LawAxis`, `MaxDiagnostic`, `ConformanceVector`). | Conformance vector data modeling. |
| **lsp-max** | `lsp-max-runtime` | Evaluates diagnostics into conformance score matrices. | `build_conformance_vector` aggregation logic. |
| **lsp-max** | `lsp-max-ast` | Document store and tree-sitter incremental parsing. | `AutoLspAdapter` integration. |
| **lsp-max** | Core crate / `src/rule_pack_server.rs` | Enforces rules via standard `RulePackServer` trait, `WorkspaceIndex`, and `WorkspaceRuleEvaluator`. | `RulePackServer`, `EvalBudget` reclass, `WorkspaceIndex`. |
| **lsp-max** | `examples/anti-llm-cheat-lsp` | Detects CalVer and dependency violations. | Custom regex/string scanning, AhoCorasick scanning. |
| **lsp-max** | `examples/pattern-lsp` | Implements rule pack server loading TOML patterns. | TOML rule deserialization, regex-based scanning. |
| **praxis** | `crates/chatman-common` | Shared utilities including Otel tracing, errors, and BLAKE3. | Content-addressing helper methods (`blake3::hash`). |
| **praxis** | `template` | Skeleton directory for scaffolding single or multi-crates. | Baseline CalVer settings, `Evidence` typestate models. |

---

## 3. Code Abstractions

### A. Generative Typestates (Crates: `mech_morphology_law`, `praxis/template`)

The typestate pattern observed in `crates/mech_morphology_law/src/machine.rs` enforces compile-time phase transitions. An object starts as `Measured`, transitions to `Validated`, and can only reach `Admitted` if validation checks succeed. 

#### 1. Transition Enforced by Constructor Consumption
```rust
use core::marker::PhantomData;

// Zero-Sized marker types (ZSTs) representing states
#[derive(Debug)]
pub struct Measured;
#[derive(Debug)]
pub struct Validated;
#[derive(Debug)]
pub struct Admitted;

// The typestate admission machine
#[derive(Debug)]
pub struct Machine<L, P> {
    law: L,
    mech: MeasuredMech,
    outcome: Option<LawOutcome>,
    _phase: PhantomData<P>,
}

// Construction starts at the Measured state
impl<L: MorphologyLaw> Machine<L, Measured> {
    pub fn with_law(law: L, mech: MeasuredMech) -> Self {
        Machine {
            law,
            mech,
            outcome: None,
            _phase: PhantomData,
        }
    }

    // The only legal transition from Measured is validate.
    // Consumes `self` (preventing reuse) and yields a Validated machine.
    pub fn validate(self) -> Machine<L, Validated> {
        let outcome = self.law.validate(&self.mech);
        Machine {
            law: self.law,
            mech: self.mech,
            outcome: Some(outcome),
            _phase: PhantomData,
        }
    }
}
```

#### 2. Gated Transition Enforced by Result Types
```rust
impl<L: MorphologyLaw> Machine<L, Validated> {
    // The gated admission transition. Available ONLY on Validated phase.
    // Checks standing at runtime; returns Machine<L, Admitted> on success,
    // or a ClaimHold on failure. There is no method to construct Admitted directly.
    pub fn admit(self) -> Result<Machine<L, Admitted>, ClaimHold> {
        let outcome = self.outcome.clone().unwrap_or_default();
        match &outcome.standing {
            Standing::Admitted => Ok(Machine {
                law: self.law,
                mech: self.mech,
                outcome: self.outcome,
                _phase: PhantomData,
            }),
            other => Err(ClaimHold {
                standing: other.clone(),
                reason: outcome.refusals.clone(),
            }),
        }
    }
}
```

#### 3. The Witness & Seal Patterns
Observed in `praxis/template/src/types.rs`:
- **Witness**: Ensures that only a module holding the `Admit` trait implementation can wrap a type into the `Admitted` state via `admit_unchecked()` (which is kept private/restricted).
- **Seal**: Structs carry a private `_seal: ()` field preventing direct instantiation (E0451 compile error) outside of their declaring module.

```rust
// Seal Pattern
pub struct Receipt {
    pub events: Vec<Event>,
    pub chain_hash: String,
    _seal: (), // Private field prevents struct-literal construction externally
}

// Witness Pattern
pub struct Evidence<T, S, W> {
    inner: T,
    _state: PhantomData<S>,
    _witness: PhantomData<W>,
}

impl<T, W> Evidence<T, Admitted, W> {
    // Private constructor: only Admit impls within the crate can construct this.
    pub(crate) fn admit_unchecked(inner: T) -> Self {
        Self { inner, _state: PhantomData, _witness: PhantomData }
    }
}
```

---

### B. `RulePackServer` (Crates: `lsp-max`, `anti-llm-cheat-lsp`, `pattern-lsp`)

The `RulePackServer` trait in `lsp-max/src/rule_pack_server.rs` acts as a protocol wrapper. The server backend defines its custom types, parser grammars, and rule configurations, while the trait supplies default implementations for text document syncing, diagnostics publishing, and workspace index aggregation.

```rust
#[allow(async_fn_in_trait)]
pub trait RulePackServer {
    /// The rule packs this server enforces, pre-validated for conflicts.
    fn rule_packs(&self) -> &ValidatedRulePackSet;

    /// Optional cross-file rules. Default: empty.
    fn cross_file_rules(&self) -> &[CrossFileRule] { &[] }

    /// The tree-sitter grammar used for incremental parsing.
    fn grammar(&self) -> tree_sitter::Language;

    /// A stable, human-readable server identifier.
    fn server_name(&self) -> &'static str;

    /// The Client handle for push-publishing diagnostics.
    fn client(&self) -> &crate::service::Client;

    /// The AutoLspAdapter that owns the incremental document store.
    fn adapter(&self) -> &AutoLspAdapter;

    /// The shared workspace index. Return Some(&self.index) for cross-file rules.
    fn workspace_index(&self) -> Option<&WorkspaceIndex> { None }

    // -- Primitives delegation --
    fn spc_monitor(&self) -> Option<&std::sync::Mutex<SpcMonitor>> { None }
    fn latency_trackers(&self) -> Option<&Arc<DashMap<String, RuleLatencyTracker>>> { None }
    fn rule_circuit_breaker(&self) -> Option<&Arc<parking_lot::Mutex<CircuitBreaker>>> { None }

    // -- Default implementations (LSP lifecycles) --
    fn server_capabilities(&self) -> ServerCapabilities { ... }
    async fn handle_did_open(&self, params: DidOpenTextDocumentParams) { ... }
    async fn handle_did_change(&self, params: DidChangeTextDocumentParams) { ... }
    
    // -- Scanning & Evaluation --
    fn scan_uri_classified(&self, uri: &DocumentUri, content: &str) -> ClassifiedFindings {
        // Enforces circuit breaker, matches Regex patterns, tracks latency, 
        // and dynamically shifts slow Sync rules to Background Tokio tasks.
    }
    
    // -- Conformance vectors --
    fn workspace_conformance(&self) -> ConformanceVector {
        // Aggregates file conformance vectors. If any file is refused, 
        // the workspace is refused. Unknown axes do not collapse.
    }
}
```

---

### C. `ggen` µ-Pipeline (Crates: `ggen-core`)

The generation pipeline in `ggen-core/src/codegen/pipeline.rs` runs the core sequence from ontology to source file:

```text
Load Ontology (TTL) 
  → Execute Inference Rules (CONSTRUCT) 
  → Run SHACL Validation & SPARQL ASK Validation
  → Execute Generation Rules (SELECT -> Tera Template Rendering)
  → Apply Poka-Yoke & Unsafe Checks 
  → Atomic Commit (FileTransaction) & Receipt Generation
```

Key structures observed in the code:

```rust
pub struct GenerationPipeline {
    manifest: GgenManifest,
    base_path: PathBuf,
    ontology_graph: Option<Graph>, // Oxigraph graph
    executed_rules: Vec<ExecutedRule>,
    generated_files: Vec<GeneratedFile>,
    validation_results: Vec<ValidationResult>,
    started_at: Instant,
    force_overwrite: bool,
    llm_service: Option<Box<dyn LlmService>>,
}
```

#### The Six Pipeline Stages
1. **Loading (`load_ontology`)**: Loads ontology source files (e.g. `all_merged.ttl`) and package imports into an oxigraph-backed memory `Graph`.
2. **Construct (`execute_inference_rules`)**: Runs SPARQL `CONSTRUCT` queries to derive new triples. Enforces `GGEN-INFER-001` (aborts if a rule adds zero triples in strict mode to detect faulty queries).
3. **Validation (`execute_shacl_validation`, `execute_validation_rules`)**: Evaluates SHACL shape TTLs and custom SPARQL `ASK` invariants. Polarity: `ASK = true` is valid; `ASK = false` raises a validation violation. If any `Error`-severity violations exist, generation aborts before files are written.
4. **Extract (`execute_generation_rules`)**: Executes SPARQL `SELECT` queries to retrieve bindings. Enforces the presence of an `ORDER BY` clause to guarantee strict compilation determinism.
5. **Template Lowering (`execute_generation_rules` loop)**: Renders Tera templates. If `output_file` contains `{{`, it runs in dynamic fan-out mode (one rendered output file per row). If static, it folds rows into an aggregate context. Supports optional LLM-based skill implementations with static stub fallback.
6. **Receipts & Poka-Yoke (`validate_generated_output`)**: Performs file-level validation checks (must be non-empty, under 10MB, and contain no path traversal `../`). If `no_unsafe` is enabled, scans output for Rust `unsafe` keyword usage. Files are written atomically using a `FileTransaction` which outputs cryptographic hashes.

---

## 4. Praxis Integration Plan

To align the `praxis` template generator (`~/praxis/template`) with the post-Chatman ecosystem, we propose injecting these patterns directly into the template skeleton. This will allow every new project generated with `cargo generate` to natively inherit these abstractions.

### A. Template Cargo.toml Upgrades (`template/Cargo.toml`)
Expose optional features and dependencies for LSP and custom code generation:

```toml
[features]
default   = []
typestate = []
lsp       = ["dep:lsp-max", "dep:tree-sitter", "dep:tokio", "dep:dashmap", "dep:parking_lot"]
ggen      = []

[dependencies]
# Standard dependencies ...
dashmap     = { version = "6", optional = true }
parking_lot = { version = "0.12", optional = true }
lsp-max     = { git = "https://github.com/seanchatmangpt/lsp-max", optional = true }
tree-sitter = { version = "0.22", optional = true }
tokio       = { version = "1", features = ["full"], optional = true }
```

### B. Scaffold Boilerplate for Generative Typestates (`template/src/types.rs`)
Inject standardized helper traits and state wrappers to make typestate modeling trivial for developers:

```rust
// In template/src/types.rs:
use std::marker::PhantomData;

/// Sealed module to prevent external state implementation.
mod sealed {
    pub trait LifecycleState {}
}

/// Baseline unvalidated raw evidence.
pub struct Raw;
impl sealed::LifecycleState for Raw {}

/// Terminal admitted state.
pub struct Admitted;
impl sealed::LifecycleState for Admitted {}

/// Enforces compile-time transition bounds.
pub struct Evidence<T, S: sealed::LifecycleState, W> {
    inner: T,
    _state: PhantomData<S>,
    _witness: PhantomData<W>,
}

impl<T, W> Evidence<T, Raw, W> {
    pub fn raw(inner: T) -> Self {
        Self { inner, _state: PhantomData, _witness: PhantomData }
    }
    pub fn inner(&self) -> &T { &self.inner }
}

impl<T, W> Evidence<T, Admitted, W> {
    pub fn inner(&self) -> &T { &self.inner }
    // Only constructible within the crate by implementing Admit
    pub(crate) fn admit_unchecked(inner: T) -> Self {
        Self { inner, _state: PhantomData, _witness: PhantomData }
    }
}

/// The standard gatekeeper trait.
pub trait Admit {
    type Input;
    type Witness;
    type Reason;

    fn admit(
        input: Evidence<Self::Input, Raw, Self::Witness>,
    ) -> Result<Evidence<Self::Input, Admitted, Self::Witness>, Self::Reason>;
}
```

### C. Scaffold LSP Module with `RulePackServer` (`template/src/lsp.rs`)
Create a default language server skeleton in the template that uses the `lsp-max` abstractions:

```rust
// In template/src/lsp.rs (active when `feature = "lsp"` is enabled):
use lsp_max::rule_pack_server::{RulePackServer, ValidatedRulePackSet, WorkspaceIndex, PrimitivesBundle};
use lsp_max_ast::AutoLspAdapter;

pub struct AppLspServer {
    packs: ValidatedRulePackSet,
    grammar: tree_sitter::Language,
    adapter: AutoLspAdapter,
    client: lsp_max::service::Client,
    index: WorkspaceIndex,
    primitives: PrimitivesBundle,
}

impl AppLspServer {
    pub fn new(client: lsp_max::service::Client) -> Self {
        Self {
            packs: ValidatedRulePackSet::empty(),
            grammar: tree_sitter_rust::LANGUAGE.into(), // template default
            adapter: AutoLspAdapter::new_default(),
            client,
            index: WorkspaceIndex::new(),
            primitives: PrimitivesBundle::new(),
        }
    }
}

impl RulePackServer for AppLspServer {
    fn rule_packs(&self) -> &ValidatedRulePackSet { &self.packs }
    fn grammar(&self) -> tree_sitter::Language { self.grammar.clone() }
    fn server_name(&self) -> &'static str { "{{project-name}}-lsp" }
    fn client(&self) -> &lsp_max::service::Client { &self.client }
    fn adapter(&self) -> &AutoLspAdapter { &self.adapter }
    fn workspace_index(&self) -> Option<&WorkspaceIndex> { Some(&self.index) }
    
    fn spc_monitor(&self) -> Option<&std::sync::Mutex<lsp_max::primitives::SpcMonitor>> {
        Some(self.primitives.spc_monitor_ref())
    }
    fn latency_trackers(&self) -> Option<&std::sync::Arc<dashmap::DashMap<String, lsp_max::primitives::RuleLatencyTracker>>> {
        Some(self.primitives.latency_trackers_ref())
    }
    fn rule_circuit_breaker(&self) -> Option<&std::sync::Arc<parking_lot::Mutex<lsp_max::primitives::CircuitBreaker>>> {
        Some(self.primitives.circuit_breaker_ref())
    }
}
```

### D. Enhance Starter `ggen.toml` and Ontology (`template/ggen.toml`)
The starter `ggen.toml` should contain boilerplate examples demonstrating:
- An inference rule (`construct`) that derives semantic relationships.
- A custom validation rule (`ask` query) checking database consistency or schema shape.
- A code generation rule linking a query to a Tera template (scaffolding code automatically).

These additions will be checked by a new programmatic validation script (`tools/hollow-gate/main.rs` or `src/bin/dod.rs`) that verifies the generated cargo project compiles and adheres to typestate structures.

---

## 5. Verification and Auditing

### Verified Aspects
- Observed the compilation-gated phase transition mechanics in `crates/mech_morphology_law/src/machine.rs` where type marker parameters (`Measured`, `Validated`, `Admitted`) prevent invalid actions at compile time.
- Observed the `RulePackServer` implementation in `lsp-max/src/rule_pack_server.rs`, verifying features like `EvalBudget` (Sync/Background), dynamic reclassification, workspace indexing, and SPC control monitoring.
- Traced the `ggen` pipeline execution structure in `ggen-core/src/codegen/pipeline.rs` verifying stages from loading ontology, running CONSTRUCT rules, checking validation ASK queries, running SELECT queries, and executing Tera generation templates.

### Unverified Aspects
- Performance limits under multi-gigabyte ontology structures for `ggen` (our mecha domain ontology is of limited scale).
- Integration test verification for the proposed `AppLspServer` within a generated template workspace (since template changes are not yet written, they remain candidates).

### Key Assumptions & Risks
- **Assumption**: The project will remain on Rust 1.82 (or compatible) where `std::marker::PhantomData` remains zero-cost.
- **Risk**: Adding `lsp-max` as a dependency in templates increases the initial compile time of the generated project due to `tree-sitter` and serialization dependencies. This is mitigated by putting it behind the optional `lsp` feature flag.

---

**Status:** PARTIAL_ALIVE  
**Verdict:** candidate  
**Receipt Required:** Successful generation of a boilerplate crate using the updated praxis templates, compile check passing, and verification script admitting the typestate structure.  
**Residuals:** Manual testing of `AppLspServer` in an editor environment remains unverified.  
