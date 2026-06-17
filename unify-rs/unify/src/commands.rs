use crate::app::{Cli, Commands};
use crate::output::Output;
use crate::version::crate_versions;
use unify_receipts::receipt::Receipt;
use unify_rdf::pipeline::{OntologyPipeline, PipelineConfig};
use unify_rdf::sparql::{PatternExecutor, SparqlExecutor};
use unify_rdf::store::TripleStore;
use unify_rdf::triple::Term;
use serde_json::json;

/// Typed error variants for all CLI command handlers.
#[derive(thiserror::Error, Debug)]
pub enum CommandError {
    #[error("JSON parse failed: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("turtle parse error: {0}")]
    TurtleParse(String),
}

/// Entry point: dispatch the parsed CLI to the matching handler.
pub fn run(cli: Cli) -> Result<Output, CommandError> {
    match cli.command {
        Commands::Receipt { label, data } => cmd_receipt(&label, &data),
        Commands::Verify { chain_json } => cmd_verify(&chain_json),
        Commands::Gate { law, data } => cmd_gate(&law, &data),
        Commands::Info => cmd_info(),
        Commands::Dispatch { namespace, noun, verb, input } => {
            cmd_dispatch(namespace.as_deref(), &noun, &verb, input.as_deref())
        }
        Commands::Query { ttl, pattern } => cmd_query(ttl.as_deref(), &pattern),
        Commands::Witnesses { domain } => cmd_witnesses(domain.as_deref()),
    }
}

/// Compute a BLAKE3 receipt for `data` under `label`.
pub fn cmd_receipt(label: &str, data: &str) -> Result<Output, CommandError> {
    let receipt = Receipt::new(label, data.as_bytes());
    let value = json!({
        "key": receipt.key,
        "hash": receipt.hash,
        "issued_at": receipt.issued_at,
    });
    Ok(Output::ok(value))
}

/// Verify a receipt (or array of receipts) against the stored hash.
///
/// Accepts either a single Receipt JSON object or an array of Receipt objects.
/// Each receipt's `key` is treated as the data to verify against the stored hash.
pub fn cmd_verify(chain_json: &str) -> Result<Output, CommandError> {
    let value: serde_json::Value = serde_json::from_str(chain_json)?;

    let receipts: Vec<Receipt> = if value.is_array() {
        serde_json::from_value(value)?
    } else {
        let r: Receipt = serde_json::from_str(chain_json)?;
        vec![r]
    };

    let results: Vec<serde_json::Value> = receipts
        .iter()
        .map(|r| {
            // Verify that the stored hash matches hashing the key itself as data.
            let valid = r.verify(r.key.as_bytes());
            json!({ "key": r.key, "valid": valid })
        })
        .collect();

    let all_valid = results
        .iter()
        .all(|r| r["valid"].as_bool().unwrap_or(false));

    Ok(Output {
        data: json!({ "results": results }),
        success: all_valid,
        message: if all_valid {
            Some("All receipts valid".into())
        } else {
            Some("One or more receipts failed verification".into())
        },
    })
}

/// Check an admission gate law against JSON data.
pub fn cmd_gate(law: &str, data: &str) -> Result<Output, CommandError> {
    let parsed: serde_json::Value = serde_json::from_str(data)?;

    let admitted = if let Some(field) = law.strip_prefix("field:") {
        parsed.get(field).is_some()
    } else if let Some(value) = law.strip_prefix("eq:") {
        // eq:field=expected_value
        if let Some((field, expected)) = value.split_once('=') {
            parsed.get(field).and_then(|v| v.as_str()) == Some(expected)
        } else {
            false
        }
    } else if law == "nonempty" {
        !parsed.as_object().map(|o| o.is_empty()).unwrap_or(true)
    } else if law == "open" {
        true
    } else {
        // Unknown law — closed gate (deny by default for unknown laws)
        false
    };

    Ok(Output {
        data: json!({ "law": law, "admitted": admitted }),
        success: admitted,
        message: Some(if admitted {
            format!("Gate '{}' admitted", law)
        } else {
            format!("Gate '{}' denied", law)
        }),
    })
}

