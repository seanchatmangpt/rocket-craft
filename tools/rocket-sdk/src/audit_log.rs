//! SOC2 CC7.2/CC7.3 compliant structured audit event log.
//!
//! This module is the structured event stream that feeds the tamper-evident
//! affidavit chain in `audit_affidavit.rs`. Think of it as the "write-ahead log"
//! for SOC2 evidence: events flow into JSONL files on disk; a daily batch job
//! seals them into BLAKE3 receipts via `seal_into_receipt`.
//!
//! # SOC2 Controls addressed
//!
//! | Control | How                                               |
//! |---------|---------------------------------------------------|
//! | CC7.2   | All event categories required by the standard     |
//! | CC7.3   | AuditLogAccessed events detect log tampering      |
//! | CC6.1   | Actor + session tracked on every event            |
//! | CC6.6   | SecurityViolation + PolicyViolation event types   |

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

// ── ID generation (timestamp + monotone counter, no uuid dep required) ─────────

static EVENT_COUNTER: AtomicU64 = AtomicU64::new(0);

fn new_event_id() -> String {
    let ts = Utc::now().timestamp_micros();
    let seq = EVENT_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("evt-{ts}-{seq:06}")
}

// ── Event version ──────────────────────────────────────────────────────────────

pub const AUDIT_LOG_VERSION: &str = "v1";

// ── AuditEventType ─────────────────────────────────────────────────────────────

/// SOC2 CC7.2 event categories.  All variants serialize with `serde(tag = "type")`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum AuditEventType {
    // ── Authentication ─────────────────────────────────────────────────────────
    AuthenticationSuccess,
    AuthenticationFailure,

    // ── Authorization ──────────────────────────────────────────────────────────
    AuthorizationGrant,
    AuthorizationDenial,

    // ── Resource lifecycle ─────────────────────────────────────────────────────
    ResourceCreated,
    ResourceModified,
    ResourceDeleted,
    ResourceAccessed,

    // ── Configuration ──────────────────────────────────────────────────────────
    ConfigurationChanged,

    // ── Audit visibility (CC7.3: log access itself must be logged) ─────────────
    AuditLogAccessed,

    // ── Security ───────────────────────────────────────────────────────────────
    SecurityViolation,

    // ── Generic system event ───────────────────────────────────────────────────
    SystemEvent { description: String },

    // ── CI / build pipeline ────────────────────────────────────────────────────
    BuildStarted,
    BuildCompleted { success: bool },

    // ── Deployment ─────────────────────────────────────────────────────────────
    DeploymentStarted,
    DeploymentCompleted { success: bool },

    // ── Secrets ────────────────────────────────────────────────────────────────
    SecretsAccessed,

    // ── Policy ─────────────────────────────────────────────────────────────────
    PolicyViolation { policy: String },

    // ── Compliance ─────────────────────────────────────────────────────────────
    ComplianceScanCompleted { passed: bool, findings: u32 },
}

// ── Actor ─────────────────────────────────────────────────────────────────────

/// Who performed the action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum ActorKind {
    Human { role: String },
    ServiceAccount(String),
    System,
    CiPipeline { job_id: String },
}

/// Full actor representation including network context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Actor {
    pub id: String,
    #[serde(flatten)]
    pub kind: ActorKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

impl Actor {
    /// Convenience constructor for a human actor.
    pub fn human(id: impl Into<String>, role: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: ActorKind::Human { role: role.into() },
            ip: None,
            session_id: None,
        }
    }

    /// Convenience constructor for the system actor.
    pub fn system() -> Self {
        Self {
            id: "system".to_string(),
            kind: ActorKind::System,
            ip: None,
            session_id: None,
        }
    }

    /// Convenience constructor for a CI pipeline actor.
    pub fn ci(job_id: impl Into<String>) -> Self {
        let job_id = job_id.into();
        Self {
            id: format!("ci:{job_id}"),
            kind: ActorKind::CiPipeline { job_id },
            ip: None,
            session_id: None,
        }
    }
}

// ── AuditResource ─────────────────────────────────────────────────────────────

/// The resource that the event targets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditResource {
    pub kind: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
}

impl AuditResource {
    pub fn new(kind: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            id: id.into(),
            name: None,
            path: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }
}

// ── Outcome ───────────────────────────────────────────────────────────────────

