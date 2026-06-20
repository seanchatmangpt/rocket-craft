# Original User Request

## 2026-06-20T01:03:33Z

You are the sub-orchestrator for the Implementation Track of target `AAA_UE4_MECH_PACK_001` (GC-AAA-UE4-MECH-001). Your working directory is `/Users/sac/rocket-craft/.agents/sub_orch_implementation_aaa_ue4_mech_pack_001`.

Your mission is to implement the parallel generation swarms and source law for the mecha asset pack:
1. Deploy swarms A-D to generate modular OpenUSD and MaterialX files using `ggen` templates without manual edits:
   - Geometry/Morphology: Replace parallel lines/barcodes with layered swept feather panels for wings. Core body must have angular armor shell hierarchy and clear torso/head shoulder distinctions. Blades must be cyan rods of correct length/angle.
   - Modular USD Identity: SPARQL queries must bind `?CURRENT_PART_ID` context so that local part USD files (SM_Torso, SM_Head, SM_WingArray_Left/Right, SM_Blade_Left/Right) contain only their respective meshes, preventing assembly duplication.
   - Materials & Textures: Headless texture compilation and MaterialX lookdev.
   - Rigging & Sockets: UsdSkel rigging, joint limits, weapon sockets, and collision bounding boxes.
2. Build the IP-Distance engine to output IP distance, mecha commons, and protected trademark signatures graphs.
3. Automate the UE4 import, material binding, and cooking verification.
4. Wait for `/Users/sac/rocket-craft/TEST_READY.md` to be published at root. Once available, execute the test runner and fix any failures by modifying the source law (Turtle ontologies, SPARQL, templates) rather than generated USD files.
5. Output the sequential, Merkle-like receipt chain (`asset_receipts.jsonl`) and OCEL event logs.
6. Coordinate the implementation via worker, reviewer, and challenger subagents. Do not write code directly. Report progress in your `progress.md` file. Use conversation ID ea452791-09f7-405c-ac17-9de880041ac5 (the top-level orchestrator) for progress updates and reports.

## 2026-06-20T01:07:45Z

**Context**: Elevating target to FLAGSHIP_UE4_MECH_PLANT_001 (F1-Grade Flagship).
**Content**: We have received an emergency correction elevating the project goal from a simple game-ready pack to a $2M-$5M flagship cinematic production asset:
1. Review the new specification file at `/Users/sac/rocket-craft/.agents/SPR_FLAGSHIP_F1_PLANT.md`.
2. Configure your generation swarms to produce a flagship cinematic mecha asset meeting gates `CTQ-F1-001` through `CTQ-F1-013`.
3. Update the source law (Turtle ontologies, SPARQL, templates) to implement:
   - Destruction states (mesh variations for broken armor, exposed mechanical frame parts, and VFX attachment sockets).
   - Heavy animations (idle, walk, deploy).
   - Multiple weapon loadouts.
   - Cinematic lookdev with 4K/8K texture policies.
4. Ensure the pipeline handles the new F1-grade command: `just verify-flagship-ue4-mech`.
**Action**: Please adjust your planning, briefing, and swarm configurations to target these F1 cinematic requirements. Wait for `TEST_READY.md` to be updated by the E2E track before finalizing verification.

## 2026-06-20T01:14:08Z

**Context**: Integrating the AI Vision Judge Cell (SPR_AI_VISION_JUDGE.md).
**Content**: We have received another emergency update introducing a qualitative visual review gate:
1. Read `/Users/sac/rocket-craft/.agents/SPR_AI_VISION_JUDGE.md`.
2. The pipeline and verification runner must ensure that all generated outputs (headless renders, UE4 screenshots, and MaterialX material closeups) are compiled and prepared for qualitative scoring against the `VJ001-VJ012` rubric.
3. Assets scoring below 4.5/5.0 will be marked as `REFUSE_AS_NON_FLAGSHIP` and must trigger a source-law patch loop.
**Action**: Adjust your implementation and verification subtasks to ensure visual outputs are formatted, recorded, and ready for the AI Vision Judge evaluation.

## 2026-06-20T01:15:28Z

**Context**: Operational Update to the AI Vision Judge (SPR_AI_VISION_JUDGE.md).
**Content**: The AI Vision Judge specification has been updated to remove subjective scalar scores (4.5/5.0) in favor of binary dispositions and a structured defect taxonomy:
1. Re-read the updated `/Users/sac/rocket-craft/.agents/SPR_AI_VISION_JUDGE.md`.
2. The asset generation and verification pipeline must prepare to receive a JSON report from the AI Vision Judge with a `disposition` (`PASS_FLAGSHIP`, etc.) and a list of `critical_defects` (`VJ-CRIT-001` through `VJ-CRIT-006`).
3. If the disposition is anything other than `PASS_FLAGSHIP`, the line must stop, the defect must be recorded as a negative fixture, and the appropriate repair route must be executed on the source law.
**Action**: Align your implementation and verification subtasks to support this binary JSON report and defect-driven repair loop.

## 2026-06-20T01:21:01Z

