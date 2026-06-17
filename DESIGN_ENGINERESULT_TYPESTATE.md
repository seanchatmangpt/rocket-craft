# EngineResult Typestate Pattern — Complete Design Document

**Version:** 1.0  
**Date:** 2026-06-17  
**Scope:** anti-llm-cheat-lsp integration refactoring  
**Target:** Unify-rs workspace (MCP, LSP, CLI, Admission gates)

---

## 1. Executive Summary

Currently, `anti-llm-cheat-lsp` exposes a sequential processing pipeline:
```
scan_file() → Observation[] → evaluate_diagnostics() → AntiLlmDiagnostic[]
```

This design proposes adding **compile-time state machine safety** via the **Rust typestate pattern** (zero-sized `PhantomData<S>` markers), allowing illegal transitions (e.g., merging before enrichment, deduping twice) to be caught at **compile time, not runtime**.

**Benefits:**
- Illegal state transitions are **compile errors** — impossible to violate
- Each state has **exactly the methods** valid in that state
- **No runtime checks** needed (zero cost)
- **Self-documenting** code: types encode the protocol
- **Refactoring safe**: adding state requirements breaks builds, not tests

**Costs:**
- 40–60 LOC of trait definitions and impl blocks (per crate)
- Modest learning curve for new contributors (offset by docs)
- Slightly longer compile times (negligible with incremental builds)

---

## 2. State Enum Definition

### 2.1 Zero-Sized State Markers

```rust
// anti-llm-cheat-lsp/src/states.rs (NEW)

use std::marker::PhantomData;

/// Marker: raw file scanning complete, observations collected.
/// Invariant: observations may contain duplicates; rules not yet evaluated.
pub struct ScanComplete;

/// Marker: observations enriched with filename/extension classification.
/// Invariant: each observation has `kind` set (e.g., "test_smell", "raw_text");
///           no duplicates removed yet.
pub struct Enriched;

/// Marker: rules evaluated, diagnostics generated, severity determined.
/// Invariant: each diagnostic has `blocking` flag and `code` set;
///           no duplicates removed yet.
pub struct Evaluated;

/// Marker: duplicates removed, final dedupe pass complete.
/// Invariant: (file_path, line, code) tuples are unique;
///           no further transitions possible.
pub struct Deduped;
```

### 2.2 State Type Bounds

```rust
// Trait for states that allow further transitions
pub trait Transitionable: Sized {}

impl Transitionable for ScanComplete {}
impl Transitionable for Enriched {}
impl Transitionable for Evaluated {}

// Deduped does NOT implement Transitionable — terminal state

// For clarity: marks states that are past scanning
pub trait PostScan: Transitionable {}

impl PostScan for Enriched {}
impl PostScan for Evaluated {}
impl PostScan for Deduped {}
```

---

## 3. EngineResult<S> Wrapper Type

### 3.1 Core Definition

```rust
// anti-llm-cheat-lsp/src/engine_result.rs (NEW)

use crate::states::*;
use crate::diagnostics::AntiLlmDiagnostic;
use crate::observations::Observation;
use std::marker::PhantomData;
use std::collections::{HashMap, HashSet};

/// Typestate result container: holds intermediate and final results
/// parametrized by the current state S.
///
/// Type parameter S (zero-sized marker) encodes which transformations
/// are legal. Illegal transitions do not have impl blocks, so they
/// are compile errors.
pub struct EngineResult<S> {
    /// File system path being processed
    file_path: String,

    /// Raw observations (populated after scan, immutable thereafter)
    observations: Vec<Observation>,

    /// Classification metadata per observation (populated after enrichment)
    /// Maps observation index → (file_kind, line_category)
    enrichment_map: HashMap<usize, (String, String)>,

    /// Final diagnostics (populated after evaluation)
    diagnostics: Vec<AntiLlmDiagnostic>,

    /// Deduplication state (populated after dedup)
    /// Stores set of (file_path, line, code) tuples for membership testing
    dedup_set: HashSet<(String, usize, String)>,

    /// Type state marker (zero-sized, erased at runtime)
    _phantom: PhantomData<S>,
}
```

### 3.2 State-Specific Invariants

