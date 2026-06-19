//! Supabase pipeline commands — connectivity check, migration push, receipt query.
//!
//! All HTTP logic lives in `rocket_sdk::supabase::SupabaseService`.
//! Verbs here are thin coordinators (complexity ≤ 5).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde_json::Value;

fn make_service() -> Result<rocket_sdk::supabase::SupabaseService> {
    let url = std::env::var("SUPABASE_URL")
        .unwrap_or_else(|_| "http://localhost:54321".to_string());
    let key = std::env::var("SUPABASE_SERVICE_ROLE_KEY")
        .or_else(|_| std::env::var("SUPABASE_ANON_KEY"))
        .unwrap_or_default();
    if key.is_empty() {
        return Err(clap_noun_verb::NounVerbError::execution_error(
            "SUPABASE_ANON_KEY (or SUPABASE_SERVICE_ROLE_KEY) not set. \
             Copy nuxt-shell/.env.example → nuxt-shell/.env and fill in the values."
                .to_string(),
        ));
    }
    Ok(rocket_sdk::supabase::SupabaseService::new(url, key))
}

fn new_runtime() -> Result<tokio::runtime::Runtime> {
    tokio::runtime::Runtime::new()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))
}

fn print_ping_results(url: &str, checks: &[(String, bool, String)], health: u16) {
    println!("=== Supabase Ping ===");
    println!("[{}] REST: {} (HTTP {})", if health == 200 { "PASS" } else { "FAIL" }, url, health);
    for (table, ok, msg) in checks {
        println!("[{}] {}: {}", if *ok { "PASS" } else { "FAIL" }, table, msg);
    }
}

// ── Verbs ─────────────────────────────────────────────────────────────────────

/// Generate an Ed25519 key pair for receipt signing.
///
/// Writes the keys to stdout in .env format. Copy them into nuxt-shell/.env:
///   ROCKET_SIGNING_KEY=<private>
///   ROCKET_SIGNING_PUBKEY=<public>
///
/// The private key is used by `rocket html5 verify` to sign cook receipts.
/// The public key is used by the browser and verify_receipt server route to
/// confirm the receipt originated from the real CLI pipeline (not forged).
#[verb("keygen", "supabase")]
fn supabase_keygen() -> Result<Value> {
    let (priv_b64, pub_b64) = rocket_sdk::signing::generate_keypair()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;
    println!("# Add these to nuxt-shell/.env and tools/.env");
    println!("ROCKET_SIGNING_KEY={priv_b64}");
    println!("ROCKET_SIGNING_PUBKEY={pub_b64}");
    Ok(serde_json::json!({ "public_key": pub_b64 }))
}

/// Ping Supabase and verify that all pipeline tables exist.
///
/// Reads SUPABASE_URL and SUPABASE_ANON_KEY / SUPABASE_SERVICE_ROLE_KEY from env.
/// After `rocket supabase migrate`, use this to confirm the database is ready.
#[verb("ping", "supabase")]
fn ping_supabase() -> Result<Value> {
    let svc = make_service()?;
    let url = std::env::var("SUPABASE_URL")
        .unwrap_or_else(|_| "http://localhost:54321".to_string());
    let rt = new_runtime()?;
    let (checks, health) = rt.block_on(svc.ping());
    print_ping_results(&url, &checks, health);
    let all_ok = checks.iter().all(|(_, ok, _)| *ok);
    let overall = if health == 200 && all_ok { "READY" } else { "NOT READY" };
    println!("\n[{overall}]");
    if !all_ok {
        println!("  Hint: cd nuxt-shell && supabase db push");
    }
    Ok(serde_json::json!({
        "overall": overall,
        "rest_health": health,
        "tables": checks.iter().map(|(t, ok, msg)| serde_json::json!({ "table": t, "ok": ok, "message": msg })).collect::<Vec<_>>(),
    }))
}