/// Show version info for all unify-rs crates.
pub fn cmd_info() -> Result<Output, CommandError> {
    let versions: Vec<serde_json::Value> = crate_versions()
        .into_iter()
        .map(|v| json!({ "name": v.name, "version": v.version }))
        .collect();
    Ok(Output::ok(json!({ "crates": versions })))
}

/// Dispatch a noun-verb command.
pub fn cmd_dispatch(
    ns: Option<&str>,
    noun: &str,
    verb: &str,
    input: Option<&str>,
) -> Result<Output, CommandError> {
    let parsed_input: serde_json::Value = match input {
        Some(s) => serde_json::from_str(s).unwrap_or(json!(s)),
        None => json!(null),
    };

    // Known dispatch table — extend as real handlers are wired up.
    let result = match (ns, noun, verb) {
        (_, "receipt", "create") => {
            let data = parsed_input["data"].as_str().unwrap_or("{}");
            let label = parsed_input["label"].as_str().unwrap_or("default");
            let r = Receipt::new(label, data.as_bytes());
            json!({ "key": r.key, "hash": r.hash, "issued_at": r.issued_at })
        }
        (_, "version", "list") => {
            let versions: Vec<serde_json::Value> = crate_versions()
                .into_iter()
                .map(|v| json!({ "name": v.name, "version": v.version }))
                .collect();
            json!({ "crates": versions })
        }
        _ => {
            return Ok(Output::error(format!(
                "Unknown dispatch: namespace={:?} noun={} verb={}",
                ns, noun, verb
            )));
        }
    };

    Ok(Output::ok(result))
}

/// Run a SPARQL-like triple pattern query over optional Turtle input.
pub fn cmd_query(ttl: Option<&str>, pattern: &str) -> Result<Output, CommandError> {
    let source = ttl.map(|_| "inline-turtle").unwrap_or("none");

    // Build a fresh triple store, optionally loading Turtle input.
    let mut store = TripleStore::new();
    if let Some(turtle) = ttl {
        let config = PipelineConfig {
            target_language: "rust".into(),
            output_dir: ".".into(),
            template_dir: None,
            namespace: "unify".into(),
        };
        let mut pipeline = OntologyPipeline::new(store, config);
        pipeline.load_turtle(turtle).map_err(|e| CommandError::TurtleParse(e.to_string()))?;
        drop(pipeline);
        store = TripleStore::new();
        parse_turtle_into_store(turtle, &mut store);
    }

    let executor = PatternExecutor(&store);
    let triple_count = store.len();

    match executor.select(pattern) {
        Ok(bindings) => {
            let results: Vec<serde_json::Value> = bindings
                .iter()
                .map(|b| {
                    let term_to_str = |t: &Term| -> String {
                        match t {
                            Term::Named(iri) => iri.clone(),
                            Term::Blank(id) => format!("_:{}", id),
                            Term::Literal { value, .. } => value.clone(),
                        }
                    };
                    json!({
                        "s": b.get("s").map(term_to_str).unwrap_or_default(),
                        "p": b.get("p").map(term_to_str).unwrap_or_default(),
                        "o": b.get("o").map(term_to_str).unwrap_or_default(),
                    })
                })
                .collect();
            Ok(Output::ok(json!({
                "pattern": pattern,
                "source": source,
                "triple_count": triple_count,
                "results": results,
            })))
        }
        Err(e) => Ok(Output {
            data: json!({
                "pattern": pattern,
                "source": source,
                "triple_count": triple_count,
                "results": [],
                "error": e.to_string(),
            }),
            success: false,
            message: Some(format!("Query failed: {}", e)),
        }),
    }
}

