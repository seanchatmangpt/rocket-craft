//! Chicago-TDD-style integration tests for `unify::commands`.
//!
//! These tests exercise the real implementations of `cmd_witnesses` and
//! `cmd_query`; no mocks are used.  Each test sets up a
//! `chicago_tdd_tools::TestEnvironment` (which provides an isolated temp
//! directory) even when it does not need the filesystem, so that the
//! harness pattern is consistent and tests remain composable.

use chicago_tdd_tools::TestEnvironment;
use unify::commands::{cmd_query, cmd_witnesses};

// ============================================================================
// cmd_witnesses tests
// ============================================================================

/// 1. `cmd_witnesses(None)` must return at least 3 witness entries.
#[test]
fn test_witnesses_no_filter_returns_at_least_three() {
    let _env = TestEnvironment::new().expect("env setup must succeed");

    let output = cmd_witnesses(None).expect("cmd_witnesses must not error");
    assert!(output.success, "output must be successful");

    let witnesses = output.data["witnesses"]
        .as_array()
        .expect("data.witnesses must be an array");

    assert!(
        witnesses.len() >= 3,
        "expected at least 3 witnesses, got {}",
        witnesses.len()
    );
}

/// 2. `cmd_witnesses(Some("rdf"))` must return only witnesses whose
///    `domain` field equals `"rdf"`.
#[test]
fn test_witnesses_domain_filter_returns_only_rdf() {
    let _env = TestEnvironment::new().expect("env setup must succeed");

    let output = cmd_witnesses(Some("rdf")).expect("cmd_witnesses(rdf) must not error");
    assert!(output.success, "output must be successful");

    let witnesses = output.data["witnesses"]
        .as_array()
        .expect("data.witnesses must be an array");

    assert!(
        !witnesses.is_empty(),
        "domain 'rdf' filter must return at least one witness"
    );

    for w in witnesses {
        assert_eq!(
            w["domain"].as_str().unwrap_or(""),
            "rdf",
            "all witnesses returned must have domain == 'rdf', got: {}",
            w
        );
    }
}

/// 5. `cmd_witnesses(None)` must include at least one entry with
///    `status == "active"`.
#[test]
fn test_witnesses_at_least_one_active() {
    let _env = TestEnvironment::new().expect("env setup must succeed");

    let output = cmd_witnesses(None).expect("cmd_witnesses must not error");
    assert!(output.success, "output must be successful");

    let witnesses = output.data["witnesses"]
        .as_array()
        .expect("data.witnesses must be an array");

    let active_count = witnesses
        .iter()
        .filter(|w| w["status"].as_str() == Some("active"))
        .count();

    assert!(
        active_count >= 1,
        "expected at least 1 active witness, found 0 among {} witnesses",
        witnesses.len()
    );
}

/// Extra: every witness entry must have `domain`, `witness`, and `status` fields.
#[test]
fn test_witnesses_entries_have_required_fields() {
    let _env = TestEnvironment::new().expect("env setup must succeed");

    let output = cmd_witnesses(None).expect("cmd_witnesses must not error");

    let witnesses = output.data["witnesses"]
        .as_array()
        .expect("data.witnesses must be an array");

    for (i, w) in witnesses.iter().enumerate() {
        assert!(
            w["domain"].is_string(),
            "witness[{}] missing 'domain' field: {}",
            i,
            w
        );
        assert!(
            w["witness"].is_string(),
            "witness[{}] missing 'witness' field: {}",
            i,
            w
        );
        assert!(
            w["status"].is_string(),
            "witness[{}] missing 'status' field: {}",
            i,
            w
        );
    }
}

// ============================================================================
// cmd_query tests
// ============================================================================

/// 3. `cmd_query` with valid Turtle input and a matching pattern returns
///    matching triples in `results`.
#[test]
fn test_query_with_turtle_returns_matching_triples() {
    let _env = TestEnvironment::new().expect("env setup must succeed");

    let turtle = "\
        <http://example.org/Alice> rdf:type <http://example.org/Person> .\n\
        <http://example.org/Bob>   rdf:type <http://example.org/Person> .\n\
        <http://example.org/Alice> <http://example.org/knows> <http://example.org/Bob> .\
    ";

    let output = cmd_query(Some(turtle), "SELECT * WHERE { ?s ?p ?o }")
        .expect("cmd_query must not return Err");

    // The query is supported so output must be successful.
    assert!(
        output.success,
        "query over valid Turtle must succeed; got: {}",
        serde_json::to_string_pretty(&output.data).unwrap_or_default()
    );

    let results = output.data["results"]
        .as_array()
        .expect("data.results must be an array");

    assert_eq!(
        results.len(),
        3,
        "three input triples must produce 3 result bindings, got {}",
        results.len()
    );

    // Every result row must carry s/p/o keys.
    for (i, row) in results.iter().enumerate() {
        assert!(
            row["s"].is_string(),
            "result[{}] missing 's' key: {}",
            i,
            row
        );
        assert!(
            row["p"].is_string(),
            "result[{}] missing 'p' key: {}",
            i,
            row
        );
        assert!(
            row["o"].is_string(),
            "result[{}] missing 'o' key: {}",
            i,
            row
        );
    }

    // Source must be reflected as "inline-turtle".
    assert_eq!(
        output.data["source"].as_str(),
        Some("inline-turtle"),
        "source field must be 'inline-turtle' when TTL is provided"
    );
}

/// 4. `cmd_query` with `ttl = None` and any pattern must return empty results
///    gracefully (the store has no triples, so SELECT returns nothing or an
///    error message — either way no panic and `results` is present).
#[test]
fn test_query_without_turtle_returns_empty_results_gracefully() {
    let _env = TestEnvironment::new().expect("env setup must succeed");

    // With no turtle input the store is empty; SELECT * WHERE { ?s ?p ?o }
    // is supported by PatternExecutor but will match zero triples.
    let output =
        cmd_query(None, "SELECT * WHERE { ?s ?p ?o }").expect("cmd_query must not return Err");

    let results = output.data["results"]
        .as_array()
        .expect("data.results must be an array even on empty store");

    assert_eq!(
        results.len(),
        0,
        "empty store must yield 0 results, got {}",
        results.len()
    );

    assert_eq!(
        output.data["source"].as_str(),
        Some("none"),
        "source field must be 'none' when no TTL is provided"
    );
}

/// Extra: `cmd_query` with valid Turtle but an unsupported pattern returns a
/// non-success Output with an `error` field rather than panicking.
#[test]
fn test_query_unsupported_pattern_returns_error_output() {
    let _env = TestEnvironment::new().expect("env setup must succeed");

    let turtle = "<http://s> <http://p> <http://o> .\n";

    // PatternExecutor only supports SELECT queries containing ?s ?p ?o.
    // An INSERT query must be reported as an error, not a panic.
    let output = cmd_query(
        Some(turtle),
        "INSERT DATA { <http://s> <http://p> <http://o> }",
    )
    .expect("cmd_query must return Ok even for unsupported patterns");

    assert!(
        !output.success,
        "unsupported pattern must produce a non-success Output"
    );

    assert!(
        output.data["error"].is_string(),
        "unsupported pattern must set an 'error' field in data: {}",
        output.data
    );
}