/// Whether the operation succeeded.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status")]
pub enum Outcome {
    Success,
    Failure { code: String, message: String },
    Partial { message: String },
}

// ── AuditEvent ────────────────────────────────────────────────────────────────

/// A single structured audit event.  Serialises to one JSONL line.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID — `evt-<microseconds>-<seq>`.
    pub id: String,
    /// UTC timestamp of the event.
    pub timestamp: DateTime<Utc>,
    /// Schema version — always `"v1"`.  Stored as `String` so the struct is
    /// `DeserializeOwned`; the const `AUDIT_LOG_VERSION` holds the canonical value.
    pub version: String,
    /// Categorised event type (SOC2 CC7.2).
    pub event_type: AuditEventType,
    /// Who performed the action.
    pub actor: Actor,
    /// Resource the action targeted.
    pub resource: AuditResource,
    /// Whether the action succeeded.
    pub outcome: Outcome,
    /// Extensible key-value context (BTreeMap ensures deterministic serialisation).
    pub metadata: BTreeMap<String, String>,
}

impl AuditEvent {
    /// Serialize this event to a single JSONL line (no trailing newline).
    pub fn to_jsonl(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

// ── AuditEventBuilder ─────────────────────────────────────────────────────────

/// Fluent builder for `AuditEvent`.
///
/// # Example
/// ```no_run
/// # use rocket_sdk::audit_log::{AuditEventBuilder, AuditEventType, Actor, AuditResource, Outcome};
/// let event = AuditEventBuilder::new(AuditEventType::BuildStarted)
///     .actor(Actor::ci("gh-actions-run-42"))
///     .resource(AuditResource::new("ue4-project", "ShooterGame"))
///     .outcome(Outcome::Success)
///     .meta("branch", "main")
///     .build();
/// ```
pub struct AuditEventBuilder {
    event_type: AuditEventType,
    actor: Option<Actor>,
    resource: Option<AuditResource>,
    outcome: Option<Outcome>,
    metadata: BTreeMap<String, String>,
    timestamp: Option<DateTime<Utc>>,
}

impl AuditEventBuilder {
    /// Start building an event of the given type.
    pub fn new(event_type: AuditEventType) -> Self {
        Self {
            event_type,
            actor: None,
            resource: None,
            outcome: None,
            metadata: BTreeMap::new(),
            timestamp: None,
        }
    }

    /// Set the actor.
    pub fn actor(mut self, actor: Actor) -> Self {
        self.actor = Some(actor);
        self
    }

    /// Set the resource.
    pub fn resource(mut self, resource: AuditResource) -> Self {
        self.resource = Some(resource);
        self
    }

    /// Set the outcome.
    pub fn outcome(mut self, outcome: Outcome) -> Self {
        self.outcome = Some(outcome);
        self
    }

    /// Add a metadata key-value pair.
    pub fn meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Override the event timestamp (useful in tests for determinism).
    pub fn timestamp(mut self, ts: DateTime<Utc>) -> Self {
        self.timestamp = Some(ts);
        self
    }

    /// Finalise the event.  Uses default System actor + no-resource + Success if
    /// fields were not explicitly set.
    pub fn build(self) -> AuditEvent {
        AuditEvent {
            id: new_event_id(),
            timestamp: self.timestamp.unwrap_or_else(Utc::now),
            version: AUDIT_LOG_VERSION.to_string(),
            event_type: self.event_type,
            actor: self.actor.unwrap_or_else(Actor::system),
            resource: self.resource.unwrap_or_else(|| AuditResource::new("unknown", "unknown")),
            outcome: self.outcome.unwrap_or(Outcome::Success),
            metadata: self.metadata,
        }
    }
}

// ── RetentionPolicy ──────────────────────────────────────────────────────────

/// SOC2 default: retain logs for 365 days.
pub struct RetentionPolicy {
    /// Maximum age of log files in days.  Files older than this are purged.
    pub max_age_days: u64,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self { max_age_days: 365 }
    }
}

// ── AuditSummary ──────────────────────────────────────────────────────────────

/// Aggregate counts for a monitoring dashboard or daily digest.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditSummary {
    pub total_events: usize,
    pub auth_failures: usize,
    pub security_violations: usize,
    pub policy_violations: usize,
    /// Earliest and latest event timestamps in the slice.
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

// ── Log file naming ───────────────────────────────────────────────────────────

fn log_filename(date: &str) -> String {
    format!("audit-{date}.jsonl")
}

fn today_str() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}

