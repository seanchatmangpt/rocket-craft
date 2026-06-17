# anti-llm-cheat-lsp Integration — Executive Summary

## Overview

`anti-llm-cheat-lsp` is a complete detection engine with:
- **Input**: Source files via `scan_file()` or directory walk via `scan_directory()`
- **Processing**: 14 parsers + 17 rule modules → observations → diagnostics
- **Output**: `Vec<AntiLlmDiagnostic>` with `blocking: bool` flag

This document maps how to integrate it across the unify-rs workspace at four distinct points, each serving a different user/system:

1. **unify-mcp**: Tools for AI agents (Claude, external MCP clients)
2. **unify-lsp**: Diagnostics for editors (VS Code, Neovim, etc.)
3. **unify CLI**: Commands for developers (batch audits, CI/CD)
4. **unify-admission**: Policy gates (commit-time enforcement)

## Summary Table

| Integration | Module | What Changes | Public API Added | Risk | Value |
|---|---|---|---|---|---|
| **MCP** | unify-mcp | +anti_llm_tools.rs, main.rs | 2 tools + 1 attach fn | Low | High (AI tooling) |
| **LSP** | unify-lsp | +anti_llm_gate.rs, compositor | AntiLlmGate + 3 methods | Medium | High (editor UX) |
| **CLI** | unify | app.rs, commands.rs | Audit variant + handler | Low | Medium (dev tooling) |
| **Admission** | unify-admission | lib.rs | Law + Gate structs | Medium | Medium (enforcement) |

## Key Design Decisions

### 1. Engine Isolation
- `anti-llm-cheat-lsp` remains **unchanged**
- All integration logic lives in bridge modules (`anti_llm_tools.rs`, `anti_llm_gate.rs`, etc.)
- Zero circular dependencies

### 2. Independent Integration Points
- Each phase (P1–P4) can be deployed separately
- No blocking dependencies between phases
- All use the same underlying engine API

### 3. Type Conversion Strategy
- Observations → Diagnostics (engine layer)
- Diagnostics → LSP Diagnostic (unify-lsp bridge)
- Diagnostics → JSON (serde, unify-mcp/unify-cli)
- Diagnostics → Refusal (admission gate, unify-admission)

### 4. Configuration
- All integrations accept `AntiLlmConfig::default()` (hardcoded rules)
- Future: load from `anti-llm.toml` per integration

---

## Critical Public API (from anti-llm-cheat-lsp)

```rust
// Engine (already public, no changes needed)
pub fn scan_file(filepath: &str) -> Vec<Observation>
pub fn scan_directory(dirpath: &str) -> Vec<Observation>
pub fn evaluate_diagnostics(obs: &[Observation]) -> Vec<AntiLlmDiagnostic>
pub fn evaluate_diagnostics_with_config(obs: &[Observation], config: &AntiLlmConfig) 
    -> Vec<AntiLlmDiagnostic>

// Types (already public)
pub struct Observation { ... }
pub struct AntiLlmDiagnostic { 
    pub blocking: bool,  // KEY field for all integrations
    ...
}
pub struct AntiLlmConfig { ... }
```

**Assumption**: These types remain stable. If they change, all four integrations must be updated.

---

## Files to Create/Modify (Summary)

### New Files
```
unify-mcp/src/anti_llm_tools.rs
unify-lsp/src/anti_llm_gate.rs
unify-admission/src/anti_llm_runtime_law.rs (optional)
unify-*/tests/anti_llm_*.rs (test files, 4 per phase)
```

### Modified Files
```
unify-mcp/src/lib.rs              (1 line: pub mod)
unify-mcp/src/main.rs             (1 line: attach call)
unify-mcp/Cargo.toml              (1 dependency)

unify-lsp/src/lib.rs              (1 line: pub mod)
unify-lsp/src/compositor.rs       (1 field, 3 methods)
unify-lsp/Cargo.toml              (2 dependencies)

unify/src/app.rs                  (10 lines: Audit variant)
unify/src/commands.rs             (30 lines: cmd_audit impl)
unify/Cargo.toml                  (1 dependency)

unify-admission/src/lib.rs        (40 lines: law + gate)
unify-admission/Cargo.toml        (1 dependency)
```

**Total impact**: ~150 lines of new code, minimal changes to existing code.

