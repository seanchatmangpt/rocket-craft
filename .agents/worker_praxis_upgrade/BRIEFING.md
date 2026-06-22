# BRIEFING — 2026-06-22T05:32:30Z

## Mission
Upgrade the praxis boilerplate generator template to natively support typestate-driven configurations and RulePackServer structures adhering to Post-Chatman Equation A = \mu(O^*).

## 🔒 My Identity
- Archetype: worker
- Roles: worker
- Working directory: /Users/sac/rocket-craft/.agents/worker_praxis_upgrade
- Original parent: 30eea61c-a259-48ef-85ca-8bca6c94e767
- Milestone: Praxis Upgrader

## 🔒 Key Constraints
- CODE_ONLY network mode.
- Minimal edits.
- Conform to Post-Chatman Equation A = \mu(O^*).

## Current Parent
- Conversation ID: 30eea61c-a259-48ef-85ca-8bca6c94e767
- Updated: 2026-06-22T05:32:30Z

## Task Summary
- **What to build**: Upgrade template to support `RulePackServer` structures and typestate-driven configurations. Implement `AppLspServer` in `src/lsp.rs` and verify conformance.
- **Success criteria**: Template compiles with `lsp` feature enabled, `AppLspServer` implements `RulePackServer`, and conformance script passes.
- **Interface contracts**: `/Users/sac/praxis/template/Cargo.toml`, `/Users/sac/praxis/template/src/lsp.rs`, `/Users/sac/praxis/tools/verify_conformance.sh`

## Key Decisions Made
- Use python script in verify_conformance.sh to copy and substitute placeholders instead of cargo-generate, since cargo-generate was not installed.

## Change Tracker
- **Files modified**:
  - `/Users/sac/praxis/template/Cargo.toml`: Added lsp feature and optional dependencies.
  - `/Users/sac/praxis/template/src/lsp.rs`: Added AppLspServer implementing RulePackServer.
  - `/Users/sac/praxis/tools/verify_conformance.sh`: Programmatic verification.
- **Build status**: PASS
- **Pending issues**: None

## Quality Status
- **Build/test result**: PASS
- **Lint status**: 0 violations
- **Tests added/modified**: verify_conformance.sh added

## Loaded Skills
- None

## Artifact Index
- `/Users/sac/praxis/template/src/lsp.rs` — AppLspServer implementation
- `/Users/sac/praxis/tools/verify_conformance.sh` — Conformance verifier script