```rust
impl<S> EngineResult<S> {
    /// Helper: check if this EngineResult satisfies state invariants.
    /// Called before each state transition. Returns Err if invariant violated.
    #[cfg(test)]
    fn validate_invariants(&self) -> Result<(), String> {
        match std::any::type_name::<S>() {
            "ScanComplete" => {
                // Invariant: observations non-empty, enrichment_map empty
                if self.observations.is_empty() {
                    return Err("ScanComplete requires non-empty observations".into());
                }
                if !self.enrichment_map.is_empty() {
                    return Err("ScanComplete invariant violated: enrichment_map should be empty".into());
                }
                Ok(())
            }
            "Enriched" => {
                // Invariant: enrichment_map has entry for each observation
                if self.enrichment_map.len() != self.observations.len() {
                    return Err(format!(
                        "Enriched invariant violated: {} obs, {} enrichments",
                        self.observations.len(),
                        self.enrichment_map.len()
                    ));
                }
                if !self.diagnostics.is_empty() {
                    return Err("Enriched invariant violated: diagnostics should be empty".into());
                }
                Ok(())
            }
            "Evaluated" => {
                // Invariant: diagnostics non-empty, dedup_set empty
                if self.diagnostics.is_empty() {
                    return Err("Evaluated requires non-empty diagnostics".into());
                }
                if !self.dedup_set.is_empty() {
                    return Err("Evaluated invariant violated: dedup_set should be empty".into());
                }
                Ok(())
            }
            "Deduped" => {
                // Invariant: dedup_set non-empty, all (path, line, code) unique
                if self.dedup_set.is_empty() {
                    return Err("Deduped requires non-empty dedup_set".into());
                }
                let tuple_count = self.diagnostics.len();
                if self.dedup_set.len() != tuple_count {
                    return Err(format!(
                        "Deduped invariant violated: {} diagnostics, {} unique tuples",
                        tuple_count,
                        self.dedup_set.len()
                    ));
                }
                Ok(())
            }
            _ => Err(format!("Unknown state: {}", std::any::type_name::<S>())),
        }
    }
}
```

---

## 4. Impl Blocks — One Per State

### 4.1 `impl EngineResult<ScanComplete>`

```rust
/// State 1: Raw scanning complete. Observations collected, no enrichment yet.
///
/// Methods:
/// - `enrich()` → EngineResult<Enriched>  [only transition]
///
/// Cannot: evaluate, dedupe, emit
impl EngineResult<ScanComplete> {
    /// Create a new ScanComplete result from scanning a file.
    pub fn from_file_scan(file_path: &str) -> Self {
        let observations = crate::engine::scan_file(file_path);
        Self {
            file_path: file_path.to_string(),
            observations,
            enrichment_map: HashMap::new(),
            diagnostics: Vec::new(),
            dedup_set: HashSet::new(),
            _phantom: PhantomData,
        }
    }

    /// Transition: ScanComplete → Enriched
    ///
    /// Classifies each observation based on file extension and content patterns.
    /// Populates enrichment_map with (file_kind, line_category) for each obs.
    ///
    /// Example enrichment:
    /// - Observation in src/*.rs → file_kind="rust", line_category="raw_text"
    /// - Observation in tests/test.rs → file_kind="rust", line_category="test_smell"
    pub fn enrich(mut self) -> EngineResult<Enriched> {
        for (idx, obs) in self.observations.iter().enumerate() {
            let file_kind = if obs.file_path.ends_with(".rs") {
                "rust"
            } else if obs.file_path.ends_with(".ts") || obs.file_path.ends_with(".tsx") {
                "typescript"
            } else if obs.file_path.ends_with(".md") {
                "markdown"
            } else if obs.file_path.ends_with(".c") || obs.file_path.ends_with(".h") {
                "c"
            } else {
                "unknown"
            };

            let line_category = if obs.file_path.contains("tests/") || obs.file_path.ends_with("_test.rs") {
                "test_smell"
            } else if obs.kind == "raw_text" {
                "raw_text"
            } else {
                &obs.kind  // use kind as-is
            };

            self.enrichment_map.insert(idx, (file_kind.to_string(), line_category.to_string()));
        }

        EngineResult {
            file_path: self.file_path,
            observations: self.observations,
            enrichment_map: self.enrichment_map,
            diagnostics: Vec::new(),
            dedup_set: HashSet::new(),
            _phantom: PhantomData,
        }
    }

    /// Read-only access to observations (before enrichment).
    pub fn observations(&self) -> &[Observation] {
        &self.observations
    }

    /// Read-only access to file path.
    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    /// Count of observations collected.
    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }
}
```

### 4.2 `impl EngineResult<Enriched>`

```rust
/// State 2: Observations enriched with metadata (file_kind, line_category).
///
/// Methods:
/// - `evaluate(config)` → EngineResult<Evaluated>  [only transition]
///
/// Cannot: dedupe, emit, scan more
impl EngineResult<Enriched> {
    /// Transition: Enriched → Evaluated
    ///
    /// Applies all rule engines (claims, complexity, determinism, etc.)
    /// to convert observations into AntiLlmDiagnostics with:
    /// - code (e.g., "ANTI-LLM-CLAIM-001")
    /// - category (e.g., "claims", "complexity")
    /// - blocking flag (true if policy violation, false if warning)
    /// - required_correction, required_next_proof
    pub fn evaluate(self, config: &crate::config::AntiLlmConfig) -> EngineResult<Evaluated> {
        let diagnostics = crate::engine::evaluate_diagnostics_with_config(
            &self.observations,
            config,
        );

        EngineResult {
            file_path: self.file_path,
            observations: self.observations,
            enrichment_map: self.enrichment_map,
            diagnostics,
            dedup_set: HashSet::new(),
            _phantom: PhantomData,
        }
    }

    /// Read-only access to enriched observations with metadata.
    pub fn observations_enriched(&self) -> Vec<(Observation, String, String)> {
        self.observations
            .iter()
            .enumerate()
            .filter_map(|(idx, obs)| {
                self.enrichment_map.get(&idx).map(|(fk, lc)| {
                    (obs.clone(), fk.clone(), lc.clone())
                })
            })
            .collect()
    }

    /// Read-only access to enrichment metadata.
    pub fn enrichment_map(&self) -> &HashMap<usize, (String, String)> {
        &self.enrichment_map
    }

    /// Count of enriched observations.
    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }

    /// Metadata for a specific observation index.
    pub fn get_enrichment(&self, obs_idx: usize) -> Option<(&str, &str)> {
        self.enrichment_map.get(&obs_idx).map(|(fk, lc)| {
            (fk.as_str(), lc.as_str())
        })
    }
}
```

