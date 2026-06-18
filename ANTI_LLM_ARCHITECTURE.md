# anti-llm-cheat-lsp Architecture & Integration Map

## Core Engine (anti-llm-cheat-lsp)

```
┌─────────────────────────────────────────────────────────────┐
│                  anti-llm-cheat-lsp Crate                   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │            engine:: (public API)                    │   │
│  │  • scan_file(filepath) → Vec<Observation>           │   │
│  │  • scan_directory(dirpath) → Vec<Observation>       │   │
│  │  • evaluate_diagnostics(obs) → Vec<Diagnostic>      │   │
│  └─────────────────────────────────────────────────────┘   │
│                          ↓                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │            parsers:: (14 parser modules)            │   │
│  │  Cargo.toml, Cargo.lock, Rust (tree-sitter),       │   │
│  │  Markdown, JSON-RPC, TypeScript, C/C++, etc.        │   │
│  └─────────────────────────────────────────────────────┘   │
│                          ↓                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │            rules:: (17 rule modules)                │   │
│  │  Raw text smells, complexity, determinism, LSP318, │   │
│  │  contract rules, OCEL rules, etc.                   │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

**Public Types:**
- `Observation` – raw pattern match (file, line, kind, construct, message)
- `AntiLlmDiagnostic` – evaluated result (code, category, blocking, required_correction)
- `AntiLlmConfig` – rule configuration

---

## Integration Points in unify-rs

```
                    anti-llm-cheat-lsp
                          ▲
            ┌─────────────┼─────────────┐
            │             │             │
            ▼             ▼             ▼
     ┌──────────┐   ┌──────────┐  ┌─────────────┐
     │ unify    │   │ unify    │  │ unify       │
     │ -mcp     │   │ -lsp     │  │ -admission  │
     └──────────┘   └──────────┘  └─────────────┘
            │             │             │
            ▼             ▼             ▼
     ┌──────────────────────────────────────────┐
     │           unify CLI binary               │
     │       (Commands::Audit)                  │
     └──────────────────────────────────────────┘
```

---

## 1. unify-mcp Integration

**Module:** `unify-mcp/src/anti_llm_tools.rs` (NEW)

**Two MCP Tools Exposed:**

```
┌─────────────────────────────────────────┐
│   MCP Tool: audit/scan_directory        │
├─────────────────────────────────────────┤
│ Input:  { "dir_path": "/path/to/code" } │
│ Output: {                               │
│   "directory": "...",                   │
│   "observation_count": 42,              │
│   "observations": [                     │
│     {                                   │
│       "file_path": "src/lib.rs",        │
│       "line": 15,                       │
│       "kind": "raw_text",               │
│       "construct": "lsp-max",         │
│       "message": "..."                  │
│     }                                   │
│   ]                                     │
│ }                                       │
└─────────────────────────────────────────┘

┌─────────────────────────────────────────┐
│ MCP Tool: audit/evaluate_diagnostics    │
├─────────────────────────────────────────┤
│ Input:  { "observations": [...] }       │
│ Output: {                               │
│   "diagnostic_count": 15,               │
│   "blocking_count": 3,                  │
│   "warning_count": 12,                  │
│   "diagnostics": [                      │
│     {                                   │
│       "code": "ANTI-LLM-...",           │
│       "category": "...",                │
│       "file_path": "...",               │
│       "line": 42,                       │
│       "blocking": true,                 │
│       "required_correction": "..."      │
│     }                                   │
│   ]                                     │
│ }                                       │
└─────────────────────────────────────────┘
```

**Use Case:** Claude Desktop, external MCP clients can audit code in real-time.

**Files to Change:**
- `unify-mcp/src/anti_llm_tools.rs` (NEW)
- `unify-mcp/src/lib.rs` (add module)
- `unify-mcp/src/main.rs` (register tools)
- `unify-mcp/Cargo.toml` (add dependency)

---

## 2. unify-lsp Integration

**Module:** `unify-lsp/src/anti_llm_gate.rs` (NEW)

**Diagnostic Flow:**

```
File opened/changed (textDocument/didOpen, didChange)
         ↓
CompositorState::scan_file_anti_llm(file_path)
         ↓
AntiLlmGate::scan_file_to_lsp()
         ↓
Converts AntiLlmDiagnostic → LSP Diagnostic
         ↓
Merge into CompositorState.diagnostics
         ↓
publishDiagnostics notification sent to editor
         ↓
Editor displays errors (red squiggle) and warnings (yellow squiggle)
```

**Diagnostic Severity Mapping:**

| AntiLlmDiagnostic | LSP Severity |
|---|---|
| `blocking: true` | DiagnosticSeverity::Error (red) |
| `blocking: false` | DiagnosticSeverity::Warning (yellow) |

**Integration with Compositor:**

```rust
pub struct CompositorState {
    servers: Vec<ServerEntry>,
    diagnostics: DiagnosticSet,
    gate: AndonGate,
    anti_llm_gate: Option<AntiLlmGate>,  // NEW
}

