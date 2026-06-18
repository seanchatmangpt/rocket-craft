# anti-llm-cheat-lsp Integration Checklist

## Pre-Integration Setup

- [ ] Ensure `anti-llm-cheat-lsp` is a registered member in `unify-rs/Cargo.toml`
- [ ] Verify `anti-llm-cheat-lsp` crate builds cleanly: `cd unify-rs && cargo build -p anti-llm-cheat-lsp`
- [ ] Confirm public API is stable:
  - [ ] `engine::scan_file(filepath) -> Vec<Observation>`
  - [ ] `engine::scan_directory(dirpath) -> Vec<Observation>`
  - [ ] `engine::evaluate_diagnostics(obs) -> Vec<AntiLlmDiagnostic>`
  - [ ] `engine::evaluate_diagnostics_with_config(obs, config) -> Vec<AntiLlmDiagnostic>`
  - [ ] `config::AntiLlmConfig` is public
  - [ ] `diagnostics::AntiLlmDiagnostic` is public
  - [ ] `observations::Observation` is public

---

## Phase 1: unify-mcp Integration

### Task 1.1: Create anti_llm_tools Module

- [ ] Create `unify-mcp/src/anti_llm_tools.rs`
- [ ] Implement `handle_scan_directory()` handler:
  ```rust
  pub fn handle_scan_directory(params: serde_json::Value) -> Result<serde_json::Value, String> {
      let dir_path = params["dir_path"].as_str()
          .ok_or_else(|| "Missing 'dir_path'".to_string())?;
      let observations = anti_llm_cheat_lsp::engine::scan_directory(dir_path);
      Ok(json!({
          "directory": dir_path,
          "observation_count": observations.len(),
          "observations": ...
      }))
  }
  ```
- [ ] Implement `handle_evaluate_diagnostics()` handler
- [ ] Implement `scan_directory_descriptor()` and `evaluate_diagnostics_descriptor()` functions
- [ ] Implement public `attach_anti_llm_tools(server: McpServer) -> McpServer` function

### Task 1.2: Update unify-mcp Module Structure

- [ ] Add `pub mod anti_llm_tools;` to `unify-mcp/src/lib.rs`
- [ ] Update `unify-mcp/src/main.rs`:
  ```rust
  let server = anti_llm_tools::attach_anti_llm_tools(server);
  ```

### Task 1.3: Update Dependencies

- [ ] Add to `unify-mcp/Cargo.toml`:
  ```toml
  [dependencies]
  anti-llm-cheat-lsp = { path = "../anti-llm-cheat-lsp" }
  ```

### Task 1.4: Test MCP Tools

- [ ] Create `unify-mcp/tests/anti_llm_tools.rs` with:
  - [ ] Test `handle_scan_directory()` with a test directory
  - [ ] Test `handle_evaluate_diagnostics()` with sample observations
  - [ ] Verify JSON response shape matches expected format
- [ ] Manual test with MCP client:
  ```bash
  cargo build -p unify-mcp
  # Test with MCP client, call audit/scan_directory
  ```
- [ ] Run: `cargo test -p unify-mcp`

---

## Phase 2: unify-lsp Integration

### Task 2.1: Create AntiLlmGate Module

- [ ] Create `unify-lsp/src/anti_llm_gate.rs`
- [ ] Implement `AntiLlmGate` struct:
  ```rust
  pub struct AntiLlmGate {
      config: AntiLlmConfig,
  }
  
  impl AntiLlmGate {
      pub fn new(config: AntiLlmConfig) -> Self { ... }
      pub fn scan_file_to_lsp(&self, file_path: &str) -> DiagnosticSet { ... }
      pub fn scan_directory_to_lsp(&self, dir_path: &str) -> DiagnosticSet { ... }
      fn to_lsp_diagnostic(&self, diag: &AntiLlmDiagnostic) -> Diagnostic { ... }
  }
  ```
