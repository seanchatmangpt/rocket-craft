# anti-llm-cheat-lsp Integration into unify-rs Workspace

## Executive Summary

`anti-llm-cheat-lsp` is a complete detection engine with scanning (`scan_file`, `scan_directory`) and diagnostic generation (`evaluate_diagnostics`). It currently lives as a standalone crate in `unify-rs/anti-llm-cheat-lsp/` and should be integrated at four key points in the workspace:

1. **unify-mcp**: Expose a `scan_directory` tool for AI-driven code audits
2. **unify-lsp**: Feed diagnostics into the LSP server for real-time editor warnings
3. **unify CLI binary**: Add `unify audit` command for CLI-driven scanning
4. **unify-admission**: Wire as a policy gate to block commits with blocking diagnostics

---

## Integration Point 1: unify-mcp (MCP Tool Registration)

### Purpose
Expose anti-llm-cheat-lsp as an MCP tool so Claude Desktop and MCP clients can audit codebases in real-time.

### Current State
- `unify-mcp/src/server.rs` defines `McpServer` with `.with_tool()` builder method
- `unify-mcp/src/rocket_tools.rs` registers Rocket-specific tools (`manifest/list`, `project/audit`, `env/doctor`, `receipt/chain`)
- Tool handler signature: `fn(serde_json::Value) -> Result<serde_json::Value, String>`

### Design

#### New Module: `unify-mcp/src/anti_llm_tools.rs`

```rust
/// Public function to attach anti-llm scanning tools to the MCP server.
pub fn attach_anti_llm_tools(server: McpServer) -> McpServer {
    server
        .with_tool(scan_directory_descriptor(), handle_scan_directory)
        .with_tool(evaluate_diagnostics_descriptor(), handle_evaluate_diagnostics)
}

/// `audit/scan_directory` – scan a directory for LLM cheat patterns.
pub fn handle_scan_directory(params: serde_json::Value) -> Result<serde_json::Value, String> {
    let dir_path = params["dir_path"]
        .as_str()
        .ok_or_else(|| "Missing 'dir_path'".to_string())?;
    
    let observations = anti_llm_cheat_lsp::engine::scan_directory(dir_path);
    
    Ok(json!({
        "directory": dir_path,
        "observation_count": observations.len(),
        "observations": observations.iter()
            .map(|o| json!({
                "file_path": o.file_path,
                "line": o.line,
                "kind": o.kind,
                "construct": o.construct,
                "message": o.message
            }))
            .collect::<Vec<_>>()
    }))
}

/// `audit/evaluate_diagnostics` – convert observations to blocking/warning diagnostics.
pub fn handle_evaluate_diagnostics(params: serde_json::Value) -> Result<serde_json::Value, String> {
    let observations_json = params["observations"]
        .as_array()
        .ok_or_else(|| "Missing 'observations' array".to_string())?;
    
    // Deserialize observations from JSON
    let observations: Vec<Observation> = observations_json
        .iter()
        .filter_map(|o| serde_json::from_value(o).ok())
        .collect();
    
    let config = AntiLlmConfig::default(); // Could be parameterized
    let diagnostics = anti_llm_cheat_lsp::engine::evaluate_diagnostics_with_config(&observations, &config);
    
    let blocking_count = diagnostics.iter().filter(|d| d.blocking).count();
    let warning_count = diagnostics.len() - blocking_count;
    
    Ok(json!({
        "diagnostic_count": diagnostics.len(),
        "blocking_count": blocking_count,
        "warning_count": warning_count,
        "diagnostics": diagnostics.iter()
            .map(|d| json!({
                "code": d.code,
                "category": d.category,
                "file_path": d.file_path,
                "line": d.line,
                "message": d.message,
                "blocking": d.blocking,
                "required_correction": d.required_correction
            }))
            .collect::<Vec<_>>()
    }))
}

fn scan_directory_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        name: "audit/scan_directory".to_string(),
        description: "Scan a directory for LLM-generated stubs, cheats, and other anti-patterns".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "dir_path": {
                    "type": "string",
                    "description": "Absolute or relative path to directory to scan"
                }
            },
            "required": ["dir_path"]
        }),
    }
}

fn evaluate_diagnostics_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        name: "audit/evaluate_diagnostics".to_string(),
        description: "Convert raw observations into blocking vs. warning diagnostics".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "observations": {
                    "type": "array",
                    "description": "Array of observation objects from scan_directory"
                }
            },
            "required": ["observations"]
        }),
    }
}
```

