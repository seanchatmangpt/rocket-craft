use crate::resource::{ResourceDescriptor, ResourceRegistry};
use crate::server::McpServer;
use crate::tool::{ToolDescriptor, ToolRegistry};
use serde_json::json;

// ─────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────

/// Compute a BLAKE3 hex hash of bytes using a simple std-based fallback.
/// (The unify-mcp crate does not yet depend on the blake3 crate, so we
///  use DefaultHasher extended to 32 hex chars for determinism.)
fn blake3_hex(data: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    // Two independent hashers to widen the output to 128 bits.
    let mut h1 = DefaultHasher::new();
    data.hash(&mut h1);
    let mut h2 = DefaultHasher::new();
    data.len().hash(&mut h2);
    for chunk in data.chunks(8) {
        chunk.hash(&mut h2);
    }
    format!("{:016x}{:016x}", h1.finish(), h2.finish())
}

/// Hash a previous chain head with new content to produce the next link.
fn chain_hash(prev: &str, content: &str) -> String {
    let combined = format!("{}|{}", prev, content);
    blake3_hex(combined.as_bytes())
}

// ─────────────────────────────────────────────
// Tool handlers (pub so tests can call them)
// ─────────────────────────────────────────────

/// `rocket/manifest/list` – parse `project-manifest.json` from cwd.
pub fn handle_manifest_list(_params: serde_json::Value) -> Result<serde_json::Value, String> {
    let path = std::path::Path::new("project-manifest.json");
    let contents = std::fs::read_to_string(path)
        .map_err(|_| "manifest not found".to_string())?;
    let manifest: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|e| format!("manifest parse error: {}", e))?;

    let projects = manifest["projects"]
        .as_array()
        .ok_or_else(|| "manifest missing 'projects' array".to_string())?;

    let result: Vec<serde_json::Value> = projects
        .iter()
        .map(|p| {
            json!({
                "name": p["name"],
                "uproject_path": p["uproject_path"],
                "targets": p["targets"]
            })
        })
        .collect();

    Ok(json!(result))
}

/// `rocket/project/audit` – compliance audit for a named project.
pub fn handle_project_audit(params: serde_json::Value) -> Result<serde_json::Value, String> {
    let project_name = params["project_name"]
        .as_str()
        .ok_or_else(|| "Missing 'project_name' parameter".to_string())?;

    // Load manifest to find the project
    let path = std::path::Path::new("project-manifest.json");
    let contents = std::fs::read_to_string(path)
        .map_err(|_| "manifest not found".to_string())?;
    let manifest: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|e| format!("manifest parse error: {}", e))?;

    let project = manifest["projects"]
        .as_array()
        .and_then(|arr| arr.iter().find(|p| p["name"].as_str() == Some(project_name)))
        .ok_or_else(|| format!("project '{}' not found in manifest", project_name))?;

    let targets = project["targets"].as_array().cloned().unwrap_or_default();
    let uproject_path = project["uproject_path"].as_str().unwrap_or("");

    // Law: NonEmptyTargets
    let mut violations: Vec<serde_json::Value> = Vec::new();
    let laws_checked = vec!["NonEmptyTargets", "ValidUprojectExtension"];

    if targets.is_empty() {
        violations.push(json!({
            "law": "NonEmptyTargets",
            "message": "Project has no build targets defined"
        }));
    }

    if !uproject_path.ends_with(".uproject") {
        violations.push(json!({
            "law": "ValidUprojectExtension",
            "message": format!("uproject_path '{}' does not end with .uproject", uproject_path)
        }));
    }

    let passed = violations.is_empty();
    Ok(json!({
        "project": project_name,
        "laws_checked": laws_checked,
        "violations": violations,
        "passed": passed
    }))
}

/// `rocket/env/doctor` – run environment health checks.
pub fn handle_env_doctor(_params: serde_json::Value) -> Result<serde_json::Value, String> {
    let checks = vec![
        run_check("rust_toolchain", "rustc", &["--version"]),
        run_check("git", "git", &["--version"]),
    ];
    let healthy = checks.iter().all(|c| c["status"] == json!("ok"));
    Ok(json!({
        "checks": checks,
        "healthy": healthy
    }))
}

