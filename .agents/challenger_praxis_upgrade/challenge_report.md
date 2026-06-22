# Challenge Report — Conformance Verification of Upgraded Praxis Generator

**Verdict**: FAIL

## Challenge Summary

**Overall risk assessment**: CRITICAL

The upgraded praxis generator template and its dependencies contain critical build-breaking bugs and unreplaced template tags that prevent a clean build and test execution from scratch.

---

## Challenges

### [Critical] Challenge 1: Git Merge Conflict in Dependency `lsp-max`

- **Assumption challenged**: The underlying dependencies are stable and compile cleanly.
- **Attack scenario**: When the cargo cache is cleaned (`cargo clean`) or a developer checks out the project in a fresh environment, compilation fails because `/Users/sac/lsp-max/src/rule_pack_server.rs` contains unresolved git merge conflict markers (`<<<<<<< HEAD` ... `=======` ... `>>>>>>>`).
- **Blast radius**: Prevents the dynamically generated project from compiling or passing check whenever `lsp-max` is recompiled.
- **Mitigation**: Resolve the merge conflict in `/Users/sac/lsp-max/src/rule_pack_server.rs`.

### [Critical] Challenge 2: Incompatible `lsp-types-max` Trait Bounds in `lsp-max`

- **Assumption challenged**: Enabling the `lsp` feature compiles `lsp-max` correctly.
- **Attack scenario**: When the generated project is compiled with `cargo test --all-features` or when `lsp-max` is recompiled from scratch, compilation fails with `error[E0277]: the trait bound max_protocol::lsp_3_18::TextDocumentContentRefreshRequest: lsp_types_max::request::Request is not satisfied` in `lsp-max/src/service/client/lsp_methods.rs`.
- **Blast radius**: The `lsp-max` crate unconditionally compiles method implementations using proposed LSP 3.18 types but does not enable the `proposed` feature on its `lsp-types-max` dependency by default. Unless the user explicitly enables `lsp-max/proposed`, compilation fails.
- **Mitigation**: Either make those methods conditionally compiled in `lsp-max` behind a `proposed` feature, or update the generated project's `Cargo.toml` feature definition to enable `lsp-max/proposed` when `lsp` is enabled: `lsp = ["dep:lsp-max", "lsp-max/proposed", ...]`.

### [High] Challenge 3: Unconditional `#[tokio::main]` in `mcp_server.rs` with Optional Tokio Dependency

- **Assumption challenged**: The generated project can run its unit tests without features enabled.
- **Attack scenario**: Running `cargo test` in `/tmp/my-conforming-project` fails to compile the `mcp_server` test target. The binary `src/bin/mcp_server.rs` unconditionally uses the `#[tokio::main]` attribute, but `tokio` is marked as an optional dependency (only enabled under `mcp` or `lsp` features).
- **Blast radius**: Running `cargo test` out-of-the-box fails.
- **Mitigation**: Guard the `#[tokio::main]` macro and `main` function (or the entire binary file) with a `#[cfg(feature = "mcp")]` configuration attribute.

### [Medium] Challenge 4: Unreplaced `{{crate_name}}` Template Tag

- **Assumption challenged**: All template tags are cleanly replaced.
- **Attack scenario**: Opening `src/cli.rs` in the generated project reveals the raw template tag `{{crate_name}}` on line 84: `/// use {{crate_name}}::cli::{collect_tools_from_cmd, Cli};`.
- **Blast radius**: Documentation code blocks in `src/cli.rs` retain template placeholders, causing confusion or doctest failures.
- **Mitigation**: Update the mock generator in `/Users/sac/praxis/tools/verify_conformance.sh` to substitute `{{crate_name}}` with `$PROJECT_NAME_SNAKE` or a corresponding value.

---

## Stress Test Results

- **Run `/Users/sac/praxis/tools/verify_conformance.sh` without clean** → `VERIFIED` (cached success) → **PASS** (false positive due to build cache)
- **Run `cargo clean && cargo check --all-targets --all-features` inside `/tmp/my-conforming-project`** → Compiler error on git merge conflict markers → **FAIL**
- **Run `cargo test` inside `/tmp/my-conforming-project`** → Compiler error due to unresolved `tokio` macro in `mcp_server.rs` → **FAIL**
- **Run `cargo test --all-features` inside `/tmp/my-conforming-project`** → Compiler error due to `lsp-types-max::request::Request` trait bound mismatch in `lsp-max` → **FAIL**
- **Verify template tags replacement in `Cargo.toml`** → No `{{` tags left → **PASS**
- **Verify template tags replacement in `src/cli.rs`** → `{{crate_name}}` left unreplaced → **FAIL**

---

## Unchallenged Areas

- None — All requested validation tasks were fully investigated.
