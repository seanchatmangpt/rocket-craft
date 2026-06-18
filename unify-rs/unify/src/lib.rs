pub mod app;
pub mod commands;
pub mod output;
pub mod version;

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    // ── Output tests ─────────────────────────────────────────────────────────

    #[test]
    fn output_ok_has_success_true() {
        let out = output::Output::ok(serde_json::json!({"x": 1}));
        assert!(out.success);
    }

    #[test]
    fn output_error_has_success_false() {
        let out = output::Output::error("boom");
        assert!(!out.success);
    }

    #[test]
    fn output_to_json_is_valid_json() {
        let out = output::Output::ok(serde_json::json!({"hello": "world"}));
        let s = out.to_json();
        let v: serde_json::Value = serde_json::from_str(&s).expect("must be valid JSON");
        assert_eq!(v["success"], true);
    }

    #[test]
    fn output_success_msg_has_message() {
        let out = output::Output::success_msg("completed");
        assert!(out.success);
        assert_eq!(out.message.as_deref(), Some("completed"));
    }

    #[test]
    fn output_to_human_contains_ok_for_success() {
        let out = output::Output::success_msg("all good");
        let human = out.to_human();
        assert!(human.contains("[OK]"));
    }

    // ── version tests ─────────────────────────────────────────────────────────

    #[test]
    fn crate_versions_returns_at_least_10_entries() {
        let vs = version::crate_versions();
        assert!(vs.len() >= 10, "expected >= 10 crates, got {}", vs.len());
    }

    #[test]
    fn crate_versions_includes_unify() {
        let vs = version::crate_versions();
        assert!(vs.iter().any(|v| v.name == "unify"));
    }

    // ── command tests ─────────────────────────────────────────────────────────

    #[test]
    fn cmd_receipt_produces_hash_in_output() {
        let out = commands::cmd_receipt("foo", "hello world").expect("cmd_receipt should succeed");
        assert!(out.success);
        let hash = out.data["hash"].as_str().expect("hash must be a string");
        assert!(!hash.is_empty(), "hash must not be empty");
    }

    #[test]
    fn cmd_info_returns_version_info() {
        let out = commands::cmd_info().expect("cmd_info should succeed");
        assert!(out.success);
        let crates = out.data["crates"]
            .as_array()
            .expect("crates must be an array");
        assert!(!crates.is_empty());
    }

    #[test]
    fn cmd_verify_on_valid_receipt_json_succeeds() {
        // Build a receipt where the key is used as its own data so verify() returns true.
        use unify_receipts::receipt::Receipt;
        let key = "test-key";
        let r = Receipt::new(key, key.as_bytes());
        let json = serde_json::to_string(&r).unwrap();
        let out = commands::cmd_verify(&json).expect("cmd_verify should not error");
        assert!(out.success, "verify should succeed for self-keyed receipt");
    }

    #[test]
    fn cmd_dispatch_unknown_noun_verb_returns_error() {
        let out = commands::cmd_dispatch(None, "unicorn", "fly", None)
            .expect("cmd_dispatch returns Ok even on unknown dispatch");
        assert!(!out.success, "unknown dispatch should set success=false");
    }

    // ── CLI parse tests ───────────────────────────────────────────────────────

    #[test]
    fn cli_parses_receipt_label_and_data() {
        let cli = app::Cli::try_parse_from(["unify", "receipt", "-l", "foo", "bar"])
            .expect("should parse receipt subcommand");
        match cli.command {
            app::Commands::Receipt { label, data } => {
                assert_eq!(label, "foo");
                assert_eq!(data, "bar");
            }
            _ => panic!("expected Receipt command"),
        }
    }

    #[test]
    fn cli_parses_info() {
        let cli =
            app::Cli::try_parse_from(["unify", "info"]).expect("should parse info subcommand");
        assert!(matches!(cli.command, app::Commands::Info));
    }

    #[test]
    fn cli_parses_witnesses_with_domain() {
        let cli = app::Cli::try_parse_from(["unify", "witnesses", "--domain", "rdf"])
            .expect("should parse witnesses subcommand");
        match cli.command {
            app::Commands::Witnesses { domain } => {
                assert_eq!(domain.as_deref(), Some("rdf"));
            }
            _ => panic!("expected Witnesses command"),
        }
    }
}
