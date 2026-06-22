# Project: Praxis Generator Upgrade (Post-Chatman Equation)

## Architecture
- **Input Ecosystems**: `~/rocket-craft` and `~/lsp-max` workspaces contain the reference implementations of Generative Typestates, `RulePackServer`, and the `ggen` µ-pipeline.
- **Generator Core**: `~/praxis` is a template-based boilerplate generator.
- **Target Abstraction**: Extract core patterns and upgrade the templates and Rust logic in `~/praxis` to dynamically output Post-Chatman conforming structures ($A = \mu(O^*)$).
- **Verification Harness**: Code verification and structural checks ensuring PhantomData typestates and `RulePackServer` compliance in the generated projects.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Ecosystem Catalog and Abstraction | Analyze and document patterns/components in rocket-craft/lsp-max | None | DONE (/Users/sac/rocket-craft/.agents/explorer_m1/catalog_report.md) |
| M2 | Praxis Generator Upgrade | Implement Rust upgrades in the praxis codebase to output typestates and RulePackServer structures | M1 | DONE (/Users/sac/rocket-craft/.agents/worker_praxis_upgrade/handoff.md) |
| M3 | Project Emission & Cargo Check | Emit sample project and verify compilation (cargo check) | M2 | DONE (/Users/sac/rocket-craft/.agents/worker_praxis_upgrade_remediation/handoff.md) |
| M4 | Programmatic Conformance Verification | Run programmatic verification script to assert typestate and RulePackServer structural compliance | M3 | DONE (/Users/sac/rocket-craft/.agents/auditor_praxis_upgrade/audit_report.md) |

## Interface Contracts
### `praxis` generator ↔ Generated Boilerplate
- The generator executes via CLI or standard command interface.
- Output: Valid Rust workspace structure containing `RulePackServer` traits and phantom-typestate state-machines.