/// Parse Turtle-like N-Triples lines into `store`.
fn parse_turtle_into_store(turtle: &str, store: &mut TripleStore) {
    use unify_rdf::triple::Triple;
    for line in turtle.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let s = parts[0].trim_matches(|c| c == '<' || c == '>');
            let p = parts[1].trim_matches(|c| c == '<' || c == '>');
            let o = parts[2].trim_matches(|c| c == '<' || c == '>' || c == '.');
            store.add(Triple::new(s, p, o));
        }
    }
}

/// Show the witness registry, optionally filtered by domain.
pub fn cmd_witnesses(domain: Option<&str>) -> Result<Output, CommandError> {
    // -- rdf / triple-store ---------------------------------------------------
    let rdf_store_status = {
        let mut store = TripleStore::new();
        store.add(unify_rdf::triple::Triple::new(
            "unify:witness",
            "unify:probe",
            "unify:triple-store",
        ));
        if store.len() == 1 {
            json!({
                "domain": "rdf",
                "witness": "triple-store",
                "status": "active",
                "triple_count": store.len(),
            })
        } else {
            json!({
                "domain": "rdf",
                "witness": "triple-store",
                "status": "unavailable",
                "triple_count": store.len(),
            })
        }
    };

    // -- rdf / pipeline -------------------------------------------------------
    let rdf_pipeline_status = {
        let config = PipelineConfig {
            target_language: "rust".into(),
            output_dir: ".".into(),
            template_dir: None,
            namespace: "unify".into(),
        };
        let mut pipeline = OntologyPipeline::new(TripleStore::new(), config);
        let n = pipeline
            .load_turtle("<unify:witness> rdf:type <unify:Pipeline> .")
            .unwrap_or(0);
        let stage_count = pipeline.stage_count();
        if n > 0 {
            json!({
                "domain": "rdf",
                "witness": "pipeline",
                "status": "active",
                "triples_loaded": n,
                "stage_count": stage_count,
            })
        } else {
            json!({
                "domain": "rdf",
                "witness": "pipeline",
                "status": "unavailable",
                "triples_loaded": n,
                "stage_count": stage_count,
            })
        }
    };

    // -- receipts / blake3-chain ----------------------------------------------
    let receipts_status = {
        let r = Receipt::new("unify:witness-probe", b"witness");
        let valid = r.verify(b"witness");
        json!({
            "domain": "receipts",
            "witness": "blake3-chain",
            "status": if valid { "active" } else { "unavailable" },
            "probe_valid": valid,
        })
    };

    // -- sem / semantic-net ---------------------------------------------------
    let sem_status = {
        #[allow(unused_imports)]
        use unify_sem as _sem;
        json!({
            "domain": "sem",
            "witness": "semantic-net",
            "status": "active",
            "note": "crate linked; no runtime state exposed yet",
        })
    };

    // -- lsp / lsp-bridge  ----------------------------------------------------
    let lsp_status = {
        use unify_lsp::gate::AndonGate;
        let gate = AndonGate::new();
        let status = if gate.is_open() { "active" } else { "blocked" };
        json!({
            "domain": "lsp",
            "witness": "andon-gate",
            "status": status,
            "events": gate.event_count(),
            "gate": format!("{:?}", gate.state())
        })
    };

    // -- wasm / wasm4pm -------------------------------------------------------
    let wasm_status = json!({
        "domain": "wasm",
        "witness": "wasm4pm",
        "status": "stub",
    });

    let all = vec![
        rdf_store_status,
        rdf_pipeline_status,
        receipts_status,
        sem_status,
        lsp_status,
        wasm_status,
    ];

    let witnesses: Vec<&serde_json::Value> = match domain {
        Some(d) => all.iter().filter(|w| w["domain"] == d).collect(),
        None => all.iter().collect(),
    };

    Ok(Output::ok(json!({ "witnesses": witnesses })))
}