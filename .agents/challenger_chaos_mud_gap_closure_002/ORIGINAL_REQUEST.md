## 2026-06-19T20:25:23Z
You are a teamwork_preview_challenger subagent.
Your role name is: challenger_chaos_mud_gap_closure_002
Your working directory is: /Users/sac/rocket-craft/.agents/challenger_chaos_mud_gap_closure_002

Your task is to:
1. Test the new Rust gap checker's resilience by temporarily injecting chaos mutations (e.g. rename a generated file like `generated/mech_factory_mud/rust/route.rs`, or break a check target).
2. Verify that `cargo run --bin mud_gap_check` immediately exits with code 1, correctly flags the failed rule, and records the defect class.
3. Restore the files after testing and verify the checker passes again.
4. Record your chaos test execution and results in `/Users/sac/rocket-craft/.agents/challenger_chaos_mud_gap_closure_002/challenger_report.md` and report back.