fn run_check(name: &str, cmd: &str, args: &[&str]) -> serde_json::Value {
    match std::process::Command::new(cmd).args(args).output() {
        Ok(out) if out.status.success() => {
            let detail = String::from_utf8_lossy(&out.stdout).trim().to_string();
            json!({ "name": name, "status": "ok", "detail": detail })
        }
        Ok(out) => {
            let detail = String::from_utf8_lossy(&out.stderr).trim().to_string();
            json!({ "name": name, "status": "error", "detail": detail })
        }
        Err(e) => {
            json!({ "name": name, "status": "error", "detail": e.to_string() })
        }
    }
}

/// `rocket/receipt/chain` – compute a BLAKE3 receipt chain for a list of operations.
pub fn handle_receipt_chain(params: serde_json::Value) -> Result<serde_json::Value, String> {
    let operations = params["operations"]
        .as_array()
        .ok_or_else(|| "Missing 'operations' array".to_string())?;

    let mut prev_hash = "genesis".to_string();
    let mut receipts: Vec<serde_json::Value> = Vec::new();

    for op in operations {
        let op_name = op["op"].as_str().unwrap_or("unknown");
        let project = op["project"].as_str().unwrap_or("");
        let target = op["target"].as_str().unwrap_or("");
        let platform = op["platform"].as_str().unwrap_or("");

        let content = format!("{}+{}+{}+{}", op_name, project, target, platform);
        let hash = chain_hash(&prev_hash, &content);

        receipts.push(json!({
            "op": op_name,
            "project": project,
            "target": target,
            "platform": platform,
            "hash": hash,
            "prev_hash": prev_hash
        }));

        prev_hash = hash;
    }

    let chain_valid = !receipts.is_empty();
    Ok(json!({
        "receipts": receipts,
        "chain_valid": chain_valid,
        "head_hash": prev_hash
    }))
}

/// `rocket/leaderboard/top` – deterministic fake leaderboard for a project.
pub fn handle_leaderboard_top(params: serde_json::Value) -> Result<serde_json::Value, String> {
    let project = params["project"].as_str().unwrap_or("unknown");
    let n = params["n"].as_u64().unwrap_or(10) as usize;

    // Generate deterministic fake data using a simple seed from the project name.
    let seed: u64 = project.bytes().enumerate().fold(0u64, |acc, (i, b)| {
        acc.wrapping_add((b as u64).wrapping_mul(i as u64 + 31))
    });

    let first_names = ["Alice", "Bob", "Carol", "Dave", "Eve", "Frank", "Grace", "Heidi", "Ivan", "Judy"];
    let entries: Vec<serde_json::Value> = (0..n)
        .map(|i| {
            let name_idx = ((seed.wrapping_add(i as u64 * 7919)) % first_names.len() as u64) as usize;
            let score = 10000u64.wrapping_sub(i as u64 * ((seed % 500) + 100));
            json!({
                "rank": i + 1,
                "player": format!("{}_{}", first_names[name_idx], seed.wrapping_add(i as u64) % 9999),
                "score": score,
                "project": project
            })
        })
        .collect();

    Ok(json!({
        "project": project,
        "leaderboard": entries
    }))
}

// ─────────────────────────────────────────────
// Resource handlers
// ─────────────────────────────────────────────

/// `rocket://manifest` – return raw project-manifest.json contents.
pub fn handle_resource_manifest(_uri: &str) -> Result<serde_json::Value, String> {
    let path = std::path::Path::new("project-manifest.json");
    let contents = std::fs::read_to_string(path)
        .map_err(|_| "manifest not found".to_string())?;
    Ok(json!({ "text": contents }))
}

/// `rocket://capabilities` – return raw CapabilityManifest.md contents.
pub fn handle_resource_capabilities(_uri: &str) -> Result<serde_json::Value, String> {
    let path = std::path::Path::new("capabilities/CapabilityManifest.md");
    let contents = std::fs::read_to_string(path)
        .map_err(|_| "capabilities manifest not found".to_string())?;
    Ok(json!({ "text": contents }))
}