// New methods:
pub fn enable_anti_llm_gate(&mut self, config: AntiLlmConfig)
pub fn scan_file_anti_llm(&mut self, file_path: &str)
pub fn scan_directory_anti_llm(&mut self, dir_path: &str)
```

**Files to Change:**
- `unify-lsp/src/anti_llm_gate.rs` (NEW)
- `unify-lsp/src/lib.rs` (add module)
- `unify-lsp/src/compositor.rs` (add anti_llm_gate field + methods)
- `unify-lsp/Cargo.toml` (add dependencies)

---

## 3. unify CLI Integration

**Command:** `unify audit [OPTIONS]`

**Subcommand Structure:**

```
┌────────────────────────────────────────────┐
│ unify audit [OPTIONS]                      │
├────────────────────────────────────────────┤
│ --directory <DIR>                          │
│   Path to scan (default: ".")              │
│                                            │
│ --blocking-only                            │
│   Show only blocking errors                │
│                                            │
│ --fail-on-blocking                         │
│   Exit non-zero if blocking errors found   │
│                                            │
│ --json                                     │
│   Output as JSON (from parent --json flag) │
└────────────────────────────────────────────┘
```

**Exit Codes:**

| Case | Exit Code |
|------|-----------|
| Success, no diagnostics | 0 |
| Success, warnings/info only | 0 |
| `--fail-on-blocking` + blocking errors | 1 |

**Human Output Example:**

```
audited: 12 warning(s), 3 blocking error(s)

Directory: .
Diagnostic Count: 15

Blocking Diagnostics:
  ANTI-LLM-CLAIM-001 (src/engine.rs:150)
    Tower-LSP raw text detected
    Correction: Remove hardcoded string literal
    Proof: Verify no LSP-specific imports remain

[... 12 more warnings ...]
```

**JSON Output:**

```json
{
  "success": false,
  "data": {
    "directory": ".",
    "diagnostic_count": 15,
    "blocking_count": 3,
    "warning_count": 12,
    "diagnostics": [...]
  },
  "message": "Audit FAILED: 3 blocking error(s) detected"
}
```

**Files to Change:**
- `unify/src/app.rs` (add Audit variant to Commands)
- `unify/src/commands.rs` (implement cmd_audit)
- `unify/Cargo.toml` (add dependency)

---

## 4. unify-admission Integration

**Static Law:** `AntiLlmAdmissionLaw`

**Gate Implementation:** `AntiLlmAdmissionGate`

**Law Registry:**

```rust
pub struct AntiLlmAdmissionLaw;

impl StaticLaw for AntiLlmAdmissionLaw {
    const NAME: &'static str = "AntiLlmAdmission";
}

pub struct AntiLlmAdmissionGate {
    config: AntiLlmConfig,
}

impl Admit<AntiLlmAdmissionLaw> for AntiLlmAdmissionGate {
    type Artifact = PathBuf;
    
    fn admit(&self, path: &PathBuf) -> Result<(), Refusal<AntiLlmAdmissionLaw>> {
        // Scan path; return Err if blocking diagnostics found
    }
}
```

**Runtime Law:** `AntiLlmRuntimeLaw` (mirrors StaticLaw but dispatched at runtime)

**Admission Flow (Pre-Commit Hook):**

```
$ git commit -m "Add feature"
         ↓
Pre-commit hook runs:
  $ unify gate --law AntiLlmAdmission --path .
         ↓
  OR: $ unify audit --fail-on-blocking
         ↓
AntiLlmAdmissionGate::admit(repo_path)
         ↓
Scan repo; evaluate diagnostics
         ↓
If blocking_count > 0:
  Return Refusal("... blocking errors found")
         ↓
Pre-commit exits non-zero
         ↓
Commit rejected
```

**Files to Change:**
- `unify-admission/src/lib.rs` (add AntiLlmAdmissionLaw + AntiLlmAdmissionGate)
- `unify-admission/src/anti_llm_runtime_law.rs` (NEW, AntiLlmRuntimeLaw)
- `unify-admission/Cargo.toml` (add dependency)

---

## Shared Type Conversions

```
┌──────────────────────────────────────────────────┐
│ anti-llm-cheat-lsp Types (source of truth)       │
├──────────────────────────────────────────────────┤
│ • Observation (raw pattern)                      │
│ • AntiLlmDiagnostic (evaluated)                  │
│ • AntiLlmConfig (rules config)                   │
└──────────────────────────────────────────────────┘
         ↓ (serde)      ↓ (bridge)      ↓ (bridge)
    ┌─────────┐     ┌──────────┐   ┌────────────┐
    │ JSON    │     │ LSP      │   │ Admission  │
    │ (MCP,   │     │ Diag     │   │ Refusal    │
    │ CLI)    │     │          │   │            │
    └─────────┘     └──────────┘   └────────────┘
