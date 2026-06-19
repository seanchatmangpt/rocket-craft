# BRIEFING — 2026-06-19T05:48:10Z

## Mission
Propose concrete RDF classes, properties, relationships, and validation shapes/rules for modeling the UE4 asset cooking pipeline and compression profiles in `/Users/sac/.ggen/packs/ue4_ontology/typestates.ttl`.

## 🔒 My Identity
- Archetype: Teamwork explorer (Explorer 1)
- Roles: Cooking and Asset Pipeline Explorer
- Working directory: /Users/sac/rocket-craft/.agents/explorer_cooking_m5_1_gen2
- Original parent: 26a55229-63bc-48fa-bc48-7ec491f0dfa3
- Milestone: Cooking and Asset Pipeline Modelling (m5_1)

## 🔒 Key Constraints
- Read-only investigation — do NOT implement (do not edit /Users/sac/.ggen/packs/ue4_ontology/ files, only read them and propose additions)
- CODE_ONLY network mode: No external queries or HTTP client requests.

## Current Parent
- Conversation ID: 26a55229-63bc-48fa-bc48-7ec491f0dfa3
- Updated: 2026-06-19T05:48:10Z

## Investigation State
- **Explored paths**: `/Users/sac/.ggen/packs/ue4_ontology/typestates.ttl`, `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`, `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl`, `/Users/sac/rocket-craft/validate_ontology.sh`
- **Key findings**:
  - `typestates.ttl` defines `CookingTypestate` but lacks target-platform differentiation, asset-type subclasses (`UStaticMesh`, `USoundWave`), or compression formats.
  - Recommended introducing `ue4:AssetPlatformRepresentation` to relate source assets to target platform configurations and compression profiles.
  - Formulated 5 custom SHACL validation shapes for HTML5 texture/audio format compatibility, mesh LODs, and file size budgets.
- **Unexplored areas**: Linking and WASM memory layout (handled by Explorer 2); Packaging and RHI target config (handled by Explorer 3).

## Key Decisions Made
- Chose an intermediate class model (`ue4:AssetPlatformRepresentation`) over direct properties on raw assets to support multi-platform builds cleanly.

## Artifact Index
- `/Users/sac/rocket-craft/.agents/explorer_cooking_m5_1_gen2/analysis.md` — Main analysis report
- `/Users/sac/rocket-craft/.agents/explorer_cooking_m5_1_gen2/handoff.md` — Handoff report