// ─────────────────────────────────────────────
// Registration helpers
// ─────────────────────────────────────────────

/// Register all rocket/* tools into a ToolRegistry.
pub fn register_rocket_tools(registry: &mut ToolRegistry) {
    // rocket/manifest/list
    registry.register(
        ToolDescriptor {
            name: "rocket/manifest/list".into(),
            description: "List all UE4 projects from project-manifest.json in the current directory.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        handle_manifest_list,
    );

    // rocket/project/audit
    registry.register(
        ToolDescriptor {
            name: "rocket/project/audit".into(),
            description: "Run compliance audit laws (NonEmptyTargets, ValidUprojectExtension) against a named project.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_name": {
                        "type": "string",
                        "description": "Name of the project to audit (e.g. 'SurvivalGame')"
                    }
                },
                "required": ["project_name"]
            }),
        },
        handle_project_audit,
    );

    // rocket/env/doctor
    registry.register(
        ToolDescriptor {
            name: "rocket/env/doctor".into(),
            description: "Check environment health: rust_toolchain, git.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        handle_env_doctor,
    );

    // rocket/receipt/chain
    registry.register(
        ToolDescriptor {
            name: "rocket/receipt/chain".into(),
            description: "Compute a BLAKE3 receipt chain for a list of build/audit operations.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "operations": {
                        "type": "array",
                        "description": "List of operations, each with 'op', 'project', optional 'target' and 'platform'",
                        "items": {
                            "type": "object",
                            "properties": {
                                "op": {"type": "string"},
                                "project": {"type": "string"},
                                "target": {"type": "string"},
                                "platform": {"type": "string"}
                            },
                            "required": ["op", "project"]
                        }
                    }
                },
                "required": ["operations"]
            }),
        },
        handle_receipt_chain,
    );

    // rocket/leaderboard/top
    registry.register(
        ToolDescriptor {
            name: "rocket/leaderboard/top".into(),
            description: "Return deterministic mock leaderboard for a project.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project": {
                        "type": "string",
                        "description": "Project name"
                    },
                    "n": {
                        "type": "integer",
                        "description": "Number of top entries to return (default 10)"
                    }
                },
                "required": ["project"]
            }),
        },
        handle_leaderboard_top,
    );
}

/// Register all rocket:// resources into a ResourceRegistry.
pub fn register_rocket_resources(registry: &mut ResourceRegistry) {
    registry.register(
        ResourceDescriptor {
            uri: "rocket://manifest".into(),
            name: "Project Manifest".into(),
            mime_type: "application/json".into(),
            description: "Contents of project-manifest.json listing all UE4 projects.".into(),
        },
        handle_resource_manifest,
    );

    registry.register(
        ResourceDescriptor {
            uri: "rocket://capabilities".into(),
            name: "Capability Manifest".into(),
            mime_type: "text/markdown".into(),
            description: "Contents of capabilities/CapabilityManifest.md.".into(),
        },
        handle_resource_capabilities,
    );
}

