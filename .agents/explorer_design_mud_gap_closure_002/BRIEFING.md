# BRIEFING — 2026-06-19T20:20:00Z

## Mission
Design the architecture of the Rust-based gap checker (`mud_gap_check`) generated from ontology metadata for Mech Factory MUD.

## 🔒 My Identity
- Archetype: explorer
- Roles: explorer_design_mud_gap_closure_002
- Working directory: /Users/sac/rocket-craft/.agents/explorer_design_mud_gap_closure_002
- Original parent: 6f2354dc-f3cf-4d95-ac26-1866cb1cb85e
- Milestone: Mech Factory MUD gap checker architecture

## 🔒 Key Constraints
- Read-only investigation — do NOT implement (do not write to source tree, only agents directory)
- Do not modify any files except files in working directory
- CODE_ONLY network mode (no external services)

## Current Parent
- Conversation ID: 6f2354dc-f3cf-4d95-ac26-1866cb1cb85e
- Updated: 2026-06-19T20:20:00Z

## Investigation State
- **Explored paths**: `crates/mech_factory_mud`, `ontology/ggen-packs/mech_factory_mud`, `ontology/mech_factory_mud.ttl`, `scripts/mud_gap_check.py`
- **Key findings**:
  - Found Python check logic in `scripts/mud_gap_check.py` checking 21 requirements (files exist, cargo test metrics, command exit statuses).
  - Determined that expected file list and check rule declarations can be represented as ontology classes (`ExpectedFile`, `GapCheckRule`) in `schema/mech_factory_mud.ttl`.
  - Defined SPARQL query using `UNION` to select expected files, rules, route nodes, and stations deterministically.
  - Drafted Tera template (`mud_gap_check.rs.tera`) that outputs a complete Rust binary `crates/mech_factory_mud/src/bin/mud_gap_check.rs`.
- **Unexplored areas**: Direct runtime verification of the generated check code, as we are in a read-only investigation constraint.

## Key Decisions Made
- Chose to propose a dedicated SPARQL query (`queries/gap_check.rq`) instead of modifying `all.rq` to follow the Combinatorial Maximalist Doctrine's Bounded Selection principle.
- Chose to parameterize the station canonical check and route connectivity check by querying ontology data dynamically in the Tera template.

## Artifact Index
- /Users/sac/rocket-craft/.agents/explorer_design_mud_gap_closure_002/design.md — Main design proposal output