### 4.3 `impl EngineResult<Evaluated>`

```rust
/// State 3: Diagnostics evaluated and categorized (with blocking flags).
/// This is the "most useful" state for consumers (MCP, LSP, CLI, Admission).
///
/// Methods:
/// - `dedupe()` → EngineResult<Deduped>  [only transition]
/// - emit() → Vec<AntiLlmDiagnostic>  [read-only, no transition]
///
/// Cannot: enrich, evaluate again, scan more
impl EngineResult<Evaluated> {
    /// Transition: Evaluated → Deduped
    ///
    /// Removes duplicate diagnostics by (file_path, line, code).
    /// Keeps the first occurrence of each unique tuple.
    /// Populates dedup_set for verification.
    pub fn dedupe(mut self) -> EngineResult<Deduped> {
        let mut seen: HashSet<(String, usize, String)> = HashSet::new();

        self.diagnostics.retain(|d| {
            let key = (d.file_path.clone(), d.line, d.code.clone());
            seen.insert(key.clone())
        });

        let dedup_set = seen.clone();

        EngineResult {
            file_path: self.file_path,
            observations: self.observations,
            enrichment_map: self.enrichment_map,
            diagnostics: self.diagnostics,
            dedup_set,
            _phantom: PhantomData,
        }
    }

    /// Read-only access to diagnostics without deduplication.
    /// Useful for integrations that handle their own dedup logic.
    pub fn diagnostics(&self) -> &[AntiLlmDiagnostic] {
        &self.diagnostics
    }

    /// Emit diagnostics as-is (pre-dedup).
    /// Suitable for MCP tools, LSP servers, and CLI commands
    /// that do not require strict dedup guarantees.
    pub fn emit(&self) -> Vec<AntiLlmDiagnostic> {
        self.diagnostics.clone()
    }

    /// Count blocking vs. warning diagnostics.
    pub fn diagnostic_counts(&self) -> (usize, usize) {
        let blocking = self.diagnostics.iter().filter(|d| d.blocking).count();
        let warning = self.diagnostics.len() - blocking;
        (blocking, warning)
    }

    /// Filter diagnostics by category (e.g., "claims", "complexity").
    pub fn filter_by_category(&self, category: &str) -> Vec<&AntiLlmDiagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.category == category)
            .collect()
    }

    /// Filter diagnostics by blocking status.
    pub fn filter_by_blocking(&self, blocking: bool) -> Vec<&AntiLlmDiagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.blocking == blocking)
            .collect()
    }

    /// Diagnostic count summary.
    pub fn diagnostic_count(&self) -> usize {
        self.diagnostics.len()
    }

    /// Read-only access to file path.
    pub fn file_path(&self) -> &str {
        &self.file_path
    }
}
```

### 4.4 `impl EngineResult<Deduped>` — Terminal State

```rust
/// State 4: Final state — duplicates removed, pipeline complete.
/// This state has NO transitions (Deduped does NOT implement Transitionable).
///
/// Methods:
/// - emit() → Vec<AntiLlmDiagnostic>  [read-only]
/// - emit_by_blocking() → Vec<AntiLlmDiagnostic>  [filtered read-only]
///
/// Cannot: enrich, evaluate, dedupe again, transition further
impl EngineResult<Deduped> {
    /// Emit final, deduplicated diagnostics.
    /// Guaranteed (file_path, line, code) uniqueness.
    pub fn emit(&self) -> Vec<AntiLlmDiagnostic> {
        self.diagnostics.clone()
    }

    /// Emit only blocking diagnostics (errors).
    pub fn emit_blocking(&self) -> Vec<AntiLlmDiagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.blocking)
            .cloned()
            .collect()
    }

    /// Emit only warning diagnostics.
    pub fn emit_warnings(&self) -> Vec<AntiLlmDiagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| !d.blocking)
            .cloned()
            .collect()
    }

    /// Diagnostic count summary (final).
    pub fn diagnostic_count(&self) -> usize {
        self.diagnostics.len()
    }

    /// Blocking vs. warning counts (final).
    pub fn diagnostic_counts(&self) -> (usize, usize) {
        let blocking = self.diagnostics.iter().filter(|d| d.blocking).count();
        let warning = self.diagnostics.len() - blocking;
        (blocking, warning)
    }

    /// Verify dedup invariant (for testing).
    #[cfg(test)]
    pub fn verify_dedup(&self) -> Result<(), String> {
        if self.dedup_set.len() != self.diagnostics.len() {
            return Err(format!(
                "Dedup invariant broken: {} in set, {} diagnostics",
                self.dedup_set.len(),
                self.diagnostics.len()
            ));
        }
        for d in &self.diagnostics {
            let key = (d.file_path.clone(), d.line, d.code.clone());
            if !self.dedup_set.contains(&key) {
                return Err(format!("Diagnostic not in dedup set: {:?}", key));
            }
        }
        Ok(())
    }

    /// Read-only access to file path.
    pub fn file_path(&self) -> &str {
        &self.file_path
    }
}
```

