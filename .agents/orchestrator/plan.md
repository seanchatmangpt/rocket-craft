# Project: BIPEDAL_KIT_COHERENCE & HIGH_FIDELITY_SCULPTING
# Scope: Mathematically sculpt blocky Mecha geometry into a high-fidelity, photorealistic 3D asset matching the Wing Gundam Snow White Prelude reference image. Implement SHACL-based bipedal kit coherence constraints, compile the mecha graph, and verify via Playwright actuation.

## Architecture
The procedural sculpting is driven by `ggen` geometry templates (`part_mesh.usda.tera` and queries) mapping to the mecha source graph. SHACL rules in `ontology/source_law/` validate the structural connectivity (kit coherence) and proportions of the bipedal model.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|---|---|---|---|
| 1 | Exploration & Gap Analysis | Inspect reference image, check current geometry templates, and identify anatomy and morphology gaps. | None | PLANNED |
| 2 | Kit Coherence Graph Constraints | Formulate SHACL shapes for neck/pelvis/joints and add kit coherence constraints to source law. | M1 | PLANNED |
| 3 | High-Fidelity Geometry Templates | Enhance `part_mesh.usda.tera` with layered armor, curved overlapping feathers, cheeks, neck, and joint sockets. | M2 | PLANNED |
| 4 | SHACL Validation | Compile the mecha graph and run `validate_shacl.py` to ensure all coherence and envelope constraints pass. | M3 | PLANNED |
| 5 | Visual Pipeline & Playwright Gate | Run verify scripts, generate fresh renders, and execute Playwright actuation validation. | M4 | PLANNED |
| 6 | Forensic Audit & Handoff | Spawn Forensic Auditor to verify verdict cleanliness and compile final reports. | M5 | PLANNED |

## Interface & Kit Coherence Constraints
### Kit Coherence Rules (BIPEDAL_KIT_COHERENCE)
- head_y_min > torso_y_max
- neck connects head to torso
- shoulder sockets attach arms to torso
- elbow exists between shoulder and wrist
- wrist connects to manipulator
- weapon must be held by manipulator, not float
- pelvis exists below torso
- hip sockets attach thighs
- knee exists between thigh and shin
- ankle exists between shin and foot
- feet contact lower body ground band
- shield attaches to forearm or hand
- wing binders attach to backpack, not arbitrary torso slabs
- wing feathers are segmented curved panels, not flat slabs
