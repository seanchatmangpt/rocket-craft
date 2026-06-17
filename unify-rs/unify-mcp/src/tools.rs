use crate::server::McpServer;
use crate::tool::{ToolDescriptor, ToolRegistry};
use serde_json::json;

/// Register all built-in unify tools into a ToolRegistry.
pub fn register_builtin_tools(registry: &mut ToolRegistry) {
    // --- unify/version ---
    registry.register(
        ToolDescriptor {
            name: "unify/version".into(),
            description: "Returns the unify-mcp package version.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        |_params| {
            Ok(json!({
                "version": env!("CARGO_PKG_VERSION")
            }))
        },
    );

    // --- unify/receipt/compute ---
    registry.register(
        ToolDescriptor {
            name: "unify/receipt/compute".into(),
            description: "Compute a BLAKE3 receipt (hash) for a given data string and key.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "The capability key for the receipt"
                    },
                    "data": {
                        "type": "string",
                        "description": "The data to hash"
                    }
                },
                "required": ["key", "data"]
            }),
        },
        |params| {
            let key = params["key"]
                .as_str()
                .ok_or_else(|| "Missing 'key' parameter".to_string())?;
            let data = params["data"]
                .as_str()
                .ok_or_else(|| "Missing 'data' parameter".to_string())?;
            let hash = blake3_hex(data.as_bytes());
            let issued_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            Ok(json!({
                "key": key,
                "hash": hash,
                "issued_at": issued_at
            }))
        },
    );

    // --- unify/cli/dispatch ---
    registry.register(
        ToolDescriptor {
            name: "unify/cli/dispatch".into(),
            description: "Dispatch a noun-verb command (e.g. noun='receipt', verb='compute').".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "noun": {
                        "type": "string",
                        "description": "The resource noun (e.g. 'receipt', 'rdf', 'pm')"
                    },
                    "verb": {
                        "type": "string",
                        "description": "The action verb (e.g. 'compute', 'query', 'count')"
                    },
                    "args": {
                        "type": "object",
                        "description": "Additional arguments for the command"
                    }
                },
                "required": ["noun", "verb"]
            }),
        },
        |params| {
            let noun = params["noun"]
                .as_str()
                .ok_or_else(|| "Missing 'noun' parameter".to_string())?;
            let verb = params["verb"]
                .as_str()
                .ok_or_else(|| "Missing 'verb' parameter".to_string())?;
            let args = params.get("args").cloned().unwrap_or(json!({}));

            // Dispatch known noun-verb combinations
            match (noun, verb) {
                ("receipt", "compute") => {
                    let key = args["key"].as_str().unwrap_or("default");
                    let data = args["data"].as_str().unwrap_or("");
                    let hash = blake3_hex(data.as_bytes());
                    Ok(json!({ "key": key, "hash": hash }))
                }
                ("rdf", "query") => {
                    use unify_rdf::store::TripleStore;
                    use unify_rdf::triple::{Term, Triple};
                    let turtle = args["turtle"].as_str().unwrap_or("");
                    let subject_f = args["subject"].as_str();
                    let predicate_f = args["predicate"].as_str();
                    let object_f = args["object"].as_str();
                    let mut store = TripleStore::new();
                    for line in turtle.lines() {
                        let line = line.trim();
                        if line.is_empty() || line.starts_with('#') { continue; }
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            let s = parts[0].trim_matches(|c| c == '<' || c == '>');
                            let p = parts[1].trim_matches(|c| c == '<' || c == '>');
                            let o = parts[2].trim_matches(|c: char| c == '<' || c == '>' || c == '.');
                            store.add(Triple::new(s, p, o));
                        }
                    }
                    let s_term = subject_f.map(|s| Term::Named(s.to_string()));
                    let p_term = predicate_f.map(|p| Term::Named(p.to_string()));
                    let o_term = object_f.map(|o| Term::Named(o.to_string()));
                    let matching: Vec<serde_json::Value> = store
                        .query_pattern(s_term.as_ref(), p_term.as_ref(), o_term.as_ref())
                        .iter()
                        .map(|t| {
                            let term_str = |term: &Term| -> String {
                                match term {
                                    Term::Named(n) => n.clone(),
                                    Term::Blank(b) => format!("_:{}", b),
                                    Term::Literal { value, .. } => value.clone(),
                                }
                            };
                            json!({ "subject": term_str(&t.subject), "predicate": term_str(&t.predicate), "object": term_str(&t.object) })
                        })
                        .collect();
                    let count = matching.len();
                    Ok(json!({ "triples": matching, "count": count }))
                }
                ("pm", "count") => {
                    use unify_pm::EventLog;
                    let mut log = EventLog::new();
                    if let Some(events_arr) = args["events"].as_array() {
                        for ev in events_arr {
                            if let (Some(id), Some(act)) = (ev["id"].as_str(), ev["activity"].as_str()) {
                                log.push(id, act, "0");
                            }
                        }
                    }
                    Ok(json!({ "count": log.events.len() }))
                }
                _ => Ok(json!({
                    "noun": noun,
                    "verb": verb,
                    "status": "dispatched",
                    "args": args
                })),
            }
        },
    );

    // --- unify/rdf/query ---
    registry.register(
        ToolDescriptor {
            name: "unify/rdf/query".into(),
            description: "Run a simple triple pattern query against an in-memory RDF store.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "triples": {
                        "type": "array",
                        "description": "Array of triple objects {subject, predicate, object} to load",
                        "items": {
                            "type": "object",
                            "properties": {
                                "subject": {"type": "string"},
                                "predicate": {"type": "string"},
                                "object": {"type": "string"}
                            }
                        }
                    },
                    "subject": {
                        "type": "string",
                        "description": "Optional subject filter (null/omit for wildcard)"
                    },
                    "predicate": {
                        "type": "string",
                        "description": "Optional predicate filter (null/omit for wildcard)"
                    },
                    "object": {
                        "type": "string",
                        "description": "Optional object filter (null/omit for wildcard)"
                    }
                },
                "required": []
            }),
        },
        |params| {
            // Parse triples from params
            let empty_arr = vec![];
            let raw_triples = params["triples"].as_array().unwrap_or(&empty_arr);

            let subject_filter = params["subject"].as_str();
            let predicate_filter = params["predicate"].as_str();
            let object_filter = params["object"].as_str();

            // Simple in-memory filtering without depending on unify-rdf
            let matching: Vec<serde_json::Value> = raw_triples
                .iter()
                .filter(|t| {
                    let s_match = subject_filter
                        .map_or(true, |f| t["subject"].as_str() == Some(f));
                    let p_match = predicate_filter
                        .map_or(true, |f| t["predicate"].as_str() == Some(f));
                    let o_match = object_filter
                        .map_or(true, |f| t["object"].as_str() == Some(f));
                    s_match && p_match && o_match
                })
                .cloned()
                .collect();

            Ok(json!({
                "triples": matching,
                "count": matching.len()
            }))
        },
    );

    // --- unify/pm/event-count ---
    registry.register(
        ToolDescriptor {
            name: "unify/pm/event-count".into(),
            description: "Count events in a JSON event log array.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "events": {
                        "type": "array",
                        "description": "Array of event objects to count",
                        "items": {"type": "object"}
                    },
                    "filter_type": {
                        "type": "string",
                        "description": "Optional event type to filter by"
                    }
                },
                "required": ["events"]
            }),
        },
        |params| {
            let empty_arr = vec![];
            let events = params["events"].as_array().unwrap_or(&empty_arr);
            let filter_type = params["filter_type"].as_str();

            let count = if let Some(ft) = filter_type {
                events
                    .iter()
                    .filter(|e| e["type"].as_str() == Some(ft))
                    .count()
            } else {
                events.len()
            };

            Ok(json!({
                "count": count,
                "total": events.len()
            }))
        },
    );
}

