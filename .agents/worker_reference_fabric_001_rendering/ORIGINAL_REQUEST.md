## 2026-06-19T17:34:06-07:00
You are a teamwork_preview_worker agent.
Your working directory is: `/Users/sac/rocket-craft/.agents/worker_reference_fabric_001_rendering`
Your task is to implement the procedural texture generation, headless rendering, similarity comparison, and evidence generation (OCEL, Receipts) for GC-MECH-ASSET-FABRIC-001.

Follow these instructions exactly:
1. Implement the procedural texture generator. It should output the following files:
   - `generated/mech_assets/reference_fabric_001/textures/T_WhiteArmor_BaseColor.png`: 512x512 white image with subtle grid/panel details.
   - `generated/mech_assets/reference_fabric_001/textures/T_WhiteArmor_Roughness.png`: 512x512 light gray image.
   - `generated/mech_assets/reference_fabric_001/textures/T_WhiteArmor_Normal.png`: 512x512 flat normal map `[128, 128, 255]`.
   - `generated/mech_assets/reference_fabric_001/textures/T_CyanBlade_Emissive.png`: 512x512 cyan image.
   - `generated/mech_assets/reference_fabric_001/textures/texture_manifest.json`: JSON listing these textures, dimensions, and purposes.
   You can write a python helper or integrate this directly into `scripts/render_reference_fabric.py`.

2. Implement `scripts/render_reference_fabric.py`:
   - It should generate the procedural textures if missing.
   - It should run the headless renderer `/usr/bin/usdrecord` (using `subprocess` or `os.system`) to render front and angled views of the generated master USD:
     `generated/mech_assets/reference_fabric_001/usd/ASSET_ReferenceFabric_001.usda`
   - It should output to `generated/mech_assets/reference_fabric_001/renders/render_front.png` and `generated/mech_assets/reference_fabric_001/renders/render_angled.png`. (If no custom cameras are set, usdrecord will frame the scene automatically, which is acceptable. If cameras are set in the USD file, specify them via `-camera` or `--camera` flags, or just let usdrecord default-frame).
   - It must post-process `renders/render_front.png` using `PIL` to output:
     - `generated/mech_assets/reference_fabric_001/renders/render_silhouette.png`: Binary silhouette mask where foreground is white (255) and background is black (0) (assume background in the render is transparent or black/white, threshold accordingly).
     - `generated/mech_assets/reference_fabric_001/renders/render_edges.png`: Edge detection map using PIL's ImageFilter.FIND_EDGES.

3. Implement `scripts/compare_reference_render.py`:
   - It must compare `renders/render_front.png` and its silhouette/edges with `reference_silhouette.png` and `reference_edges.png`.
   - It must calculate the required metrics:
     - `silhouette_iou`: Intersection over Union of the render silhouette vs reference silhouette.
     - `edge_similarity`: Cross-correlation or pixel similarity of the edge maps.
     - `color_palette_similarity`: Compare proportions of white, dark, cyan, yellow, and red colors.
     - `cyan_region_similarity`
     - `symmetry_delta`
     - `wing_span_delta`
     - `body_mass_delta`
     - `usd_prim_count` (parse/count lines/objects in `ASSET_ReferenceFabric_001.usda`)
     - `material_binding_count` (e.g. count material binds in `ASSET_ReferenceFabric_001.usda`)
     - `wing_feather_count` (count feather panels)
   - It must write the output to `generated/mech_assets/reference_fabric_001/reports/visual_gap_report.json`.
   - Ensure the threshold check: `silhouette_iou >= 0.25` and `color_palette_similarity >= 0.50`.
   - Output the reports: `visual_gap_report.md`, `verifier_report.json`, `verifier_report.md`.

4. Generate the Object-Centric Event Log (OCEL):
   - Output to `generated/mech_assets/reference_fabric_001/ocel/asset_manufacturing.ocel.json`.
   - Include the events: `CV_Extraction`, `Ontology_Merge`, `Ggen_Sync_Compilation`, `Texture_Generation`, `USD_Rendering`, `Visual_Comparison`, `Verification`.
   - Link these events to the created files/objects.

5. Generate the Cryptographic Receipts Chain:
   - Output to `generated/mech_assets/reference_fabric_001/receipts/asset_receipts.jsonl`.
   - Hash all emitted artifacts (using SHA-256 or BLAKE3) and record the receipt entries.

6. Run `python3 scripts/render_reference_fabric.py` followed by `python3 scripts/compare_reference_render.py` to produce all these files and verify their contents.
7. Create a handoff report in `/Users/sac/rocket-craft/.agents/worker_reference_fabric_001_rendering/handoff.md` detailing the implemented scripts, metrics calculated, and hashes.
8. Send a completion message to the parent (conversation ID: d4e41fa1-3eb0-465c-ab89-89d6805b1b6d).