#### Changes to `unify-mcp/src/main.rs`

```rust
use unify_mcp::anti_llm_tools;

fn main() {
    let server = McpServer::new(ServerInfo { ... });
    let server = tools::register_server_tools(server);
    let server = rocket_tools::attach_rocket_tools(server);
    let server = anti_llm_tools::attach_anti_llm_tools(server);  // NEW
    
    // stdin/stdout loop ...
}
```

#### Cargo.toml Changes

In `unify-mcp/Cargo.toml`, add:
```toml
[dependencies]
anti-llm-cheat-lsp = { path = "../anti-llm-cheat-lsp" }
```

And ensure `unify-rs/Cargo.toml` workspace declares `anti-llm-cheat-lsp` as a member.

### Input/Output Shape

**Request (audit/scan_directory):**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "audit/scan_directory",
    "arguments": {
      "dir_path": "/path/to/codebase"
    }
  },
  "id": 1
}
```

**Response:**
```json
{
  "directory": "/path/to/codebase",
  "observation_count": 42,
  "observations": [
    {
      "file_path": "src/lib.rs",
      "line": 15,
      "kind": "raw_text",
      "construct": "lsp-max",
      "message": "Raw text pattern 'lsp-max' detected"
    }
  ]
}
```

### Dependencies
- `anti-llm-cheat-lsp` (path dependency)
- `serde_json` (already in workspace)

---

## Integration Point 2: unify-lsp (Diagnostic Compositor)

### Purpose
Feed anti-llm-cheat-lsp diagnostics into the LSP server's diagnostic compositor so the editor (VS Code, Neovim, etc.) displays warnings and errors in real-time.

### Current State
- `unify-lsp/src/diagnostic.rs` defines `Diagnostic`, `DiagnosticSet`, `DiagnosticSeverity`
- `unify-lsp/src/compositor.rs` manages a `CompositorState` with multiple language servers
- LSP server will call `compositor.merge_diagnostics()` to aggregate results from all servers

### Design

#### New Module: `unify-lsp/src/anti_llm_gate.rs`

```rust
/// Wrapper that converts anti-llm observations/diagnostics into LSP diagnostics.
pub struct AntiLlmGate {
    config: AntiLlmConfig,
}

impl AntiLlmGate {
    pub fn new(config: AntiLlmConfig) -> Self {
        Self { config }
    }
    
    /// Scan a file and convert to LSP diagnostics.
    pub fn scan_file_to_lsp(&self, file_path: &str) -> DiagnosticSet {
        let observations = anti_llm_cheat_lsp::engine::scan_file(file_path);
        let anti_llm_diags = anti_llm_cheat_lsp::engine::evaluate_diagnostics_with_config(&observations, &self.config);
        
        let mut diag_set = DiagnosticSet::new();
        for diag in anti_llm_diags {
            let uri = url::Url::from_file_path(file_path)
                .ok()
                .map(|u| u.to_string())
                .unwrap_or_else(|| file_path.to_string());
            
            let lsp_diag = self.to_lsp_diagnostic(&diag);
            diag_set.add(uri, lsp_diag);
        }
        diag_set
    }
    
    /// Scan a directory and return aggregated LSP diagnostics.
    pub fn scan_directory_to_lsp(&self, dir_path: &str) -> DiagnosticSet {
        let observations = anti_llm_cheat_lsp::engine::scan_directory(dir_path);
        let anti_llm_diags = anti_llm_cheat_lsp::engine::evaluate_diagnostics_with_config(&observations, &self.config);
        
        let mut diag_set = DiagnosticSet::new();
        for diag in anti_llm_diags {
            let uri = url::Url::from_file_path(&diag.file_path)
                .ok()
                .map(|u| u.to_string())
                .unwrap_or_else(|| diag.file_path.clone());
            
            let lsp_diag = self.to_lsp_diagnostic(&diag);
            diag_set.add(uri, lsp_diag);
        }
        diag_set
    }
    
