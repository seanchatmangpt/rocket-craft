## 2026-06-20T21:11:43Z
MANDATORY INTEGRITY WARNING — include this verbatim in the Worker's dispatch prompt:
> DO NOT CHEAT. All implementations must be genuine. DO NOT
> hardcode test results, create dummy/facade implementations, or
> circumvent the intended task. A Forensic Auditor will independently
> verify your work. Integrity violations WILL be detected and your
> work WILL be rejected.

You are dispatched as the replacement Worker (Worker M2 gen 2) for Milestone 2 (R1: Deterministic Geometry & Modular USD).
The previous worker was hung. You must proceed with the following steps:
1. Examine the file `/Users/sac/rocket-craft/ggen.toml`. Locate the rules for SM_Limb_Left, SM_Limb_Right, SM_Loadout, SM_TankTreads, SM_InterleavedWheels, and SM_KwK36Gun.
2. Insert the SPARQL filter:
   `FILTER (?part = ?CURRENT_PART_ID || (?type = "socket" && ?part != ?CURRENT_PART_ID))`
   into each of these queries, just before the closing brace `}`. This ensures that the query returns only primitives belonging to the current part (or outward-pointing sockets).
3. View the file `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera`.
4. Modify this template to add explicit branches for the following primitiveFamily types in the `render_primitive` macro:
   - "hard_surface_shell": Map it to the same rendering code as "angular_armor_shell" and "tapered_box" (i.e. the loop generating cubes/pistons).
   - "fin": Add a custom rendering branch that generates a thin, tapered blade-like box or mesh representing control surfaces/fins.
   - "wing": Add a custom rendering branch that generates a cylinder/spar representing the wing root structure.
   - "arm": Add a custom rendering branch that generates an arm segment consisting of a main bone cylinder and a joint sphere.
   - "leg": Add a custom rendering branch that generates a leg segment consisting of a main bone cylinder and a joint sphere.
   - "core": Add a custom rendering branch that generates a box/cube structure for the backpack core.
   - "thruster": Add a custom rendering branch that generates a cone representing a thruster nozzle.
   Make sure these custom branches do NOT fall back to the default nested mechanical subframe (the `{% else %}` block), which was generating duplicate/identical geometries.
5. Run `bash scripts/verify_asset.sh` (which merges the ontology, runs ggen sync, deletes old renders, renders fresh PNGs, and scores them) to compile the assets and verify success.
6. Verify that the duplicate geometry issue is resolved (i.e., part files like SM_Limb_Left.usda no longer contain identical subframe cylinders/spheres) and that modularity checks pass.
7. Print the metrics and vis_errors from `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/reports/visual_gap_report.json` in your report.

Please write your progress logs to `/Users/sac/rocket-craft/.agents/worker_modularity_gen2/progress.md` and your final report to `/Users/sac/rocket-craft/.agents/worker_modularity_gen2/handoff.md`.