**Context**: Controlled DOE Run (GC-FLAGSHIP-UE4-MECH-001).
**Content**: The E2E test harness has successfully verified the measurement pipeline. We are now executing the first Design of Experiments (DOE) run.
1. Read `/Users/sac/rocket-craft/.agents/SPR_CONTROLLED_DOE_RUN.md`.
2. Implement the following parallel patches:
   - **Chassis Cell**: Replace primitive Xform and `parallel_line_array` with part-scoped `angular_armor_shell`, `layered_swept_feather_panel`, `beveled_hard_surface_plate`, and `nested_mechanical_subframe` primitives. Enforce unique geometry fingerprints per part.
   - **Surface Cell**: Generate complete PBR texture manifests and nonblank maps for BaseColor, Normal, Roughness, Metallic, AO, Emissive, DecalMask, WearMask, PanelLineMask, and DamageMask.
   - **Rig Cell**: Generate `skeleton.usda`, `sockets.json`, `joint_limits.json`, `loadout_mounts.json`, `damage_zones.json`, and `destruction_states.json` for every candidate.
3. Once patched, configure the **DOE Enumeration Cell** to run a bounded combinatorial design across chassis, surface, rig, loadout, and destruction factors.
4. Produce a batch of 100+ candidates and funnel every candidate through `just verify-flagship-ue4-mech`.
5. Classify every failure and emit a Pareto report of refusal causes mapping the X->Y transfer function.
**Action**: Execute the patches and run the combinatorial DOE, reporting back with the Pareto report when complete.

## 2026-06-20T01:30:40Z

**Context**: Modular USD Identity & Part Scope Conformance (SPR_MODULAR_IDENTITY.md).
**Content**: We have received an emergency update regarding a source-law violation at the `MODULAR_USD` station:
1. Review the updated `/Users/sac/rocket-craft/.agents/SPR_MODULAR_IDENTITY.md`.
2. Stop the DOE launch immediately. The `SM_Torso.usda` file currently renders foreign geometry (arms, chains, side modules), violating modular USD part scope.
3. Preserve the current invalid torso file as a negative fixture named `torso_contains_foreign_parts`.
4. Apply a source-law patch to:
   - Restrict `SM_Torso.usda` to torso prims and sockets only. Ensure: `∀ prim ∈ SM_Torso.usda: prim.owner_part_id == "torso" OR prim.kind == "socket"`.
   - Connected assemblies must be emitted as `socket` or mount declarations, NOT geometry.
   - Enforce the new LSP diagnostics (`USD303`, `USD307`, `USD308`, `USD309`, `USD310`).
5. Ensure the verifier rejects this negative fixture with `REFUSE_MODULAR_USD` at the `MODULAR_USD` gate before it consumes downstream queue capacity.
**Action**: Halt the DOE run, save the negative fixture, patch the generator source law, implement the new diagnostics, verify that the negative fixture is rejected correctly, and only then restart the DOE run.

## 2026-06-20T01:33:08Z

**Context**: Additional Modularity Rules and Smoke Run (SPR_MODULAR_IDENTITY.md).
**Content**: We have received another emergency update to protect against geometry smuggling inside sockets:
1. Read the updated `/Users/sac/rocket-craft/.agents/SPR_MODULAR_IDENTITY.md`.
2. Implement `USD311` (socket prim contains mesh payload) and `USD312` (part file references assembly root) check gates in the verifier.
3. Deploy two more negative fixtures for pre-flight testing:
   - `socket_contains_mesh_payload`
   - `assembly_reference_inside_part_file`
4. Do NOT launch the 100-seed DOE batch yet.
5. First, run a 3-seed `MODULAR_IDENTITY_SMOKE` batch to verify that the generator and verifier satisfy:
   - owner_part_id scope check.
   - forbidden family check.
   - bounding envelope check.
   - geometry fingerprint uniqueness check.
   - no full-assembly reference/payload check.
   - socket-only outward relation check.
6. Once the 3-seed smoke batch passes cleanly with USD303-USD312 all successfully validated, you are cleared to proceed with the 100-seed DOE batch and generate the five reports.
**Action**: Implement `USD311`/`USD312`, verify the negative fixtures, run the 3-seed smoke batch, and proceed with the DOE batch only after the smoke batch passes.

## 2026-06-20T01:34:54Z

**Context**: Enforcing Modular Identity Andon Hold (SPR_MODULAR_IDENTITY.md).
**Content**: We have received another operational instruction to maintain the Andon Hold and patch the geometry generation code:
1. Maintain the Andon Hold. Do NOT launch the 100-seed DOE batch yet.
2. In your geometry generation swarms, patch `nexus-gundam/src/mech_primitives.rs` and the USD emission path to ensure:
   - Every part file is strictly `owner_part_id` scoped.
   - Socket declarations contain NO mesh payloads.
   - Assembly/root references are strictly forbidden inside part-local files.
3. Run the 3-seed `MODULAR_IDENTITY_SMOKE` batch.
4. Emit the required smoke reports, including `MODULAR_IDENTITY_SMOKE_REPORT.json`.
5. Prove that the 5 required negative fixtures (including `torso_contains_foreign_parts` and `socket_contains_mesh_payload`) are halted at the Chassis station with `REFUSE_MODULAR_USD` and do not consume downstream verification cycles.
6. When the 3-seed smoke batch passes and all negative fixtures are rejected, you are cleared to release the 100-seed DOE.
**Action**: Implement the `nexus-gundam/src/mech_primitives.rs` patch, run the smoke batch, verify the negative fixtures are rejected at the Chassis station, emit the reports, and only then proceed to the 100-seed DOE batch.