/// Helper to register all built-in tools on the server (builder pattern).
pub fn register_server_tools(server: McpServer) -> McpServer {
    let mut registry = crate::tool::ToolRegistry::new();
    register_builtin_tools(&mut registry);

    let mut s = server;
    // Transfer all tools from local registry to server
    // Since we can't drain a ToolRegistry directly, we rebuild via with_tool.
    // We use a helper that gives us back the server with tools attached.
    s = attach_builtin_tools(s);
    s
}

/// Attach all built-in tools to the server using the builder pattern.
fn attach_builtin_tools(server: McpServer) -> McpServer {
    use crate::tool::ToolDescriptor;

    let server = server.with_tool(
        ToolDescriptor {
            name: "unify/version".into(),
            description: "Returns the unify-mcp package version.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        |_params| {
            Ok(json!({
                "version": env!("CARGO_PKG_VERSION")
            }))
        },
    );

    let server = server.with_tool(
        ToolDescriptor {
            name: "unify/receipt/compute".into(),
            description: "Compute a BLAKE3 receipt (hash) for a given data string and key.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "key": {"type": "string", "description": "The capability key for the receipt"},
                    "data": {"type": "string", "description": "The data to hash"}
                },
                "required": ["key", "data"]
            }),
        },
        |params| {
            let key = params["key"]
                .as_str()
                .ok_or_else(|| "Missing 'key' parameter".to_string())?;
            let data = params["data"]
                .as_str()
                .ok_or_else(|| "Missing 'data' parameter".to_string())?;
            let hash = blake3_hex(data.as_bytes());
            let issued_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            Ok(json!({ "key": key, "hash": hash, "issued_at": issued_at }))
        },
    );

    let server = server.with_tool(
        ToolDescriptor {
            name: "unify/cli/dispatch".into(),
            description: "Dispatch a noun-verb command.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "noun": {"type": "string"},
                    "verb": {"type": "string"},
                    "args": {"type": "object"}
                },
                "required": ["noun", "verb"]
            }),
        },
        |params| {
            let noun = params["noun"]
                .as_str()
                .ok_or_else(|| "Missing 'noun' parameter".to_string())?;
            let verb = params["verb"]
                .as_str()
                .ok_or_else(|| "Missing 'verb' parameter".to_string())?;
            let args = params.get("args").cloned().unwrap_or(json!({}));
            match (noun, verb) {
                ("receipt", "compute") => {
                    let key = args["key"].as_str().unwrap_or("default");
                    let data = args["data"].as_str().unwrap_or("");
                    let hash = blake3_hex(data.as_bytes());
                    Ok(json!({ "key": key, "hash": hash }))
                }
                ("rdf", "query") => {
                    use unify_rdf::store::TripleStore;
                    use unify_rdf::triple::{Term, Triple};
                    let turtle = args["turtle"].as_str().unwrap_or("");
                    let subject_f = args["subject"].as_str();
                    let predicate_f = args["predicate"].as_str();
                    let object_f = args["object"].as_str();
                    let mut store = TripleStore::new();
                    for line in turtle.lines() {
                        let line = line.trim();
                        if line.is_empty() || line.starts_with('#') { continue; }
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            let s = parts[0].trim_matches(|c| c == '<' || c == '>');
                            let p = parts[1].trim_matches(|c| c == '<' || c == '>');
                            let o = parts[2].trim_matches(|c: char| c == '<' || c == '>' || c == '.');
                            store.add(Triple::new(s, p, o));
                        }
                    }
                    let s_term = subject_f.map(|s| Term::Named(s.to_string()));
                    let p_term = predicate_f.map(|p| Term::Named(p.to_string()));
                    let o_term = object_f.map(|o| Term::Named(o.to_string()));
                    let matching: Vec<serde_json::Value> = store
                        .query_pattern(s_term.as_ref(), p_term.as_ref(), o_term.as_ref())
                        .iter()
                        .map(|t| {
                            let term_str = |term: &Term| -> String {
                                match term {
                                    Term::Named(n) => n.clone(),
                                    Term::Blank(b) => format!("_:{}", b),
                                    Term::Literal { value, .. } => value.clone(),
                                }
                            };
                            json!({ "subject": term_str(&t.subject), "predicate": term_str(&t.predicate), "object": term_str(&t.object) })
                        })
                        .collect();
                    let count = matching.len();
                    Ok(json!({ "triples": matching, "count": count }))
                }
                ("pm", "count") => {
                    use unify_pm::EventLog;
                    let mut log = EventLog::new();
                    if let Some(events_arr) = args["events"].as_array() {
                        for ev in events_arr {
                            if let (Some(id), Some(act)) = (ev["id"].as_str(), ev["activity"].as_str()) {
                                log.push(id, act, "0");
                            }
                        }
                    }
                    Ok(json!({ "count": log.events.len() }))
                }
                _ => Ok(json!({
                    "noun": noun,
                    "verb": verb,
                    "status": "dispatched",
                    "args": args
                })),
            }
        },
    );

    let server = server.with_tool(
        ToolDescriptor {
            name: "unify/rdf/query".into(),
            description: "Run a simple triple pattern query against an in-memory RDF store.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "triples": {"type": "array", "items": {"type": "object"}},
                    "subject": {"type": "string"},
                    "predicate": {"type": "string"},
                    "object": {"type": "string"}
                }
            }),
        },
        |params| {
            let empty_arr = vec![];
            let raw_triples = params["triples"].as_array().unwrap_or(&empty_arr);
            let subject_filter = params["subject"].as_str();
            let predicate_filter = params["predicate"].as_str();
            let object_filter = params["object"].as_str();
            let matching: Vec<serde_json::Value> = raw_triples
                .iter()
                .filter(|t| {
                    subject_filter.map_or(true, |f| t["subject"].as_str() == Some(f))
                        && predicate_filter.map_or(true, |f| t["predicate"].as_str() == Some(f))
                        && object_filter.map_or(true, |f| t["object"].as_str() == Some(f))
                })
                .cloned()
                .collect();
            let count = matching.len();
            Ok(json!({ "triples": matching, "count": count }))
        },
    );

    server.with_tool(
        ToolDescriptor {
            name: "unify/pm/event-count".into(),
            description: "Count events in a JSON event log array.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "events": {"type": "array", "items": {"type": "object"}},
                    "filter_type": {"type": "string"}
                },
                "required": ["events"]
            }),
        },
        |params| {
            let empty_arr = vec![];
            let events = params["events"].as_array().unwrap_or(&empty_arr);
            let filter_type = params["filter_type"].as_str();
            let count = if let Some(ft) = filter_type {
                events.iter().filter(|e| e["type"].as_str() == Some(ft)).count()
            } else {
                events.len()
            };
            Ok(json!({ "count": count, "total": events.len() }))
        },
    )
}

