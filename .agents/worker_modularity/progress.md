# Progress

Last visited: 2026-06-20T21:23:45Z

- [x] Initialized ORIGINAL_REQUEST.md
- [x] Updated BRIEFING.md
- [x] Investigate `ggen.toml` queries and locations for SM_Limb_Left, SM_Limb_Right, SM_Loadout, SM_TankTreads, SM_InterleavedWheels, and SM_KwK36Gun
- [x] Insert SPARQL filter `FILTER (?part = ?CURRENT_PART_ID || (?type = "socket" && ?part != ?CURRENT_PART_ID))` into the queries in `ggen.toml` (managed via `patch_geometry_generator.py`)
- [x] View `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera` and identify where to insert the new primitiveFamily branches
- [x] Modify `part_mesh.usda.tera` (via `patch_geometry_generator.py`) to explicitly handle:
  - "hard_surface_shell" -> same branch as "angular_armor_shell" and "tapered_box"
  - "fin" -> thin blade/tapered box
  - "wing" -> cylinder/wing spar
  - "arm" -> arm segment (cylinder + sphere)
  - "leg" -> leg segment (cylinder + sphere)
  - "core" -> box/cube
  - "thruster" -> cone/thruster nozzle
- [x] Run `bash scripts/verify_asset.sh` and inspect outputs/gap report
- [x] Verify that duplicate geometries are eliminated and modular USD files contain only correct prims
- [x] Document final results in handoff.md and progress.md
