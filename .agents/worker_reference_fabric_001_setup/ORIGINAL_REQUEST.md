## 2026-06-19T17:23:46Z

You are a teamwork_preview_worker agent.
Your working directory is: `/Users/sac/rocket-craft/.agents/worker_reference_fabric_001_setup`
Your task is to perform the initial directory setup and implement the reference visual extraction logic for GC-MECH-ASSET-FABRIC-001.

Follow these instructions exactly:
1. Create all required output directories under `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/`:
   - `reference/`
   - `graph/`
   - `queries/`
   - `templates/`
   - `usd/`
   - `materialx/`
   - `textures/`
   - `renders/`
   - `reports/`
   - `ocel/`
   - `receipts/`

2. Copy the reference image from `/Users/sac/Documents/Papers/61gOtV1wnAL._AC_SL1200_.jpg` to:
   - `generated/mech_assets/reference_fabric_001/reference/reference_original.jpg`
   - `references/mech/61gOtV1wnAL._AC_SL1200_.jpg` (create the `references/mech/` parent directory first)

3. Implement `scripts/extract_reference_visual_targets.py` to extract visual measurements from the reference image:
   - Use `PIL` (Pillow) library for image processing (since OpenCV/cv2 is missing in the environment).
   - The script should support being run as `python3 scripts/extract_reference_visual_targets.py`.
   - It must read the local reference image `references/mech/61gOtV1wnAL._AC_SL1200_.jpg`.
   - Calculate silhouette mask (threshold background vs foreground, assuming a light/white background, e.g. RGB average > 240 is background, else foreground). Save binary mask (black background, white foreground) to `generated/mech_assets/reference_fabric_001/reference/reference_silhouette.png`.
   - Calculate edge map using PIL's `ImageFilter.FIND_EDGES` (or equivalent) and save to `generated/mech_assets/reference_fabric_001/reference/reference_edges.png`.
   - Calculate dominant color palette and color proportions (white, dark/black, cyan, yellow/gold, and red) for foreground pixels. Save the detailed histogram/proportions to `generated/mech_assets/reference_fabric_001/reference/reference_color_histogram.json`.
   - Calculate bounding box, aspect ratio, wing span estimate, central torso mass estimate, left/right symmetry estimate, cyan weapon regions, and head/visor highlight regions.
   - Write all these calculated targets to `generated/mech_assets/reference_fabric_001/reference/reference_measurements.json`.
   - Keep the code clean, robust, and handle exceptions.

4. Run `python3 scripts/extract_reference_visual_targets.py` to generate these reference files. Verify their existence.
5. Create a detailed handoff report in `/Users/sac/rocket-craft/.agents/worker_reference_fabric_001_setup/handoff.md` detailing the output files and contents of `reference_measurements.json`.
6. Send a completion message to the parent (conversation ID: d4e41fa1-3eb0-465c-ab89-89d6805b1b6d).

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.