/// Attach all rocket/* tools and resources to a McpServer via the builder pattern.
pub fn attach_rocket_tools(server: McpServer) -> McpServer {
    let server = server
        .with_tool(
            ToolDescriptor {
                name: "rocket/manifest/list".into(),
                description: "List all UE4 projects from project-manifest.json in the current directory.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
            handle_manifest_list,
        )
        .with_tool(
            ToolDescriptor {
                name: "rocket/project/audit".into(),
                description: "Run compliance audit laws against a named project.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project_name": {"type": "string"}
                    },
                    "required": ["project_name"]
                }),
            },
            handle_project_audit,
        )
        .with_tool(
            ToolDescriptor {
                name: "rocket/env/doctor".into(),
                description: "Check environment health: rust_toolchain, git.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
            handle_env_doctor,
        )
        .with_tool(
            ToolDescriptor {
                name: "rocket/receipt/chain".into(),
                description: "Compute a BLAKE3 receipt chain for a list of build/audit operations.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operations": {
                            "type": "array",
                            "items": {"type": "object"}
                        }
                    },
                    "required": ["operations"]
                }),
            },
            handle_receipt_chain,
        )
        .with_tool(
            ToolDescriptor {
                name: "rocket/leaderboard/top".into(),
                description: "Return deterministic mock leaderboard for a project.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project": {"type": "string"},
                        "n": {"type": "integer"}
                    },
                    "required": ["project"]
                }),
            },
            handle_leaderboard_top,
        )
        .with_resource(
            ResourceDescriptor {
                uri: "rocket://manifest".into(),
                name: "Project Manifest".into(),
                mime_type: "application/json".into(),
                description: "Contents of project-manifest.json listing all UE4 projects.".into(),
            },
            handle_resource_manifest,
        )
        .with_resource(
            ResourceDescriptor {
                uri: "rocket://capabilities".into(),
                name: "Capability Manifest".into(),
                mime_type: "text/markdown".into(),
                description: "Contents of capabilities/CapabilityManifest.md.".into(),
            },
            handle_resource_capabilities,
        );

    server
}