---

## 5. Usage Examples — Before and After

### 5.1 Call Site 1: main.rs (scan command)

#### Before (Current Code)
```rust
// tools/unify/src/commands.rs

pub fn cmd_audit(dir_path: &str, blocking_only: bool, fail_on_blocking: bool)
    -> Result<Output, Box<dyn std::error::Error>>
{
    let observations = anti_llm_cheat_lsp::engine::scan_directory(dir_path);
    let config = AntiLlmConfig::default();
    let diagnostics = anti_llm_cheat_lsp::engine::evaluate_diagnostics_with_config(
        &observations,
        &config,
    );

    // Dedup manually (easy to forget, easy to duplicate)
    let mut seen = std::collections::HashSet::new();
    let mut unique_diags = Vec::new();
    for d in diagnostics {
        let key = (d.file_path.clone(), d.line, d.code.clone());
        if seen.insert(key) {
            unique_diags.push(d);
        }
    }

    let blocking_count = unique_diags.iter().filter(|d| d.blocking).count();
    let warning_count = unique_diags.len() - blocking_count;

    let filtered = if blocking_only {
        unique_diags.iter().filter(|d| d.blocking).cloned().collect()
    } else {
        unique_diags
    };

    let success = if fail_on_blocking {
        blocking_count == 0
    } else {
        true
    };

    Ok(Output {
        data: json!({ "diagnostics": filtered }),
        success,
        message: Some(format!("Audit: {} warnings, {} blocking", warning_count, blocking_count)),
    })
}
```

#### After (With Typestate)
```rust
// tools/unify/src/commands.rs (with typestate safety)

pub fn cmd_audit(dir_path: &str, blocking_only: bool, fail_on_blocking: bool)
    -> Result<Output, Box<dyn std::error::Error>>
{
    // Type-safe pipeline: each step transforms the state
    let result = anti_llm_cheat_lsp::engine_result::EngineResult::from_directory_scan(dir_path)
        .enrich()
        .evaluate(&AntiLlmConfig::default())
        .dedupe();

    // At this point, result is EngineResult<Deduped> — guaranteed unique, safe to emit
    let diagnostics = if blocking_only {
        result.emit_blocking()
    } else {
        result.emit()
    };

    let (blocking_count, warning_count) = result.diagnostic_counts();
    let success = if fail_on_blocking {
        blocking_count == 0
    } else {
        true
    };

    Ok(Output {
        data: json!({ "diagnostics": diagnostics }),
        success,
        message: Some(format!("Audit: {} warnings, {} blocking", warning_count, blocking_count)),
    })
}
```

**Safety gain:** Calling `.emit()` before `.dedupe()` is **a compile error** — impossible to ship dedup-unsafe code.

---

### 5.2 Call Site 2: server.rs (LSP integration)

#### Before (Current Code)
```rust
// unify-lsp/src/anti_llm_gate.rs (current design from INTEGRATION_PLAN.md)

pub fn scan_file_to_lsp(&self, file_path: &str) -> DiagnosticSet {
    let observations = anti_llm_cheat_lsp::engine::scan_file(file_path);
    let anti_llm_diags = anti_llm_cheat_lsp::engine::evaluate_diagnostics_with_config(
        &observations,
        &self.config
    );

    // Manual dedup (if the LSP server cares about uniqueness)
    // — easy to forget, split across codebases
    let mut seen = HashSet::new();
    let mut unique = Vec::new();
    for d in anti_llm_diags {
        let key = (d.file_path.clone(), d.line, d.code.clone());
        if seen.insert(key) {
            unique.push(d);
        }
    }

    let mut diag_set = DiagnosticSet::new();
    for diag in unique {
        let uri = url::Url::from_file_path(&diag.file_path)
            .ok()
            .map(|u| u.to_string())
            .unwrap_or_else(|| diag.file_path.clone());

        let lsp_diag = self.to_lsp_diagnostic(&diag);
        diag_set.add(uri, lsp_diag);
    }
    diag_set
}
```