fn log_path(log_dir: &Path, date: &str) -> PathBuf {
    log_dir.join(log_filename(date))
}

fn rotate_path(log_dir: &Path, date: &str) -> PathBuf {
    log_dir.join(format!("audit-{date}.jsonl.1"))
}

// ── AuditLogger ───────────────────────────────────────────────────────────────

/// Append-only JSONL audit logger with automatic daily file rotation.
///
/// # Thread safety
///
/// `AuditLogger` takes `&mut self` so it is `!Sync`.  Wrap in a `Mutex<AuditLogger>`
/// when sharing across threads.
pub struct AuditLogger {
    log_dir: PathBuf,
    max_file_size_bytes: u64,
    current_date: String,
    file: File,
}

impl AuditLogger {
    /// Open (or create) the audit log for today in `log_dir`.
    ///
    /// `max_file_size_bytes` is the soft limit at which the current log file is
    /// rotated by renaming it to `audit-YYYY-MM-DD.jsonl.1` before opening a
    /// fresh one.
    pub fn new(log_dir: PathBuf, max_file_size_bytes: u64) -> Result<Self> {
        fs::create_dir_all(&log_dir)
            .with_context(|| format!("creating audit log dir {}", log_dir.display()))?;

        let current_date = today_str();
        let path = log_path(&log_dir, &current_date);
        let file = open_append(&path)?;

        Ok(Self {
            log_dir,
            max_file_size_bytes,
            current_date,
            file,
        })
    }

    /// Append `event` as a single JSONL line.
    ///
    /// Rotates the active log file if it would exceed `max_file_size_bytes`.
    /// Also handles day roll-over automatically.
    pub fn log(&mut self, event: AuditEvent) -> Result<()> {
        // Day roll-over
        let today = today_str();
        if today != self.current_date {
            let path = log_path(&self.log_dir, &today);
            self.file = open_append(&path)?;
            self.current_date = today;
        }

        // Size-based rotation
        let current_path = log_path(&self.log_dir, &self.current_date);
        let size = current_path.metadata().map(|m| m.len()).unwrap_or(0);
        if size >= self.max_file_size_bytes {
            let rotate = rotate_path(&self.log_dir, &self.current_date);
            fs::rename(&current_path, &rotate)?;
            self.file = open_append(&current_path)?;
        }

        let line = event.to_jsonl()?;
        writeln!(&self.file, "{}", line)?;
        Ok(())
    }

    /// Return a fluent builder pre-configured with this logger's defaults.
    pub fn log_builder(&self) -> AuditEventBuilder {
        AuditEventBuilder::new(AuditEventType::SystemEvent {
            description: String::new(),
        })
    }

    /// Read all events for `date` (format `YYYY-MM-DD`), or today if `None`.
    /// Malformed lines are skipped with a warning printed to stderr.
    pub fn read_events(log_dir: &Path, date: Option<&str>) -> Result<Vec<AuditEvent>> {
        let date_str = date.map(str::to_owned).unwrap_or_else(today_str);
        let path = log_path(log_dir, &date_str);

        if !path.exists() {
            return Ok(vec![]);
        }

        parse_jsonl_file(&path)
    }

