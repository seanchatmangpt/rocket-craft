## 2026-06-19T17:26:30Z
You are a teamwork_preview_worker agent.
Your working directory is: `/Users/sac/rocket-craft/.agents/worker_reference_fabric_001_generation`
Your task is to define the ontology triples, SPARQL queries, Tera templates, and generation rules in `ggen.toml`, then run `ggen sync` to manufacture the OpenUSD and MaterialX files for GC-MECH-ASSET-FABRIC-001.

Follow these instructions exactly:
1. Write the Turtle ontology files under `generated/mech_assets/reference_fabric_001/graph/`:
   - `asset_fabric.ttl`: Defines classes and instances representing our mech part grammar (torso_core, head_unit, wing_root_left/right, primary_wing_feathers_left/right, secondary_wing_feathers_left/right, blade_left/right, backpack_core, thruster_cluster, shoulder_left/right, arm_left/right, leg_left/right, v_fin_left/right) and geometry primitive families.
   - `visual_targets.ttl`: Contains the measured target values from the setup phase (e.g. aspect ratio, symmetry, color proportions, bounding box).
   - `generator_parameters.ttl`: Defines generator parameters and geometry primitive instances. Make sure to define exactly 120 or more primitive instances, including at least 48 wing-feather panels (24 left, 24 right), and bind them to the 4 materials (M_WhiteArmor, M_CyanBlade, M_DarkFrame, M_GoldVisor).
   Ensure all Turtle files use prefix `mud:` mapping to `https://rocket-craft.com/ontology/mud#` (matching the project namespace prefix) or `core:` mapping to `https://ggen.io/ontology/core/`.

2. Append these new triples or load them into `/Users/sac/rocket-craft/ontology/all_merged.ttl` to ensure `ggen sync` reads them. You can append the Turtle content of your graph files directly to the end of `ontology/all_merged.ttl` (ensuring valid syntax and namespace prefix definitions).

3. Write the required SPARQL queries under `generated/mech_assets/reference_fabric_001/queries/`:
   - `candidate_parts.rq`: Selects the list of parts and their properties.
   - `usd_prims.rq`: Selects all geometry primitives with their part, type, transforms (translateX/Y/Z, scaleX/Y/Z, rotateX/Y/Z), and material binding. MUST use an explicit `ORDER BY` clause.
   - `materials.rq`: Selects the materials and their parameters (color, roughness, emissive, metallic). MUST use an explicit `ORDER BY` clause.
   - `texture_programs.rq`: Selects texture specifications. MUST use an explicit `ORDER BY` clause.
   - `verifier_expectations.rq`: Selects target metrics from visual targets. MUST use an explicit `ORDER BY` clause.

4. Write the Tera templates under `generated/mech_assets/reference_fabric_001/templates/`:
   - `templates/usd/asset.usda.tera`: Generates the master USD file `usd/ASSET_ReferenceFabric_001.usda` referencing materials and referencing the part meshes.
   - `templates/usd/part_mesh.usda.tera`: Loops over results and generates USD geometry meshes (Torso, Head, etc.) using custom vertex data for tapered boxes, feather panels, blade prisms, cylinders, etc., and applies translation/rotation/scale transforms and material bindings.
   - `templates/materialx/materials.mtlx.tera`: Generates MaterialX `.mtlx` files for looking up white armor, cyan blade, etc.
   - `templates/texture_program.rs.tera`: A template representing texture program configuration.
   - `templates/visual_gap_report.md.tera`: A template for report generation.

5. Update `/Users/sac/rocket-craft/ggen.toml` to add the `[[generation.rules]]` blocks for generating the 11 target USD and MaterialX files:
   - `ASSET_ReferenceFabric_001.usda`
   - `SM_Torso.usda`
   - `SM_Head.usda`
   - `SM_WingArray_Left.usda`
   - `SM_WingArray_Right.usda`
   - `SM_Blade_Left.usda`
   - `SM_Blade_Right.usda`
   - `M_WhiteArmor.mtlx`
   - `M_CyanBlade.mtlx`
   - `M_DarkFrame.mtlx`
   - `M_GoldVisor.mtlx`
   Ensure each rule uses the correct query and template files you created.

6. Run `ggen sync` (or `make build`) to trigger generation. Check the output files under `generated/mech_assets/reference_fabric_001/usd/` and `materialx/` to confirm that all 11 files are successfully generated and contain actual meshes and material definitions (not empty placeholders).
7. Create a handoff report in `/Users/sac/rocket-craft/.agents/worker_reference_fabric_001_generation/handoff.md` detailing the queries, templates, rules, and generated hashes.
8. Send a completion message to the parent (conversation ID: d4e41fa1-3eb0-465c-ab89-89d6805b1b6d).
