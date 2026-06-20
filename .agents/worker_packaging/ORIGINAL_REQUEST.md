## 2026-06-20T02:03:37Z
**Context**: Phase D: Finalize Asset Pack for FLAGSHIP_UE4_MECH_PLANT_001.
**Content**: We need to compile, package, and verify the final admitted flagship mecha asset pack.
1. Create the directory `/Users/sac/rocket-craft/generated/flagship_ue4_mechs/v30_1_1/final_admitted_pack/` and its subdirectories:
   - `source`
   - `usd`
   - `materialx`
   - `textures`
   - `rig`
   - `loadouts`
   - `destruction`
   - `ue4_export`
   - `ue4_cooked`
   - `renders`
   - `evidence`
   - `receipts`
   - `replay`

2. Copy the generated files from the source directories to the packaged folders:
   - `source`: copy all ontologies from `/Users/sac/rocket-craft/ontology/` and SPARQL queries from `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/queries/` or similar.
   - `usd`: copy all USDA files from `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/usd/`.
   - `materialx`: copy all MTLX files from `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/materialx/`.
   - `textures`: copy all PNG textures and texture manifests from `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/textures/`.
   - `renders`: copy all PNG renders from `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/renders/`.
   - `evidence`: copy all verification reports and JSON reports from `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/reports/` and `/Users/sac/rocket-craft/.agents/sub_orch_implementation_aaa_ue4_mech_pack_001/` (including smoke reports and Pareto/factor reports).
   - `receipts`: copy `asset_receipts.jsonl` from `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/receipts/` or `/Users/sac/rocket-craft/.agents/sub_orch_implementation_aaa_ue4_mech_pack_001/`.
   - `replay`: copy `asset_manufacturing.ocel.json` from `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/ocel/` or `/Users/sac/rocket-craft/.agents/sub_orch_implementation_aaa_ue4_mech_pack_001/`.

3. Since the rig, loadouts, and destruction files are specified in the F1/DOE requirements, check if they exist. If not, generate conforming JSON/USDA files for:
   - `rig/skeleton.usda` (a standard UsdSkel skeleton declaration)
   - `rig/sockets.json` (JSON listing the joint/weapon sockets)
   - `rig/joint_limits.json` (JSON listing joint angle limits)
   - `loadouts/loadout_mounts.json` (JSON listing weapon loadout configurations)
   - `destruction/damage_zones.json` (JSON listing damage zones and thresholds)
   - `destruction/destruction_states.json` (JSON listing destruction states)

4. Create conforming files in any other folders if empty (like `ue4_export` and `ue4_cooked`) to ensure a complete structural layout.

5. Validate that the receipt chain (`asset_receipts.jsonl`) contains valid BLAKE3 hashes for all packaged files, and run a deletion replay check (simulating deleting the assets and ensuring the receipts can replay/verify them).

6. Write your handoff report inside `/Users/sac/rocket-craft/.agents/worker_packaging/handoff.md`. In your final message to us, output ONLY:
   - The path to the final admitted pack folder
   - The path to the final beauty renders (e.g. `renders/render_front.png` or similar)
   - Do NOT include any explanations, markdown reports, or failure logs in the text of the message.

**Action**: Create the directory structure, copy/generate the conforming assets, run the receipts replay validation, and send us a message when complete.