    /// Return the last `n` events across *all* `.jsonl` log files in `log_dir`,
    /// sorted oldest-first.
    pub fn tail_events(log_dir: &Path, n: usize) -> Result<Vec<AuditEvent>> {
        if n == 0 {
            return Ok(vec![]);
        }

        let mut files: Vec<PathBuf> = fs::read_dir(log_dir)
            .with_context(|| format!("reading log dir {}", log_dir.display()))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension().map(|e| e == "jsonl").unwrap_or(false)
            })
            .collect();

        // Sort by file name ascending (date order); process newest files last so we
        // can keep a trailing window.
        files.sort();

        // Collect all events then return the last n.
        let mut all: Vec<AuditEvent> = Vec::new();
        for path in &files {
            let events = parse_jsonl_file(path)?;
            all.extend(events);
        }

        let start = all.len().saturating_sub(n);
        Ok(all.into_iter().skip(start).collect())
    }

    /// Compute a BLAKE3 chain hash over the serialised events in `events`.
    ///
    /// The algorithm is compatible with `audit_affidavit`'s chain: each event is
    /// hashed as canonical (sorted-key) JSON and folded into a running hash using
    /// `blake3::hash(prev_hex || event_bytes)`.
    ///
    /// Returns the final chain hash as a lowercase hex string.
    pub fn seal_into_receipt(events: &[AuditEvent]) -> Result<String> {
        const GENESIS_SEED: &[u8] = b"affidavit-v26.6.14-genesis";
        let mut running = blake3::hash(GENESIS_SEED).to_hex().to_string();

        for event in events {
            let value = serde_json::to_value(event)?;
            let canonical = serde_json::to_vec(&sort_value(value))?;

            let mut fold_input = Vec::with_capacity(running.len() + canonical.len());
            fold_input.extend_from_slice(running.as_bytes());
            fold_input.extend_from_slice(&canonical);

            running = blake3::hash(&fold_input).to_hex().to_string();
        }

        Ok(running)
    }

    /// Delete log files in `log_dir` older than `policy.max_age_days` days.
    ///
    /// Only files matching `audit-YYYY-MM-DD.jsonl` (and their `.1` rotated
    /// variants) are considered.  Returns the number of files deleted.
    pub fn purge_old_logs(&self, policy: &RetentionPolicy) -> Result<u64> {
        let cutoff = Utc::now()
            .date_naive()
            .checked_sub_days(chrono::Days::new(policy.max_age_days))
            .unwrap_or_else(|| Utc::now().date_naive());

        let mut deleted = 0u64;

        let entries = fs::read_dir(&self.log_dir)
            .with_context(|| format!("reading log dir {}", self.log_dir.display()))?;

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let fname = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // Matches audit-YYYY-MM-DD.jsonl or audit-YYYY-MM-DD.jsonl.1
            if let Some(date_str) = extract_date_from_filename(&fname) {
                if let Ok(file_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    if file_date < cutoff {
                        fs::remove_file(&path).with_context(|| {
                            format!("deleting old log {}", path.display())
                        })?;
                        deleted += 1;
                    }
                }
            }
        }

        Ok(deleted)
    }

    /// Build a summary over a slice of events for monitoring dashboards.
    pub fn summarize(events: &[AuditEvent]) -> AuditSummary {
        let total_events = events.len();
        let mut auth_failures = 0usize;
        let mut security_violations = 0usize;
        let mut policy_violations = 0usize;
        let mut min_ts: Option<DateTime<Utc>> = None;
        let mut max_ts: Option<DateTime<Utc>> = None;

        for ev in events {
            // Timestamp range
            let ts = ev.timestamp;
            min_ts = Some(min_ts.map_or(ts, |m: DateTime<Utc>| m.min(ts)));
            max_ts = Some(max_ts.map_or(ts, |m: DateTime<Utc>| m.max(ts)));

            match &ev.event_type {
                AuditEventType::AuthenticationFailure => auth_failures += 1,
                AuditEventType::SecurityViolation => security_violations += 1,
                AuditEventType::PolicyViolation { .. } => policy_violations += 1,
                _ => {}
            }
        }

        let date_range = min_ts.zip(max_ts);

        AuditSummary {
            total_events,
            auth_failures,
            security_violations,
            policy_violations,
            date_range,
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn open_append(path: &Path) -> Result<File> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("opening audit log {}", path.display()))
}

fn parse_jsonl_file(path: &Path) -> Result<Vec<AuditEvent>> {
    let file =
        File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();

    for (line_no, line_result) in reader.lines().enumerate() {
        let line: String = line_result.with_context(|| {
            format!("reading line {} of {}", line_no + 1, path.display())
        })?;
        let line = line.trim().to_owned();
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str::<AuditEvent>(&line) {
            Ok(ev) => events.push(ev),
            Err(e) => {
                // SOC2 graceful degradation: skip malformed lines, warn on stderr.
                eprintln!(
                    "[audit_log] WARNING: skipping malformed line {} in {}: {}",
                    line_no + 1,
                    path.display(),
                    e
                );
            }
        }
    }

    Ok(events)
}

/// Recursively sort all JSON object keys so serialisation is canonical.
fn sort_value(v: serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(map) => {
            let sorted: serde_json::Map<_, _> = map
                .into_iter()
                .map(|(k, v)| (k, sort_value(v)))
                .collect::<BTreeMap<_, _>>()
                .into_iter()
                .collect();
            serde_json::Value::Object(sorted)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(sort_value).collect())
        }
        other => other,
    }
}

