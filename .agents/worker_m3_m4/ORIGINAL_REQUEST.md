## 2026-06-18T21:36:44-07:00
Perform Milestone 3 (Validation & SHACL Hardening) and Milestone 4 (The ALIVE Proof Generation) for the `eden_server` ontology pack.

You must do the following:
1. Create a new Turtle file `/Users/sac/.ggen/packs/eden_server/ontology/instances.ttl` containing concrete individuals and properties representing all entities and metrics needed for the 10 acceptances of the ALIVE Proof:
   - **Walkable GMF factory**: A walkable facility with 6+ zones, locations, exits, routes, waypoints, and connectivity matching the MUD walkthrough.
   - **Complete mech assembly line**: A mech root with sockets, subassemblies, parts, scale/grade, materials (PS, ABS, etc.), Jidoka gates, and Kanban signals.
   - **Race facility**: A racing vehicle, exactly 4 tires, engine, grip/heat classes, pit strategy, and sector time records.
   - **Market facility**: Dimensional assets, ownership records, risk/proof classes, and proof receipts.
   - **Deterministic MUD walkthrough**: Sequence of MUD commands (look, verify, go) and their outcomes.
   - **Renderable BOM / LOD / Authority / Resolution**: Assign the properties: `eden:lodClass`, `eden:materialClass`, `eden:instancingClass`, `eden:silhouetteImportanceClass`, `eden:interactionDistanceClass`. Assign the 12 authority properties to `AssemblyComponent` individuals. Assign resolution levels (ResGlobal, etc.).
2. Update `/Users/sac/.ggen/packs/eden_server/ggen.toml`:
   - Add `"ontology/instances.ttl"` to the `imports` array under `[ontology]`.
   - Add 10 generation rules under `[generation]`, each combining a SPARQL SELECT query (which MUST have an explicit `ORDER BY` clause to guarantee determinism) and a Tera template.
   - Target the output files under a `src/` subdirectory (e.g., `src/walkable_gmf_factory.txt`), NOT in the pack root, and NOT containing `output/` or `generated/` in the path (to avoid GGEN linter errors).
3. Run `ggen sync` in `/Users/sac/.ggen/packs/eden_server/` to generate all 10 files.
4. Verify that the sync completes with exit code 0, no validation warnings or errors are raised, and all 10 generated files exist and contain the correct structured data from the instances.