---

## Rollout Phases

### Phase 1: unify-mcp (Week 1)
- [ ] Create anti_llm_tools.rs with two tools
- [ ] Wire into main.rs
- [ ] Add dependency to Cargo.toml
- [ ] Test MCP tool handlers
- **Deliverable**: MCP tools `audit/scan_directory` and `audit/evaluate_diagnostics`
- **Validation**: MCP client calls tool and gets expected JSON response

### Phase 2: unify-lsp (Week 1–2, concurrent with P1)
- [ ] Create anti_llm_gate.rs
- [ ] Update compositor.rs with anti_llm_gate field
- [ ] Add dependencies
- [ ] Test diagnostic conversion
- **Deliverable**: LSP diagnostics feed via compositor
- **Validation**: Editor displays warnings/errors from anti-llm rules

### Phase 3: unify CLI (Week 2, concurrent with P1–P2)
- [ ] Add Audit variant to app.rs
- [ ] Implement cmd_audit in commands.rs
- [ ] Add dependency
- [ ] Test CLI output and exit codes
- **Deliverable**: `unify audit [opts]` command
- **Validation**: CLI runs, exits 0 for clean, 1 for `--fail-on-blocking`

### Phase 4: unify-admission (Week 3, after P1–P3)
- [ ] Add law + gate to lib.rs
- [ ] Optionally add runtime law
- [ ] Test gate admit/refuse
- [ ] Document pre-commit integration
- **Deliverable**: AntiLlmAdmissionLaw + gate for policy enforcement
- **Validation**: Gate blocks pathological commits, passes clean ones

---

## Testing Strategy