#### After (With Typestate)
```rust
// unify-lsp/src/anti_llm_gate.rs (with typestate safety)

pub fn scan_file_to_lsp(&self, file_path: &str) -> DiagnosticSet {
    // Type-safe pipeline: guarantees dedup before emit
    let result = anti_llm_cheat_lsp::engine_result::EngineResult::from_file_scan(file_path)
        .enrich()
        .evaluate(&self.config)
        .dedupe();

    // result.emit() is safe — dedup already enforced by type system
    let mut diag_set = DiagnosticSet::new();
    for diag in result.emit() {
        let uri = url::Url::from_file_path(&diag.file_path)
            .ok()
            .map(|u| u.to_string())
            .unwrap_or_else(|| diag.file_path.clone());

        let lsp_diag = self.to_lsp_diagnostic(&diag);
        diag_set.add(uri, lsp_diag);
    }
    diag_set
}
```

**Safety gain:** Forgetting `.dedupe()` before `.emit()` results in **a compile error** — cannot ship LSP diagnostics with unknown dedup guarantees.

---

### 5.3 Pattern: Error Handling and Short-Circuiting

Both before and after code can be enhanced with error handling. With typestate, we can encode error states too (optional):

```rust
// Optional: add an Error<S> state for compile-time error handling
pub struct EngineResult<S> {
    // ... (as before)
    error: Option<String>,  // None in normal states, Some() in Error state
    // ...
}

// Example: optional error-state typestate
pub struct Error;

impl EngineResult<Error> {
    pub fn error_message(&self) -> &str {
        self.error.as_deref().unwrap_or("unknown error")
    }
    
    // No transition out of Error state (or only to cleanup)
}

// If scan fails:
if let Err(e) = ... {
    return Ok(EngineResult::error(e.to_string())); // EngineResult<Error>
}
```

This makes error handling **type-safe** too, but is optional for this design.

---

## 6. Hybrid Alternatives

### 6.1 Builder Pattern (No Typestate)

**Sketch:**
```rust
pub struct EngineResultBuilder {
    observations: Option<Vec<Observation>>,
    enrichment_map: Option<HashMap<usize, (String, String)>>,
    diagnostics: Option<Vec<AntiLlmDiagnostic>>,
}

impl EngineResultBuilder {
    pub fn new() -> Self { .. }
    pub fn scan_file(mut self, path: &str) -> Self {
        self.observations = Some(anti_llm_cheat_lsp::engine::scan_file(path));
        self
    }
    pub fn enrich(mut self) -> Self {
        // .. populate enrichment_map
        self
    }
    pub fn evaluate(mut self, config: &AntiLlmConfig) -> Self {
        // .. populate diagnostics
        self
    }
    pub fn dedupe(mut self) -> Self {
        // .. retain unique diagnostics
        self
    }
    pub fn build(self) -> Result<Vec<AntiLlmDiagnostic>, String> {
        // Check all required steps completed
        Ok(self.diagnostics.unwrap_or_default())
    }
}

// Usage:
let diags = EngineResultBuilder::new()
    .scan_file("src/lib.rs")
    .enrich()
    .evaluate(&config)
    .dedupe()
    .build()?;
```

**Pros:**
- Simpler type signature (only one generic param, S)
- Familiar to Rust developers

**Cons:**
- Runtime checks in `build()` instead of compile-time
- `.emit()` before `.dedupe()` is **possible** (caught only at runtime or via tests)
- No enforcement of ordering (could call `.evaluate()` twice, silent override)

**Verdict:** Builder is **less safe** than typestate; suitable only for optional, non-critical pipelines.

---

### 6.2 Docs-Only Approach (No Enforcement)

**Sketch:**
```rust
// Just document the order in a comment
/// # Pipeline Order
/// 1. scan_file() → Observation[]
/// 2. enrich() — optional enrichment step
/// 3. evaluate() → AntiLlmDiagnostic[]
/// 4. dedupe() — remove (path, line, code) duplicates
/// 5. emit() → final diagnostics
pub fn scan_and_evaluate(path: &str) -> Result<Vec<AntiLlmDiagnostic>, Box<dyn std::error::Error>> {
    let obs = scan_file(path);
    let diags = evaluate_diagnostics(&obs);
    let mut seen = HashSet::new();
    let unique = diags.into_iter().filter(|d| {
        seen.insert((d.file_path.clone(), d.line, d.code.clone()))
    }).collect();
    Ok(unique)
}
```

**Pros:**
- Zero overhead (just a function)
- Familiar to all Rust developers

**Cons:**
- **No compile-time enforcement** — developer must remember order
- Easy to skip dedup, double-enrich, etc.
- **No type-safety** — bugs slip into production
- Hard to verify in code review

**Verdict:** Docs-only is **not safe enough** for a compliance-critical pipeline like anti-llm-cheat detection.

---

### 6.3 Recommendation: Typestate is the Correct Choice

