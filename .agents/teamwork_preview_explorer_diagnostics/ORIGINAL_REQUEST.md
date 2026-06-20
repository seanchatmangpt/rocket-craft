## 2026-06-19T19:17:39Z
You are a Diagnostic Explorer (teamwork_preview_explorer).
Your working directory is `/Users/sac/rocket-craft/.agents/teamwork_preview_explorer_diagnostics/`.
Please perform the following exploration:
1. Run the gap check script: `python3 scripts/mud_gap_check.py` and capture its stdout/stderr.
2. Run `cargo check -p mech_factory_mud` and `cargo test -p mech_factory_mud` and capture the outputs.
3. Inspect `crates/mech_factory_mud/` to understand the code structure (e.g. src/, tests/, Cargo.toml, etc.).
4. Find the location of ggen configurations, ontologies (e.g., .ttl files), and Tera templates in the codebase.
5. Write your findings and verification commands in `handoff.md` in your working directory.
6. Send a message back to the parent (conversation ID: 35f1bedd-29d2-4018-9ff0-9132ef45c113) when done with the path to your handoff.md.