    /// Convert an anti-llm diagnostic to LSP format.
    fn to_lsp_diagnostic(&self, diag: &AntiLlmDiagnostic) -> Diagnostic {
        let severity = if diag.blocking {
            DiagnosticSeverity::Error
        } else {
            DiagnosticSeverity::Warning
        };
        
        Diagnostic {
            range: Range {
                start: Position {
                    line: (diag.line.saturating_sub(1)) as u32,
                    character: (diag.column.saturating_sub(1)) as u32,
                },
                end: Position {
                    line: (diag.line.saturating_sub(1)) as u32,
                    character: (diag.column.saturating_sub(1) + 10) as u32,
                },
            },
            severity,
            code: Some(diag.code.clone()),
            source: Some("anti-llm-cheat-lsp".to_string()),
            message: format!(
                "{}\nForbidden: {}\nCorrection: {}",
                diag.message,
                diag.forbidden_implication,
                diag.required_correction
            ),
        }
    }
}
```

#### Update `unify-lsp/src/compositor.rs`

Add anti-llm gate to the `CompositorState`:

```rust
pub struct CompositorState {
    servers: Vec<ServerEntry>,
    diagnostics: DiagnosticSet,
    gate: AndonGate,
    anti_llm_gate: Option<AntiLlmGate>,  // NEW
}

impl CompositorState {
    pub fn new() -> Self {
        Self {
            servers: Vec::new(),
            diagnostics: DiagnosticSet::new(),
            gate: AndonGate::new(),
            anti_llm_gate: None,  // Enabled optionally
        }
    }
    
    /// Enable anti-llm scanning with the given config.
    pub fn enable_anti_llm_gate(&mut self, config: AntiLlmConfig) {
        self.anti_llm_gate = Some(AntiLlmGate::new(config));
    }
    
    /// Scan a file and merge anti-llm diagnostics into the compositor.
    pub fn scan_file_anti_llm(&mut self, file_path: &str) {
        if let Some(gate) = &self.anti_llm_gate {
            let diag_set = gate.scan_file_to_lsp(file_path);
            self.diagnostics.merge(diag_set);
        }
    }
    
    /// Scan a directory and merge anti-llm diagnostics into the compositor.
    pub fn scan_directory_anti_llm(&mut self, dir_path: &str) {
        if let Some(gate) = &self.anti_llm_gate {
            let diag_set = gate.scan_directory_to_lsp(dir_path);
            self.diagnostics.merge(diag_set);
            
            // If any blocking diagnostics exist, raise the ANDON gate
            let blocking_diags = self.diagnostics.all_errors();
            if !blocking_diags.is_empty() {
                self.raise_andon("Anti-LLM cheat patterns detected; blocking errors present");
            }
        }
    }
}
```

#### Cargo.toml Changes

In `unify-lsp/Cargo.toml`, add:
```toml
[dependencies]
anti-llm-cheat-lsp = { path = "../anti-llm-cheat-lsp" }
url = "2.5"
```

### Integration with LSP Server

The LSP server implementation would call `scan_file_anti_llm()` when:
- A document opens (`textDocument/didOpen`)
- A document changes (`textDocument/didChange`)
- A manual audit is requested (`$/audit`)

Example pseudo-code in the LSP server:
```rust
async fn on_document_opened(&self, params: DidOpenTextDocumentParams) {
    let uri = &params.text_document.uri;
    let file_path = uri.path();
    self.compositor.scan_file_anti_llm(file_path);
    
    let diagnostics = self.compositor.get_diagnostics_for_uri(uri);
    self.client.publish_diagnostics(uri.clone(), diagnostics, None).await;
}
```

---

## Integration Point 3: unify CLI (`unify audit` subcommand)

### Purpose
Provide a CLI command so developers can audit their codebase from the terminal or in CI/CD pipelines.

### Current State
- `unify/src/app.rs` defines the `Commands` enum with subcommands
- `unify/src/commands.rs` implements handlers (`cmd_receipt`, `cmd_verify`, `cmd_gate`, etc.)
- All commands return `Result<Output, Box<dyn std::error::Error>>`

### Design

#### Update `unify/src/app.rs`

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

#### Update `unify/src/commands.rs`

```rust
pub fn run(cli: Cli) -> Result<Output, Box<dyn std::error::Error>> {
    match cli.command {
        // ... existing commands ...
        Commands::Audit { directory, blocking_only, fail_on_blocking } => {
            cmd_audit(&directory, blocking_only, fail_on_blocking)
        }
    }
}

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
    
    let success = if fail_on_blocking {
        blocking_count == 0
    } else {
        true
    };
    
    let output = Output {
        data: json!({
            "directory": dir_path,
            "diagnostic_count": filtered.len(),
            "blocking_count": blocking_count,
            "warning_count": warning_count,
            "diagnostics": filtered.iter()
                .map(|d| json!({
                    "code": d.code,
                    "category": d.category,
                    "file_path": d.file_path,
                    "line": d.line,
                    "message": d.message,
                    "blocking": d.blocking,
                    "required_correction": d.required_correction,
                    "required_next_proof": d.required_next_proof
                }))
                .collect::<Vec<_>>()
        }),
        success,
        message: if success {
            Some(format!("audited: {} warning(s), {} blocking error(s)", warning_count, blocking_count))
        } else {
            Some(format!("Audit FAILED: {} blocking error(s) detected", blocking_count))
        },
    };
    
    Ok(output)
}
```

#### Cargo.toml Changes

In `unify/Cargo.toml`, add:
```toml
[dependencies]
anti-llm-cheat-lsp = { path = "../anti-llm-cheat-lsp" }
```

### Usage Examples

```bash
# Scan current directory
unify audit

