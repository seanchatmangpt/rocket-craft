# anti-llm-cheat-lsp Integration Diagrams

## 1. System Architecture (High Level)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           unify-rs Workspace                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │           anti-llm-cheat-lsp (Detection Engine)                 │  │
│  │                                                                  │  │
│  │  • scan_file() → Vec<Observation>                              │  │
│  │  • scan_directory() → Vec<Observation>                         │  │
│  │  • evaluate_diagnostics() → Vec<AntiLlmDiagnostic>            │  │
│  │                                                                  │  │
│  │  14 parsers × 17 rule modules → blocking: bool flag            │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                  ▲                                     │
│                    ┌─────────────┼─────────────┐                      │
│                    │             │             │                      │
│          ┌─────────▼────┐ ┌─────▼──────┐ ┌────▼─────────┐            │
│          │ unify-mcp    │ │ unify-lsp  │ │ unify (CLI)  │            │
│          │              │ │            │ │              │            │
│          │ MCP Tools:   │ │ Compositor │ │ `unify audit`│            │
│          │ • audit/scan │ │ integration│ │ subcommand   │            │
│          │ • audit/eval │ │            │ │              │            │
│          └──────────────┘ └────────────┘ └──────────────┘            │
│                    │             │             │                      │
│                    └─────────────┼─────────────┘                      │
│                                  │                                    │
│                          ┌───────▼────────┐                          │
│                          │ unify-admission│                          │
│                          │                │                          │
│                          │ Admission Gate │                          │
│                          │ (policy)       │                          │
│                          └────────────────┘                          │
└─────────────────────────────────────────────────────────────────────────┘
        │                    │                    │
        ▼                    ▼                    ▼
    [MCP Client]        [LSP Client]        [CLI User]
  (Claude, etc.)     (Editor, etc.)     (Developer, CI)
```

---

## 2. Data Flow: From File to Diagnostic

```
INPUT: Source file or directory
│
├─ scan_file(filepath) or scan_directory(dirpath)
│  ├─ Read file contents
│  ├─ Dispatch to appropriate parser (14 available)
│  │  ├─ rust_tree_sitter.rs    [Rust via tree-sitter]
│  │  ├─ typescript.rs          [TypeScript/JavaScript]
│  │  ├─ cargo_toml.rs          [Cargo.toml]
│  │  ├─ markdown_claims.rs     [Markdown claims]
│  │  └─ ... (11 more parsers)
│  └─ Return: Vec<Observation>
│
├─ evaluate_diagnostics(observations, config)
│  ├─ Pass observations through rule engines (17 available)
│  │  ├─ surface.rs            [Surface-level patterns]
│  │  ├─ authority.rs          [Authority violations]
│  │  ├─ receipts.rs           [Receipt validation]
│  │  ├─ routes.rs             [Route validation]
│  │  ├─ rust_smells.rs        [Code smells]
│  │  ├─ lsp318.rs             [LSP 3.18 spec]
│  │  ├─ ocel_rules.rs         [OCEL validation]
│  │  └─ ... (10 more rules)
│  └─ Return: Vec<AntiLlmDiagnostic>
│       └─ Each diagnostic has: code, category, line, blocking
│
OUTPUT: Structured diagnostics with severity (blocking: bool)
```

---

## 3. Phase 1: MCP Integration

```
┌─────────────────────────────────────────────────────┐
│            MCP Client (Claude, etc.)                │
└────────────────────┬────────────────────────────────┘
                     │ JSON-RPC
                     │ {method: "tools/call",
                     │  params: {name: "audit/scan_directory"}}
                     ▼
        ┌────────────────────────────┐
        │    unify-mcp Server        │
        ├────────────────────────────┤
        │                            │
        │  anti_llm_tools.rs (NEW)  │
        │  ├─ handle_scan_directory │
        │  └─ handle_evaluate_diag  │
        │                            │
        │  Registered tools:         │
        │  • audit/scan_directory    │
        │  • audit/evaluate_diag     │
        │                            │
        └────────────────────────────┘
                     │
                     │ Calls engine
                     ▼
        ┌────────────────────────────┐
        │ anti_llm_cheat_lsp::engine │
        │ • scan_directory()         │
        │ • evaluate_diagnostics()   │
        └────────────────────────────┘
                     │
                     │ Returns
                     ▼
        ┌────────────────────────────┐
        │     Vec<Diagnostic>        │
        │     + blocking flag        │
        └────────────────────────────┘
                     │
                     │ Serializes to JSON
                     ▼