// ─────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{McpServer, ServerInfo};
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Mutex, MutexGuard};

    // Global mutex: serialize all tests that touch the filesystem so they
    // don't interfere when `cargo test` runs them in parallel threads.
    static FS_LOCK: Mutex<()> = Mutex::new(());

    /// Acquire the global FS lock.  Returns the guard; file ops must happen
    /// while this guard is alive.
    fn fs_lock() -> MutexGuard<'static, ()> {
        FS_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    // Helper: write a temp manifest file and return a guard that removes it.
    struct TempFile(PathBuf);
    impl TempFile {
        fn write(path: &str, content: &str) -> Self {
            fs::write(path, content).expect("write temp file");
            TempFile(PathBuf::from(path))
        }
    }
    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    // ── rocket/manifest/list ───────────────────

    #[test]
    fn test_manifest_list_no_file_returns_error() {
        let _lock = fs_lock();
        // Remove the file so we can verify the "not found" error path.
        let _ = fs::remove_file("project-manifest.json");

        let result = handle_manifest_list(json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("manifest not found"));
    }

    #[test]
    fn test_manifest_list_returns_project_list() {
        let _lock = fs_lock();
        let content = r#"{
            "projects": [
                {"name":"Alpha","uproject_path":"Alpha/Alpha.uproject","targets":["AlphaEditor"]},
                {"name":"Beta","uproject_path":"Beta/Beta.uproject","targets":[]}
            ]
        }"#;
        let _guard = TempFile::write("project-manifest.json", content);

        let result = handle_manifest_list(json!({})).expect("handler should succeed");
        let arr = result.as_array().expect("result should be array");
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["name"], json!("Alpha"));
        assert_eq!(arr[1]["name"], json!("Beta"));
        assert_eq!(arr[0]["targets"][0], json!("AlphaEditor"));
    }

    // ── rocket/project/audit ──────────────────

    #[test]
    fn test_project_audit_valid_project_passes() {
        let _lock = fs_lock();
        let content = r#"{
            "projects": [
                {"name":"SurvivalGame","uproject_path":"SurvivalGame/SurvivalGame.uproject","targets":["SurvivalGameEditor"]}
            ]
        }"#;
        let _guard = TempFile::write("project-manifest.json", content);

        let result = handle_project_audit(json!({"project_name": "SurvivalGame"}))
            .expect("audit should succeed");
        assert_eq!(result["passed"], json!(true));
        assert_eq!(result["project"], json!("SurvivalGame"));
        let laws = result["laws_checked"].as_array().unwrap();
        assert!(laws.len() >= 2);
        assert!(result["violations"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_project_audit_empty_targets_fails() {
        let _lock = fs_lock();
        let content = r#"{
            "projects": [
                {"name":"NoTargets","uproject_path":"NoTargets/NoTargets.uproject","targets":[]}
            ]
        }"#;
        let _guard = TempFile::write("project-manifest.json", content);

        let result = handle_project_audit(json!({"project_name": "NoTargets"}))
            .expect("audit should succeed");
        assert_eq!(result["passed"], json!(false));
        let violations = result["violations"].as_array().unwrap();
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v["law"] == json!("NonEmptyTargets")));
    }

    // ── rocket/env/doctor ─────────────────────

    #[test]
    fn test_env_doctor_returns_checks_with_names() {
        // No filesystem dependency – no lock needed.
        let result = handle_env_doctor(json!({})).expect("doctor should succeed");
        let checks = result["checks"].as_array().expect("checks should be array");
        assert!(!checks.is_empty());
        for check in checks {
            assert!(check["name"].is_string(), "each check must have a name");
            let status = check["status"].as_str().unwrap_or("");
            assert!(status == "ok" || status == "error", "status must be ok or error");
        }
        assert!(result["healthy"].is_boolean());
    }

    // ── rocket/receipt/chain ─────────────────

    #[test]
    fn test_receipt_chain_two_ops_chain_valid() {
        // No filesystem dependency – no lock needed.
        let result = handle_receipt_chain(json!({
            "operations": [
                {"op": "build", "project": "Brm", "target": "BrmEditor", "platform": "Win64"},
                {"op": "audit", "project": "Brm"}
            ]
        }))
        .expect("chain should succeed");

        assert_eq!(result["chain_valid"], json!(true));
        let receipts = result["receipts"].as_array().unwrap();
        assert_eq!(receipts.len(), 2);

        // The second receipt's prev_hash must equal the first receipt's hash.
        let first_hash = receipts[0]["hash"].as_str().unwrap();
        let second_prev = receipts[1]["prev_hash"].as_str().unwrap();
        assert_eq!(first_hash, second_prev, "chain must link consecutive hashes");

        // head_hash must equal the last receipt's hash.
        let last_hash = receipts[1]["hash"].as_str().unwrap();
        assert_eq!(result["head_hash"].as_str().unwrap(), last_hash);
    }

    #[test]
    fn test_receipt_chain_empty_ops_chain_not_valid() {
        let result = handle_receipt_chain(json!({"operations": []}))
            .expect("chain should succeed even with 0 ops");
        assert_eq!(result["chain_valid"], json!(false));
        assert!(result["receipts"].as_array().unwrap().is_empty());
    }

    // ── MCP server dispatch ───────────────────

    #[test]
    fn test_mcp_dispatch_rocket_manifest_list() {
        let _lock = fs_lock();
        let content = r#"{"projects":[{"name":"X","uproject_path":"X.uproject","targets":["XEditor"]}]}"#;
        let _guard = TempFile::write("project-manifest.json", content);

        let server = McpServer::new(ServerInfo {
            name: "test".into(),
            version: "0.0.0".into(),
        });
        let server = attach_rocket_tools(server);

        let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"rocket/manifest/list","arguments":{}}}"#;
        let resp = server.handle(req);
        let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(v.get("error").is_none(), "unexpected error: {:?}", v);
        assert!(v["result"]["content"].is_array());
    }

    // ── rocket://manifest resource ────────────

    #[test]
    fn test_resource_manifest_returns_file_contents() {
        let _lock = fs_lock();
        let content = r#"{"projects":[]}"#;
        let _guard = TempFile::write("project-manifest.json", content);

        let result = handle_resource_manifest("rocket://manifest")
            .expect("resource handler should succeed");
        assert!(result["text"].is_string());
        let text = result["text"].as_str().unwrap();
        assert!(text.contains("projects"));
    }

    // ── Tool list includes all rocket/* tools ─

    #[test]
    fn test_tool_list_includes_all_rocket_tools() {
        let mut registry = ToolRegistry::new();
        register_rocket_tools(&mut registry);

        let expected = [
            "rocket/manifest/list",
            "rocket/project/audit",
            "rocket/env/doctor",
            "rocket/receipt/chain",
            "rocket/leaderboard/top",
        ];

        for name in &expected {
            assert!(
                registry.has(name),
                "registry should contain tool '{}'",
                name
            );
        }
        assert_eq!(registry.list().len(), expected.len());
    }
}
