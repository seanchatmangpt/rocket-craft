## 2026-06-19T17:51:50Z
Your identity: You are Worker 4 (archetype: worker/teamwork_preview_worker).
Your working directory is /Users/sac/rocket-craft/.agents/worker_modularity
Your task: Implement the Modular Identity checks (USD300 series) in `crates/ggen-asset-lsp`.

Specifically, you must:
1. Update `crates/ggen-asset-lsp/src/diagnostics.rs` to support the new `USD300` series diagnostic taxonomy for modularity failures.
2. Read the spec in `/Users/sac/rocket-craft/.agents/SPR_MODULAR_IDENTITY.md` carefully.
3. In `diagnostics.rs` (inside `run_diagnostics` function), check all `.usda` files in the asset `usd/` directory (e.g. `generated/mech_assets/reference_fabric_001/usd/`):
   - USD301: If two or more part files (excluding the master `ASSET_ReferenceFabric_001.usda`) share the exact same geometry fingerprint (e.g., identical file contents or identical hashes), emit a Diagnostic with code "USD301" and message "USD301 ERROR: duplicate USD geometry fingerprint" on the first line.
   - USD302: If a part file (e.g. `SM_Head.usda`) is a copy of or renders the full assembly (for example, if it contains the full assembly root `/World` or defines all component types), emit a Diagnostic with code "USD302" and message "USD302 ERROR: part file renders full assembly" on the first line.
   - USD303: If a part file contains foreign component prims (for example, if `SM_Head.usda` contains a prim declaration for a `Torso` or `Wing` or `Blade` component, where each part file is only allowed to contain its own local components), emit a Diagnostic with code "USD303" and message "USD303 ERROR: part-local file contains foreign component prims" on the matching prim declaration line.
     Define the local boundaries:
     - `SM_Head.usda`: only head-local prims.
     - `SM_Torso.usda`: only torso-local prims.
     - `SM_Blade_Left.usda`: only blade-left-local prims.
     - `SM_Blade_Right.usda`: only blade-right-local prims.
     - `SM_WingArray_Left.usda`: only left-wing-local prims.
     - `SM_WingArray_Right.usda`: only right-wing-local prims.
   - USD304: If a part file is missing its expected part root prim (e.g., `SM_Head.usda` must have a prim declaration for `SM_Head` or `Head`), emit a Diagnostic with code "USD304" and message "USD304 ERROR: expected part root missing" on the first line.
   - USD305: If mirrored variant files (like left and right wings/blades) have identical mesh coordinates or identical file content without any transform/sign inversion (e.g. having exactly identical translations/scaling instead of sign inversion on the X axis), emit a Diagnostic with code "USD305" and message "USD305 ERROR: mirrored part lacks mirror transform proof" on the first line.
   - USD306: If part files share identical source template expansion (exactly identical duplicate copies), emit a Diagnostic with code "USD306" and message "USD306 ERROR: generated USD files share identical source template expansion" on the first line.
   - USD307: If a part file specifies bounding box extents (e.g., `float3[] extents` or `double3[] extents` or `extents = `) that exactly match the full asset's bounds (suggesting it was not properly isolated), emit a Diagnostic with code "USD307" and message "USD307 ERROR: part bounding box overlaps full-asset bounds" on the first line.
4. Add robust unit tests covering all these modularity errors (`USD301` to `USD307`) and assert they are triggered correctly on the expected lines.
5. Rerun all tests via `cargo test -p ggen-asset-lsp` and ensure all tests pass cleanly.
6. Write your report to `/Users/sac/rocket-craft/.agents/worker_modularity/handoff.md` and send a message back to the orchestrator.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.
