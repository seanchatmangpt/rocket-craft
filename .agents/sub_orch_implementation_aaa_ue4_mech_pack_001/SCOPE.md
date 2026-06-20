# Scope: FLAGSHIP_UE4_MECH_PLANT_001 Implementation Track

## Architecture
The Implementation Track generates a flagship cinematic UE4 Mech Unit from source ontologies, SPARQL, and ggen templates. It supports hero geometry, modular USD meshes, cinematic Lookdev (MaterialX) with 4K/8K texture policies, skeletal rigging (UsdSkel), weapon sockets, heavy animations (idle, walk, deploy), destruction states (exposed frames, VFX sockets, broken armor meshes), multiple weapon loadouts, IP-Distance verification, and automated UE4 import, cooking, and receipt/replay.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | F1 Geometry & Morphology | Layered swept feather panels, angular armor shells, torso/head distinctions, and cyan blade rods. | None | PLANNED |
| 2 | Modular USD Identity | SPARQL ?CURRENT_PART_ID binding to isolate part meshes and prevent assembly duplication. | Milestone 1 | PLANNED |
| 3 | Cinematic Lookdev & 4K/8K Textures | Lookdev MaterialX definitions and headless texture generation with 4K/8K policies. | Milestone 2 | PLANNED |
| 4 | Rigging, VFX Sockets & Bounds | UsdSkel rigging, joint limits, weapon sockets, collision bounding boxes, and VFX attachment sockets. | Milestone 2 | PLANNED |
| 5 | Destruction & Animation States | Turtle/template declarations for destruction states (broken armor, exposed mechanical frame) and heavy animations (idle, walk, deploy). | Milestone 4 | PLANNED |
| 6 | Multiple Weapon Loadouts | Multiple loadout configurations and attachment variants. | Milestone 5 | PLANNED |
| 7 | IP-Distance Engine | Output IP distance, mecha commons, and trademark signature graphs. | Milestone 3 | PLANNED |
| 8 | UE4 Import & Cooking Automation | Automated FBX translation, import tables, DataTable material binding, and cooking verification. | Milestone 6, 7 | PLANNED |
| 9 | Verification & Receipts | Run F1 command `just verify-flagship-ue4-mech` against E2E test suite (TEST_READY.md), compile visual outputs for AI Vision Judge (evaluating VJ-CRIT-001 through VJ-CRIT-006, requiring PASS_FLAGSHIP disposition), and output receipts/OCEL logs. | Milestone 8 | PLANNED |

## Interface Contracts
### ggen ↔ USD / MaterialX
- Ontology graphs define part topology, materials, textures, rigs, animations, destruction states, and loadouts.
- SPARQL queries extract parameters to populate templates.
- Composed USDA assembly references part files.
### USD / MaterialX ↔ UE4
- FBX translation/imports via import tables.
- DataTable mapping for material binding in UE4.