# Scan specific directory
unify audit --directory /path/to/rocket-craft

# Only show blocking errors
unify audit --blocking-only

# Fail if any blocking errors found (useful in CI)
unify audit --fail-on-blocking

# Output as JSON
unify audit --json
```

### CLI Output Example

```
audited: 12 warning(s), 3 blocking error(s)

Directory: .
Diagnostic Count: 15
Blocking: 3
Warnings: 12

Blocking Diagnostics:
  ANTI-LLM-CLAIM-001 (src/engine.rs:150)
    Tower-LLP raw text detected
    Required Correction: Remove hardcoded lsp-max string literal
    Required Next Proof: Verify no LSP-specific imports remain

...
```

---

## Integration Point 4: unify-admission (Policy Gate)

### Purpose
Wire anti-llm-cheat-lsp as a policy gate in the admission system, so commits with blocking diagnostics are automatically rejected.

### Current State
- `unify-admission/src/lib.rs` defines `StaticLaw`, `Admit<L>`, and `RuntimeLaw` traits
- Laws are registered in a `LawRegistry` and validated via `validate_all(path)`
- No integration with anti-llm-cheat-lsp yet

### Design

#### New Law: `AntiLlmAdmissionLaw`

```rust
// In unify-admission/src/lib.rs, add:

/// Law: code must not contain LLM-generated stubs or cheats (blocking: true).
pub struct AntiLlmAdmissionLaw;

impl StaticLaw for AntiLlmAdmissionLaw {
    const NAME: &'static str = "AntiLlmAdmission";
}

/// Gate that enforces `AntiLlmAdmissionLaw`.
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
        let diagnostics = anti_llm_cheat_lsp::engine::evaluate_diagnostics_with_config(&observations, &self.config);
        
        let blocking_diags: Vec<_> = diagnostics.iter().filter(|d| d.blocking).collect();
        
        if blocking_diags.is_empty() {
            Ok(())
        } else {
            let msg = format!(
                "Anti-LLM admission gate blocked commit: {} blocking diagnostic(s) detected\n{}",
                blocking_diags.len(),
                blocking_diags.iter()
                    .map(|d| format!("  - {} ({}:{}): {}", d.code, d.file_path, d.line, d.message))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            Err(Refusal::new(msg))
        }
    }
}

// Tests in lib.rs
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn anti_llm_admission_gate_admits_clean_directory() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("lib.rs"), "fn main() {}").unwrap();
        
        let gate = AntiLlmAdmissionGate::new(Default::default());
        assert!(gate.admit(&dir.path().to_path_buf()).is_ok());
    }
    
    #[test]
    fn anti_llm_admission_gate_rejects_blocked_pattern() {
        let dir = TempDir::new().unwrap();
        // Write a file with a known blocking pattern (e.g., lsp-max if it's blocking)
        fs::write(
            dir.path().join("lib.rs"),
            "// TODO: lsp-max integration",
        ).unwrap();
        
        let gate = AntiLlmAdmissionGate::new(Default::default());
        let result = gate.admit(&dir.path().to_path_buf());
        // This will depend on whether lsp-max is marked as blocking
        // Adjust test based on actual rule configuration
    }
}
```

#### New Module: `unify-admission/src/anti_llm_runtime_law.rs`

For runtime dispatch (no static law required), implement `RuntimeLaw`:

```rust
use super::RuntimeLaw;

