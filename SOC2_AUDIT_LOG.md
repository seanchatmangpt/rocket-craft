# SOC2 Audit Log — Integration & Operations Guide

`tools/rocket-sdk/src/audit_log.rs` implements a SOC2 CC7.2/CC7.3 compliant structured
event log for the Rocket Craft monorepo.  It is the **write-ahead log** layer that feeds
the tamper-evident BLAKE3 receipt chain in `audit_affidavit.rs`.

---

## SOC2 Controls Addressed

| Control | Description | How this module satisfies it |
|---------|-------------|------------------------------|
| CC7.2 | Monitors system components for anomalies | `AuditEventType` covers all required event categories: authentication, authorisation, resource lifecycle, configuration change, security violations, policy violations, compliance scans |
| CC7.3 | Evaluates security events and communicates findings | `AuditLogAccessed` events ensure that audit log reads are themselves logged; `seal_into_receipt` ties daily batches into a BLAKE3 tamper-evident chain compatible with `affidavit verify` |
| CC6.1 | Logical access security measures | `Actor` struct captures `id`, `kind` (Human/ServiceAccount/System/CiPipeline), `ip`, and `session_id` on every event |
| CC6.6 | Security event detection | `SecurityViolation` and `PolicyViolation { policy }` event types are counted by `summarize()` and surfaced in monitoring dashboards |

---

## Wiring into `rocket-sdk`

Add one line to `tools/rocket-sdk/src/lib.rs`:

```rust
pub mod audit_log;
```

No new Cargo dependencies are required.  The module uses only crates already declared in
`tools/rocket-sdk/Cargo.toml`:

| Crate | Feature used |
|-------|-------------|
| `blake3` `1.5` | `blake3::hash()` for receipt sealing |
| `serde` `1.0` | `Serialize` / `Deserialize` derives |
| `serde_json` `1.0` | JSONL serialisation, canonical JSON for hashing |
| `chrono` `0.4` (serde feature) | `DateTime<Utc>` timestamps |
| `anyhow` `1.0` | `Result<T>` error propagation |
| `tempfile` `3.10` (dev-dep) | Temp directories in `#[cfg(test)]` tests |

---

## Adding `rocket audit-log` subcommands to `rocket-cmd`

### `Cargo.toml` (`rocket-cmd`)

No new deps — `rocket-sdk` already re-exports the module.

### `main.rs` additions

```rust
// In the top-level Cli Args struct (clap derive):
#[command(subcommand)]
command: Commands,

// New variant in Commands:
AuditLog(AuditLogArgs),

// New sub-command struct:
#[derive(Debug, clap::Args)]
pub struct AuditLogArgs {
    #[command(subcommand)]
    pub sub: AuditLogCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum AuditLogCommand {
    /// Print the last N audit events
    Tail {
        #[arg(long, default_value_t = 50)]
        n: usize,
    },
    /// Seal all events for a date into a BLAKE3 receipt
    Seal {
        #[arg(long)]
        date: Option<String>,
    },
    /// Print a summary of audit events for a date
    Summary {
        #[arg(long)]
        date: Option<String>,
    },
}

// Handler (add inside the main match block):
Commands::AuditLog(args) => {
    use rocket_sdk::audit_log::AuditLogger;
    let log_dir = std::path::PathBuf::from("audit-logs");
    match args.sub {
        AuditLogCommand::Tail { n } => {
            // rocket audit-log tail --n 50
            let events = AuditLogger::tail_events(&log_dir, n)?;
            for ev in &events {
                println!("{}", serde_json::to_string(ev)?);
            }
            eprintln!("[audit-log] {} events", events.len());
        }
        AuditLogCommand::Seal { date } => {
            // rocket audit-log seal --date 2026-06-19
            let date_str = date.as_deref();
            let events = AuditLogger::read_events(&log_dir, date_str)?;
            let hash = AuditLogger::seal_into_receipt(&events)?;
            println!("chain_hash: {hash}");
            eprintln!("[audit-log] sealed {} events", events.len());
        }
        AuditLogCommand::Summary { date } => {
            // rocket audit-log summary
            let date_str = date.as_deref();
            let events = AuditLogger::read_events(&log_dir, date_str)?;
            let summary = AuditLogger::summarize(&events);
            println!("total_events:        {}", summary.total_events);
            println!("auth_failures:       {}", summary.auth_failures);
            println!("security_violations: {}", summary.security_violations);
            println!("policy_violations:   {}", summary.policy_violations);
            if let Some((min, max)) = summary.date_range {
                println!("date_range:          {} — {}", min, max);
            }
        }
    }
}
```

### Example invocations

```bash
# Tail the 50 most recent audit events (JSONL output)
./rocket audit-log tail --n 50

# Seal yesterday's events into a BLAKE3 receipt hash
./rocket audit-log seal --date 2026-06-19

# Summarise today's events for a monitoring dashboard
./rocket audit-log summary

# Summarise a specific date
./rocket audit-log summary --date 2026-06-18
```

---

## Daily batch job — seal + purge

Suggested cron (or GitHub Actions scheduled workflow):

```bash
#!/usr/bin/env bash
set -euo pipefail
YESTERDAY=$(date -d "yesterday" +%Y-%m-%d)

# Seal yesterday's log into a receipt hash and store it
HASH=$(./rocket audit-log seal --date "$YESTERDAY" | grep chain_hash | awk '{print $2}')
echo "$YESTERDAY $HASH" >> audit-receipts.log

# Purge logs older than 365 days (SOC2 default retention)
./rocket audit-log purge --max-age-days 365
```

---

## Log file locations

| File pattern | Description |
|---|---|
| `audit-logs/audit-YYYY-MM-DD.jsonl` | Active daily log (JSONL, one event per line) |
| `audit-logs/audit-YYYY-MM-DD.jsonl.1` | Rotated overflow file when size limit exceeded |

Default size rotation threshold: configurable via `AuditLogger::new(dir, max_bytes)`.
Recommended: `50 * 1024 * 1024` (50 MB) per day.

---

## Retention policy (SOC2 default: 365 days)

```rust
use rocket_sdk::audit_log::{AuditLogger, RetentionPolicy};

let logger = AuditLogger::new("audit-logs".into(), 50 * 1024 * 1024)?;
let policy = RetentionPolicy { max_age_days: 365 };
let deleted = logger.purge_old_logs(&policy)?;
println!("Purged {deleted} old log files");
```

---

## BLAKE3 chain compatibility with `affidavit`

`seal_into_receipt` uses the same algorithm as `audit_affidavit::record_audit`:

1. Seed: `blake3::hash(b"affidavit-v26.6.14-genesis")`
2. For each event: `blake3::hash(prev_hex_bytes || canonical_json_bytes)`
3. Canonical JSON = all object keys sorted recursively (BTreeMap order)

This means the final hash of a batch of audit events can be submitted to
`affi verify` for external chain verification.