### Unit Tests (per crate)
- **unify-mcp/tests/**: Tool handler logic, JSON serialization
- **unify-lsp/tests/**: Diagnostic conversion, LSP type mapping
- **unify/tests/**: Command parsing, exit codes, JSON output
- **unify-admission/tests/**: Gate admit/refuse logic, law constants

### Integration Tests
- **unify-integration-tests/**: End-to-end: scan → diagnostics → output
  - MCP tool output matches engine output
  - LSP diagnostics match MCP output (with format conversion)
  - CLI audit matches diagnostics
  - Admission gate matches CLI exit code logic

### Coverage Target
- All new code: 100% branch coverage
- No regression in existing tests
- Performance: scans < 1s on typical codebase

---

## Risk Assessment

| Phase | Risk | Mitigation |
|-------|------|-----------|
| **P1: MCP** | Low – isolated tool handlers | Unit tests for each handler, test with real MCP client |
| **P2: LSP** | Medium – affects editor UX | Type conversion tests, manual editor validation |
| **P3: CLI** | Low – new command, backward compatible | Exit code tests, CLI parsing tests |
| **P4: Admission** | Medium – enforces policy | Gate tests with pathological cases, pre-commit hook validation |

**Rollback**: Each phase can be reverted independently via `git revert`.

---

## Success Criteria

- [ ] All new tests pass (100% of new code)
- [ ] No regression in existing tests
- [ ] No compiler warnings
- [ ] Code review sign-off for each phase
- [ ] Manual end-to-end test for each phase
- [ ] Documentation (CLAUDE.md) updated
- [ ] No performance degradation (< 5% slowdown in CI)

---

## Deployment Checklist

**Pre-deployment:**
- [ ] All tests pass locally
- [ ] CI passes (if available)
- [ ] Code reviewed
- [ ] CLAUDE.md updated

**Deployment (per phase):**
- [ ] Merge to main
- [ ] Tag release (e.g., `unify-mcp-v0.X.0`)
- [ ] Build release binaries
- [ ] Smoke test in staging

**Post-deployment:**
- [ ] Monitor for errors
- [ ] Gather user feedback
- [ ] Plan next phase

---

## Future Enhancements

1. **Config Loading**: `anti-llm.toml` in project root
2. **Incremental Scanning**: MCP tool with `--changed-files-only`
3. **Rule Filtering**: CLI `--rule-codes` flag
4. **Custom Severity**: Per-rule severity override
5. **Batch Operations**: MCP tool to audit multiple dirs
6. **Caching**: Cache observations for repeated scans
7. **Parallel Scanning**: Multi-threaded file walk

---

## Detailed Documentation

For complete details, see:

1. **INTEGRATION_PLAN.md** – 600+ lines of detailed design
   - Input/output shapes for each tool
   - Public API signatures
   - Type conversion logic
   - Configuration strategy

2. **ANTI_LLM_ARCHITECTURE.md** – Visual diagrams
   - Core engine architecture
   - Data flow across integrations
   - Type conversion pipeline
   - Shared APIs

3. **INTEGRATION_CHECKLIST.md** – Step-by-step implementation
   - Task breakdown per phase
   - Code snippets (copy-paste ready)
   - Test specifications
   - Deployment procedures

4. **QUICK_REFERENCE.md** – Cheat sheet
   - Common errors & fixes
   - Example implementations
   - MCP/CLI usage
   - Useful commands

---

## Quick Start for Implementation

1. **Read QUICK_REFERENCE.md** (10 min) – understand what's being integrated
2. **Read ANTI_LLM_ARCHITECTURE.md** (15 min) – understand data flow
3. **Start Phase 1**: Create `unify-mcp/src/anti_llm_tools.rs` (1 hour)
4. **Test Phase 1**: Run `cargo test -p unify-mcp` (30 min)
5. **Code review**: Get sign-off (varies)
6. **Merge Phase 1** (5 min)
7. **Repeat for Phases 2–4**

**Total time estimate**: 3–4 weeks (4 weeks if strict review process, concurrent phases can shorten)

---

## Contact & Escalation

| Issue | Owner | Reference |
|-------|-------|-----------|
| Anti-LLM rule definitions | anti-llm-cheat-lsp maintainer | `src/rules/*.rs` |
| LSP integration questions | unify-lsp maintainer | ANTI_LLM_ARCHITECTURE.md |
| MCP protocol questions | unify-mcp maintainer | INTEGRATION_PLAN.md (P1) |
| Admission gate logic | unify-admission maintainer | INTEGRATION_CHECKLIST.md (P4) |

---

## Key Numbers

- **Lines of new code**: ~150 (across all 4 crates)
- **Files created**: 4 new files (anti_llm_tools.rs, anti_llm_gate.rs, anti_llm_runtime_law.rs, tests)
- **Files modified**: 8 (lib.rs, main.rs, app.rs, commands.rs, compositor.rs, + 3 Cargo.toml)
- **Public APIs added**: 8 (2 tools + 1 attach fn + AntiLlmGate + 3 methods + 2 law/gate structs)
- **Test files**: 4 (one per phase)
- **Dependencies added**: 4 (anti-llm-cheat-lsp to each of mcp, lsp, unify, admission) + 1 url (lsp)

---

## Documentation Files

This integration is fully documented in 5 markdown files:

1. **INTEGRATION_PLAN.md** (this directory) – Comprehensive design
2. **ANTI_LLM_ARCHITECTURE.md** (this directory) – Visual architecture
3. **INTEGRATION_CHECKLIST.md** (this directory) – Step-by-step tasks
4. **QUICK_REFERENCE.md** (this directory) – Cheat sheet
5. **INTEGRATION_SUMMARY.md** (this directory) – This file

Start with **QUICK_REFERENCE.md**, then dive into the appropriate detailed document.

---

## Version Compatibility

**Requires:**
- `anti-llm-cheat-lsp` crate already in unify-rs
- Rust 1.70+ (standard for workspace)
- No new external dependencies (all already in workspace)

**Produces:**
- unify-mcp v0.X.0+ (with MCP tools)
- unify-lsp v0.X.0+ (with LSP diagnostics)
- unify v0.X.0+ (with audit CLI command)
- unify-admission v0.X.0+ (with anti-llm gate)

---

## Conclusion

`anti-llm-cheat-lsp` is well-positioned for workspace-wide integration at four complementary points. The engine is stable and modular; integration focuses on bridge/wrapper modules with minimal impact to existing code.

**Start with Phase 1 (MCP), validate thoroughly, then proceed to subsequent phases in parallel. All four integrations are independent and can be deployed separately.**

For implementation guidance, follow the **INTEGRATION_CHECKLIST.md** step-by-step; refer to **ANTI_LLM_ARCHITECTURE.md** for visual diagrams and **INTEGRATION_PLAN.md** for detailed API specifications.
