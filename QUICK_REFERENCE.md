# anti-llm-cheat-lsp Integration — Quick Reference

## What This Integrates

`anti-llm-cheat-lsp` is a complete LLM cheat detection engine with:
- **Scanning**: `scan_file()`, `scan_directory()`
- **Diagnosis**: `evaluate_diagnostics()` + rule-based categorization
- **Blocking**: Some diagnostics marked `blocking: true` to enforce strict policies

## Four Integration Points

| Point | Module | Use Case | Tool | Exit Code |
|-------|--------|----------|------|-----------|
| **MCP** | unify-mcp | AI-driven audits | `audit/scan_directory` tool | JSON response |
| **LSP** | unify-lsp | Real-time editor warnings | Diagnostic feed to editor | Severity level |
| **CLI** | unify | Batch audits, CI/CD | `unify audit` command | 0=clean, 1=blocked (if flag) |
| **Admission** | unify-admission | Commit gates | `AntiLlmAdmissionLaw` gate | Refusal if blocked |

## Key Files to Create/Modify

### Phase 1: unify-mcp (MCP Tools)

**Create:**
```
unify-mcp/src/anti_llm_tools.rs  [NEW]
```

**Modify:**
```
unify-mcp/src/lib.rs               [add pub mod]
unify-mcp/src/main.rs              [call attach_anti_llm_tools]
unify-mcp/Cargo.toml               [add dependency]
```

**Exposes:**
- `audit/scan_directory` → observations
- `audit/evaluate_diagnostics` → blocking/warning diagnostics

---

### Phase 2: unify-lsp (LSP Diagnostics)

**Create:**
```
unify-lsp/src/anti_llm_gate.rs  [NEW]
```

**Modify:**
```
unify-lsp/src/lib.rs            [add pub mod]
unify-lsp/src/compositor.rs     [add anti_llm_gate field + methods]
unify-lsp/Cargo.toml            [add dependencies]
```

**Maps:**
- `AntiLlmDiagnostic.blocking → Diagnostic.Error`
- `AntiLlmDiagnostic.warning → Diagnostic.Warning`

---

### Phase 3: unify CLI (Command)

**Modify:**
```
unify/src/app.rs                [add Audit variant]
unify/src/commands.rs           [implement cmd_audit]
unify/Cargo.toml                [add dependency]
```

**Usage:**
```bash
unify audit [--directory .] [--blocking-only] [--fail-on-blocking] [--json]
```

---

### Phase 4: unify-admission (Admission Gate)

**Modify:**
```
unify-admission/src/lib.rs      [add AntiLlmAdmissionLaw + gate]
unify-admission/Cargo.toml      [add dependency]
```

**Create (optional):**
```
unify-admission/src/anti_llm_runtime_law.rs  [NEW]
```

**Enforces:**
- Commits blocked if `blocking: true` diagnostics found
- Pre-commit hook integration

---

## Core Data Types

### Observation (raw pattern)
```rust
pub struct Observation {
    pub file_path: String,
    pub line: usize,
    pub kind: String,           // "raw_text", "test_smell", etc.
    pub construct: String,      // "lsp-max", "assert_contains", etc.
    pub message: String,
}
```

### AntiLlmDiagnostic (evaluated)
```rust
pub struct AntiLlmDiagnostic {
    pub code: String,                   // "ANTI-LLM-CLAIM-001"
    pub category: String,               // "claims", "complexity", etc.
    pub file_path: String,
    pub line: usize,
    pub message: String,
    pub blocking: bool,                 // Key: determines severity
    pub required_correction: String,
    pub required_next_proof: String,
}
```

### AntiLlmConfig (rule configuration)
```rust
pub struct AntiLlmConfig {
    pub claim: ClaimConfig,
    // ... other rule configs
}
```

---

## Type Conversions

```
Observation (raw)
    ↓ [parser modules]
    → Observation (16 location fields)
    ↓ [rules modules]
    → AntiLlmDiagnostic (code, category, blocking)
    ↓ [bridge converters]
    → LSP Diagnostic (message, severity, code, range)
    → JSON (for MCP, CLI)
    → Refusal (for admission gate)
```

---

## Testing (Quick Setup)

```bash
# Phase 1
cargo test -p unify-mcp

# Phase 2
cargo test -p unify-lsp

# Phase 3
cargo test -p unify

# Phase 4
cargo test -p unify-admission

# All
cargo test --workspace
```

---

## Common Errors & Solutions

| Error | Cause | Fix |
|-------|-------|-----|
| `unrereaddressed import anti_llm_cheat_lsp` | Dependency not in Cargo.toml | Add to `[dependencies]` |
| `Observation not public` | API stability issue | Ensure it's `pub struct` in anti-llm-cheat-lsp |
| `DiagnosticSet not found` | LSP types not imported | Import from `unify_lsp::diagnostic` |
| `AntiLlmConfig not found` | Config type not imported | `use anti_llm_cheat_lsp::config::*` |

---

## Minimal Example: Add Audit to CLI

```rust
// 1. app.rs
#[derive(Subcommand, Debug)]
pub enum Commands {
    Audit {
        #[arg(short, long, default_value = ".")]
        directory: String,
    },
}

// 2. commands.rs
pub fn run(cli: Cli) -> Result<Output, Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Audit { directory } => cmd_audit(&directory),
    }
}

pub fn cmd_audit(dir: &str) -> Result<Output, Box<dyn std::error::Error>> {
    let obs = anti_llm_cheat_lsp::engine::scan_directory(dir);
    let diags = anti_llm_cheat_lsp::engine::evaluate_diagnostics(&obs);
    Ok(Output {
        data: json!({ "diagnostics": diags }),
        success: true,
        message: Some(format!("Found {} diagnostics", diags.len())),
    })
}

// 3. Cargo.toml
[dependencies]
anti-llm-cheat-lsp = { path = "../anti-llm-cheat-lsp" }

// 4. Test
cargo run -p unify -- audit --directory .
```

