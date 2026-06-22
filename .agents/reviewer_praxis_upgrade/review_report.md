# Quality & Adversarial Review Report — Praxis Template Upgrade

## Review Summary

**Verdict**: REQUEST_CHANGES

## Findings

### Critical Finding 1: Mismatched & Hardcoded Template Placeholders (Leak)

- **What**: The developer hardcoded the project name `praxis_template` inside some files while leaving `{{project-name}}` placeholders in others.
- **Where**:
  - `template/Cargo.toml` (lines 2, 9, 10, 11)
  - `template/Cargo.workspace.toml` (lines 6, 34, 35)
  - `template/src/types.rs` (lines 1, 18, 81, 140, 238, 769, 1079, 1082, 1085)
  - `template/src/lsp.rs` (lines 1, 37, 65)
  - Unmodified files like `template/src/chain.rs`, `template/src/cli.rs`, and `template/src/error.rs` still contain `{{project-name}}` placeholders.
- **Why**: Hardcoding the project name breaks the scaffold template's functionality. When instantiated using a template-generation tool (which replaces `{{project-name}}` with a custom name like `praxis_test_project`), the crate compiles under the hardcoded name `praxis_template` but doc tests in the unmodified files try to import `use praxis_test_project::...`, leading to compilation errors.
- **Suggestion**: Restore all `{{project-name}}` placeholders in `Cargo.toml`, `Cargo.workspace.toml`, `src/types.rs`, and `src/lsp.rs` so they remain templated.

### Major Finding 2: Missing Re-export of `ProfileId` in Crate Root

- **What**: `ProfileId` is defined in `src/types.rs` but is not exported in `src/lib.rs`.
- **Where**: `template/src/types.rs` (line 406) and `template/src/lib.rs` (line 22).
- **Why**: The doctest for `ProfileId` (lines 391-401) attempts to import it via `use praxis_template::ProfileId;`. This fails compilation since `ProfileId` is only accessible under `praxis_template::types::ProfileId`.
- **Suggestion**: Add `ProfileId` to the `pub use` list in `src/lib.rs`:
  ```rust
  pub use types::{Blake3Hash, ObjectRef, ProfileId, canonical_bytes};
  ```

### Major Finding 3: Unresolved Git Merge Conflict in `lsp-max` Dependency

- **What**: The local dependency `/Users/sac/lsp-max` contains git merge conflict markers in `src/rule_pack_server.rs`.
- **Where**: `/Users/sac/lsp-max/src/rule_pack_server.rs` (lines 1416-1420).
- **Why**: The conflict markers cause compilation failures when the template is checked or compiled with the `lsp` feature enabled (which depends on `lsp-max`).
- **Suggestion**: Resolve the merge conflict in the `/Users/sac/lsp-max` repository.

---

## Verified Claims

- **Evidence typestate and Admit trait implementation** → verified by inspecting `src/types.rs` and running `cargo test --features mcp` → **PASS**
  - Robust use of sealed state markers (`mod sealed { pub trait LifecycleState {} }`), ZST tags (`Raw`, `Validated`, `Admitted`), and PhantomData constraints.
  - One-way transition enforced at compile time.
- **MCP server rmcp attributes usage** → verified by compiling and running `mcp_server` under the `mcp` feature → **PASS**
  - Attributes `#[tool_router]`, `#[tool_handler]`, and `#[tool]` are correctly set up and compile without errors.
- **ProfileId repr(u8) and size assertion** → verified by inspecting `src/types.rs` and checking compile-time size assertion → **PASS**
  - `#[repr(u8)]` is present and asserted to be exactly 1 byte:
    `assert!(size_of::<ProfileId>() == 1);`

---

## Coverage Gaps

- **LSP Feature Integration Verification** — Risk level: **HIGH** — recommendation: Resolve the conflict in `lsp-max` and re-run compilation checks with `--features lsp` to ensure the template server builds correctly against the new `RulePackServer` api.

---

## Unverified Items

- **LSP Feature Compilation & Conformance** — Reason: blocked by local dependency (`lsp-max`) code syntax error.

---

## Challenge Summary (Adversarial Critic)

**Overall risk assessment**: MEDIUM

## Challenges

### [Medium] Challenge 1: Non-Exhaustive Enum Representation Mismatch

- **Assumption challenged**: The assumption that `#[repr(u8)]` and compile-time size assertions will prevent all downstream serialization size mismatches on non-exhaustive enums.
- **Attack scenario**: While the compile-time size assertion checks `size_of::<ProfileId>() == 1` within the crate, downstream crates that compile against this crate may experience layout shifts if new variants are added that exceed the `u8` capacity (greater than 255 variants). Although extremely unlikely for a protocol profile enum, `#[non_exhaustive]` allows variant expansion, but `#[repr(u8)]` restricts it.
- **Blast radius**: If the variant list grows beyond 256, it will fail compile-time validation.
- **Mitigation**: Keep both `#[repr(u8)]` and the compile-time layout assertion `_PROFILE_ID_SIZE` to fail fast at compile-time instead of shifting sizes silently on wire protocol.

### [Low] Challenge 2: Evading the `Admit` Gate via Private `validate_unchecked` / `admit_unchecked`

- **Assumption challenged**: The assumption that the `admit_unchecked` or `validate_unchecked` constructors are only callable by the appropriate validator authority.
- **Attack scenario**: Because the constructors are `pub(crate)` rather than private to a specific submodule, any code within the same crate (including tests, CLI routing, or other modules) can call `admit_unchecked` to bypass validation. If a developer accidentally imports and uses `admit_unchecked` directly in `cli.rs` or `main.rs`, they bypass the validation logic defined in the `Admit` implementation.
- **Blast radius**: Security check bypass if unchecked constructors are called directly within the crate instead of using the `Admit` trait.
- **Mitigation**: Move the unchecked constructors and the `Admit` implementation into a separate submodule (e.g. `src/types/admit.rs`) and keep the constructors private to that module, exposing only the `Admit` trait publicly.