┌─────────────────────────────────────────────────────┐
│ JSON Response to MCP Client                         │
│ {                                                   │
│   "directory": ".",                                 │
│   "diagnostic_count": 15,                           │
│   "blocking_count": 3,                              │
│   "warning_count": 12,                              │
│   "diagnostics": [...]                              │
│ }                                                   │
└─────────────────────────────────────────────────────┘
```

---

## 4. Phase 2: LSP Integration

```
┌─────────────────────────────────────────────────────┐
│        LSP Client (Editor, Neovim, etc.)            │
│                                                     │
│  User opens file → textDocument/didOpen            │
└────────────────────┬────────────────────────────────┘
                     │
                     ▼
        ┌────────────────────────────┐
        │    LSP Server              │
        │  (unify-lsp)               │
        │                            │
        │  On document opened:       │
        │  compositor.scan_file_anti │
        │  _llm(file_path)           │
        │                            │
        └────────────────────────────┘
                     │
                     ▼
        ┌────────────────────────────┐
        │  CompositorState           │
        │  ├─ anti_llm_gate: Option  │
        │  ├─ diagnostics: DiagSet   │
        │  └─ gate: AndonGate        │
        │                            │
        │  Method:                   │
        │  pub fn scan_file_anti_llm │
        │  (&mut self, file_path)    │
        │                            │
        └────────────────────────────┘
                     │
                     ▼
        ┌────────────────────────────┐
        │   AntiLlmGate (NEW)        │
        │                            │
        │  • scan_file_to_lsp()      │
        │  • to_lsp_diagnostic()     │
        │                            │
        │  Converts:                 │
        │  blocking=true  → Error    │
        │  blocking=false → Warning  │
        │                            │
        └────────────────────────────┘
                     │
                     ▼
        ┌────────────────────────────┐
        │   LSP Diagnostic           │
        │   ├─ range                 │
        │   ├─ severity              │
        │   ├─ message               │
        │   ├─ code                  │
        │   └─ source: "anti-llm"    │
        │                            │
        └────────────────────────────┘
                     │
                     │ publishDiagnostics notification
                     ▼
┌─────────────────────────────────────────────────────┐
│        LSP Client (Editor)                          │
│                                                     │
│  File content with squiggles:                       │
│  ┌─────────────────────────────────┐               │
│  │ let x = lsp-max;              │               │
│  │           ^^^^^^^^^^^ 🔴         │ Error         │
│  │ // ANTI-LLM-CLAIM-001           │               │
│  │    ^^^^^^^^^^^^^^^^^ 🟡          │ Warning       │
│  └─────────────────────────────────┘               │
│                                                     │
│  Red = blocking (Error severity)                    │
│  Yellow = warning (Warning severity)                │
│                                                     │
└─────────────────────────────────────────────────────┘
```

---

## 5. Phase 3: CLI Integration

```
┌─────────────────────────────────────────────────────┐
│             Developer Terminal                      │
│                                                     │
│  $ unify audit --directory . --fail-on-blocking    │
│                                                     │
└────────────────────┬────────────────────────────────┘
                     │
                     ▼
        ┌────────────────────────────┐
        │   unify CLI (unify/src/)   │
        │                            │
        │   Commands enum            │
        │   ├─ Audit {              │
        │   │   directory,           │
        │   │   blocking_only,       │
        │   │   fail_on_blocking     │
        │   └─ }                     │
        │                            │
        └────────────────────────────┘
                     │
                     ▼
        ┌────────────────────────────┐
        │    cmd_audit() handler     │
        │                            │
        │  1. scan_directory(dir)    │
        │  2. evaluate_diagnostics() │
        │  3. Filter if -blocking    │
        │  4. Serialize to JSON      │
        │  5. Determine success flag │
        │                            │
        │  success =                 │
        │    !fail_on_blocking ||    │
        │    blocking_count == 0     │
        │                            │
        └────────────────────────────┘
                     │
                     ▼
        ┌────────────────────────────┐
        │    Output struct           │
        │    ├─ success: bool        │
        │    ├─ data: JSON value     │
        │    └─ message: String      │
        │                            │
        └────────────────────────────┘
                     │
                     │
        ┌────────────┴────────────┐
        │                         │
        ▼ (human-readable)   ▼ (--json flag)
