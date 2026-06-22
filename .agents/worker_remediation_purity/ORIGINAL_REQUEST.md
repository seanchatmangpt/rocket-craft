## 2026-06-21T00:25:25Z
You are the Purity Remediation Worker.
Your working directory is: `/Users/sac/rocket-craft/.agents/worker_remediation_purity`.
Your parent is the Project Orchestrator (conversation ID: 4de4468a-b1c4-4f05-8ff3-cb26e5516fd2).
Your task is to remediate the Python control surface purity violation on the Rocket-Craft geometry pipeline.

Specifically, you must:
1. Freeze `patch_geometry_generator.py` and analyze it. Do not delete it yet.
2. Emit `PYTHON_MORPHOLOGY_VIOLATION_REPORT.json` containing:
   - affected file
   - hardcoded constants
   - geometry decisions found
   - generated artifacts influenced
   - claimed standing invalidated
   - required TTL replacement facts
3. Create TTL source-law replacements for every morphology decision:
   - blade scale bands, owner part id, socket attachment, edge count
   - subdivision density class, armor density band
   - curvature or sweep class
   - material zone bindings
   - metric units via QUDT
4. Create SHACL refusal rules in `110_bipedal_metric_envelope_law.ttl` (or a dedicated shape file) to:
   - refuse hardcoded Python morphology
   - refuse magic geometry constants not backed by TTL
   - refuse generated USD whose dimensions lack source-law provenance
   - refuse density escalation not selected by SPARQL
5. Refactor `patch_geometry_generator.py` (or the generator files) so it ONLY:
   - reads graph-selected rows
   - validates required graph facts exist
   - lowers TTL/SPARQL results into deterministic generated output (templates like part_mesh.usda.tera)
   - emits provenance and receipts
6. Add a negative fixture: `python_hardcoded_blade_scale_must_refuse`
7. Run the full verification pipeline:
   - source-law merge
   - SHACL validation
   - SPARQL extraction
   - ggen sync
   - fresh render
   - visual residual report
   - delete-and-resync replay
   - BLAKE3 receipt chain
8. Ensure all metrics reproduce from TTL source law with NO Python morphology decisions.
9. Demote downstream reports: set standing to CLAIM_HOLD / REFUSED.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Write your handoff report to `/Users/sac/rocket-craft/.agents/worker_remediation_purity/handoff.md` and report back.
