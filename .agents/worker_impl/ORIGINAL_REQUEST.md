## 2026-06-20T00:32:46Z
Your identity: You are Worker 2 (archetype: worker/teamwork_preview_worker).
Your working directory is /Users/sac/rocket-craft/.agents/worker_impl
Your task: Implement the Core LSP Server & Diagnostics and Code Actions for the Asset Manufacturing LSP (ggen-asset-lsp) inside `crates/ggen-asset-lsp/`.

Specifically, you need to create/update:
1. `crates/ggen-asset-lsp/Cargo.toml`: Add dependencies on `serde_json`, `walkdir`, and any other crates you need.
2. `crates/ggen-asset-lsp/src/main.rs`: Delegate to your server implementation.
3. `crates/ggen-asset-lsp/src/server.rs`: Implement the `LanguageServer` trait from `lsp-max`. Keep track of the workspace root and run checks when files are opened, changed, or saved.
4. `crates/ggen-asset-lsp/src/diagnostics.rs`: Implement parsing and linting of `.usda` and `.mtlx` files inside `generated/mech_assets/reference_fabric_001/` to detect:
   - Missing payloads (e.g. if a Mesh prim is missing `payload = @mesh.usd@` or does not contain `payload = @`).
   - Missing material bindings (e.g. if a prim does not contain a `rel material:binding` statement, or if it points to a material name that is not defined in any `.mtlx` file).
   - Unreceipted USD prims (e.g. if a prim named `Panel_001` is not receipted. A prim is receipted if a file like `generated/mech_assets/reference_fabric_001/receipts/<prim_name>.json` exists, or if `receipt.json` exists in that directory and references the prim).
   - Headless render outputs (`visual_gap_report.json`): Parse this file. If `silhouette_iou` is < 0.90, or `status` is `"FAILED"`, project a diagnostic error on the root Xform/Mesh (the first `def` line in the USDA file).
   - `usdchecker` logs (`reports/usdchecker.log`): Parse lines. If there are failures on specific prim paths, project them onto the matching prim name declaration lines in the USDA.
5. `crates/ggen-asset-lsp/src/code_actions.rs`: Implement LSP Code Actions that point to the generator parameter source (e.g., templates `template.usda.tera` / `part_mesh.usda.tera`, or `generator_parameters.ttl`) for repair instead of editing the generated USD files. Provide title options like:
   - "Fix payload reference in template.usda.tera"
   - "Add material binding in generator_parameters.ttl"
   - If there is a `# ggen-source: <uri>` comment, provide "Edit source parameter in generator_parameters.ttl for <uri>".
6. `crates/ggen-asset-lsp/src/ocel.rs`: Implement OCEL event logging. Emit OCEL events (e.g. `Validate` and `Repair` activities) to `generated/mech_assets/reference_fabric_001/ocel/lsp_log.json` whenever validation or code actions are triggered.
7. Verify your implementation by running `cargo check -p ggen-asset-lsp` to ensure it compiles cleanly with zero errors.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.