- [ ] Implement conversion from `AntiLlmDiagnostic` to LSP `Diagnostic`:
  - [ ] Map `blocking: true` → `DiagnosticSeverity::Error`
  - [ ] Map `blocking: false` → `DiagnosticSeverity::Warning`
  - [ ] Include code, message, and source in LSP diagnostic
  - [ ] Handle line/column conversion (1-based → 0-based)

### Task 2.2: Update CompositorState

- [ ] Modify `unify-lsp/src/compositor.rs`:
  - [ ] Add field: `anti_llm_gate: Option<AntiLlmGate>`
  - [ ] Add method: `pub fn enable_anti_llm_gate(&mut self, config: AntiLlmConfig)`
  - [ ] Add method: `pub fn scan_file_anti_llm(&mut self, file_path: &str)`
  - [ ] Add method: `pub fn scan_directory_anti_llm(&mut self, dir_path: &str)`
  - [ ] In `scan_directory_anti_llm()`, raise ANDON gate if blocking diagnostics found

### Task 2.3: Update Module Exports

- [ ] Add `pub mod anti_llm_gate;` to `unify-lsp/src/lib.rs`

### Task 2.4: Update Dependencies

- [ ] Add to `unify-lsp/Cargo.toml`:
  ```toml
  [dependencies]
  anti-llm-cheat-lsp = { path = "../anti-llm-cheat-lsp" }
  url = "2.5"
  ```

### Task 2.5: Test LSP Diagnostics

- [ ] Create `unify-lsp/tests/anti_llm_gate.rs` with:
  - [ ] Test `scan_file_to_lsp()` returns correctly formatted diagnostics
  - [ ] Test `scan_directory_to_lsp()` aggregates across files
  - [ ] Test blocking/warning severity mapping
  - [ ] Test line/column conversion
- [ ] Run: `cargo test -p unify-lsp`

---

## Phase 3: unify CLI Integration

### Task 3.1: Add Audit Subcommand

- [ ] Modify `unify/src/app.rs`:
  ```rust
  #[derive(Subcommand, Debug)]
  pub enum Commands {
      // ... existing commands ...
      
      /// Scan a directory for LLM cheats and anti-patterns
      Audit {
          /// Path to scan (defaults to current directory)
          #[arg(short, long, default_value = ".")]
          directory: String,
          
          /// Show only blocking diagnostics (errors)
          #[arg(long)]
          blocking_only: bool,
          
          /// Exit with non-zero if any blocking diagnostics found
          #[arg(long)]
          fail_on_blocking: bool,
      },
  }
  ```

### Task 3.2: Implement Audit Handler

- [ ] Add to `unify/src/commands.rs`:
  ```rust
  pub fn cmd_audit(
      dir_path: &str,
      blocking_only: bool,
      fail_on_blocking: bool,
  ) -> Result<Output, Box<dyn std::error::Error>> {
      let observations = anti_llm_cheat_lsp::engine::scan_directory(dir_path);
      let config = AntiLlmConfig::default();
      let diagnostics = anti_llm_cheat_lsp::engine::evaluate_diagnostics_with_config(&observations, &config);
      
      let mut filtered = diagnostics.clone();
      if blocking_only {
          filtered.retain(|d| d.blocking);
      }
      
      let blocking_count = diagnostics.iter().filter(|d| d.blocking).count();
      let warning_count = diagnostics.len() - blocking_count;
      let success = if fail_on_blocking { blocking_count == 0 } else { true };
      
      Ok(Output {
          data: json!({ /* ... */ }),
          success,
          message: Some(format!("Audit: {} warning(s), {} blocking", warning_count, blocking_count)),
      })
  }
  ```
- [ ] Wire into `run()` function:
  ```rust
  Commands::Audit { directory, blocking_only, fail_on_blocking } => {
      cmd_audit(&directory, blocking_only, fail_on_blocking)
  }
  ```

### Task 3.3: Update Dependencies

- [ ] Add to `unify/Cargo.toml`:
  ```toml
  [dependencies]
  anti-llm-cheat-lsp = { path = "../anti-llm-cheat-lsp" }
  ```

### Task 3.4: Test CLI Command

