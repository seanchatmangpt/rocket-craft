use crate::app::{Cli, Commands};
use crate::output::Output;
use crate::version::crate_versions;
use unify_receipts::receipt::Receipt;
use serde_json::json;

/// Entry point: dispatch the parsed CLI to the matching handler.
pub fn run(cli: Cli) -> Result<Output, Box<dyn std::error::Error>> {
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
pub fn cmd_receipt(label: &str, data: &str) -> Result<Output, Box<dyn std::error::Error>> {
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
pub fn cmd_verify(chain_json: &str) -> Result<Output, Box<dyn std::error::Error>> {
    let value: serde_json::Value = serde_json::from_str(chain_json)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let receipts: Vec<Receipt> = if value.is_array() {
        serde_json::from_value(value).map_err(|e| format!("Invalid receipt array: {}", e))?
    } else {
        let r: Receipt =
            serde_json::from_str(chain_json).map_err(|e| format!("Invalid receipt: {}", e))?;
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
pub fn cmd_gate(law: &str, data: &str) -> Result<Output, Box<dyn std::error::Error>> {
    // Parse data as JSON to validate it is well-formed.
    let parsed: serde_json::Value = serde_json::from_str(data)
        .map_err(|e| format!("Invalid data JSON: {}", e))?;

    // Stub: interpret law as a field-exists check of the form "field:<name>"
    let admitted = if let Some(field) = law.strip_prefix("field:") {
        parsed.get(field).is_some()
    } else {
        // Unknown law — admitted by default (open gate)
        true
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
pub fn cmd_info() -> Result<Output, Box<dyn std::error::Error>> {
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
) -> Result<Output, Box<dyn std::error::Error>> {
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
pub fn cmd_query(ttl: Option<&str>, pattern: &str) -> Result<Output, Box<dyn std::error::Error>> {
    // Stub implementation: parse the pattern and echo back with turtle source info.
    let source = ttl.map(|_| "inline-turtle").unwrap_or("none");
    Ok(Output::ok(json!({
        "pattern": pattern,
        "source": source,
        "results": [],
        "note": "SPARQL query stub — wire unify-rdf for full evaluation",
    })))
}

/// Show the witness registry, optionally filtered by domain.
pub fn cmd_witnesses(domain: Option<&str>) -> Result<Output, Box<dyn std::error::Error>> {
    // Stub registry — real implementation would query unify-rdf / unify-sem.
    let all = vec![
        json!({ "domain": "rdf",      "witness": "triple-store", "status": "active" }),
        json!({ "domain": "receipts", "witness": "blake3-chain", "status": "active" }),
        json!({ "domain": "sem",      "witness": "semantic-net", "status": "active" }),
        json!({ "domain": "lsp",      "witness": "lsp-bridge",   "status": "stub"   }),
        json!({ "domain": "wasm",     "witness": "wasm4pm",      "status": "stub"   }),
    ];

    let witnesses: Vec<&serde_json::Value> = match domain {
        Some(d) => all.iter().filter(|w| w["domain"] == d).collect(),
        None => all.iter().collect(),
    };

    Ok(Output::ok(json!({ "witnesses": witnesses })))
}
