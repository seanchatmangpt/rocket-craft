## 2026-06-20T20:40:39Z

MANDATORY INTEGRITY WARNING — include this verbatim in the Worker's dispatch prompt:
> DO NOT CHEAT. All implementations must be genuine. DO NOT
> hardcode test results, create dummy/facade implementations, or
> circumvent the intended task. A Forensic Auditor will independently
> verify your work. Integrity violations WILL be detected and your
> work WILL be rejected.

You are dispatched to implement Milestone 2 (R1: Deterministic Geometry & Modular USD).
Your task:
1. Modify `/Users/sac/rocket-craft/ggen.toml` to insert the SPARQL filter:
   `FILTER (?part = ?CURRENT_PART_ID || (?type = "socket" && ?part != ?CURRENT_PART_ID))`
   into the queries for SM_Limb_Left, SM_Limb_Right, SM_Loadout, SM_TankTreads, SM_InterleavedWheels, and SM_KwK36Gun.
2. Modify `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera` to handle the following primitiveFamily types explicitly:
   - "hard_surface_shell": Map it to the same branch as "angular_armor_shell" and "tapered_box".
   - "fin": Add a branch rendering a thin blade/tapered box.
   - "wing": Add a branch rendering a cylinder/wing spar.
   - "arm": Add a branch rendering an arm segment (cylinder + sphere).
   - "leg": Add a branch rendering a leg segment (cylinder + sphere).
   - "core": Add a branch rendering a box/cube.
   - "thruster": Add a branch rendering a cone/thruster nozzle.
   Ensure that these do not fall back to the default nested mechanical subframe (which generates duplicate cylinders and spheres).
3. Run `bash scripts/verify_asset.sh` to generate the new USD assets, renders, and gap report.
4. Verify that duplicate geometries are eliminated and that the modular USD files contain only their correct prims.
5. Report the metrics and vis_errors from `generated/mech_assets/reference_fabric_001/reports/visual_gap_report.json`.

Please write your progress to `/Users/sac/rocket-craft/.agents/worker_modularity/progress.md` and your final report to `/Users/sac/rocket-craft/.agents/worker_modularity/handoff.md`.