┌──────────────────────────┐ ┌──────────────────────────┐
│ Human Output             │ │ JSON Output              │
│                          │ │                          │
│ audited:          │ │ {                        │
│ 12 warning(s)            │ │   "success": false,      │
│ 3 blocking error(s)      │ │   "data": {              │
│                          │ │     "diagnostic_count": 15,
│ Blocking:                │ │     "blocking_count": 3, │
│ • ANTI-LLM-... (file:42) │ │     "diagnostics": [...]│
│ • ANTI-LLM-... (file:99) │ │   },                     │
│ • ...                    │ │   "message": "..."       │
│                          │ │ }                        │
└──────────────────────────┘ └──────────────────────────┘
        │
        ▼ (exit code)
     ┌─────────────────────────────┐
     │ 0 = success (clean)         │
     │   (unless fail_on_blocking) │
     │                             │
     │ 1 = failure (has blocking)  │
     │   (if fail_on_blocking)     │
     └─────────────────────────────┘
```

---

## 6. Phase 4: Admission Integration

```
┌─────────────────────────────────────────────────────┐
│           Developer commits code                    │
│                                                     │
│  $ git commit -m "Add feature"                      │
│                                                     │
└────────────────────┬────────────────────────────────┘
                     │
                     ▼ (if pre-commit hook configured)
        ┌────────────────────────────┐
        │   .git/hooks/pre-commit    │
        │                            │
        │  #!/bin/bash               │
        │  unify audit --fail-on-... │
        │   ↓                        │
        │  exit $?                   │
        │                            │
        └────────────────────────────┘
                     │
                     ▼
        ┌────────────────────────────┐
        │ unify-admission            │
        │                            │
        │ AntiLlmAdmissionLaw        │
        │ AntiLlmAdmissionGate       │
        │                            │
        │ Gate::admit(repo_path):    │
        │  1. scan_directory()       │
        │  2. evaluate_diagnostics() │
        │  3. Filter blocking=true   │
        │  4. Return Refusal if any  │
        │                            │
        └────────────────────────────┘
                     │
        ┌────────────┴────────────┐
        │                         │
        ▼ (No blocking)      ▼ (Blocking found)
    ┌────────────┐       ┌──────────────────────┐
    │  Ok(())    │       │  Err(Refusal)        │
    │            │       │  "Anti-LLM admission │
    │ Commit     │       │   gate blocked:      │
    │ allowed    │       │   - ANTI-LLM-... ... │
    │            │       │   - ...              │
    │ exit 0     │       │   "                  │
    │            │       │                      │
    └────────────┘       │  exit 1              │
        │                └──────────────────────┘
        │                        │
        ▼                        ▼
    Commit succeeds       Commit rejected
    Code pushed           Developer fixes issues
    CI runs               Tries again