```

| From Type | To Type | How |
|-----------|---------|-----|
| `Observation` | `serde_json::Value` | Native serde derive |
| `AntiLlmDiagnostic` | `serde_json::Value` | Native serde derive |
| `AntiLlmDiagnostic` | `Diagnostic` (LSP) | `AntiLlmGate::to_lsp_diagnostic()` |
| `Vec<AntiLlmDiagnostic>` | `Refusal<AntiLlmAdmissionLaw>` | Format message in gate |

---

## New Public APIs (Summary)

### unify-mcp

```rust
pub fn attach_anti_llm_tools(server: McpServer) -> McpServer;
pub fn handle_scan_directory(params: serde_json::Value) -> Result<serde_json::Value, String>;
pub fn handle_evaluate_diagnostics(params: serde_json::Value) -> Result<serde_json::Value, String>;
```

### unify-lsp

```rust
pub struct AntiLlmGate {
    pub fn new(config: AntiLlmConfig) -> Self;
    pub fn scan_file_to_lsp(&self, file_path: &str) -> DiagnosticSet;
    pub fn scan_directory_to_lsp(&self, dir_path: &str) -> DiagnosticSet;
}

// Added to CompositorState:
pub fn enable_anti_llm_gate(&mut self, config: AntiLlmConfig);
pub fn scan_file_anti_llm(&mut self, file_path: &str);
pub fn scan_directory_anti_llm(&mut self, dir_path: &str);
```

### unify (CLI)

```rust
// New variant in Commands enum:
Audit {
    #[arg(short, long, default_value = ".")]
    directory: String,
    
    #[arg(long)]
    blocking_only: bool,
    
    #[arg(long)]
    fail_on_blocking: bool,
}

// New handler:
pub fn cmd_audit(dir_path: &str, blocking_only: bool, fail_on_blocking: bool) 
    -> Result<Output, Box<dyn std::error::Error>>;
```

### unify-admission

```rust
pub struct AntiLlmAdmissionLaw;
pub struct AntiLlmAdmissionGate { ... }

impl StaticLaw for AntiLlmAdmissionLaw { ... }
impl Admit<AntiLlmAdmissionLaw> for AntiLlmAdmissionGate { ... }

pub struct AntiLlmRuntimeLaw { ... }
impl RuntimeLaw for AntiLlmRuntimeLaw { ... }
```

---

## Dependencies

**Added to workspace (Cargo.toml):**

| Crate | Dependencies |
|-------|---|
| unify-mcp | `anti-llm-cheat-lsp` (path) |
| unify-lsp | `anti-llm-cheat-lsp` (path), `url` |
| unify | `anti-llm-cheat-lsp` (path) |
| unify-admission | `anti-llm-cheat-lsp` (path) |

No external crate versions change; all dependencies already in workspace.

---

## Rollout Sequence

```
Phase 1: unify-mcp
  ├─ Implement anti_llm_tools.rs
  ├─ Register in main.rs
  ├─ Test with MCP client
  └─ Deploy (lowest risk)

Phase 2: unify-lsp (concurrent with Phase 1)
  ├─ Implement anti_llm_gate.rs
  ├─ Integrate with compositor
  ├─ Test with LSP client
  └─ Deploy

Phase 3: unify CLI (concurrent with Phases 1-2)
  ├─ Add Audit subcommand
  ├─ Test cmd_audit
  ├─ Verify exit codes
  └─ Deploy

Phase 4: unify-admission (optional, after Phase 3)
  ├─ Implement AntiLlmAdmissionLaw + Gate
  ├─ Add to pre-commit hooks
  └─ Deploy (strictest enforcement)
```

All phases are independent; each can be merged/deployed separately.

---

## Testing Summary

**Unit Tests:**
- `unify-mcp/tests/anti_llm_tools.rs` – Tool handler logic
- `unify-lsp/tests/anti_llm_gate.rs` – Diagnostic conversion
- `unify/tests/cmd_audit.rs` – CLI command behavior
- `unify-admission/tests/anti_llm_law.rs` – Gate admit/refuse

**Integration Tests:**
- `unify-integration-tests/` – End-to-end: scan → diagnostics → CLI output

**Manual Testing:**
- MCP client calls `audit/scan_directory` and verifies output shape
- LSP editor displays diagnostics from anti-llm gate
- CLI command `unify audit` produces expected exit codes
- Pre-commit hook blocks commits with blocking errors

---

## Configuration

All integrations accept `AntiLlmConfig`:

```rust
pub struct AntiLlmConfig {
    pub claim: ClaimConfig,
    // ... other rule configs
}
```

Future enhancements:
- Load from `anti-llm.toml` in project root
- Environment variable overrides
- Per-integration config variants (stricter LSP, lenient MCP)

---

## Conclusion

The `anti-llm-cheat-lsp` engine is integrated across unify-rs in four complementary ways:

1. **MCP** – AI-driven audits (external tools)
2. **LSP** – Real-time editor feedback (developers)
3. **CLI** – Batch audits and CI/CD gates
4. **Admission** – Automated policy enforcement (commits)

Each integration is self-contained, independently testable, and deployable. No changes to anti-llm-cheat-lsp itself are required.
