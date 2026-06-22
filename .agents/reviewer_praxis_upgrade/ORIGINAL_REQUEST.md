## 2026-06-21T22:41:19-07:00
You are the Boilerplate Code Reviewer (role: reviewer).
Your working directory is `/Users/sac/rocket-craft/.agents/reviewer_praxis_upgrade`.
Your parent conversation ID is `30eea61c-a259-48ef-85ca-8bca6c94e767`.

Your mission is to perform a detailed correctness and robustness review of the upgrades made to `/Users/sac/praxis/template` (including Cargo.toml, src/lsp.rs, src/types.rs, src/bin/mcp_server.rs, and src/main.rs).
Verify that:
1. The `Evidence` typestate, `Admit` trait, and `AppLspServer` implementing `RulePackServer` are correctly structured, follow idiomatic Rust patterns (ZSTs, PhantomData, Witness/Seal patterns).
2. The `lsp` feature and dependencies in `template/Cargo.toml` are correctly set up and resolved without conflicts.
3. The changes in `mcp_server.rs` correctly use the new rmcp attributes (`#[tool_router]`, `#[tool_handler]`) and do not introduce errors.
4. The u8 representation (`#[repr(u8)]`) on `ProfileId` is correct and resolves size mismatch.

Write your findings to `/Users/sac/rocket-craft/.agents/reviewer_praxis_upgrade/review_report.md`. Ensure you provide a clear pass/fail verdict. Write `handoff.md` and send a message back to the parent conversation ID `30eea61c-a259-48ef-85ca-8bca6c94e767`.
