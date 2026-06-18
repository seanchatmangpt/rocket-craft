use crate::tool::ToolDescriptor;
use anti_llm_cheat_lsp::engine;
use serde_json::json;

/// Attach anti-llm-cheat-lsp scanning tools to the MCP server.
pub fn attach_anti_llm_tools(server: crate::server::McpServer) -> crate::server::McpServer {
    server
        .with_tool(scan_directory_descriptor(), handle_scan_directory)
        .with_tool(
            evaluate_diagnostics_descriptor(),
            handle_evaluate_diagnostics,
        )
}

/// MCP tool: `audit/scan_directory` - scan a directory for LLM cheat patterns
pub fn handle_scan_directory(params: serde_json::Value) -> Result<serde_json::Value, String> {
    let dir_path = params["dir_path"]
        .as_str()
        .ok_or_else(|| "Missing 'dir_path' parameter".to_string())?;

    let observations = engine::scan_directory(dir_path);
    let blocking_count = observations
        .iter()
        .filter(|o| o.kind.contains("stub") || o.kind.contains("oracle"))
        .count();

    Ok(json!({
        "directory": dir_path,
        "observation_count": observations.len(),
        "blocking_count": blocking_count,
        "observations": observations
            .iter()
            .map(|o| json!({
                "file_path": o.file_path,
                "line": o.line,
                "column": o.column,
                "kind": o.kind,
                "construct": o.construct,
                "message": o.message,
                "context": o.context
            }))
            .collect::<Vec<_>>()
    }))
}

/// MCP tool: `audit/evaluate_diagnostics` - convert observations to diagnostics
pub fn handle_evaluate_diagnostics(params: serde_json::Value) -> Result<serde_json::Value, String> {
    use anti_llm_cheat_lsp::config::AntiLlmConfig;
    use anti_llm_cheat_lsp::observations::Observation;

    let observations_json = params["observations"]
        .as_array()
        .ok_or_else(|| "Missing 'observations' array".to_string())?;

    let observations: Vec<Observation> = observations_json
        .iter()
        .filter_map(|o| serde_json::from_value(o.clone()).ok())
        .collect();

    let config = AntiLlmConfig::default();
    let diagnostics = engine::evaluate_diagnostics_with_config(&observations, &config);

    let blocking_count = diagnostics.iter().filter(|d| d.blocking).count();
    let warning_count = diagnostics.len() - blocking_count;

    Ok(json!({
        "diagnostic_count": diagnostics.len(),
        "blocking_count": blocking_count,
        "warning_count": warning_count,
        "diagnostics": diagnostics
            .iter()
            .map(|d| json!({
                "code": d.code,
                "category": d.category,
                "file_path": d.file_path,
                "line": d.line,
                "column": d.column,
                "message": d.message,
                "blocking": d.blocking,
                "required_correction": d.required_correction,
                "required_next_proof": d.required_next_proof
            }))
            .collect::<Vec<_>>()
    }))
}

fn scan_directory_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        name: "audit/scan_directory".to_string(),
        description:
            "Scan a directory for LLM-generated stubs, debug artifacts, and other anti-patterns."
                .to_string(),
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
        description:
            "Convert raw observations into blocking vs. warning diagnostics with remediation guidance."
                .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "observations": {
                    "type": "array",
                    "description": "Array of observation objects (from audit/scan_directory)",
                    "items": {
                        "type": "object"
                    }
                }
            },
            "required": ["observations"]
        }),
    }
}