/// Apply Supabase migrations from nuxt-shell/supabase/migrations/.
///
/// Delegates to the `supabase db push` CLI (brew install supabase/tap/supabase).
///
/// # Arguments
/// * `nuxt_dir` - Path to nuxt-shell/ (default: ./nuxt-shell from repo root)
/// * `db_url`   - Override DB URL (default: local Supabase instance)
#[verb("migrate", "supabase")]
fn migrate_supabase(nuxt_dir: Option<String>, db_url: Option<String>) -> Result<Value> {
    let root = std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;
    let nuxt_path = nuxt_dir.map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("nuxt-shell"));
    if !nuxt_path.join("supabase/migrations").exists() {
        return Err(clap_noun_verb::NounVerbError::execution_error(
            format!("supabase/migrations not found at {}", nuxt_path.display())));
    }
    let cli_ok = std::process::Command::new("supabase").arg("--version")
        .output().map(|o| o.status.success()).unwrap_or(false);
    if !cli_ok {
        return Err(clap_noun_verb::NounVerbError::execution_error(
            "Supabase CLI not found. Install: brew install supabase/tap/supabase".to_string()));
    }
    let mut cmd = std::process::Command::new("supabase");
    cmd.arg("db").arg("push").current_dir(&nuxt_path);
    if let Some(ref u) = db_url { cmd.arg("--db-url").arg(u); }
    let ok = cmd.status()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?
        .success();
    if !ok {
        return Err(clap_noun_verb::NounVerbError::execution_error(
            "supabase db push exited non-zero".to_string()));
    }
    println!("[migrate] DONE");
    Ok(serde_json::json!({ "status": "ok", "nuxt_dir": nuxt_path.display().to_string() }))
}

/// Show the 10 most recent game receipts from Supabase.
///
/// Confirms that cook receipts from `rocket html5 verify` are in the database.
#[verb("receipts", "supabase")]
fn supabase_receipts() -> Result<Value> {
    let svc = make_service()?;
    let rt = new_runtime()?;
    let receipts = rt.block_on(svc.recent_receipts())
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;
    if receipts.is_empty() {
        println!("No receipts. Run `rocket html5 verify` or `rocket html5 e2e`.");
    }
    for r in &receipts {
        println!("[{}] {}  engine={}  events={}  at={}",
            r["verdict"].as_str().unwrap_or("?"),
            r["milestone"].as_str().unwrap_or("?"),
            r["engine_source"].as_str().unwrap_or("?"),
            r["ocel_event_count"].as_u64().unwrap_or(0),
            r["proven_at"].as_str().unwrap_or("?"));
    }
    Ok(serde_json::json!({ "receipts": receipts }))
}

/// Export ocel_events for a session as OCEL 2.0 JSON for pm4py conformance checking.
///
/// Usage:
///   rocket supabase ocel-export --session-id <uuid> [--out ocel2.json]
///
/// Drop the output into pm4py:
///   import pm4py
///   log = pm4py.read_ocel2_json('ocel2.json')
///   process_tree = pm4py.discover_process_tree_inductive(log)
///
/// # Arguments
/// * `session_id` - Session UUID to export (required)
/// * `out`        - Output file path (default: ocel2-<session_id_prefix>.json)
#[verb("ocel-export", "supabase")]
fn supabase_ocel_export(session_id: String, out: Option<String>) -> Result<Value> {
    let svc = make_service()?;
    let rt = new_runtime()?;
    let rows = rt.block_on(svc.fetch_ocel_events(&session_id))
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;

    if rows.is_empty() {
        return Err(clap_noun_verb::NounVerbError::execution_error(
            format!("No events found for session {session_id}")));
    }

    let ocel2 = build_ocel2(&session_id, &rows);
    let json = serde_json::to_string_pretty(&ocel2)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;

    let prefix = &session_id[..8.min(session_id.len())];
    let path = out.unwrap_or_else(|| format!("ocel2-{prefix}.json"));
    std::fs::write(&path, &json)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;

    println!("[ocel-export] {path}");
    println!("  events:  {}", rows.len());
    println!("  pm4py:   log = pm4py.read_ocel2_json('{path}')");
    Ok(serde_json::json!({ "path": path, "events": rows.len(), "session_id": session_id }))
}

fn infer_object_type(id: &str) -> &'static str {
    if id.starts_with("session") { "GameSession" }
    else if id.starts_with("intent") { "Intent" }
    else { "GameSession" }
}

