## 2026-06-20T01:23:03Z
**Context**: Milestone 1: F1 Patches and Controlled DOE Run for target FLAGSHIP_UE4_MECH_PLANT_001.
**Content**: Synthesize the findings of the three explorers (detailed in `/Users/sac/rocket-craft/.agents/sub_orch_implementation_aaa_ue4_mech_pack_001/explorer_geom_1_report.md`, `explorer_geom_2_report.md`, and `explorer_geom_3_report.md`).
Implement the following F1 Flagship requirements and Controlled DOE run:
1. **Chassis Cell**:
   - Replace the primitive Xform and `parallel_line_array` outputs in the ontologies (like `generator_parameters.ttl`, `core.ttl`) and templates (`part_mesh.usda.tera`) with part-scoped `angular_armor_shell`, `layered_swept_feather_panel`, `beveled_hard_surface_plate`, and `nested_mechanical_subframe` primitives.
   - Enforce unique geometry fingerprints per part by refining coordinate offsets and translation/scale/rotation values dynamically.
2. **Surface Cell**:
   - Generate complete PBR texture manifests and nonblank maps (e.g. 4K/8K texture compilation policies) for BaseColor, Normal, Roughness, Metallic, AO, Emissive, DecalMask, WearMask, PanelLineMask, and DamageMask.
3. **Rig Cell**:
   - Generate `skeleton.usda`, `sockets.json`, `joint_limits.json`, `loadout_mounts.json`, `damage_zones.json`, and `destruction_states.json` for each candidate.
4. **DOE Enumeration Cell**:
   - Implement a Python script to run a bounded combinatorial design across chassis, surface, rig, loadout, and destruction factors.
   - Generate a batch of 100+ candidates.
   - Funnel every candidate through the strict flow discipline:
     `source-law candidate -> static artifact checks -> modular USD checks -> PBR manifest checks -> rig/socket checks -> render checks -> AI Vision Judge -> UE4 cook eligibility -> UE4 cook -> final disposition`
     Stop execution of a candidate immediately at the first failure to conserve UE4 cook resources.
   - For candidate evaluation, use `just verify-flagship-ue4-mech` (or the underlying `./verify_mecha_pipeline.sh`).
5. **Output Reports**:
   - `DOE_FACTOR_MATRIX.json` (or `.csv`)
   - `CANDIDATE_DISPOSITIONS.jsonl`
   - `PARETO_FAILURE_REPORT.md` (identifying where the line fails first and which defects dominate)
   - `TRANSFER_FUNCTION_REPORT.md` (mapping X factors to Y failures)
   - `NEXT_PATCH_PRIORITY_REPORT.md` (identifying the highest yield increase patch)
   - OCEL manufacturing logs and receipt chain.

Write all of these output report files inside `/Users/sac/rocket-craft/.agents/sub_orch_implementation_aaa_ue4_mech_pack_001/` or copy them there.

**Action**: Implement the patches, execute the combinatorial DOE run funneling candidates through verification, and generate the 5 required reports. Send us a message when complete with paths to all generated reports.

## 2026-06-20T01:50:18Z
**Context**: Monitoring F1 patches and smoke run progress.
**Content**: Hi worker_doe, checking in on the progress of the 3-seed smoke batch, negative fixtures runs, and the modular identity diagnostics. Are they compiling/running successfully?
**Action**: Please reply with your current status or intermediate log outputs if available.

