# BRIEFING — 2026-06-19T00:50:00Z

## Mission
Investigate the ggen.toml schema and configuration by researching the local ~/ggen/ repository and analyzing project requirements.

## 🔒 My Identity
- Archetype: explorer
- Roles: read-only investigator
- Working directory: /Users/sac/rocket-craft/.agents/explorer_ggen_spec_2/
- Original parent: b6f958b7-c50a-4ec3-8e16-40ef0a23f032
- Milestone: explorer_ggen_spec_2

## 🔒 Key Constraints
- Read-only investigation — do NOT implement
- CODE_ONLY network mode: no external web access, no curl/wget/lynx to external URLs

## Current Parent
- Conversation ID: b6f958b7-c50a-4ec3-8e16-40ef0a23f032
- Updated: 2026-06-19T00:50:00Z

## Investigation State
- **Explored paths**: 
  - `/Users/sac/rocket-craft/.agents/orchestrator_ggen_spec/PROJECT.md`
  - `/Users/sac/ggen/ggen.toml`
  - `/Users/sac/ggen/crates/ggen-config/src/config_lib/schema.rs`
  - `/Users/sac/ggen/crates/ggen-core/src/manifest/types.rs`
  - `/Users/sac/ggen/crates/ggen-core/src/manifest/parser.rs`
  - `/Users/sac/ggen/README.md`
- **Key findings**: 
  - `ggen.toml` is a dual-purpose configuration file, mapped to `GgenManifest` in `ggen-core` and `GgenConfig` in `ggen-config`.
  - The core generator rules rely on `[project]`, `[ontology]`, `[inference]`, `[[generation.rules]]`, and `[validation]` blocks.
  - The BIG BANG 80/20 criteria are defined in reference comments and `README.md` as 5 gate conditions enforcing specification-first design.
- **Unexplored areas**: None.

## Key Decisions Made
- Fully documented and structured both configurations in analysis.md.
- Created handoff.md mapping findings to parent orchestrator.

## Artifact Index
- `/Users/sac/rocket-craft/.agents/explorer_ggen_spec_2/analysis.md` — Complete ggen.toml schema analysis.
- `/Users/sac/rocket-craft/.agents/explorer_ggen_spec_2/handoff.md` — Canonical handoff report.