/// A `RuntimeLaw` that validates code against anti-llm rules.
pub struct AntiLlmRuntimeLaw {
    config: anti_llm_cheat_lsp::config::AntiLlmConfig,
}

impl AntiLlmRuntimeLaw {
    pub fn new(config: anti_llm_cheat_lsp::config::AntiLlmConfig) -> Self {
        Self { config }
    }
}

impl RuntimeLaw for AntiLlmRuntimeLaw {
    fn name(&self) -> &str {
        "AntiLlmRuntime"
    }
    
    fn description(&self) -> &str {
        "Detects LLM-generated stubs, cheats, and anti-patterns in source code"
    }
    
    fn validate_path(&self, path: &std::path::Path) -> Result<(), LawViolation> {
        let path_str = path.to_string_lossy();
        let observations = anti_llm_cheat_lsp::engine::scan_directory(&path_str);
        let diagnostics = anti_llm_cheat_lsp::engine::evaluate_diagnostics_with_config(&observations, &self.config);
        
        let blocking_diags: Vec<_> = diagnostics.iter().filter(|d| d.blocking).collect();
        
        if blocking_diags.is_empty() {
            Ok(())
        } else {
            Err(LawViolation {
                law_name: self.name().to_string(),
                message: format!(
                    "{} blocking diagnostic(s): {}",
                    blocking_diags.len(),
                    blocking_diags.iter()
                        .map(|d| format!("{} ({}:{})", d.code, d.file_path, d.line))
                        .collect::<Vec<_>>()
                        .join("; ")
                ),
            })
        }
    }
}
```

#### Integration with CLI (optional)

If desired, add a `Gate` subcommand variant that uses anti-llm:

```rust
// In unify/src/app.rs, expand the Gate command:

Commands::Gate {
    #[arg(short, long)]
    law: String,  // e.g., "AntiLlmAdmission", "NonEmptyName", etc.
    #[arg(short, long)]
    path: Option<String>,  // e.g., "." for current directory
    // ...
}
```

#### Cargo.toml Changes

In `unify-admission/Cargo.toml`, add:
```toml
[dependencies]
anti-llm-cheat-lsp = { path = "../anti-llm-cheat-lsp" }
```

#### Usage in Pre-Commit Hook

Developers can integrate this into `.git/hooks/pre-commit`:

```bash
#!/bin/bash
set -e

