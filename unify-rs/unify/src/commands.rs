use crate::app::{AutomlSubcommands, Cli, Commands, DevSubcommands, GenieSubcommands};
use crate::output::Output;
use crate::version::crate_versions;
use genie_core::spec::WorldSpec;
use serde_json::json;
use unify_rdf::pipeline::{OntologyPipeline, PipelineConfig};
use unify_rdf::sparql::{PatternExecutor, SparqlExecutor};
use unify_rdf::store::TripleStore;
use unify_rdf::triple::Term;
use unify_receipts::receipt::Receipt;

/// Typed error variants for all CLI command handlers.
#[derive(thiserror::Error, Debug)]
pub enum CommandError {
    #[error("JSON parse failed: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("turtle parse error: {0}")]
    TurtleParse(String),
    #[error("Genie error: {0}")]
    Genie(String),
    #[error("AutoML error: {0}")]
    Automl(String),
}

/// Entry point: dispatch the parsed CLI to the matching handler.
pub fn run(cli: Cli) -> Result<Output, CommandError> {
    match cli.command {
        Commands::Receipt { label, data } => cmd_receipt(&label, &data),
        Commands::Verify { chain_json } => cmd_verify(&chain_json),
        Commands::Gate { law, data } => cmd_gate(&law, &data),
        Commands::Info => cmd_info(),
        Commands::Dispatch {
            namespace,
            noun,
            verb,
            input,
        } => cmd_dispatch(namespace.as_deref(), &noun, &verb, input.as_deref()),
        Commands::Query { ttl, pattern } => cmd_query(ttl.as_deref(), &pattern),
        Commands::Witnesses { domain } => cmd_witnesses(domain.as_deref()),
        Commands::Genie { subcommand } => cmd_genie(subcommand),
        Commands::WorldParse { intent, output } => cmd_world_parse(&intent, &output),
        Commands::WorldValidate { spec } => cmd_world_validate(&spec),
        Commands::WorldGenerate { spec, output } => cmd_world_generate(&spec, &output),
        Commands::WorldDeploy { spec, log } => cmd_world_deploy(&spec, &log),
        Commands::WorldEvolve {
            spec,
            intent,
            output,
        } => cmd_world_evolve(&spec, &intent, &output),
        Commands::Automl { subcommand } => cmd_automl(subcommand),
        Commands::Dev { subcommand } => cmd_dev(subcommand),
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
        pipeline
            .load_turtle(turtle)
            .map_err(|e| CommandError::TurtleParse(e.to_string()))?;
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
        unify_sem::init();
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

/// Genie 26 World Manufacturing Platform CLI command handler.
pub fn cmd_genie(subcmd: GenieSubcommands) -> Result<Output, CommandError> {
    match subcmd {
        GenieSubcommands::Manufacture {
            intent,
            out_spec,
            out_t3d,
        } => {
            // Load intent (either file path or raw string)
            let intent_str = if std::path::Path::new(&intent).exists() {
                std::fs::read_to_string(&intent).map_err(|e| {
                    CommandError::Genie(format!("Failed to read intent file {}: {}", intent, e))
                })?
            } else {
                intent
            };

            // Parse
            let mut spec = genie_core::parse_intent(&intent_str)
                .map_err(|e| CommandError::Genie(format!("Parse failed: {}", e)))?;

            // Validate
            let gate = genie_core::laws::WorldCoherenceGate::new();
            if let Err(errors) = gate.validate(&spec) {
                return Err(CommandError::Genie(format!(
                    "World coherence validation failed:\n{}",
                    errors.join("\n")
                )));
            }

            // Receipts
            genie_core::receipt_chain::ReceiptChainManager::generate_receipt_chain(
                &mut spec,
                b"genie_salt",
            )
            .map_err(|e| CommandError::Genie(format!("Receipt chaining failed: {}", e)))?;

            // Save JSON Spec
            let spec_json = serde_json::to_string_pretty(&spec)?;
            std::fs::write(&out_spec, spec_json).map_err(|e| {
                CommandError::Genie(format!("Failed to write spec JSON to {}: {}", out_spec, e))
            })?;

            // Save T3D Layout
            let t3d_str = genie_core::layout::LayoutCompiler::compile(&spec);
            std::fs::write(&out_t3d, t3d_str).map_err(|e| {
                CommandError::Genie(format!("Failed to write T3D layout to {}: {}", out_t3d, e))
            })?;

            Ok(Output::ok(json!({
                "status": "manufactured",
                "spec_path": out_spec,
                "t3d_path": out_t3d,
                "places_count": spec.places.len(),
                "actors_count": spec.actors.len(),
                "objects_count": spec.objects.len(),
            })))
        }
        GenieSubcommands::Evolve {
            spec,
            intent,
            out_spec,
            out_t3d,
        } => {
            // Load initial spec
            let spec_content = std::fs::read_to_string(&spec).map_err(|e| {
                CommandError::Genie(format!("Failed to read spec file {}: {}", spec, e))
            })?;
            let world_spec: WorldSpec = serde_json::from_str(&spec_content)?;

            // Load modification intent
            let intent_str = if std::path::Path::new(&intent).exists() {
                std::fs::read_to_string(&intent).map_err(|e| {
                    CommandError::Genie(format!("Failed to read intent file {}: {}", intent, e))
                })?
            } else {
                intent
            };

            // Evolve
            let evolved_spec =
                genie_core::evolution::WorldEvolver::evolve(&world_spec, &intent_str)
                    .map_err(|e| CommandError::Genie(format!("Evolution failed: {}", e)))?;

            // Validate
            let gate = genie_core::laws::WorldCoherenceGate::new();
            if let Err(errors) = gate.validate(&evolved_spec) {
                return Err(CommandError::Genie(format!(
                    "Evolved world coherence validation failed:\n{}",
                    errors.join("\n")
                )));
            }

            // Save JSON Spec
            let spec_json = serde_json::to_string_pretty(&evolved_spec)?;
            std::fs::write(&out_spec, spec_json).map_err(|e| {
                CommandError::Genie(format!(
                    "Failed to write evolved spec JSON to {}: {}",
                    out_spec, e
                ))
            })?;

            // Save T3D Layout
            let t3d_str = genie_core::layout::LayoutCompiler::compile(&evolved_spec);
            std::fs::write(&out_t3d, t3d_str).map_err(|e| {
                CommandError::Genie(format!(
                    "Failed to write evolved T3D layout to {}: {}",
                    out_t3d, e
                ))
            })?;

            Ok(Output::ok(json!({
                "status": "evolved",
                "spec_path": out_spec,
                "t3d_path": out_t3d,
                "places_count": evolved_spec.places.len(),
                "actors_count": evolved_spec.actors.len(),
                "objects_count": evolved_spec.objects.len(),
            })))
        }
        GenieSubcommands::Deploy { spec, log } => {
            // Load spec
            let spec_content = std::fs::read_to_string(&spec).map_err(|e| {
                CommandError::Genie(format!("Failed to read spec file {}: {}", spec, e))
            })?;
            let world_spec: WorldSpec = serde_json::from_str(&spec_content)?;

            // Deploy
            genie_core::deployment::DeploymentManager::deploy(
                &world_spec,
                std::path::Path::new(&log),
            )
            .map_err(|e| CommandError::Genie(format!("Deployment failed: {}", e)))?;

            Ok(Output::ok(json!({
                "status": "deployed",
                "log_path": log,
            })))
        }
    }
}

fn load_intent(intent: &str) -> Result<String, CommandError> {
    if std::path::Path::new(intent).exists() {
        std::fs::read_to_string(intent).map_err(|e| {
            CommandError::Genie(format!("Failed to read intent file {}: {}", intent, e))
        })
    } else {
        Ok(intent.to_string())
    }
}

fn load_world_spec(spec_path: &str) -> Result<WorldSpec, CommandError> {
    let spec_content = std::fs::read_to_string(spec_path).map_err(|e| {
        CommandError::Genie(format!("Failed to read spec file {}: {}", spec_path, e))
    })?;
    let spec: WorldSpec = serde_json::from_str(&spec_content)
        .map_err(|e| CommandError::Genie(format!("Failed to parse spec JSON: {}", e)))?;
    Ok(spec)
}

pub fn cmd_world_parse(intent: &str, output: &str) -> Result<Output, CommandError> {
    let intent_str = load_intent(intent)?;
    let spec = genie_core::parse_intent(&intent_str)
        .map_err(|e| CommandError::Genie(format!("Parse failed: {}", e)))?;
    let spec_json = serde_json::to_string_pretty(&spec)?;
    std::fs::write(output, spec_json)
        .map_err(|e| CommandError::Genie(format!("Failed to write parsed spec: {}", e)))?;
    Ok(Output::ok(json!({
        "status": "parsed",
        "output": output,
        "places": spec.places.len(),
        "actors": spec.actors.len(),
        "objects": spec.objects.len(),
    })))
}

pub fn cmd_world_validate(spec: &str) -> Result<Output, CommandError> {
    let world_spec = load_world_spec(spec)?;
    let gate = genie_core::laws::WorldCoherenceGate::new();
    match gate.validate(&world_spec) {
        Ok(_) => Ok(Output {
            data: json!({ "valid": true }),
            success: true,
            message: Some("World specification is coherent".to_string()),
        }),
        Err(errors) => Ok(Output {
            data: json!({ "valid": false, "errors": errors }),
            success: false,
            message: Some(format!(
                "Coherence validation failed:\n{}",
                errors.join("\n")
            )),
        }),
    }
}

pub fn cmd_world_generate(spec: &str, output: &str) -> Result<Output, CommandError> {
    let mut world_spec = load_world_spec(spec)?;

    // Regenerate and verify the BLAKE3 receipt chain using ReceiptChainManager and secret salt b"genie_salt"
    genie_core::receipt_chain::ReceiptChainManager::generate_receipt_chain(
        &mut world_spec,
        b"genie_salt",
    )
    .map_err(|e| CommandError::Genie(format!("Receipt chaining failed: {}", e)))?;

    // Save the updated spec back to its original path to store the new receipts
    let spec_json = serde_json::to_string_pretty(&world_spec)?;
    std::fs::write(spec, spec_json)
        .map_err(|e| CommandError::Genie(format!("Failed to write updated spec: {}", e)))?;

    // Verify receipt chain
    let is_valid = genie_core::receipt_chain::ReceiptChainManager::verify_receipt_chain(
        &world_spec,
        b"genie_salt",
    );
    if !is_valid {
        return Err(CommandError::Genie(
            "Failed to verify newly generated receipt chain".to_string(),
        ));
    }

    // Compile layout to T3D
    let t3d_str = genie_core::layout::LayoutCompiler::compile(&world_spec);
    std::fs::write(output, t3d_str).map_err(|e| {
        CommandError::Genie(format!("Failed to write T3D layout to {}: {}", output, e))
    })?;

    Ok(Output::ok(json!({
        "status": "generated",
        "output": output,
        "receipt_chain_valid": is_valid,
    })))
}

pub fn cmd_world_deploy(spec: &str, log: &str) -> Result<Output, CommandError> {
    let world_spec = load_world_spec(spec)?;
    genie_core::deployment::DeploymentManager::deploy(&world_spec, std::path::Path::new(log))
        .map_err(|e| CommandError::Genie(format!("Deployment failed: {}", e)))?;

    use std::io::Write;
    let _ = writeln!(
        std::io::stdout(),
        "Deployment active! Server running on http://127.0.0.1:8080"
    );
    let _ = writeln!(std::io::stdout(), "Press Ctrl+C to terminate.");
    if std::env::var("GENIE_TEST_NO_BLOCK").is_ok() {
        return Ok(Output::ok(json!({
            "status": "deployed",
            "log_path": log,
            "test_mode": true
        })));
    }
    // Loop to keep main thread alive in CLI
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}

pub fn cmd_world_evolve(spec: &str, intent: &str, output: &str) -> Result<Output, CommandError> {
    let world_spec = load_world_spec(spec)?;
    let intent_str = load_intent(intent)?;

    // Evolve spec
    let mut evolved_spec = genie_core::evolution::WorldEvolver::evolve(&world_spec, &intent_str)
        .map_err(|e| CommandError::Genie(format!("Evolution failed: {}", e)))?;

    // Regenerate and verify the BLAKE3 receipt chain using ReceiptChainManager and secret salt b"genie_salt"
    genie_core::receipt_chain::ReceiptChainManager::generate_receipt_chain(
        &mut evolved_spec,
        b"genie_salt",
    )
    .map_err(|e| CommandError::Genie(format!("Receipt chaining failed: {}", e)))?;

    // Verify receipt chain
    let is_valid = genie_core::receipt_chain::ReceiptChainManager::verify_receipt_chain(
        &evolved_spec,
        b"genie_salt",
    );
    if !is_valid {
        return Err(CommandError::Genie(
            "Failed to verify evolved receipt chain".to_string(),
        ));
    }

    // Save evolved spec
    let spec_json = serde_json::to_string_pretty(&evolved_spec)?;
    std::fs::write(output, spec_json).map_err(|e| {
        CommandError::Genie(format!(
            "Failed to write evolved spec JSON to {}: {}",
            output, e
        ))
    })?;

    Ok(Output::ok(json!({
        "status": "evolved",
        "output": output,
        "receipt_chain_valid": is_valid,
        "places": evolved_spec.places.len(),
        "actors": evolved_spec.actors.len(),
        "objects": evolved_spec.objects.len(),
    })))
}

pub fn cmd_automl(subcmd: AutomlSubcommands) -> Result<Output, CommandError> {
    let args = match subcmd {
        AutomlSubcommands::Discover { path } => vec!["discover".to_string(), path],
        AutomlSubcommands::Optimize {
            points,
            target,
            sims,
        } => vec![
            "optimize".to_string(),
            points.to_string(),
            target.to_string(),
            sims.to_string(),
        ],
    };
    let out = unify_automl::cli::dispatch_command(&args)
        .map_err(|e| CommandError::Automl(e.to_string()))?;
    Ok(Output {
        data: out.data,
        success: out.success,
        message: Some(out.message),
    })
}

pub fn cmd_dev(subcmd: DevSubcommands) -> Result<Output, CommandError> {
    let args = match subcmd {
        DevSubcommands::Init { path } => vec!["init".to_string(), path],
        DevSubcommands::Start { path } => vec!["start".to_string(), path],
    };
    let out = unify_automl::cli::dispatch_dev_command(&args)
        .map_err(|e| CommandError::Automl(e.to_string()))?;
    Ok(Output {
        data: out.data,
        success: out.success,
        message: Some(out.message),
    })
}