- [ ] Create `unify/tests/cmd_audit.rs` with:
  - [ ] Test `cmd_audit()` with clean directory → success
  - [ ] Test `cmd_audit()` with violations → diagnostics
  - [ ] Test `--blocking-only` flag
  - [ ] Test `--fail-on-blocking` flag → exit code 1
  - [ ] Test JSON output format
- [ ] Manual test:
  ```bash
  cd unify-rs
  cargo build -p unify
  ./target/debug/unify audit --directory .
  ./target/debug/unify audit --directory . --json
  ./target/debug/unify audit --directory . --fail-on-blocking && echo "clean" || echo "blocked"
  ```
- [ ] Run: `cargo test -p unify`

---

## Phase 4: unify-admission Integration

### Task 4.1: Add AntiLlmAdmissionLaw to Core

- [ ] Modify `unify-admission/src/lib.rs`:
  ```rust
  /// Law: code must not contain LLM-generated stubs or cheats (blocking: true).
  pub struct AntiLlmAdmissionLaw;
  
  impl StaticLaw for AntiLlmAdmissionLaw {
      const NAME: &'static str = "AntiLlmAdmission";
  }
  ```

### Task 4.2: Implement AntiLlmAdmissionGate

- [ ] Add to `unify-admission/src/lib.rs`:
  ```rust
  pub struct AntiLlmAdmissionGate {
      config: anti_llm_cheat_lsp::config::AntiLlmConfig,
  }
  
  impl AntiLlmAdmissionGate {
      pub fn new(config: anti_llm_cheat_lsp::config::AntiLlmConfig) -> Self {
          Self { config }
      }
  }
  
  impl Admit<AntiLlmAdmissionLaw> for AntiLlmAdmissionGate {
      type Artifact = std::path::PathBuf;
      
      fn admit(&self, path: &std::path::PathBuf) -> Result<(), Refusal<AntiLlmAdmissionLaw>> {
          let path_str = path.to_string_lossy();
          let observations = anti_llm_cheat_lsp::engine::scan_directory(&path_str);
          let diags = anti_llm_cheat_lsp::engine::evaluate_diagnostics_with_config(&observations, &self.config);
          let blocking: Vec<_> = diags.iter().filter(|d| d.blocking).collect();
          
          if blocking.is_empty() {
              Ok(())
          } else {
              let msg = format!(
                  "Anti-LLM admission gate blocked commit: {} blocking error(s)\n{}",
                  blocking.len(),
                  blocking.iter()
                      .map(|d| format!("  - {} ({}:{}): {}", d.code, d.file_path, d.line, d.message))
                      .collect::<Vec<_>>()
                      .join("\n")
              );
              Err(Refusal::new(msg))
          }
      }
  }
  ```

### Task 4.3: Create AntiLlmRuntimeLaw (Optional)

- [ ] Create `unify-admission/src/anti_llm_runtime_law.rs`:
  ```rust
  pub struct AntiLlmRuntimeLaw {
      config: anti_llm_cheat_lsp::config::AntiLlmConfig,
  }
  
  impl AntiLlmRuntimeLaw {
      pub fn new(config: anti_llm_cheat_lsp::config::AntiLlmConfig) -> Self {
          Self { config }
      }
  }
  
  impl RuntimeLaw for AntiLlmRuntimeLaw {
      fn name(&self) -> &str { "AntiLlmRuntime" }
      fn description(&self) -> &str { "LLM cheat detection" }
      fn validate_path(&self, path: &Path) -> Result<(), LawViolation> {
          let path_str = path.to_string_lossy();
          let observations = anti_llm_cheat_lsp::engine::scan_directory(&path_str);
          let diags = anti_llm_cheat_lsp::engine::evaluate_diagnostics_with_config(&observations, &self.config);
          let blocking: Vec<_> = diags.iter().filter(|d| d.blocking).collect();
          
          if blocking.is_empty() {
              Ok(())
          } else {
              Err(LawViolation {
                  law_name: self.name().to_string(),
                  message: format!("{} blocking error(s)", blocking.len()),
              })
          }
      }
  }
  ```
