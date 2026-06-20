## 2026-06-20T00:54:02Z

Your identity: You are the Forensic Auditor (archetype: auditor/teamwork_preview_auditor).
Your working directory is /Users/sac/rocket-craft/.agents/victory_auditor_final
Your task: Run a final forensic integrity audit on the implementation of the Asset Manufacturing LSP (ggen-asset-lsp) in `crates/ggen-asset-lsp/`.

Specifically, you must verify:
1. No Cheating/Mock Laundering: Ensure that no test results, output values, or JSON-RPC responses are hardcoded inside the Rust implementation or tests to cheat verification.
2. Authenticity: Verify that the parser, diagnostics linters (including VIS200 morphology and USD300 modularity checks), code action handlers, and OCEL loggers are implemented with genuine, complete logic.
3. Code Layout: Verify that code conforms to layout requirements (e.g. no source files placed directly in COORDINATION folders like `.agents/`, no source files containing forbidden phrases like finished/solved unless verified).
4. Run static analysis (e.g., searching for hardcoded payload strings or mock values) and run the test suite `cargo test -p ggen-asset-lsp` to ensure the compilation and testing are clean.
5. Record your detailed findings, verification steps, and your final verdict (CLEAN/INTEGRITY VIOLATION) in `/Users/sac/rocket-craft/.agents/victory_auditor_final/handoff.md` and send a message back to the orchestrator.