```

---

## 7. Type Conversion Pipeline

```
                         SOURCE
                            │
                   ┌────────▼────────┐
                   │ Source Code     │
                   │ (*.rs, *.ts, .) │
                   └────────┬────────┘
                            │
                ┌───────────▼───────────┐
                │  scan_file() [engine] │
                │  or                   │
                │  scan_directory()     │
                └───────────┬───────────┘
                            │
                ┌───────────▼───────────┐
                │   Observation         │
                │   (raw patterns)      │
                │                       │
                │ • file_path           │
                │ • line, column        │
                │ • kind, construct     │
                │ • message             │
                └───────────┬───────────┘
                            │
            ┌───────────────▼───────────────┐
            │ evaluate_diagnostics() [17    │
            │ rule engines]                 │
            └───────────────┬───────────────┘
                            │
        ┌───────────────────▼───────────────────┐
        │   AntiLlmDiagnostic                   │
        │   (evaluated result)                  │
        │                                       │
        │ • code: String                        │
        │ • category: String                    │
        │ • file_path: String                   │
        │ • line: usize                         │
        │ • blocking: bool  ◄─── KEY FIELD     │
        │ • required_correction: String         │
        │ • required_next_proof: String         │
        │                                       │
        └───────────────────┬───────────────────┘
                            │
    ┌───────────────────────┼───────────────────────┐
    │                       │                       │
    ▼                       ▼                       ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────────┐
│ MCP Tool     │    │ LSP Diag     │    │ Admission Gate   │
│              │    │              │    │                  │
│ JSON:        │    │ LSP:         │    │ Refusal:         │
│ {            │    │ • range      │    │ "Blocked:        │
│  "diag": {   │    │ • severity   │    │  - ANTI-LLM-..  │
│   "code": ..,    │ • message    │    │  "              │
│   "blocking":    │ • code       │    │                  │
│   true       │    │ • source     │    │ Exit code: 1    │
│  }           │    │              │    │                  │
│ }            │    │ Severity:    │    │ Commit blocked   │
│              │    │ blocking =   │    │                  │
│ ─────────────│    │ ERROR (red)  │    │ ─────────────────│
│ Returns JSON │    │ warning =    │    │ Enforce policy   │
│ to MCP client    │ WARNING      │    │ at commit time   │
└──────────────┘    │ (yellow)     │    └──────────────────┘
                    └──────────────┘
```

---

## 8. Dependency Graph

```
anti-llm-cheat-lsp (core engine)
        │
        ├─── unify-mcp
        │    └─ Exposes: audit/scan_directory tool
        │             audit/evaluate_diagnostics tool
        │
        ├─── unify-lsp
        │    └─ Exposes: AntiLlmGate (compositor integration)
        │             Converts to LSP Diagnostic
        │
        ├─── unify (CLI)
        │    └─ Exposes: unify audit command
        │             Handler: cmd_audit()
        │
        └─── unify-admission
             └─ Exposes: AntiLlmAdmissionLaw (static law)
                      AntiLlmAdmissionGate (enforcement)
                      AntiLlmRuntimeLaw (optional)

⚠️  NO CIRCULAR DEPENDENCIES
Each crate independently consumes anti-llm-cheat-lsp;
no crate depends on another.
```

---

## 9. Execution Timeline (All Phases)

```
┌──────────────────────────────────────────────────────────────────┐
│                    Development Timeline                          │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│ Week 1:                                                          │
│ ┌─ Phase 1: unify-mcp  ──────────────────────────────────────┐  │
│ │ • Create anti_llm_tools.rs (1h)                             │  │
│ │ • Test + review (2h)                                        │  │
│ │ • Merge (1h)                                                │  │
│ └──────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ ┌─ Phase 2: unify-lsp  ──────────────────────────────────────┐  │
│ │ (concurrent with P1)                                         │  │
│ │ • Create anti_llm_gate.rs (1.5h)                            │  │
│ │ • Update compositor.rs (1h)                                  │  │
│ │ • Test + review (2h)                                        │  │
│ │ • Merge (1h)                                                │  │
│ └──────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ ┌─ Phase 3: unify CLI  ──────────────────────────────────────┐  │
│ │ (concurrent with P1-P2)                                     │  │
│ │ • Update app.rs + commands.rs (1h)                          │  │
│ │ • Test + review (1.5h)                                      │  │
│ │ • Merge (1h)                                                │  │
│ └──────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ Week 2:                                                         │
│ ┌─ Phase 4: unify-admission  ──────────────────────────────┐  │
│ │ (after P1-P3)                                             │  │
│ │ • Add law + gate (1h)                                      │  │
│ │ • Test + review (1.5h)                                     │  │
│ │ • Merge (1h)                                               │  │
│ │ • Document pre-commit (0.5h)                               │  │
│ └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ Integration Testing & Validation: 1-2 days                      │
│                                                                  │
│ Total: 2-3 weeks (sequential) or 1-2 weeks (parallel)          │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