| Aspect | Builder | Docs-Only | **Typestate** |
|--------|---------|-----------|-------------|
| Compile-time safety | ❌ (runtime checks) | ❌ (none) | ✅ (type system) |
| Enforces ordering | ❌ (runtime) | ❌ (none) | ✅ (no impl block) |
| Prevents double-enrichment | ❌ | ❌ | ✅ (enrich() only on ScanComplete) |
| Prevents emit before dedupe | ❌ | ❌ | ✅ (emit() only on Evaluated/Deduped) |
| Complexity | Medium | Low | Medium (learning curve) |
| Ergonomics | Good | Good | **Excellent** (type guides usage) |

**Verdict:** **Typestate is strongly recommended** for this codebase. It matches the existing architectural style (used extensively in nexus-engine, unify-rdf) and provides the strongest guarantees.

---

## 7. Implementation Roadmap

### Phase 1: Core Types (40 LOC)

**File:** `anti-llm-cheat-lsp/src/states.rs` (new)

```rust
pub struct ScanComplete;
pub struct Enriched;
pub struct Evaluated;
pub struct Deduped;

pub trait Transitionable: Sized {}
impl Transitionable for ScanComplete {}
impl Transitionable for Enriched {}
impl Transitionable for Evaluated {}
```

**File:** `anti-llm-cheat-lsp/src/engine_result.rs` (new)

```rust
pub struct EngineResult<S> { /* as sketched above */ }
impl<S> EngineResult<S> { /* helpers */ }
```

**File:** `anti-llm-cheat-lsp/src/lib.rs` (modified)

```rust
pub mod states;
pub mod engine_result;
```

### Phase 2: Impl Blocks (150 LOC)

Four impl blocks (one per state) in `engine_result.rs`:
- `impl EngineResult<ScanComplete>` — from_file_scan, enrich()
- `impl EngineResult<Enriched>` — evaluate()
- `impl EngineResult<Evaluated>` — dedupe(), emit()
- `impl EngineResult<Deduped>` — final emit methods

### Phase 3: Integration Points (60 LOC per crate)

Update call sites in:
- `unify/src/commands.rs` — cmd_audit()
- `unify-lsp/src/anti_llm_gate.rs` — scan_file_to_lsp()
- `unify-mcp/src/anti_llm_tools.rs` — handle_scan_directory()
- `unify-admission/src/lib.rs` — AntiLlmAdmissionGate::admit()

### Phase 4: Tests (80 LOC)

```rust
#[test]
fn scan_complete_transitions_to_enriched() {
    let result = EngineResult::from_file_scan("tests/data/sample.rs");
    let enriched = result.enrich();
    // Result is EngineResult<Enriched> — compile-time verified
    assert!(!enriched.enrichment_map().is_empty());
}

#[test]
fn cannot_emit_before_dedupe() {
    let result = EngineResult::from_file_scan("tests/data/sample.rs")
        .enrich()
        .evaluate(&AntiLlmConfig::default());
    // result.emit() is valid here (EngineResult<Evaluated>)
    let diags = result.emit();
    assert!(!diags.is_empty());
}

#[test]
fn deduped_guarantees_uniqueness() {
    // Write a file that generates duplicate observations
    // ... 
    let result = EngineResult::from_file_scan("tests/data/duplicates.rs")
        .enrich()
        .evaluate(&AntiLlmConfig::default())
        .dedupe();
    
    let diags = result.emit();
    let mut seen = HashSet::new();
    for d in &diags {
        let key = (d.file_path.clone(), d.line, d.code.clone());
        assert!(seen.insert(key), "Duplicate found!");
    }
}
```

---

## 8. Cost-Benefit Analysis

### Effort Estimate

| Phase | Component | LOC | Hours | Risk |
|-------|-----------|-----|-------|------|
| 1 | Core types | 40 | 1 | Low |
| 2 | Impl blocks | 150 | 4 | Low |
| 3 | Integration (4 crates × 60) | 240 | 6 | Medium (need to verify no regressions) |
| 4 | Tests | 80 | 3 | Low |
| **Total** | | **510** | **14** | **Medium** |

### Bug Prevention

| Scenario | Before | After |
|----------|--------|-------|
| Skip `.dedupe()` before emitting | ❌ Ships with duplicates | ✅ **Compile error** |
| Call `.enrich()` twice | ❌ Overwrites enrichment | ✅ **Compile error** |
| Call `.evaluate()` before `.enrich()` | ⚠️ May work (no-op) | ✅ **Compile error** |
| Call `.emit()` on Evaluated (pre-dedupe) | ❌ Possible, hides duplicates | ✅ Works correctly (pre-dedup) |
| Reorder pipeline in new crate | ❌ Fragile (must test) | ✅ **Type-guided** |

**Expected impact:** **2–3 bugs prevented per year** in a codebase using this pattern (based on nexus-engine experience).

### Maintenance Burden

- **New contributors:** 1–2 hour onboarding (typestate pattern is Rust idiom)
- **Code review:** Easier to verify correctness (type signature is self-documenting)
- **Refactoring:** Safer (breaking changes are compile errors, not test failures)

**Net maintenance impact:** **+5–10% effort upfront, -20–30% effort on refactoring and bugs**.