/// Extract the `YYYY-MM-DD` part from filenames like `audit-2026-06-19.jsonl`
/// or `audit-2026-06-19.jsonl.1`.
fn extract_date_from_filename(fname: &str) -> Option<&str> {
    // Expected prefix: "audit-"
    let rest = fname.strip_prefix("audit-")?;
    // The date is exactly 10 chars: YYYY-MM-DD
    if rest.len() >= 10 {
        Some(&rest[..10])
    } else {
        None
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use tempfile::tempdir;

    // ── helpers ────────────────────────────────────────────────────────────────

    fn fixed_ts() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 19, 12, 0, 0).unwrap()
    }

    fn simple_event(etype: AuditEventType) -> AuditEvent {
        AuditEventBuilder::new(etype)
            .actor(Actor::system())
            .resource(AuditResource::new("test", "res-1"))
            .outcome(Outcome::Success)
            .timestamp(fixed_ts())
            .build()
    }

    fn make_logger(dir: &Path) -> AuditLogger {
        AuditLogger::new(dir.to_path_buf(), 1024 * 1024).unwrap()
    }

    // ── 1. Event serialises to valid JSON ──────────────────────────────────────

    #[test]
    fn event_serializes_to_valid_json() {
        let ev = simple_event(AuditEventType::BuildStarted);
        let json = ev.to_jsonl().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["version"], "v1");
        assert!(parsed["id"].as_str().unwrap().starts_with("evt-"));
    }

    // ── 2. JSONL has no embedded newlines ─────────────────────────────────────

    #[test]
    fn jsonl_is_single_line() {
        let ev = simple_event(AuditEventType::AuthenticationSuccess);
        let line = ev.to_jsonl().unwrap();
        assert_eq!(line.lines().count(), 1, "JSONL must be exactly one line");
    }

    // ── 3. Event type tag is present ──────────────────────────────────────────

    #[test]
    fn event_type_tag_serialized() {
        let ev = simple_event(AuditEventType::AuthenticationFailure);
        let json = ev.to_jsonl().unwrap();
        assert!(json.contains("\"type\":\"AuthenticationFailure\""));
    }

    // ── 4. SystemEvent description is preserved ───────────────────────────────

    #[test]
    fn system_event_description_round_trips() {
        let ev = simple_event(AuditEventType::SystemEvent {
            description: "disk full on /var".into(),
        });
        let json = ev.to_jsonl().unwrap();
        let back: AuditEvent = serde_json::from_str(&json).unwrap();
        if let AuditEventType::SystemEvent { description } = &back.event_type {
            assert_eq!(description, "disk full on /var");
        } else {
            panic!("wrong variant");
        }
    }

    // ── 5. PolicyViolation round-trip ─────────────────────────────────────────

    #[test]
    fn policy_violation_round_trips() {
        let ev = simple_event(AuditEventType::PolicyViolation {
            policy: "no-html5-apex".into(),
        });
        let json = ev.to_jsonl().unwrap();
        let back: AuditEvent = serde_json::from_str(&json).unwrap();
        if let AuditEventType::PolicyViolation { policy } = &back.event_type {
            assert_eq!(policy, "no-html5-apex");
        } else {
            panic!("wrong variant");
        }
    }

    // ── 6. read_events round-trip ─────────────────────────────────────────────

    #[test]
    fn read_events_round_trips() {
        let dir = tempdir().unwrap();
        let mut logger = make_logger(dir.path());

        let ev1 = simple_event(AuditEventType::BuildStarted);
        let ev2 = simple_event(AuditEventType::BuildCompleted { success: true });
        logger.log(ev1.clone()).unwrap();
        logger.log(ev2.clone()).unwrap();

        let events = AuditLogger::read_events(dir.path(), None).unwrap();
        assert_eq!(events.len(), 2);
        // Timestamps are the same in our fixtures
        assert_eq!(events[0].timestamp, fixed_ts());
    }

    // ── 7. read_events returns empty for missing date ─────────────────────────

    #[test]
    fn read_events_empty_for_missing_date() {
        let dir = tempdir().unwrap();
        let events = AuditLogger::read_events(dir.path(), Some("1990-01-01")).unwrap();
        assert!(events.is_empty());
    }

    // ── 8. read_events skips malformed lines ──────────────────────────────────

    #[test]
    fn read_events_skips_malformed_lines() {
        let dir = tempdir().unwrap();
        let date = today_str();
        let path = log_path(dir.path(), &date);

        // Write one good and one bad line
        let good = simple_event(AuditEventType::ResourceAccessed);
        let good_line = good.to_jsonl().unwrap();
        fs::write(&path, format!("{good_line}\nnot-json-at-all\n")).unwrap();

        let events = AuditLogger::read_events(dir.path(), None).unwrap();
        assert_eq!(events.len(), 1, "malformed line must be skipped");
    }

    // ── 9. Log rotation triggers at size limit ────────────────────────────────

    #[test]
    fn rotation_triggers_at_size_limit() {
        let dir = tempdir().unwrap();
        // Set a tiny max_file_size_bytes so the first event causes rotation on the
        // second write.
        let mut logger = AuditLogger::new(dir.path().to_path_buf(), 1).unwrap();

        let ev1 = simple_event(AuditEventType::BuildStarted);
        logger.log(ev1).unwrap();

        let ev2 = simple_event(AuditEventType::BuildCompleted { success: false });
        logger.log(ev2).unwrap();

        let date = &logger.current_date.clone();
        let rotated = rotate_path(dir.path(), date);
        assert!(rotated.exists(), "rotated file must exist after size overflow");
    }

    // ── 10. tail_events returns last N ────────────────────────────────────────

    #[test]
    fn tail_events_returns_last_n() {
        let dir = tempdir().unwrap();
        let mut logger = make_logger(dir.path());

        for i in 0..10u32 {
            let ev = simple_event(AuditEventType::ComplianceScanCompleted {
                passed: true,
                findings: i,
            });
            logger.log(ev).unwrap();
        }

        let tail = AuditLogger::tail_events(dir.path(), 3).unwrap();
        assert_eq!(tail.len(), 3);
        // Last event should be the one with findings == 9
        if let AuditEventType::ComplianceScanCompleted { findings, .. } = tail[2].event_type {
            assert_eq!(findings, 9);
        } else {
            panic!("wrong event type");
        }
    }

    // ── 11. tail_events with n > total returns all ────────────────────────────

    #[test]
    fn tail_events_returns_all_when_n_exceeds_count() {
        let dir = tempdir().unwrap();
        let mut logger = make_logger(dir.path());
        logger.log(simple_event(AuditEventType::AuthenticationSuccess)).unwrap();
        logger.log(simple_event(AuditEventType::AuthorizationGrant)).unwrap();

        let tail = AuditLogger::tail_events(dir.path(), 100).unwrap();
        assert_eq!(tail.len(), 2);
    }

    // ── 12. tail_events with n == 0 returns empty ─────────────────────────────

    #[test]
    fn tail_events_zero_returns_empty() {
        let dir = tempdir().unwrap();
        let mut logger = make_logger(dir.path());
        logger.log(simple_event(AuditEventType::SystemEvent {
            description: "test".into(),
        }))
        .unwrap();

        let tail = AuditLogger::tail_events(dir.path(), 0).unwrap();
        assert!(tail.is_empty());
    }

    // ── 13. seal_into_receipt is deterministic ────────────────────────────────

    #[test]
    fn seal_hash_is_deterministic() {
        let events: Vec<AuditEvent> = (0..5)
            .map(|_| simple_event(AuditEventType::ResourceCreated))
            .collect();

        let h1 = AuditLogger::seal_into_receipt(&events).unwrap();
        let h2 = AuditLogger::seal_into_receipt(&events).unwrap();
        assert_eq!(h1, h2, "same input must produce same hash");
    }

    // ── 14. seal_into_receipt changes when events change ─────────────────────

    #[test]
    fn seal_hash_changes_with_different_events() {
        let events1 = vec![simple_event(AuditEventType::BuildStarted)];
        let events2 = vec![simple_event(AuditEventType::BuildCompleted { success: true })];

        let h1 = AuditLogger::seal_into_receipt(&events1).unwrap();
        let h2 = AuditLogger::seal_into_receipt(&events2).unwrap();
        assert_ne!(h1, h2, "different events must produce different hashes");
    }

    // ── 15. seal_into_receipt on empty returns genesis hash ───────────────────

    #[test]
    fn seal_empty_equals_genesis() {
        const GENESIS_SEED: &[u8] = b"affidavit-v26.6.14-genesis";
        let genesis = blake3::hash(GENESIS_SEED).to_hex().to_string();
        let hash = AuditLogger::seal_into_receipt(&[]).unwrap();
        assert_eq!(hash, genesis);
    }

    // ── 16. seal hash is a valid blake3 hex string ────────────────────────────

    #[test]
    fn seal_hash_is_valid_hex() {
        let events = vec![simple_event(AuditEventType::SecretsAccessed)];
        let hash = AuditLogger::seal_into_receipt(&events).unwrap();
        assert_eq!(hash.len(), 64, "blake3 hex is 64 chars");
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // ── 17. builder produces valid event ─────────────────────────────────────

    #[test]
    fn builder_produces_valid_event() {
        let ev = AuditEventBuilder::new(AuditEventType::DeploymentCompleted { success: true })
            .actor(Actor::human("alice@example.com", "engineer"))
            .resource(AuditResource::new("service", "pwa-staff").with_name("PWA Staff"))
            .outcome(Outcome::Success)
            .meta("env", "production")
            .meta("region", "us-east-1")
            .build();

        assert_eq!(ev.version, "v1");
        assert_eq!(ev.metadata["env"], "production");
        assert_eq!(ev.metadata["region"], "us-east-1");
        if let ActorKind::Human { role } = &ev.actor.kind {
            assert_eq!(role, "engineer");
        } else {
            panic!("wrong actor kind");
        }
    }

    // ── 18. builder defaults to System actor ─────────────────────────────────

    #[test]
    fn builder_defaults_to_system_actor() {
        let ev = AuditEventBuilder::new(AuditEventType::AuditLogAccessed).build();
        assert!(matches!(ev.actor.kind, ActorKind::System));
    }

    // ── 19. retention purge deletes old files ────────────────────────────────

    #[test]
    fn retention_purge_deletes_old_files() {
        let dir = tempdir().unwrap();

        // Write a "fake old" log file (dated well in the past)
        let old_path = dir.path().join("audit-2020-01-01.jsonl");
        fs::write(&old_path, b"").unwrap();

        // Write a recent file (today)
        let today_path = log_path(dir.path(), &today_str());
        fs::write(&today_path, b"").unwrap();

        let logger = make_logger(dir.path());
        let policy = RetentionPolicy { max_age_days: 365 };
        let deleted = logger.purge_old_logs(&policy).unwrap();

        assert_eq!(deleted, 1, "exactly one old file should be deleted");
        assert!(!old_path.exists(), "old file must be deleted");
        assert!(today_path.exists(), "recent file must be kept");
    }

    // ── 20. retention purge respects rotated variants (.jsonl.1) ──────────────

    #[test]
    fn retention_purge_deletes_rotated_old_files() {
        let dir = tempdir().unwrap();

        let old_rotated = dir.path().join("audit-2019-03-15.jsonl.1");
        fs::write(&old_rotated, b"").unwrap();

        let logger = make_logger(dir.path());
        let policy = RetentionPolicy::default();
        let deleted = logger.purge_old_logs(&policy).unwrap();

        assert_eq!(deleted, 1);
        assert!(!old_rotated.exists());
    }

    // ── 21. summary counts are correct ───────────────────────────────────────

    #[test]
    fn summary_counts_are_correct() {
        let events = vec![
            simple_event(AuditEventType::AuthenticationFailure),
            simple_event(AuditEventType::AuthenticationFailure),
            simple_event(AuditEventType::SecurityViolation),
            simple_event(AuditEventType::PolicyViolation {
                policy: "test-policy".into(),
            }),
            simple_event(AuditEventType::BuildStarted),
            simple_event(AuditEventType::AuthenticationSuccess),
        ];

        let summary = AuditLogger::summarize(&events);
        assert_eq!(summary.total_events, 6);
        assert_eq!(summary.auth_failures, 2);
        assert_eq!(summary.security_violations, 1);
        assert_eq!(summary.policy_violations, 1);
    }

    // ── 22. summary on empty slice ────────────────────────────────────────────

    #[test]
    fn summary_on_empty_slice() {
        let summary = AuditLogger::summarize(&[]);
        assert_eq!(summary.total_events, 0);
        assert_eq!(summary.auth_failures, 0);
        assert_eq!(summary.date_range, None);
    }

    // ── 23. summary date_range tracks min/max timestamps ─────────────────────

    #[test]
    fn summary_date_range_tracks_min_max() {
        let ts1 = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let ts2 = Utc.with_ymd_and_hms(2026, 6, 19, 0, 0, 0).unwrap();

        let ev1 = AuditEventBuilder::new(AuditEventType::BuildStarted)
            .actor(Actor::system())
            .resource(AuditResource::new("t", "1"))
            .outcome(Outcome::Success)
            .timestamp(ts1)
            .build();

        let ev2 = AuditEventBuilder::new(AuditEventType::BuildCompleted { success: true })
            .actor(Actor::system())
            .resource(AuditResource::new("t", "2"))
            .outcome(Outcome::Success)
            .timestamp(ts2)
            .build();

        let summary = AuditLogger::summarize(&[ev1, ev2]);
        let (min, max) = summary.date_range.unwrap();
        assert_eq!(min, ts1);
        assert_eq!(max, ts2);
    }

    // ── 24. AuditResource with_name / with_path ───────────────────────────────

    #[test]
    fn audit_resource_builder_methods() {
        let r = AuditResource::new("file", "42")
            .with_name("CLAUDE.md")
            .with_path("/home/user/CLAUDE.md");
        assert_eq!(r.name.as_deref(), Some("CLAUDE.md"));
        assert_eq!(r.path, Some(PathBuf::from("/home/user/CLAUDE.md")));
    }

    // ── 25. Actor convenience constructors ───────────────────────────────────

    #[test]
    fn actor_convenience_constructors() {
        let h = Actor::human("bob", "admin");
        assert_eq!(h.id, "bob");

        let s = Actor::system();
        assert_eq!(s.id, "system");

        let ci = Actor::ci("run-42");
        assert_eq!(ci.id, "ci:run-42");
        assert!(matches!(ci.kind, ActorKind::CiPipeline { .. }));
    }

    // ── 26. Outcome::Failure serialisation ───────────────────────────────────

    #[test]
    fn outcome_failure_round_trips() {
        let ev = AuditEventBuilder::new(AuditEventType::AuthenticationFailure)
            .actor(Actor::human("eve", "attacker"))
            .resource(AuditResource::new("auth", "login"))
            .outcome(Outcome::Failure {
                code: "INVALID_PASSWORD".into(),
                message: "bad credentials".into(),
            })
            .build();

        let json = ev.to_jsonl().unwrap();
        let back: AuditEvent = serde_json::from_str(&json).unwrap();
        if let Outcome::Failure { code, message } = &back.outcome {
            assert_eq!(code, "INVALID_PASSWORD");
            assert_eq!(message, "bad credentials");
        } else {
            panic!("wrong outcome variant");
        }
    }

    // ── 27. extract_date_from_filename helper ────────────────────────────────

    #[test]
    fn extract_date_helper() {
        assert_eq!(
            extract_date_from_filename("audit-2026-06-19.jsonl"),
            Some("2026-06-19")
        );
        assert_eq!(
            extract_date_from_filename("audit-2020-01-01.jsonl.1"),
            Some("2020-01-01")
        );
        assert_eq!(extract_date_from_filename("something-else.log"), None);
        assert_eq!(extract_date_from_filename("audit-.jsonl"), None);
    }

    // ── 28. metadata BTreeMap is sorted in serialisation ──────────────────────

    #[test]
    fn metadata_is_btreemap_ordered() {
        let ev = AuditEventBuilder::new(AuditEventType::ConfigurationChanged)
            .actor(Actor::system())
            .resource(AuditResource::new("config", "ue4.ini"))
            .outcome(Outcome::Success)
            .meta("z_key", "last")
            .meta("a_key", "first")
            .meta("m_key", "middle")
            .build();

        let json = ev.to_jsonl().unwrap();
        // The metadata object keys in JSON should appear in sorted order because
        // BTreeMap serialises in sorted key order.
        let a_pos = json.find("a_key").unwrap();
        let m_pos = json.find("m_key").unwrap();
        let z_pos = json.find("z_key").unwrap();
        assert!(a_pos < m_pos && m_pos < z_pos);
    }
}
