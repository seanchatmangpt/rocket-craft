## 2026-06-21T00:06:28Z
You are the Upper Body Sculpting Candidate Worker.
Your working directory is: `/Users/sac/rocket-craft/.agents/worker_upper_sculpting_m3`.
Your parent is the Project Orchestrator (conversation ID: 4de4468a-b1c4-4f05-8ff3-cb26e5516fd2).
Your task is to prepare candidate source-law patches for Milestone 3 (Upper Body Geometry: Head, Torso, Shoulders).

Objective:
1. Sculpt the blocky torso into layered chest armor, waist, abdomen, and inner frame.
2. Sculpt the blocky head into a detailed helmet with cheek guards, face depth, and neck connection.
3. Sculpt the shoulder blocks into pauldrons.
4. Integrate the parts with correct connectivity sockets pointing outward (from torso to head/limbs/wings/loadouts). Sockets must not contain mesh payloads.

Work scope:
- Investigate `ontology/source_law/104_reference_fabric.ttl` and `generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera`.
- Modify/create TTL files in `ontology/source_law/` defining the upper body geometry primitives, and edit `part_mesh.usda.tera` to support detailed chest plates, helmet armor layers, and pauldrons.
- Note: Your outputs must remain `CANDIDATE_GEOMETRY`. Do not claim ADMITTED or final standing. Compile and run verify_asset.sh locally to check metrics (silhouette_iou, shape bands) but keep status as candidate.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Write your final candidate patches and verification outcomes to `/Users/sac/rocket-craft/.agents/worker_upper_sculpting_m3/handoff.md` and report back.
