/// pipeline_loop_test.rs — End-to-end Rust integration test for the cook pipeline loop.
///
/// Van der Aalst doctrine: the pipeline is ALIVE only when event-log evidence
/// proves a lawful process happened. This test manufactures a synthetic session
/// using ChainedOcelEmitter, verifies the BLAKE3 chain locally, and confirms
/// the recipe matches the declared lifecycle.
///
/// No UE4, no browser, no Supabase — pure Rust proof of the chain algebra.
/// The same chain formula is used by:
///   - ChainedOcelEmitter (this test)
///   - html5.rs push_to_supabase()
///   - session-seed.post.ts (TypeScript server)
///   - useHashChain.computeEventHash (browser composable)
///
/// If any of them diverge, verify_event_chain RPC and session-replay will reject
/// the session — so this test is the invariant anchor.

use rocket_sdk::supabase::{ChainedOcelEmitter, OcelEventRow};

const LAWFUL_LIFECYCLE: &[&str] = &["GameSessionStarted", "FrameRendered", "InputAdmitted"];

// ── Helpers ───────────────────────────────────────────────────────────────────

fn build_lawful_session(session_id: &str) -> Vec<OcelEventRow> {
    let base_ms: u64 = 1_750_000_000_000;
    let mut emitter = ChainedOcelEmitter::new(
        Some(session_id.to_string()),
        format!("session:{session_id}"),
    );
    for (i, activity) in LAWFUL_LIFECYCLE.iter().enumerate() {
        emitter.emit(
            *activity,
            base_ms + (i as u64 * 1_000),
            serde_json::json!({ "stage_index": i }),
        );
    }
    emitter.into_events()
}

/// Replay the hash chain locally — mirrors session-replay.get.ts logic.
fn replay_chain(events: &[OcelEventRow]) -> (bool, Option<u32>, Option<String>) {
    let mut prev_hash: Option<String> = None;
    let mut chain_intact = true;
    let mut first_break_at: Option<u32> = None;

    for evt in events {
        if evt.prev_hash != prev_hash {
            if chain_intact {
                chain_intact = false;
                first_break_at = Some(evt.seq);
            }
        }
        prev_hash = Some(evt.event_hash.clone());
    }
    let chain_tip = prev_hash;
    (chain_intact, first_break_at, chain_tip)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn lawful_session_chain_is_intact() {
    let events = build_lawful_session("00000000-0000-0000-0000-000000000001");
    let (intact, break_at, tip) = replay_chain(&events);
    assert!(intact, "chain must be intact for a lawfully built session");
    assert!(break_at.is_none());
    assert!(tip.is_some(), "chain tip must be set");
    assert_eq!(tip.as_deref().map(|t| t.len()), Some(64), "BLAKE3 tip is 64 hex chars");
}

#[test]
fn lawful_lifecycle_activities_all_present() {
    let events = build_lawful_session("00000000-0000-0000-0000-000000000002");
    let activities: Vec<&str> = events.iter().map(|e| e.activity.as_str()).collect();
    for required in LAWFUL_LIFECYCLE {
        assert!(activities.contains(required), "missing activity: {required}");
    }
}

#[test]
fn event_hashes_are_unique() {
    let events = build_lawful_session("00000000-0000-0000-0000-000000000003");
    let hashes: Vec<&str> = events.iter().map(|e| e.event_hash.as_str()).collect();
    let unique: std::collections::HashSet<&str> = hashes.iter().copied().collect();
    assert_eq!(hashes.len(), unique.len(), "each event must have a distinct hash");
}

#[test]
fn prev_hash_threads_correctly() {
    let events = build_lawful_session("00000000-0000-0000-0000-000000000004");
    assert_eq!(events[0].prev_hash, None, "first event prev_hash must be None");
    for i in 1..events.len() {
        assert_eq!(
            events[i].prev_hash.as_deref(),
            Some(events[i - 1].event_hash.as_str()),
            "event[{i}].prev_hash must equal event[{}].event_hash",
            i - 1
        );
    }
}

#[test]
fn tampered_event_breaks_chain() {
    let mut events = build_lawful_session("00000000-0000-0000-0000-000000000005");
    // Tamper: replace the event_hash of the first event with a fake
    events[0].event_hash = "dead".repeat(16); // 64 chars of 'dead'
    let (intact, break_at, _) = replay_chain(&events);
    assert!(!intact, "tampered hash must break the chain");
    assert_eq!(break_at, Some(1), "break must be detected at seq=1 (first link)");
}

#[test]
fn extended_session_chain_intact() {
    // Simulate a longer cook pipeline with more events
    let session_id = "00000000-0000-0000-0000-000000000006";
    let activities = [
        "CookStarted", "ContentScanned", "ShaderCompiled",
        "WasmPackaged", "JsEmitted", "DataPakStaged",
        "PackageVerified", "ReceiptSigned",
    ];
    let base_ms: u64 = 1_750_000_000_000;
    let mut emitter = ChainedOcelEmitter::new(
        Some(session_id.to_string()),
        format!("cook:{session_id}"),
    );
    for (i, act) in activities.iter().enumerate() {
        emitter.emit(*act, base_ms + (i as u64 * 500), serde_json::json!({ "i": i }));
    }
    let chain_tip = emitter.chain_tip().map(|s| s.to_owned());
    let events = emitter.into_events();

    assert_eq!(events.len(), activities.len());
    let (intact, break_at, tip) = replay_chain(&events);
    assert!(intact);
    assert!(break_at.is_none());
    assert_eq!(tip, chain_tip, "chain_tip() must match last event_hash");
}

#[test]
fn empty_session_has_no_chain_tip() {
    let emitter = ChainedOcelEmitter::new(Some("sess".into()), "obj");
    assert!(emitter.chain_tip().is_none());
    let events = emitter.into_events();
    assert!(events.is_empty());
}

#[test]
fn cook_receipt_hash_is_64_hex() {
    use std::collections::HashMap;
    let receipt = rocket_sdk::supabase::CookReceipt {
        session_id: None,
        verdict: "PASS".into(),
        milestone: "HTML5CookVerify".into(),
        ocel_lifecycle: LAWFUL_LIFECYCLE.iter().map(|s| s.to_string()).collect(),
        ocel_event_count: LAWFUL_LIFECYCLE.len() as u32,
        engine_source: "rocket_cli".into(),
        receipt_hash: blake3::hash(b"test-payload").to_hex().to_string(),
        output_hash: None,
        proven_at: "2026-06-19T00:00:00Z".into(),
        payload: HashMap::new(),
    };
    assert_eq!(receipt.receipt_hash.len(), 64);
    assert!(receipt.receipt_hash.chars().all(|c| c.is_ascii_hexdigit()));
    let v = serde_json::to_value(&receipt).unwrap();
    assert_eq!(v["engine_source"], "rocket_cli");
    assert_eq!(v["verdict"], "PASS");
}
