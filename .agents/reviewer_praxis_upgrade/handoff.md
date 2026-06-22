# Handoff Report — Praxis Template Upgrade Review

## 1. Observation

- **Hardcoded template placeholder leak**:
  We ran `git diff --name-only` inside `/Users/sac/praxis/template` and found the following modified files:
  ```
  template/Cargo.toml
  template/Cargo.workspace.toml
  template/src/lsp.rs
  template/src/main.rs
  template/src/types.rs
  ```
  Looking at `git diff -- Cargo.toml`, the project name and description were hardcoded to `praxis_template`:
  ```diff
  -[package]
  -name = "{{project-name}}"
  +name = "praxis_template"
  ```
  Similarly in `src/types.rs`:
  ```diff
  -//! Core domain types for {{project-name}}.
  +//! Core domain types for praxis_template.
  ```
  But files like `src/chain.rs`, `src/cli.rs`, and `src/error.rs` were not modified and still contain `{{project-name}}` placeholders.

- **Doctest failures**:
  We copied the template, replaced placeholders, and ran `cargo test --workspace --features mcp` in the copy, which produced FAILED results for the doctests:
  ```
  failures:
      src/chain.rs - chain::ChainAssembler (line 45)
      src/cli.rs - cli::collect_tools_from_cmd (line 83)
      src/error.rs - error::ValidationChain (line 111)
      src/types.rs - types::ProfileId (line 439)
  ```
  - For `src/types.rs - types::ProfileId`:
    ```
    error[E0432]: unresolved import `praxis_template::ProfileId`
       --> src/types.rs:441:5
        |
    441 | use praxis_template::ProfileId;
        |     ^^^^^^^^^^^^^^^^^^^^^^^^^^ no `ProfileId` in the root
    ```
  - For the others, they tried to use the template name which resolved to `praxis_test_project`, but since `Cargo.toml` name is hardcoded to `praxis_template`, it resulted in `use of unresolved module or unlinked crate`.

- **Dependency git conflict**:
  We ran `cargo check --workspace --features lsp` and got a compilation error indicating conflict markers in `/Users/sac/lsp-max/src/rule_pack_server.rs`:
  ```
  error: encountered diff marker
      --> /Users/sac/lsp-max/src/rule_pack_server.rs:1416:1
       |
  1416 | <<<<<<< HEAD
       | ^^^^^^^ between this marker and `=======` is the code that you are merging into
  ```

---

## 2. Logic Chain

1. **Premise 1**: A template scaffold must be generic and dynamically replace placeholders like `{{project-name}}` during generation. Hardcoding names breaks generation compatibility.
2. **Premise 2**: Since `Cargo.toml` name was hardcoded to `praxis_template` while code files still reference `{{project-name}}`, any instantiation breaks (proven by doctest failures of `ChainAssembler` etc. trying to import the instantiated crate name `praxis_test_project`).
3. **Premise 3**: Documentation tests must compile. `ProfileId`'s doctest fails because `ProfileId` is defined under `src/types.rs` but is not re-exported in `src/lib.rs` (proven by the compiler error `no ProfileId in the root`).
4. **Premise 4**: Dependencies of the `lsp` feature must compile. However, local path dependency `/Users/sac/lsp-max` is currently blocked by an unresolved merge conflict (proven by compiler error pointing to `<<<<<<< HEAD`).
5. **Conclusion**: Therefore, the current state of the upgrades fails validation, and the verdict must be `REQUEST_CHANGES`.

---

## 3. Caveats

- We assumed that `/Users/sac/praxis/template` is intended to be a reusable template scaffold (corroborated by `cargo-generate.toml` in the same directory). If the intention was to completely abandon the templated/generic nature of this repository, the hardcoded values would not be a defect, but it would contradict the presence of remaining placeholders and `cargo-generate.toml`.
- We did not manually fix the merge conflict in `/Users/sac/lsp-max` since our working boundaries are review-only.

---

## 4. Conclusion

The upgrades made to the template are rejected (`REQUEST_CHANGES` verdict).
- The template suffers from placeholder leak (project name hardcoded in Cargo.toml and some source files, but left in others).
- `ProfileId` is not exported from the crate root, breaking its doctest.
- Local dependency `/Users/sac/lsp-max` is broken due to a git merge conflict, blocking the `lsp` feature check.

---

## 5. Verification Method

To verify these findings:
1. Check `git status` or `git diff` inside `/Users/sac/praxis/template` to see the modifications that hardcoded `praxis_template`.
2. Instantiate the template using the script `/tmp/instantiate_template.py` and run `cargo test --workspace --features mcp` to witness doctest failures.
3. Run `cargo check --workspace --features lsp` in `/Users/sac/praxis/template` or `/tmp/praxis_test_project` to witness the compilation blocker caused by local dependency `/Users/sac/lsp-max`.