# Scan staged files for anti-llm violations
unify audit --directory . --fail-on-blocking || {
    echo "Anti-LLM admission gate rejected commit (see above)"
    exit 1
}
```

Or via the gate CLI:

```bash
unify gate --law AntiLlmAdmission --path .
```

---

## Summary: Files to Change

### 1. New Crate Integration (unify-rs/Cargo.toml)

Ensure `anti-llm-cheat-lsp` is registered as a workspace member if not already:

```toml
[workspace]
members = [
    "unify",
    "unify-core",
    # ... existing ...
    "anti-llm-cheat-lsp",  # Ensure it's listed
]
```

### 2. unify-mcp Integration

| File | Change |
|------|--------|
| `unify-mcp/src/anti_llm_tools.rs` | **NEW** – Tool handlers and descriptors |
| `unify-mcp/src/lib.rs` | Add `pub mod anti_llm_tools;` |
| `unify-mcp/src/main.rs` | Call `anti_llm_tools::attach_anti_llm_tools(server)` |
| `unify-mcp/Cargo.toml` | Add `anti-llm-cheat-lsp` dependency |

### 3. unify-lsp Integration

| File | Change |
|------|--------|
| `unify-lsp/src/anti_llm_gate.rs` | **NEW** – LSP diagnostic wrapper |
| `unify-lsp/src/lib.rs` | Add `pub mod anti_llm_gate;` |
| `unify-lsp/src/compositor.rs` | Add `anti_llm_gate: Option<AntiLlmGate>` and related methods |
| `unify-lsp/Cargo.toml` | Add `anti-llm-cheat-lsp`, `url` dependencies |

### 4. unify CLI Integration

| File | Change |
|------|--------|
| `unify/src/app.rs` | Add `Commands::Audit { ... }` variant |
| `unify/src/commands.rs` | Implement `cmd_audit()` and wire in `run()` |
| `unify/Cargo.toml` | Add `anti-llm-cheat-lsp` dependency |

### 5. unify-admission Integration

| File | Change |
|------|--------|
| `unify-admission/src/lib.rs` | Add `AntiLlmAdmissionLaw` and `AntiLlmAdmissionGate` |
| `unify-admission/src/anti_llm_runtime_law.rs` | **NEW** – RuntimeLaw implementation |
| `unify-admission/Cargo.toml` | Add `anti-llm-cheat-lsp` dependency |

---

## API Dependencies and Public Types

### From anti-llm-cheat-lsp (must be exposed as public)

```rust
// Already public in anti-llm-cheat-lsp/src/lib.rs
pub mod config;        // AntiLlmConfig
pub mod diagnostics;   // AntiLlmDiagnostic
pub mod engine;        // scan_file, scan_directory, evaluate_diagnostics
pub mod observations;  // Observation
```

These types are already public; no changes needed to anti-llm-cheat-lsp's API.

### Type Conversions Required

| From | To | Bridge |
|------|----|---------| 
| `AntiLlmDiagnostic` | `Diagnostic` (unify-lsp) | `AntiLlmGate::to_lsp_diagnostic()` |
| `Observation` | `serde_json::Value` | Native serde |
| `AntiLlmDiagnostic` | `serde_json::Value` | Native serde |

---

## Testing Strategy

### Unit Tests

1. **unify-mcp/tests/anti_llm_tools.rs** – Test tool handlers in isolation
2. **unify-lsp/tests/anti_llm_gate.rs** – Test diagnostic conversion
3. **unify/tests/cmd_audit.rs** – Test CLI command parsing and execution
4. **unify-admission/tests/anti_llm_law.rs** – Test gate admit/refuse logic

### Integration Tests

1. **unify-integration-tests/** – End-to-end test:
   - Scan a directory with known violations
   - Verify MCP tool returns expected diagnostics
   - Verify LSP diagnostics are correctly formatted
   - Verify CLI command exits with correct code
   - Verify admission gate rejects blocked paths

### Example Test Structure

```rust
#[test]
fn audit_scan_detects_raw_text_patterns() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.rs"), "let x = lsp-max;").unwrap();
    
    let obs = anti_llm_cheat_lsp::engine::scan_directory(dir.path().to_str().unwrap());
    assert!(obs.len() > 0);
    assert!(obs.iter().any(|o| o.construct.contains("lsp-max")));
}

#[test]
fn anti_llm_gate_rejects_blocking_diagnostics() {
    let dir = TempDir::new().unwrap();
    // Write file with blocking pattern
    fs::write(dir.path().join("blocked.rs"), "// ANTI-LLM-CLAIM-004").unwrap();
    
    let gate = AntiLlmAdmissionGate::new(Default::default());
    let result = gate.admit(&dir.path().to_path_buf());
    assert!(result.is_err());
}
```

---

## Rollout Plan

1. **Phase 1**: Implement unify-mcp integration (lowest risk, highest value for AI tooling)
2. **Phase 2**: Implement unify-lsp integration (requires LSP server setup, may be in-progress)
3. **Phase 3**: Implement unify CLI integration (easy win, useful for developers)
4. **Phase 4**: Implement unify-admission integration (optional, for stricter enforcement)

Each phase can be deployed independently.

---

## Configuration and Customization

All integrations should accept an `AntiLlmConfig` parameter (currently defaulting in the engine). Future enhancements:

- Read config from `anti-llm.toml` in the project root
- Allow per-integration config overrides (e.g., MCP tools vs. LSP gates)
- Support rule whitelisting/blacklisting per integration

---

## Conclusion

`anti-llm-cheat-lsp` is well-positioned for workspace-wide integration. The engine's scanning and diagnostic generation are already modular; integrating them into unify-mcp, unify-lsp, the unify CLI, and unify-admission requires minimal changes to anti-llm-cheat-lsp itself and focuses on bridge/wrapper modules in each target crate.

The four integration points serve distinct use cases:
- **MCP**: AI-driven code audits (Claude Desktop, external clients)
- **LSP**: Real-time editor diagnostics (VS Code, Neovim, etc.)
- **CLI**: Developer-driven audits and CI/CD gates
- **Admission**: Automated policy enforcement on commits

All four can coexist without conflicts.