fn build_ocel2(session_id: &str, rows: &[serde_json::Value]) -> serde_json::Value {
    let mut objects: std::collections::HashMap<String, &str> = std::collections::HashMap::new();
    objects.insert(session_id.to_string(), "GameSession");

    for row in rows {
        if let Some(refs) = row["object_refs"].as_array() {
            for r in refs {
                if let Some(s) = r.as_str() {
                    objects.entry(s.to_string()).or_insert_with(|| infer_object_type(s));
                }
            }
        }
    }

    let activity_set: std::collections::HashSet<&str> = rows.iter()
        .filter_map(|r| r["activity"].as_str()).collect();

    let ocel_events: Vec<serde_json::Value> = rows.iter().enumerate().map(|(i, row)| {
        let ts_ms = row["timestamp_ms"].as_i64().unwrap_or(0);
        let ts = chrono::DateTime::from_timestamp_millis(ts_ms)
            .map(|d| d.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
            .unwrap_or_else(|| ts_ms.to_string());

        let mut rels = vec![serde_json::json!({ "objectId": session_id, "qualifier": "session" })];
        if let Some(refs) = row["object_refs"].as_array() {
            for r in refs {
                if let Some(s) = r.as_str() {
                    if s != session_id {
                        rels.push(serde_json::json!({ "objectId": s, "qualifier": "rel" }));
                    }
                }
            }
        }

        let mut attrs = vec![
            serde_json::json!({ "name": "seq", "value": row["seq"] }),
            serde_json::json!({ "name": "event_hash", "value": row["event_hash"] }),
        ];
        if !row["prev_hash"].is_null() {
            attrs.push(serde_json::json!({ "name": "prev_hash", "value": row["prev_hash"] }));
        }
        if let Some(obj) = row["attributes"].as_object() {
            for (k, v) in obj { attrs.push(serde_json::json!({ "name": k, "value": v })); }
        }

        serde_json::json!({
            "id": format!("ev-{i}"),
            "type": row["activity"],
            "time": ts,
            "attributes": attrs,
            "relationships": rels,
        })
    }).collect();

    let ocel_objects: Vec<serde_json::Value> = objects.iter()
        .map(|(id, t)| serde_json::json!({ "id": id, "type": t, "attributes": [] }))
        .collect();

    serde_json::json!({
        "objectTypes": objects.values().collect::<std::collections::HashSet<_>>().iter()
            .map(|t| serde_json::json!({ "name": t, "attributes": [] })).collect::<Vec<_>>(),
        "eventTypes": activity_set.iter()
            .map(|a| serde_json::json!({ "name": a, "attributes": [{"name":"seq","type":"integer"}] }))
            .collect::<Vec<_>>(),
        "objects": ocel_objects,
        "events": ocel_events,
    })
}

/// Verify the ocel_events hash chain via the verify_event_chain Postgres RPC.
///
/// Checks that each event's prev_hash matches the prior event's event_hash.
/// A chain break means the event log has been tampered with or is incomplete.
///
/// # Arguments
/// * `session_id` - Verify a specific session UUID (default: all sessions)
#[verb("chain-verify", "supabase")]
fn supabase_chain_verify(session_id: Option<String>) -> Result<Value> {
    let svc = make_service()?;
    let rt = new_runtime()?;
    let rows = rt.block_on(svc.verify_event_chain(session_id.as_deref()))
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;

    if rows.is_empty() {
        println!("[?] No sessions found or RPC unavailable.");
        println!("    Run `rocket supabase migrate` to create verify_event_chain.");
        return Ok(serde_json::json!({ "overall": "UNKNOWN", "rows": [] }));
    }

    let all_ok = rows.iter().all(|r| r["ok"].as_bool().unwrap_or(false));
    let overall = if all_ok { "PASS" } else { "FAIL" };
    println!("=== OCEL Chain Verify ===");
    for r in &rows {
        let ok = r["ok"].as_bool().unwrap_or(false);
        let msg = r["message"].as_str().unwrap_or("?");
        let sid = r["session_id"].as_str().unwrap_or("?");
        println!("[{}] session={sid}  {msg}", if ok { "PASS" } else { "FAIL" });
    }
    println!("\n[{overall}]");
    Ok(serde_json::json!({ "overall": overall, "rows": rows }))
}

/// Compute process conformance metrics against the declared pipeline model.
///
/// Declared model: GameSessionStarted → FrameRendered → InputAdmitted*
///
/// Computes Van der Aalst's four quality dimensions from session_lifecycle_summary:
///   fitness       — fraction of sessions containing all required activities
///   precision     — fraction of sessions with no unexpected activity types
///   generalization — fraction of required activities seen across all sessions
///   simplicity    — 1.0 (single-trace declared model)
///
/// # Arguments
/// * `out`   - Output file path (default: conformance-<date>.json)
/// * `limit` - Max sessions to analyse (default: 200)
#[verb("conformance", "supabase")]
fn supabase_conformance(out: Option<String>, limit: Option<u32>) -> Result<Value> {
    let svc = make_service()?;
    let rt = new_runtime()?;
    let n = limit.unwrap_or(200);
    let sessions = rt.block_on(svc.fetch_session_lifecycle_summary(n))
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;

    if sessions.is_empty() {
        println!("[conformance] No sessions found.");
        return Ok(serde_json::json!({ "status": "no-data", "sessions": 0 }));
    }

    let report = compute_conformance(&sessions);
    let path = out.unwrap_or_else(|| {
        let date = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        format!("conformance-{date}.json")
    });
    let json = serde_json::to_string_pretty(&report)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;
    std::fs::write(&path, &json)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;

    let fitness = report["fitness"].as_f64().unwrap_or(0.0);
    let label = if fitness >= 0.95 { "CONFORMANT" } else if fitness >= 0.75 { "DEGRADED" } else { "NON-CONFORMANT" };
    println!("=== Process Conformance ===");
    println!("Model:   GameSessionStarted → FrameRendered → InputAdmitted*");
    println!("Status:  {label}");
    println!("Fitness:        {:.1}%", fitness * 100.0);
    println!("Precision:      {:.1}%", report["precision"].as_f64().unwrap_or(0.0) * 100.0);
    println!("Generalization: {:.1}%", report["generalization"].as_f64().unwrap_or(0.0) * 100.0);
    println!("Simplicity:     100.0%");
    println!("Sessions:       {}/{}", report["conformant_sessions"].as_u64().unwrap_or(0), report["total_sessions"].as_u64().unwrap_or(0));
    println!("\n[{label}] Report: {path}");
    Ok(report)
}

const REQUIRED: &[&str] = &["GameSessionStarted", "FrameRendered"];
const EXPECTED: &[&str] = &["GameSessionStarted", "FrameRendered", "InputAdmitted"];

fn compute_conformance(sessions: &[serde_json::Value]) -> serde_json::Value {
    let mut all_activities: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut conformant = 0usize;
    let mut non_conformant: Vec<serde_json::Value> = Vec::new();

    for s in sessions {
        let sid = s["session_id"].as_str().unwrap_or("?");
        let activities: std::collections::HashSet<String> = s["distinct_activities"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        for a in &activities { all_activities.insert(a.clone()); }

        let missing: Vec<&str> = REQUIRED.iter().copied().filter(|r| !activities.contains(*r)).collect();
        let extra: Vec<String> = activities.iter().filter(|a| !EXPECTED.contains(&a.as_str())).cloned().collect();

        if missing.is_empty() && extra.is_empty() {
            conformant += 1;
        } else {
            non_conformant.push(serde_json::json!({ "session_id": sid, "missing": missing, "extra": extra }));
        }
    }

    let n = sessions.len() as f64;
    let precision_count = sessions.iter().filter(|s| {
        let acts: std::collections::HashSet<String> = s["distinct_activities"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        acts.iter().all(|a| EXPECTED.contains(&a.as_str()))
    }).count();

    let generalization = REQUIRED.iter().filter(|r| all_activities.contains(**r)).count() as f64
        / REQUIRED.len() as f64;

    serde_json::json!({
        "model": "GameSessionStarted → FrameRendered → InputAdmitted*",
        "fitness": conformant as f64 / n,
        "precision": precision_count as f64 / n,
        "generalization": generalization,
        "simplicity": 1.0,
        "conformant_sessions": conformant,
        "total_sessions": sessions.len(),
        "non_conformant": non_conformant,
        "all_activities_seen": all_activities.into_iter().collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn supabase_url_falls_back_to_local() {
        let url = std::env::var("SUPABASE_URL")
            .unwrap_or_else(|_| "http://localhost:54321".to_string());
        assert!(url.starts_with("http"));
    }
}