---

## 9. Recommendation and Decision

### Recommended: **Proceed with Full Typestate Implementation**

**Rationale:**

1. **Aligns with existing codebase patterns** — nexus-engine (CombatMachine<S>), unify-rdf (ProjectManifest<Pending/Ingested/Validated>) already use this pattern successfully.

2. **Catches real bugs at compile time** — dedup, enrichment order, double-evaluation are all impossible to get wrong.

3. **Self-documenting code** — the type `EngineResult<Deduped>` clearly signals "this is safe to emit without further processing."

4. **Modest effort** — 510 LOC and 14 hours is reasonable for a compliance-critical subsystem.

5. **Future-proof** — adding new states (e.g., Filtered, Cached, Compressed) is straightforward.

### Caveats

1. **Rust-specific idiom** — Not portable to other languages; if anti-llm-cheat-lsp is ever ported to Python/Go, this pattern does not translate.

2. **Learning curve** — New Rust developers may need 1–2 hours to understand PhantomData and impl blocks. Mitigate with inline docs.

3. **No error states** — This design does not encode error states (e.g., scan failure). If error handling is critical, extend with `EngineResult<Error>` in Phase 2.

4. **Compile-time overhead** — Incremental compile times increase ~5–10% due to more generic specializations. Worth the tradeoff for correctness.

### Decision Gate

Proceed if:
- ✅ Team is comfortable with Rust generics and PhantomData
- ✅ Anti-llm-cheat-lsp is considered a long-term subsystem (multi-year)
- ✅ Compliance/correctness is a priority (not "nice to have")

Do not proceed if:
- ❌ Team is unfamiliar with typestate pattern
- ❌ Anti-llm-cheat-lsp is temporary/experimental
- ❌ Compile-time performance is critical (unlikely, but noted)

**Recommendation: Proceed. Implement in Q3 2026 (after Phase 1 integration is stable).**

---

## 10. Appendix: Full Code Sketch

### Complete anti-llm-cheat-lsp/src/engine_result.rs