/// Compute a real BLAKE3 hex hash of bytes.
fn blake3_hex(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::ToolRegistry;

    #[test]
    fn test_builtin_tools_registered() {
        let mut registry = ToolRegistry::new();
        register_builtin_tools(&mut registry);
        assert!(registry.has("unify/version"));
        assert!(registry.has("unify/receipt/compute"));
        assert!(registry.has("unify/cli/dispatch"));
        assert!(registry.has("unify/rdf/query"));
        assert!(registry.has("unify/pm/event-count"));
        assert_eq!(registry.list().len(), 5);
    }

    #[test]
    fn test_version_tool_returns_version() {
        let mut registry = ToolRegistry::new();
        register_builtin_tools(&mut registry);
        let result = registry.call("unify/version", serde_json::json!({})).unwrap();
        assert!(result["version"].is_string());
    }

    #[test]
    fn test_receipt_compute_returns_hash() {
        let mut registry = ToolRegistry::new();
        register_builtin_tools(&mut registry);
        let result = registry
            .call(
                "unify/receipt/compute",
                serde_json::json!({"key": "test-key", "data": "hello world"}),
            )
            .unwrap();
        assert_eq!(result["key"], serde_json::json!("test-key"));
        assert!(result["hash"].is_string());
        assert!(result["issued_at"].is_number());
    }

    #[test]
    fn test_rdf_query_filters_triples() {
        let mut registry = ToolRegistry::new();
        register_builtin_tools(&mut registry);
        let result = registry
            .call(
                "unify/rdf/query",
                serde_json::json!({
                    "triples": [
                        {"subject": "http://s1", "predicate": "http://p", "object": "http://o1"},
                        {"subject": "http://s2", "predicate": "http://p", "object": "http://o2"}
                    ],
                    "subject": "http://s1"
                }),
            )
            .unwrap();
        assert_eq!(result["count"], serde_json::json!(1));
    }

    #[test]
    fn test_pm_event_count_counts_all() {
        let mut registry = ToolRegistry::new();
        register_builtin_tools(&mut registry);
        let result = registry
            .call(
                "unify/pm/event-count",
                serde_json::json!({
                    "events": [
                        {"type": "A", "id": 1},
                        {"type": "B", "id": 2},
                        {"type": "A", "id": 3}
                    ]
                }),
            )
            .unwrap();
        assert_eq!(result["count"], serde_json::json!(3));
        assert_eq!(result["total"], serde_json::json!(3));
    }

    #[test]
    fn test_pm_event_count_filters_by_type() {
        let mut registry = ToolRegistry::new();
        register_builtin_tools(&mut registry);
        let result = registry
            .call(
                "unify/pm/event-count",
                serde_json::json!({
                    "events": [
                        {"type": "A", "id": 1},
                        {"type": "B", "id": 2},
                        {"type": "A", "id": 3}
                    ],
                    "filter_type": "A"
                }),
            )
            .unwrap();
        assert_eq!(result["count"], serde_json::json!(2));
        assert_eq!(result["total"], serde_json::json!(3));
    }
}
