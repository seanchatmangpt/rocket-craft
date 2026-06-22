# BRIEFING — 2026-06-21T22:47:56-07:00

## Mission
Remediate the defects in `/Users/sac/praxis` identified by the reviews, reverting hardcoded names, fixing placeholders, and updating crate configurations.

## 🔒 My Identity
- Archetype: worker
- Roles: implementer, qa, specialist
- Working directory: /Users/sac/rocket-craft/.agents/worker_praxis_upgrade_remediation
- Original parent: 30eea61c-a259-48ef-85ca-8bca6c94e767
- Milestone: Praxis Upgrade Remediation

## 🔒 Key Constraints
- CODE_ONLY network mode
- Mandated integrity: no cheating, no hardcoding verification, maintain real state.

## Current Parent
- Conversation ID: 30eea61c-a259-48ef-85ca-8bca6c94e767
- Updated: not yet

## Task Summary
- **What to build**: Remediation of template and code issues in `/Users/sac/praxis` template to match required placeholder behavior, dependencies, and types re-exporting.
- **Success criteria**: Local verification passes, generated template project under `/tmp/my-conforming-project` compiles cleanly and all tests pass with and without all-features.
- **Interface contracts**: `/Users/sac/praxis/template`
- **Code layout**: Rust project structure

## Key Decisions Made
- Replaced project name placeholders with `{{project_name}}` in all Rust source doc-tests so the generated crate compiles properly using snake_case names.
- Configured the mcp_server binary target in Cargo.toml with `required-features = ["mcp"]` to prevent it from compiling when the `mcp` feature is disabled.
- Redirected cargo targets to a custom non-temporary directory `/Users/sac/cargo_target_conforming` during test runs to prevent intermediate files from being cleaned up by the OS.

## Artifact Index
- `/Users/sac/rocket-craft/.agents/worker_praxis_upgrade_remediation/handoff.md` — Detailed handoff report for the upgrades
- `/Users/sac/rocket-craft/.agents/worker_praxis_upgrade_remediation/progress.md` — Progress tracker

## Change Tracker
- **Files modified**:
  - `template/Cargo.toml`: Revert package name, description, URLs; map lsp-max/proposed; restrict mcp_server binary target.
  - `template/Cargo.workspace.toml`: Revert name in comment; revert URLs to placeholders.
  - `template/src/lib.rs`: Re-export all domain and typestate types.
  - `template/src/types.rs`: Revert/standardize doctests to project_name placeholder.
  - `template/src/chain.rs`: Revert doctest to project_name.
  - `template/src/error.rs`: Revert doctest to project_name.
  - `template/src/verbs/verifier.rs`: Revert doctest to project_name.
  - `template/src/cli.rs`: Revert doctest to project_name.
  - `tools/verify_conformance.sh`: Change target dir back to my-conforming-project.
- **Build status**: Pass (all checks and tests compiled and passed)
- **Pending issues**: None

## Quality Status
- **Build/test result**: Pass (51 unit/doc tests on all features, 52 on default features)
- **Lint status**: 0 compile/lint errors
- **Tests added/modified**: None (conformance tests verified)

## Loaded Skills
- **Source**: /Users/sac/.gemini/antigravity-cli/builtin/skills/antigravity_guide/SKILL.md
- **Local copy**: /Users/sac/rocket-craft/.agents/worker_praxis_upgrade_remediation/SKILL.md
- **Core methodology**: Comprehensive guide for Antigravity, AGY, CLI, rules, and customization.