- [ ] Add `pub mod anti_llm_runtime_law;` to `unify-admission/src/lib.rs`

### Task 4.4: Add Unit Tests

- [ ] Add to `unify-admission/src/lib.rs`:
  ```rust
  #[cfg(test)]
  mod anti_llm_tests {
      use super::*;
      use std::fs;
      use tempfile::TempDir;
      
      #[test]
      fn anti_llm_gate_admits_clean_directory() {
          let dir = TempDir::new().unwrap();
          fs::write(dir.path().join("lib.rs"), "fn main() {}").unwrap();
          let gate = AntiLlmAdmissionGate::new(Default::default());
          assert!(gate.admit(&dir.path().to_path_buf()).is_ok());
      }
      
      #[test]
      fn anti_llm_gate_name_constant() {
          assert_eq!(AntiLlmAdmissionLaw::NAME, "AntiLlmAdmission");
      }
  }
  ```

### Task 4.5: Update Dependencies

- [ ] Add to `unify-admission/Cargo.toml`:
  ```toml
  [dependencies]
  anti-llm-cheat-lsp = { path = "../anti-llm-cheat-lsp" }
  ```

### Task 4.6: Test Admission Gates

- [ ] Create `unify-admission/tests/anti_llm_law.rs` with:
  - [ ] Test gate admits clean directory
  - [ ] Test gate rejects directory with blocking patterns (if available)
  - [ ] Test gate name matches constant
- [ ] Run: `cargo test -p unify-admission`

---

## Integration Testing

### Task I.1: End-to-End Test Suite

- [ ] Create `unify-integration-tests/src/anti_llm_integration.rs`:
  ```rust
  #[test]
  fn anti_llm_scan_produces_consistent_results() {
      // Create temp dir with known pattern
      let dir = TempDir::new().unwrap();
      fs::write(dir.path().join("test.rs"), "let x = lsp-max;").unwrap();
      
      // Scan via engine
      let obs = anti_llm_cheat_lsp::engine::scan_directory(dir.path().to_str().unwrap());
      assert!(obs.len() > 0);
      
      // Convert to diagnostics
      let diags = anti_llm_cheat_lsp::engine::evaluate_diagnostics(&obs);
      assert!(diags.len() > 0);
  }
  
  #[test]
  fn anti_llm_mcp_tool_matches_engine() {
      // Test that MCP tool returns same results as direct engine call
  }
  
  #[test]
  fn anti_llm_lsp_diagnostic_conversion() {
      // Test that LSP conversion preserves diagnostic info
  }
  
  #[test]
  fn anti_llm_cli_audit_exit_codes() {
      // Test that CLI exits 0 for clean, non-zero for blocked
  }
  
  #[test]
  fn anti_llm_gate_integration() {
      // Test that admission gate blocks commits with blocking errors
  }
  ```

### Task I.2: Run Full Test Suite

- [ ] `cd unify-rs && cargo test --workspace`
- [ ] Verify all tests pass for:
  - [ ] unify-mcp
  - [ ] unify-lsp
  - [ ] unify
  - [ ] unify-admission
  - [ ] unify-integration-tests

### Task I.3: Cross-Component Verification

- [ ] [ ] MCP tool output matches engine output
- [ ] [ ] LSP diagnostics match MCP output (with proper format conversion)
- [ ] [ ] CLI audit matches LSP diagnostics (same underlying engine)
- [ ] [ ] Admission gate matches CLI audit (same engine, stricter enforcement)

---

## Documentation

### Task D.1: Update CLAUDE.md

- [ ] Add section to `/home/user/rocket-craft/unify-rs/CLAUDE.md`:
  ```markdown
  ### anti-llm-cheat-lsp Integration
  
  The `anti-llm-cheat-lsp` crate is integrated at four points:
  1. **unify-mcp**: Expose `audit/scan_directory` and `audit/evaluate_diagnostics` tools
  2. **unify-lsp**: Feed diagnostics into LSP server via AntiLlmGate
  3. **unify**: CLI command `unify audit` for batch scans
  4. **unify-admission**: Admission gate to block commits with blocking errors
  
  [Details...]
  ```

