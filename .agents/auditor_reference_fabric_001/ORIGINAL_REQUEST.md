## 2026-06-20T00:46:47Z

You are a teamwork_preview_auditor agent.
Your working directory is: `/Users/sac/rocket-craft/.agents/auditor_reference_fabric_001`
Your task is to run the integrity forensic verification for GC-MECH-ASSET-FABRIC-001.

Ensure you perform these checks:
1. Verify that the generated assets (`generated/mech_assets/reference_fabric_001/usd/*`, `generated/mech_assets/reference_fabric_001/materialx/*`) have been successfully generated from Turtle ontologies (`all_merged.ttl`), SPARQL queries (`queries/`), and Tera templates (`templates/`) via `ggen sync` and that NO hand-written code or block proxies exist in the manufactured files.
2. Verify that there is NO hardcoding of test results or expected values in the python scripts or generated files.
3. Validate that the visual similarity metrics (`silhouette_iou >= 0.25` and `color_palette_similarity >= 0.50`) are genuinely calculated and that the renders (`renders/render_front.png`, `renders/render_angled.png`) exist, match the master USD structure, and show a visible mech.
4. Verify the 7-event OCEL log and 12-entry cryptographic receipt chain are valid and sequential.
5. Verify the gap checker report (`gap_closure_report.json` and `.md`) and its falsification/counterfactual tests pass.
6. Create an audit handoff report at `/Users/sac/rocket-craft/.agents/auditor_reference_fabric_001/handoff.md` with your verdict (CLEAN or INTEGRITY VIOLATION) and exact findings.
7. Send a completion message to the parent (conversation ID: d4e41fa1-3eb0-465c-ab89-89d6805b1b6d).
