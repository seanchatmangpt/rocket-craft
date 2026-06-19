use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Player {
    pub id: i64,
    pub name: String,
    pub score: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LeaderboardEntry {
    pub id: i64,
    pub player_id: i64,
    pub score: i64,
    pub rank: i64,
}

/// A cook receipt pushed from the Rust CLI to Supabase `game_receipts`.
///
/// Mirrors the shape written by `Html5PackageReport::write_receipt()` and
/// accepted by the Nuxt server route `POST /api/game/receipt`.
#[derive(Serialize, Debug)]
pub struct CookReceipt {
    /// `null` — CLI-initiated sessions have no browser player_id.
    pub session_id: Option<String>,
    /// "PASS" or "FAIL"
    pub verdict: String,
    /// Human label: "HTML5CookVerify", "WasmMagicCheck", etc.
    pub milestone: String,
    /// Ordered list of OCEL activities witnessed during the cook.
    pub ocel_lifecycle: Vec<String>,
    /// Total number of OCEL events emitted by the cook pipeline.
    pub ocel_event_count: u32,
    /// "rocket_cli" — distinguishes Rust-sourced receipts from browser receipts.
    pub engine_source: String,
    /// SHA-256 hex of the serialised receipt payload.
    pub receipt_hash: String,
    /// SHA-256 hex of the WASM artifact bytes (cook-to-game cross-check, Gap 6).
    /// None when no WASM file is present (dry-run or cook failure).
    pub output_hash: Option<String>,
    /// RFC 3339 timestamp.
    pub proven_at: String,
    /// Free-form metadata (wasm_mb, archive_dir, companion files, …).
    pub payload: HashMap<String, serde_json::Value>,
}

/// An OCEL event batch item, for bulk-ingesting cook-pipeline events into Supabase.
#[derive(Serialize, Debug)]
pub struct OcelEventRow {
    pub session_id: Option<String>,
    pub activity: String,
    pub timestamp_ms: u64,
    pub object_refs: Vec<String>,
    pub attributes: serde_json::Value,
    pub prev_hash: Option<String>,
    pub event_hash: String,
    pub seq: u32,
}

pub struct SupabaseService {
    client: Client,
    url: String,
    anon_key: String,
}

impl SupabaseService {
    pub fn new(url: String, anon_key: String) -> Self {
        Self {
            client: Client::new(),
            url,
            anon_key,
        }
    }

    fn rest_headers(&self) -> reqwest::header::HeaderMap {
        let mut h = reqwest::header::HeaderMap::new();
        h.insert("apikey", self.anon_key.parse().unwrap());
        h.insert(
            "Authorization",
            format!("Bearer {}", self.anon_key).parse().unwrap(),
        );
        h.insert("Content-Type", "application/json".parse().unwrap());
        h.insert("Prefer", "return=minimal".parse().unwrap());
        h
    }

    pub async fn get_players(&self) -> Result<Vec<Player>> {
        let response = self
            .client
            .get(format!("{}/rest/v1/players?select=*", self.url))
            .headers(self.rest_headers())
            .send()
            .await?;
        response.error_for_status_ref()?;
        Ok(response.json::<Vec<Player>>().await?)
    }

    pub async fn get_leaderboard(&self) -> Result<Vec<LeaderboardEntry>> {
        let response = self
            .client
            .get(format!("{}/rest/v1/leaderboard?select=*", self.url))
            .headers(self.rest_headers())
            .send()
            .await?;
        response.error_for_status_ref()?;
        Ok(response.json::<Vec<LeaderboardEntry>>().await?)
    }

    /// Push a cook receipt to `game_receipts`.
    ///
    /// The leaderboard Postgres trigger fires automatically on PASS verdicts,
    /// so this single call closes the cook→leaderboard pipeline.
    pub async fn push_cook_receipt(&self, receipt: &CookReceipt) -> Result<()> {
        // Sign the receipt with the Ed25519 private key if ROCKET_SIGNING_KEY is set.
        // The signature covers (proven_at, receipt_hash, session_id, verdict) in sorted-key
        // canonical JSON — same payload the browser verify route checks.
        let proven_at = chrono::Utc::now().to_rfc3339();
        let ed25519_sig = std::env::var("ROCKET_SIGNING_KEY").ok().and_then(|key| {
            let payload = crate::signing::receipt_signing_payload(
                receipt.session_id.as_deref(),
                &receipt.verdict,
                &receipt.receipt_hash,
                &proven_at,
            );
            crate::signing::sign(&key, &payload).ok()
        });

        let mut value = serde_json::to_value(receipt).context("serialise CookReceipt")?;
        value["proven_at"] = serde_json::Value::String(proven_at);
        if let Some(sig) = ed25519_sig {
            value["ed25519_sig"] = serde_json::Value::String(sig);
        }

        let body = serde_json::to_string(&value).context("serialise signed receipt")?;
        let resp = self
            .client
            .post(format!("{}/rest/v1/game_receipts", self.url))
            .headers(self.rest_headers())
            .body(body)
            .send()
            .await
            .context("POST game_receipts failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("game_receipts insert {status}: {body}");
        }
        Ok(())
    }

    /// Open a `game_sessions` row for a CLI cook run.
    ///
    /// Creates an `is_alive=true` row with `engine_source='rocket_cli'` and returns
    /// the UUID string. Pass this to `Html5PackageVerifier::with_cook_session_id()`
    /// so all OCEL events emitted by the cook pipeline are linked to this session
    /// and can be verified by `verify_event_chain(session_id)`.
    ///
    /// The caller is responsible for closing the session by calling
    /// `close_cook_session(session_id, verdict)` after the cook completes.
    pub async fn create_cook_session(&self, project_name: &str, archive_dir: &str) -> Result<String> {
        let body = serde_json::json!({
            "is_alive": true,
            "engine_source": "rocket_cli",
            "metadata": {
                "project_name": project_name,
                "archive_dir": archive_dir,
                "opened_by": "rocket html5 cook"
            }
        });
        let mut headers = self.rest_headers();
        // PostgREST: return the inserted row so we can extract the generated UUID.
        headers.insert("Prefer", "return=representation".parse().unwrap());
        let resp = self.client
            .post(format!("{}/rest/v1/game_sessions", self.url))
            .headers(headers)
            .json(&body)
            .send()
            .await
            .context("POST game_sessions (cook session) failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let msg = resp.text().await.unwrap_or_default();
            anyhow::bail!("create_cook_session {status}: {msg}");
        }
        let rows: Vec<serde_json::Value> = resp.json().await?;
        rows.into_iter()
            .next()
            .and_then(|r| r["id"].as_str().map(str::to_owned))
            .ok_or_else(|| anyhow::anyhow!("create_cook_session: no id returned"))
    }

    /// Close a cook session by setting `is_alive=false` and recording the final verdict.
    ///
    /// Call after `push_cook_receipt` completes. Sets `session_ended_at` and tags
    /// the receipt hash in the metadata for cross-reference.
    pub async fn close_cook_session(
        &self,
        session_id: &str,
        verdict: &str,
        receipt_hash: Option<&str>,
    ) -> Result<()> {
        let body = serde_json::json!({
            "is_alive": false,
            "session_ended_at": chrono::Utc::now().to_rfc3339(),
            "receipt_hash": receipt_hash,
            "metadata": serde_json::json!({
                "verdict": verdict,
                "receipt_hash": receipt_hash,
                "closed_by": "rocket html5 cook"
            })
        });
        let resp = self.client
            .patch(format!("{}/rest/v1/game_sessions?id=eq.{session_id}", self.url))
            .headers(self.rest_headers())
            .json(&body)
            .send()
            .await
            .context("PATCH game_sessions (close cook session) failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let msg = resp.text().await.unwrap_or_default();
            anyhow::bail!("close_cook_session {status}: {msg}");
        }
        Ok(())
    }

    /// Fetch the most recent cook receipt from `game_receipts` for a given project.
    ///
    /// Used by `rocket html5 diff` to compare the last cook against a new verify run.
    pub async fn last_cook_receipt(&self, limit: u32) -> Result<Vec<serde_json::Value>> {
        let url = format!(
            "{}/rest/v1/game_receipts\
             ?select=id,verdict,milestone,engine_source,ocel_event_count,proven_at,receipt_hash,payload\
             &engine_source=eq.rocket_cli\
             &order=proven_at.desc\
             &limit={limit}",
            self.url
        );
        let resp = self.client.get(&url).headers(self.rest_headers()).send().await?;
        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            Ok(vec![])
        }
    }

    /// Ping the Supabase REST API and verify each pipeline table is accessible.
    ///
    /// Returns `(table_name, is_ok, http_status_or_error)` for each table.
    pub async fn ping(&self) -> (Vec<(String, bool, String)>, u16) {
        let tables = ["game_receipts", "ocel_events", "game_sessions", "leaderboard", "players"];
        let mut checks = Vec::new();

        let health = self.client
            .get(format!("{}/rest/v1/", self.url))
            .header("apikey", &self.anon_key)
            .send()
            .await
            .map(|r| r.status().as_u16())
            .unwrap_or(0);

        for table in &tables {
            let resp = self.client
                .get(format!("{}/rest/v1/{table}?limit=0", self.url))
                .headers(self.rest_headers())
                .send()
                .await;
            let (ok, msg) = match resp {
                Ok(r) => {
                    let s = r.status().as_u16();
                    (s == 200 || s == 406, format!("HTTP {s}"))
                }
                Err(e) => (false, format!("error: {e}")),
            };
            checks.push((table.to_string(), ok, msg));
        }
        (checks, health)
    }

    /// Call close_stale_sessions(timeout_minutes) Postgres function.
    pub async fn close_stale_sessions(&self, timeout_minutes: u32) -> Result<Vec<serde_json::Value>> {
        let url = format!("{}/rest/v1/rpc/close_stale_sessions", self.url);
        let body = serde_json::json!({ "p_timeout_minutes": timeout_minutes });
        let resp = self.client
            .post(&url)
            .headers(self.rest_headers())
            .json(&body)
            .send()
            .await
            .context("close_stale_sessions RPC failed")?;
        if resp.status().is_success() {
            Ok(resp.json::<Vec<serde_json::Value>>().await?)
        } else {
            let status = resp.status();
            let msg = resp.text().await.unwrap_or_default();
            anyhow::bail!("close_stale_sessions {status}: {msg}");
        }
    }

    /// Fetch session_lifecycle_summary view (distinct activities per session).
    /// Used by `rocket supabase conformance` to compute Van der Aalst metrics.
    pub async fn fetch_session_lifecycle_summary(&self, limit: u32) -> Result<Vec<serde_json::Value>> {
        let url = format!(
            "{}/rest/v1/session_lifecycle_summary\
             ?select=session_id,event_count,distinct_activities,duration_ms,latest_verdict\
             &order=session_id.desc\
             &limit={limit}",
            self.url
        );
        let resp = self.client
            .get(&url)
            .headers(self.rest_headers())
            .send()
            .await
            .context("GET session_lifecycle_summary failed")?;
        if resp.status().is_success() {
            Ok(resp.json::<Vec<serde_json::Value>>().await?)
        } else {
            let status = resp.status();
            let msg = resp.text().await.unwrap_or_default();
            anyhow::bail!("session_lifecycle_summary fetch {status}: {msg}");
        }
    }

    /// Fetch all ocel_events for a session ordered by seq ASC.
    pub async fn fetch_ocel_events(&self, session_id: &str) -> Result<Vec<serde_json::Value>> {
        let url = format!(
            "{}/rest/v1/ocel_events\
             ?select=id,session_id,activity,timestamp_ms,object_refs,attributes,prev_hash,event_hash,seq\
             &session_id=eq.{session_id}\
             &order=seq.asc",
            self.url
        );
        let resp = self.client
            .get(&url)
            .headers(self.rest_headers())
            .send()
            .await
            .context("GET ocel_events failed")?;
        if resp.status().is_success() {
            Ok(resp.json::<Vec<serde_json::Value>>().await?)
        } else {
            let status = resp.status();
            let msg = resp.text().await.unwrap_or_default();
            anyhow::bail!("ocel_events fetch {status}: {msg}");
        }
    }

    /// Call the `verify_event_chain` Postgres RPC.
    ///
    /// Returns an array of `{ ok, message, broken_at, session_id }` rows.
    /// When `session_id` is `None`, all sessions are checked.
    pub async fn verify_event_chain(&self, session_id: Option<&str>) -> Result<Vec<serde_json::Value>> {
        let url = format!("{}/rest/v1/rpc/verify_event_chain", self.url);
        let body = match session_id {
            Some(sid) => format!(r#"{{"p_session_id": "{sid}"}}"#),
            None => "{}".to_string(),
        };
        let resp = self.client
            .post(&url)
            .headers(self.rest_headers())
            .body(body)
            .send()
            .await
            .context("POST verify_event_chain failed")?;
        if resp.status().is_success() {
            Ok(resp.json::<Vec<serde_json::Value>>().await?)
        } else {
            let status = resp.status();
            let msg = resp.text().await.unwrap_or_default();
            anyhow::bail!("verify_event_chain {status}: {msg}");
        }
    }

    /// Fetch the 10 most recent game receipts ordered by proven_at DESC.
    pub async fn recent_receipts(&self) -> Result<Vec<serde_json::Value>> {
        let resp = self.client
            .get(format!(
                "{}/rest/v1/game_receipts\
                 ?select=id,verdict,milestone,engine_source,ocel_event_count,proven_at\
                 &order=proven_at.desc&limit=10",
                self.url
            ))
            .headers(self.rest_headers())
            .send()
            .await
            .context("GET game_receipts failed")?;
        if resp.status().is_success() {
            Ok(resp.json::<Vec<serde_json::Value>>().await?)
        } else {
            Ok(vec![])
        }
    }

    /// Bulk-insert OCEL events from a cook pipeline run into `ocel_events`.
    ///
    /// Uses PostgREST bulk insert (JSON array body).  Non-fatal on partial
    /// failure — caller should log but not abort the cook pipeline.
    pub async fn push_ocel_events(&self, events: &[OcelEventRow]) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }
        let body = serde_json::to_string(events)
            .context("failed to serialise OCEL events")?;
        let mut headers = self.rest_headers();
        // Overwrite Prefer so PostgREST performs a bulk insert without returning rows.
        headers.insert("Prefer", "return=minimal,resolution=merge-duplicates".parse().unwrap());
        let resp = self
            .client
            .post(format!("{}/rest/v1/ocel_events", self.url))
            .headers(headers)
            .body(body)
            .send()
            .await
            .context("POST ocel_events failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("ocel_events bulk insert {status}: {body}");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn supabase_service_new_stores_url_and_key() {
        let svc = SupabaseService::new("http://localhost:54321".into(), "my-key".into());
        assert_eq!(svc.url, "http://localhost:54321");
        assert_eq!(svc.anon_key, "my-key");
    }

    #[test]
    fn player_deserializes_from_json() {
        let p: Player = serde_json::from_value(json!({
            "id": 1, "name": "Alice", "score": 9001
        })).unwrap();
        assert_eq!(p.id, 1);
        assert_eq!(p.name, "Alice");
        assert_eq!(p.score, 9001);
    }

    #[test]
    fn player_serializes_to_json() {
        let p = Player { id: 2, name: "Bob".into(), score: 42 };
        let v = serde_json::to_value(&p).unwrap();
        assert_eq!(v["name"], "Bob");
        assert_eq!(v["score"], 42);
    }

    #[test]
    fn leaderboard_entry_roundtrips() {
        let entry = LeaderboardEntry { id: 1, player_id: 7, score: 500, rank: 3 };
        let json = serde_json::to_string(&entry).unwrap();
        let back: LeaderboardEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.player_id, 7);
        assert_eq!(back.rank, 3);
    }

    #[test]
    fn player_debug_format_contains_name() {
        let p = Player { id: 1, name: "Tester".into(), score: 0 };
        assert!(format!("{:?}", p).contains("Tester"));
    }

    #[test]
    fn cook_receipt_serializes_with_all_fields() {
        let r = CookReceipt {
            session_id: None,
            verdict: "PASS".into(),
            milestone: "HTML5CookVerify".into(),
            ocel_lifecycle: vec!["CookStarted".into(), "WasmPackaged".into()],
            ocel_event_count: 2,
            engine_source: "rocket_cli".into(),
            receipt_hash: "abc123".into(),
            output_hash: None,
            proven_at: "2026-06-19T00:00:00Z".into(),
            payload: {
                let mut m = HashMap::new();
                m.insert("wasm_mb".into(), json!(175.4));
                m
            },
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["verdict"], "PASS");
        assert_eq!(v["engine_source"], "rocket_cli");
        assert_eq!(v["ocel_lifecycle"][0], "CookStarted");
    }

    #[test]
    fn ocel_event_row_serializes() {
        let evt = OcelEventRow {
            session_id: None,
            activity: "WasmPackaged".into(),
            timestamp_ms: 1_700_000_000_000,
            object_refs: vec!["brm-wasm".into()],
            attributes: json!({ "size_mb": 175.4 }),
            prev_hash: None,
            event_hash: "deadbeef".into(),
            seq: 1,
        };
        let v = serde_json::to_value(&evt).unwrap();
        assert_eq!(v["activity"], "WasmPackaged");
        assert_eq!(v["seq"], 1);
    }
}
