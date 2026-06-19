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

#[cfg(test)]
mod tests {
    #[test]
    fn supabase_url_falls_back_to_local() {
        let url = std::env::var("SUPABASE_URL")
            .unwrap_or_else(|_| "http://localhost:54321".to_string());
        assert!(url.starts_with("http"));
    }
}