---

## 10. Feature Matrix

```
┌──────────────────────────────────────────────────────────────────┐
│  Feature                          P1    P2    P3    P4           │
│  (MCP) (LSP) (CLI) (Admission)   │
├──────────────────────────────────────────────────────────────────┤
│ Scan files                        ✓     ✓     ✓     ✓           │
│ Scan directories                  ✓     ✓     ✓     ✓           │
│ Evaluate diagnostics              ✓     ✓     ✓     ✓           │
│                                                                  │
│ JSON output (MCP)                 ✓                             │
│ MCP tools                         ✓                             │
│ MCP integration                   ✓                             │
│                                                                  │
│ LSP diagnostic format                   ✓                       │
│ LSP severity mapping                    ✓                       │
│ Editor integration                      ✓                       │
│ Real-time scanning                      ✓                       │
│ ANDON gate triggering                   ✓                       │
│                                                                  │
│ CLI command                                   ✓                 │
│ Exit codes                                    ✓                 │
│ Human-readable output                        ✓                 │
│ JSON flag support                            ✓                 │
│ Filtering (--blocking-only)                  ✓                 │
│                                                                  │
│ Admission gate (static law)                        ✓            │
│ Pre-commit enforcement                           ✓            │
│ Commit blocking                                  ✓            │
│ Runtime law support                             ✓            │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

---

## 11. Error Handling Flow

```
┌─────────────────────────────┐
│  Execution Point            │
│  (any of 4 integration)     │
└────────────┬────────────────┘
             │
             ▼
    ┌────────────────────┐
    │ Can scan dir/file? │
    ├────────────────────┤
    │ NO → Return error  │
    │ YES ↓              │
    └────────┬───────────┘
             │
             ▼
    ┌────────────────────┐
    │ Run parser         │
    │ (14 available)     │
    ├────────────────────┤
    │ Parse error?       │
    │ NO → Continue      │
    │ YES → Skip file    │
    └────────┬───────────┘
             │
             ▼
    ┌────────────────────┐
    │ Generate           │
    │ observations       │
    ├────────────────────┤
    │ Empty set OK       │
    │ (no violations)    │
    └────────┬───────────┘
             │
             ▼
    ┌────────────────────┐
    │ Evaluate rules     │
    │ (17 available)     │
    ├────────────────────┤
    │ Accumulate         │
    │ diagnostics        │
    └────────┬───────────┘
             │
             ▼
    ┌────────────────────┐
    │ Process output     │
    │ per phase:         │
    │                    │
    │ P1: JSON → MCP     │
    │ P2: LSP Diag       │
    │ P3: CLI output     │
    │ P4: Refusal/block  │
    └────────────────────┘
```

---

## Summary

These diagrams show:

1. **Architecture**: How anti-llm-cheat-lsp integrates into the workspace
2. **Data flow**: Files → observations → diagnostics → phase-specific output
3. **Phase 1–4**: Each integration point's specific workflow
4. **Type conversions**: How data transforms across layers
5. **Dependencies**: Isolation between phases
6. **Timeline**: Rollout sequence (can be parallel)
7. **Features**: What each phase enables
8. **Error handling**: How failures are handled

For code details, refer to **INTEGRATION_PLAN.md** and **INTEGRATION_CHECKLIST.md**.