---

## Diagnostic Severity Mapping

| AntiLlmDiagnostic | → | LSP Severity | → | Editor Display |
|---|---|---|---|---|
| `blocking: true` | → | `Error` | → | 🔴 Red squiggle |
| `blocking: false` | → | `Warning` | → | 🟡 Yellow squiggle |
| (info-only) | → | `Information` | → | ℹ️ Lightbulb |

---

## MCP Tool Shape

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "audit/scan_directory",
    "arguments": { "dir_path": "." }
  },
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [{
      "type": "text",
      "text": "{\"directory\": \".\", \"observation_count\": 42, \"observations\": [...]}"
    }]
  },
  "id": 1
}
```

---

## CLI Usage Examples

```bash
# Basic audit (exit 0 unless --fail-on-blocking)
unify audit

# Scan specific directory
unify audit --directory /path/to/code

# Only show blocking errors
unify audit --blocking-only

# Fail CI if any blocking errors found
unify audit --fail-on-blocking && echo "Pass" || echo "Fail"

# JSON output for parsing
unify audit --json | jq '.data.blocking_count'

# In pre-commit hook
#!/bin/bash
unify audit --directory . --fail-on-blocking || exit 1
```

---

## Dependency Tree

```
anti-llm-cheat-lsp (root)
    ↓
unify-mcp (MCP tools)
unify-lsp (LSP gate)
unify (CLI)
unify-admission (policy gate)
    ↓
All use engine::{scan_file, scan_directory, evaluate_diagnostics}
```

**No circular dependencies.** Each crate uses anti-llm-cheat-lsp independently.

---

## Performance Characteristics

- `scan_file()`: ~1-10ms per file (depends on size)
- `scan_directory()`: ~500ms per 1000 files (sequential walk)
- `evaluate_diagnostics()`: ~10ms per 100 observations (rule execution)

**Optimization Tips:**
- Cache observations between calls
- Scan changed files only (future enhancement)
- Run admission gate on staging, not all files

---

## Configuration

All integrations use `AntiLlmConfig::default()` (hardcoded rules). Future enhancements:

```toml
# anti-llm.toml (future)
[claim]
domain_terms = ["Rocket", "Nexus"]

[rules]
blocking_codes = ["ANTI-LLM-CLAIM-004", "ANTI-LLM-DETERMINISM-*"]
```

---

## Checklist: Verify Integration Works

- [ ] MCP tool `audit/scan_directory` returns observations
- [ ] MCP tool `audit/evaluate_diagnostics` returns diagnostics
- [ ] LSP diagnostics appear in editor with correct severity
- [ ] CLI `unify audit` runs and produces output
- [ ] CLI exits 1 if `--fail-on-blocking` and blocking errors found
- [ ] Admission gate blocks pathological paths
- [ ] All workspace tests pass
- [ ] No new compiler warnings

---

## Useful Commands

```bash
# Build entire workspace
cd unify-rs && cargo build --workspace

# Test specific crate
cargo test -p unify-mcp

# Check without building
cargo check --workspace

# Format and lint
cargo fmt --all && cargo clippy --all

# Build MCP server standalone
cargo build -p unify-mcp --release
./target/release/unify-mcp  # Ready for stdio mode

# Quick CLI test
cargo build -p unify && ./target/debug/unify audit .

# Run integration tests only
cargo test -p unify-integration-tests
```

---

## Next Steps

1. **Start with Phase 1**: MCP tools are lowest risk, highest value
2. **Test thoroughly**: Each phase has its own test suite
3. **Integrate in order**: P1 → P2 → P3 → P4
4. **Document as you go**: Update CLAUDE.md with integration notes
5. **Gather feedback**: Verify output quality before Phase 4 (strict enforcement)

---

## Questions?

Refer to:
- **INTEGRATION_PLAN.md** — detailed design & API specs
- **ANTI_LLM_ARCHITECTURE.md** — visual architecture & type conversions
- **INTEGRATION_CHECKLIST.md** — step-by-step implementation guide

---

## Nexus Engine Typestates & Builders Quick Reference

| Crate / Machine | Typestate States | Builder Type | Runtime State Enum | Runtime Rejection Error |
|---|---|---|---|---|
| `nexus-combat` | `Idle`, `Attacking`, `Parrying`, `PerfectParrying`, `Dodging` | `CombatMachineBuilder` | `CombatState` | `CombatTransitionError` |
| `nexus-session` | `Connecting`, `Authenticated`, `InLobby`, `InMatch`, `Spectating`, `Disconnected` | `PlayerSessionBuilder` | `SessionState` | `SessionTransitionError` |
| `nexus-net` | `Disconnected`, `Handshaking`, `Connected`, `Authenticated`, `InLobby`, `InMatch` | `ConnectionBuilder` | `ConnectionState` | `ConnectionTransitionError` |
| `nexus-economy` | `OpenForBids`, `BidAccepted`, `AuctionClosed`, `AuctionCancelled` | `AuctionBuilder` | `AuctionState` | `AuctionTransitionError` |