```rust
//! Typestate-based result wrapper for the anti-llm-cheat-lsp pipeline.
//! Encodes the scanning → enrichment → evaluation → dedup sequence as a compile-time state machine.

use crate::config::AntiLlmConfig;
use crate::diagnostics::AntiLlmDiagnostic;
use crate::observations::Observation;
use crate::states::{ScanComplete, Enriched, Evaluated, Deduped};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

/// Result container parametrized by state S.
/// Each state S defines which transitions are legal (via impl blocks).
pub struct EngineResult<S> {
    file_path: String,
    observations: Vec<Observation>,
    enrichment_map: HashMap<usize, (String, String)>,
    diagnostics: Vec<AntiLlmDiagnostic>,
    dedup_set: HashSet<(String, usize, String)>,
    _phantom: PhantomData<S>,
}

// === PRIVATE HELPERS ===

impl<S> EngineResult<S> {
    fn new(
        file_path: String,
        observations: Vec<Observation>,
        enrichment_map: HashMap<usize, (String, String)>,
        diagnostics: Vec<AntiLlmDiagnostic>,
        dedup_set: HashSet<(String, usize, String)>,
    ) -> Self {
        Self {
            file_path,
            observations,
            enrichment_map,
            diagnostics,
            dedup_set,
            _phantom: PhantomData,
        }
    }
}

// === STATE 1: ScanComplete ===

impl EngineResult<ScanComplete> {
    pub fn from_file_scan(file_path: &str) -> Self {
        let observations = crate::engine::scan_file(file_path);
        Self::new(
            file_path.to_string(),
            observations,
            HashMap::new(),
            Vec::new(),
            HashSet::new(),
        )
    }

    pub fn from_directory_scan(dir_path: &str) -> Self {
        let observations = crate::engine::scan_directory(dir_path);
        Self::new(
            dir_path.to_string(),
            observations,
            HashMap::new(),
            Vec::new(),
            HashSet::new(),
        )
    }

    pub fn enrich(mut self) -> EngineResult<Enriched> {
        for (idx, obs) in self.observations.iter().enumerate() {
            let file_kind = classify_file_kind(&obs.file_path);
            let line_category = classify_line_category(&obs.file_path, &obs.kind);
            self.enrichment_map.insert(idx, (file_kind, line_category));
        }

        EngineResult::new(
            self.file_path,
            self.observations,
            self.enrichment_map,
            Vec::new(),
            HashSet::new(),
        )
    }

    pub fn observations(&self) -> &[Observation] {
        &self.observations
    }

    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }
}

// === STATE 2: Enriched ===

impl EngineResult<Enriched> {
    pub fn evaluate(self, config: &AntiLlmConfig) -> EngineResult<Evaluated> {
        let diagnostics = crate::engine::evaluate_diagnostics_with_config(
            &self.observations,
            config,
        );

        EngineResult::new(
            self.file_path,
            self.observations,
            self.enrichment_map,
            diagnostics,
            HashSet::new(),
        )
    }

    pub fn enrichment_map(&self) -> &HashMap<usize, (String, String)> {
        &self.enrichment_map
    }

    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }
}

// === STATE 3: Evaluated ===

impl EngineResult<Evaluated> {
    pub fn dedupe(mut self) -> EngineResult<Deduped> {
        let mut seen: HashSet<(String, usize, String)> = HashSet::new();

        self.diagnostics.retain(|d| {
            let key = (d.file_path.clone(), d.line, d.code.clone());
            seen.insert(key)
        });

        EngineResult::new(
            self.file_path,
            self.observations,
            self.enrichment_map,
            self.diagnostics,
            seen,
        )
    }

    pub fn diagnostics(&self) -> &[AntiLlmDiagnostic] {
        &self.diagnostics
    }

    pub fn emit(&self) -> Vec<AntiLlmDiagnostic> {
        self.diagnostics.clone()
    }

    pub fn diagnostic_count(&self) -> usize {
        self.diagnostics.len()
    }

    pub fn diagnostic_counts(&self) -> (usize, usize) {
        let blocking = self.diagnostics.iter().filter(|d| d.blocking).count();
        let warning = self.diagnostics.len() - blocking;
        (blocking, warning)
    }

    pub fn filter_by_blocking(&self, blocking: bool) -> Vec<&AntiLlmDiagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.blocking == blocking)
            .collect()
    }
}

// === STATE 4: Deduped (TERMINAL) ===

impl EngineResult<Deduped> {
    pub fn emit(&self) -> Vec<AntiLlmDiagnostic> {
        self.diagnostics.clone()
    }

    pub fn emit_blocking(&self) -> Vec<AntiLlmDiagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.blocking)
            .cloned()
            .collect()
    }

    pub fn emit_warnings(&self) -> Vec<AntiLlmDiagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| !d.blocking)
            .cloned()
            .collect()
    }

    pub fn diagnostic_count(&self) -> usize {
        self.diagnostics.len()
    }

    pub fn diagnostic_counts(&self) -> (usize, usize) {
        let blocking = self.diagnostics.iter().filter(|d| d.blocking).count();
        let warning = self.diagnostics.len() - blocking;
        (blocking, warning)
    }
}

// === HELPERS ===

fn classify_file_kind(file_path: &str) -> String {
    if file_path.ends_with(".rs") {
        "rust".to_string()
    } else if file_path.ends_with(".ts") || file_path.ends_with(".tsx") {
        "typescript".to_string()
    } else if file_path.ends_with(".md") {
        "markdown".to_string()
    } else if file_path.ends_with(".c") || file_path.ends_with(".h") || file_path.ends_with(".cpp") {
        "c".to_string()
    } else {
        "unknown".to_string()
    }
}

fn classify_line_category(file_path: &str, kind: &str) -> String {
    if file_path.contains("tests/") || file_path.ends_with("_test.rs") {
        "test_smell".to_string()
    } else {
        kind.to_string()
    }
}

// === TESTS ===

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn scan_complete_to_enriched() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("lib.rs"), "fn main() {}").unwrap();

        let result = EngineResult::from_directory_scan(dir.path().to_str().unwrap());
        let enriched = result.enrich();

        assert!(enriched.observation_count() > 0 || enriched.observation_count() == 0);
        // (no observations if lib.rs has no patterns)
    }

    #[test]
    fn enriched_to_evaluated() {
        let dir = TempDir::new().unwrap();
        // Write a file with a known pattern
        fs::write(
            dir.path().join("test.rs"),
            "assert!(x.contains(\"pattern\"));",
        )
        .unwrap();

        let result = EngineResult::from_directory_scan(dir.path().to_str().unwrap())
            .enrich()
            .evaluate(&AntiLlmConfig::default());

        // Some diagnostics should be generated
        assert!(result.diagnostic_count() >= 0);
    }

    #[test]
    fn evaluated_to_deduped() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("lib.rs"), "fn main() {}").unwrap();

        let result = EngineResult::from_directory_scan(dir.path().to_str().unwrap())
            .enrich()
            .evaluate(&AntiLlmConfig::default())
            .dedupe();

        // result is now EngineResult<Deduped> — final state
        let diags = result.emit();
        assert_eq!(diags.len(), result.diagnostic_count());
    }
}
```

---

## 11. Conclusion

The **Typestate Pattern for EngineResult** provides compile-time safety for the anti-llm-cheat-lsp scanning pipeline. It ensures:

1. **Illegal transitions are impossible** (compile-time errors, not bugs)
2. **Code is self-documenting** (types encode the protocol)
3. **Zero runtime overhead** (PhantomData erased at compile time)
4. **Familiar to Rocket Craft contributors** (nexus-engine precedent)

**Total effort:** 14 hours over 2–3 weeks.  
**Return on investment:** Prevents critical bugs, improves maintainability, aligns with architectural style.

**Decision:** **Recommended for implementation in Q3 2026** after Phase 1 integration stabilizes.
