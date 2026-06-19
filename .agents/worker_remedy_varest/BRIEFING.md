# BRIEFING — 2026-06-19T05:41:45Z

## Mission
Remediate the VaRest plugin build blocker (exit code 25) by integrating X3D, CityGML, BOT, and GeoSPARQL in the eden_server and ue4_ontology packs, removing VaRest from project descriptor files, and disabling/removing runtime Blueprint dynamic HTTP/data-loading paths.

## 🔒 My Identity
- Archetype: worker_remedy_varest
- Roles: implementer, qa, specialist
- Working directory: /Users/sac/rocket-craft/.agents/worker_remedy_varest/
- Original parent: 4aba8fb0-9db3-4e8c-9ad3-b7944b912853
- Milestone: Remediation of VaRest Build Blocker

## 🔒 Key Constraints
- CODE_ONLY network mode: no external HTTP client, curl, wget, etc.
- No cheating: DO NOT hardcode test results, expected outputs, or verification strings.
- Verify output files target valid consumer subdirectories (like `src/`), NOT the pack root, and contain no "DO NOT EDIT" banners.

## Current Parent
- Conversation ID: 4aba8fb0-9db3-4e8c-9ad3-b7944b912853
- Updated: not yet

## Task Summary
- **What to build**: Ontologies with minimum required terms from X3D, CityGML, BOT, and GeoSPARQL in `eden_server/ontology/pack.ttl` and `instances.ttl`, mapping them to `ggen` rules in `ggen.toml`, removing VaRest plugin requirement from `Brm.uproject` or similar, removing Blueprint dynamic HTTP loading, compiling/cooking successfully and verifying output files.
- **Success criteria**: No VaRest build blocker, WebGL build packages successfully, walkthrough loads and generates the receipt with PASS verdict, valid output locations, no "DO NOT EDIT" banners, no cheating.
- **Interface contracts**: /Users/sac/rocket-craft/GEMINI.md, /Users/sac/rocket-craft/.agents/AGENTS.md
- **Code layout**: /Users/sac/rocket-craft/ (and files inside)

## Change Tracker
- **Files modified**: None
- **Build status**: Unknown
- **Pending issues**: None

## Quality Status
- **Build/test result**: Unknown
- **Lint status**: Unknown
- **Tests added/modified**: None

## Loaded Skills
- None

## Key Decisions Made
- None

## Artifact Index
- /Users/sac/rocket-craft/.agents/worker_remedy_varest/ORIGINAL_REQUEST.md — Original user request