### Task D.2: Add Examples

- [ ] Create `unify-rs/examples/audit_directory.rs`:
  ```rust
  use anti_llm_cheat_lsp::engine;
  
  fn main() {
      let obs = engine::scan_directory(".");
      let diags = engine::evaluate_diagnostics(&obs);
      for d in diags {
          println!("{}: {}", d.code, d.message);
      }
  }
  ```

### Task D.3: Update README

- [ ] Add usage examples to main README

---

## Deployment & Rollout

### Phase 1 Deployment

- [ ] All Phase 1 tests pass
- [ ] Code review: anti_llm_tools.rs
- [ ] Merge to main
- [ ] Tag: `unify-mcp-v0.X.0`

### Phase 2 Deployment

- [ ] All Phase 2 tests pass
- [ ] Code review: anti_llm_gate.rs, compositor updates
- [ ] Merge to main
- [ ] Tag: `unify-lsp-v0.X.0`

### Phase 3 Deployment

- [ ] All Phase 3 tests pass
- [ ] Code review: app.rs, commands.rs
- [ ] Merge to main
- [ ] Tag: `unify-v0.X.0`
- [ ] Update CLI documentation

### Phase 4 Deployment (Optional)

- [ ] All Phase 4 tests pass
- [ ] Code review: admission law and gate
- [ ] Merge to main
- [ ] Tag: `unify-admission-v0.X.0`
- [ ] Document pre-commit hook setup

---

## Post-Deployment Validation

- [ ] [ ] MCP client can call `audit/scan_directory`
- [ ] [ ] LSP editor displays anti-llm diagnostics
- [ ] [ ] CLI: `unify audit` produces correct output
- [ ] [ ] Admission gate blocks test case with blocking errors
- [ ] [ ] All workspace tests still pass: `cargo test --workspace`

---

## Rollback Plan

If issues arise after deployment:

1. **Phase 1 Rollback:**
   - Remove `anti_llm_tools.rs`
   - Revert `unify-mcp/src/main.rs`
   - Revert `unify-mcp/Cargo.toml`

2. **Phase 2 Rollback:**
   - Remove `anti_llm_gate.rs`
   - Revert `unify-lsp/src/compositor.rs`
   - Revert `unify-lsp/Cargo.toml`

3. **Phase 3 Rollback:**
   - Remove `Audit` command variant
   - Remove `cmd_audit()` function
   - Revert `unify/Cargo.toml`

4. **Phase 4 Rollback:**
   - Remove `AntiLlmAdmissionLaw` and gate
   - Revert `unify-admission/Cargo.toml`

In all cases: `git revert <commit_hash>` is preferred over manual cleanup.

---

## Success Criteria

- [ ] All new tests pass (100% coverage of new code)
- [ ] No regression in existing tests
- [ ] Code compiles with no warnings
- [ ] All four integration points are documented
- [ ] At least one manual end-to-end test completed for each phase
- [ ] Performance impact is negligible (scans < 1s on typical codebase)

---

## Known Issues / Future Enhancements

- [ ] **Config Loading**: Currently uses `AntiLlmConfig::default()`. Future: load from `anti-llm.toml`.
- [ ] **Incremental Scanning**: MCP tool could accept `--changed-files-only` flag for faster audits.
- [ ] **Output Filtering**: CLI could accept `--rule-codes` to filter by specific rule codes.
- [ ] **Custom Severity**: Per-rule severity override (e.g., treat warnings as errors).
- [ ] **Batch Operations**: MCP tool to audit multiple directories in one call.

---

## Contacts & Questions

- **Code Questions**: Refer to INTEGRATION_PLAN.md and ANTI_LLM_ARCHITECTURE.md
- **Anti-LLM Rules**: See `anti-llm-cheat-lsp/src/rules/` for rule implementations
- **Test Failures**: Check `anti-llm-cheat-lsp/src/parsers/` for observation generation
