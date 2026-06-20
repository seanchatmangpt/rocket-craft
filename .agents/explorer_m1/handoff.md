# Handoff Report - Asset Manufacturing LSP (ggen-asset-lsp) Architecture

## 1. Observation

Direct observations of the lsp-max framework and the rocket-craft workspace files:

### 1.1 lsp-max Core and Examples
* **Location**: `/Users/sac/lsp-max`
* **Crates Structure**: Standard cargo workspace containing `lsp-max` (server core), `lsp-max-protocol` (types and `max/*` methods), `lsp-max-runtime` (state machine), and `examples/`.
* **LanguageServer Trait**: Defined in `/Users/sac/lsp-max/src/language_server.rs` with default implementations for all LSP 3.18 methods.
* **powl-lsp Reference**: `/Users/sac/lsp-max/examples/powl-lsp/src/server.rs` overrides handlers like `initialize`, `did_open`, `did_change`, and `did_save` using `Client` to publish diagnostics (`self.client.publish_diagnostics`).
* **anti-llm-cheat-lsp Reference**: `/Users/sac/lsp-max/examples/anti-llm-cheat-lsp/src/server.rs` overrides `code_action` (returning `Option<CodeActionResponse>`), which collects Quickfix actions from `recommend::repair_actions` defined in `src/server/recommend.rs`. It attaches originating diagnostics to the code action:
  ```rust
  CodeActionOrCommand::CodeAction(CodeAction {
      title: format!("{}: {}", d.code, brief(&d.required_correction)),
      kind: Some(CodeActionKind::QUICKFIX),
      diagnostics: Some(vec![d.to_lsp()]),
      command: Some(Command {
          title: "Open receipt ledger".to_string(),
          command: "anti-llm.openReceiptLedger".to_string(),
          arguments: None,
      }),
      ..Default::default()
  })
  ```

### 1.2 Asset Directory `generated/mech_assets/reference_fabric_001/`
* **Directories present**: `graph`, `materialx`, `ocel`, `queries`, `receipts`, `reference`, `renders`, `reports`, `templates`, `textures`, `usd` (observed from `list_dir`).
* **Asset Population**: USD, MaterialX, and report directories are currently empty, as the manufacturing worker agent (`worker_reference_fabric_001_generation`) has not yet generated them.
* **Reference Targets**: The folder `reference/` contains the following extracted targets (from `extract_reference_visual_targets.py`):
  * `reference_original.jpg` (SHA-256: `7693fdb87e7fc7f9151550830e6f5447f8ba8d1912f4c39bc06ec71467f14f27`)
  * `reference_silhouette.png` (binary mask)
  * `reference_edges.png` (PIL Find Edges map)
  * `reference_color_histogram.json` (dominant color palette proportions)
  * `reference_measurements.json`:
    * aspect_ratio: `1.2024048096192386`
    * wing_span_estimate_px: `1200`
    * central_torso_mass_estimate: `torso_pixel_count: 294339, ratio: 0.3008, density: 0.8170`
    * left_right_symmetry_estimate: `0.9594455577822312`
    * cyan_weapon_regions and head_visor_highlight_regions metadata.

### 1.3 Generator Parameter Sources in `rocket-craft`
* **Ontology**: Merged Turtle file at `/Users/sac/rocket-craft/ontology/all_merged.ttl`. Target source triples for `reference_fabric_001` are planned to be written to:
  * `generated/mech_assets/reference_fabric_001/graph/asset_fabric.ttl` (mech part grammar)
  * `generated/mech_assets/reference_fabric_001/graph/generator_parameters.ttl` (120+ primitive instances and parameter bindings)
  * `generated/mech_assets/reference_fabric_001/graph/visual_targets.ttl` (extracted measurements)
* **SPARQL Queries**: Planned to be written under `generated/mech_assets/reference_fabric_001/queries/`:
  * `usd_prims.rq` (selects primitives with transforms/materials, ordered by key)
  * `materials.rq` (selects material parameters)
  * `verifier_expectations.rq` (selects target metrics)
* **Tera Templates**: Planned to be written under `generated/mech_assets/reference_fabric_001/templates/`:
  * `templates/usd/asset.usda.tera` (generates master USD file)
  * `templates/usd/part_mesh.usda.tera` (generates mesh geom prims)
  * `templates/materialx/materials.mtlx.tera` (generates material definitions)
* **Configuration Mapping**: Declarations are registered in `/Users/sac/rocket-craft/ggen.toml` as `[[generation.rules]]`.

---

## 2. Logic Chain

1. **Diagnostics Surface**: The compiler surface includes generated assets (`.usda`, `.mtlx`) and validation outputs (`usdchecker` logs, `visual_gap_report.json`).
2. **Line and Range Resolution**:
   * For `usdchecker` logs (e.g. `Failed verification on prim /World/Torso/Panel_001: Mesh has no normals`), the LSP can scan the generated USDA file for the declaration `def Mesh "Panel_001"` or `def "Panel_001"` to resolve the line number and range, then project a `PublishDiagnostics` error on that line.
   * For `visual_gap_report.json` errors (e.g. `silhouette_iou < 0.90`), the error represents a global failure of the assembly mesh. The LSP should project this error on the root Xform definition in the master `ASSET_ReferenceFabric_001.usda` file (typically line 1 or the main Xform prim).
   * For unresolved material bindings or missing payloads, the LSP can parse the generated `.usda` syntax, check paths, and highlight the faulty property lines.
3. **Traceability to Generator Sources**:
   * The rule definition in `ggen.toml` links each generated file to its template and SPARQL query.
   * Inside the template (`part_mesh.usda.tera`), the output is structured by looping over query rows.
   * Each query row in `usd_prims.rq` corresponds to an RDF resource (instance) in `generator_parameters.ttl` (e.g. `mud:TorsoPrimitive_001`).
   * By including source metadata (e.g., as comments `# ggen-source: mud:TorsoPrimitive_001` or as custom OpenUSD metadata attributes `custom string ggen:source_uri` on the prim), the LSP can instantly map any generated prim in the `.usda` back to its source RDF resource.
4. **Code Actions Actuation**:
   * Quickfix actions must point to the *source* files, not the generated files, since the generated files are overwritten during `ggen sync`.
   * For a given diagnostic on `/World/Torso/Panel_001` in `SM_Torso.usda`:
     * The LSP identifies the source RDF resource `mud:TorsoPrimitive_001`.
     * It searches `generator_parameters.ttl` (and other loaded Turtle files) for the text block defining `mud:TorsoPrimitive_001`.
     * It constructs an LSP `CodeAction` with a `WorkspaceEdit` that targets `generator_parameters.ttl` at the resolved range, or a `Command` (like `vscode.open` or a custom LSP command) to jump to the Turtle definition or the `part_mesh.usda.tera` template line.

---

## 3. Caveats

* The asset directory `generated/mech_assets/reference_fabric_001/` currently does not contain the generated USDA and MaterialX files. We assume their schema, structure, and query maps conform to standard GGen compiler projections.
* Tracing generated prims back to their Turtle sources is highly robust if the templates append metadata comments (e.g., `# ggen-source: <uri>`) or custom attributes to the generated USDA. If this metadata is omitted, the LSP must fall back to resolving the name (e.g. `Panel_001`) via executing the SPARQL query against the merged ontology, which adds runtime overhead.

---

## 4. Conclusion & Architectural Recommendation

We recommend the following architecture for `ggen-asset-lsp`:

### 4.1 Crate Structure
* Establish `crates/ggen-asset-lsp` depending on `lsp-max`, `lsp-types-max`, `serde_json`, and a basic TTL/SPARQL parser.
* Implement `AssetLsp` implementing `lsp_max::LanguageServer`.

### 4.2 Diagnostic Resolution and Projections
* **usdchecker parser**: The server monitors changes to `.usda` files. It parses `usdchecker` output logs and maps warnings/errors to specific prim line numbers in the editor.
* **Visual Gap parser**: On save, it parses `reports/visual_gap_report.json`. If `status` is `FAILED` or a key metric (e.g. `silhouette_iou`) is below the threshold, it publishes a diagnostic on the root `Xform` prim of the master `.usda` file.
* **Static Asset Linter**: Detects missing payload references (empty paths or missing files) and invalid material bindings by verifying if the targeted `.mtlx` path exists and contains the material.

### 4.3 Diagnostic Mapping and Code Actions
* **Turtle Trace Mapping**: The LSP reads `ggen.toml` to link output files to templates/queries, and parses comments (or attributes) in the generated `.usda` files to locate the source RDF resource URI.
* **LSP Code Actions**:
  * `Go to Source Parameter`: Traces the prim name to its RDF resource in `generator_parameters.ttl`, resolves its range, and offers a navigation command.
  * `Go to Generator Template`: Resolves the template path in `ggen.toml` (e.g. `part_mesh.usda.tera`) and maps the prim type to the template file range.
  * `Fix Material Binding`: If a material binding is missing, queries the ontology for valid material URIs and offers a quickfix edit to change the `mud:materialBinding` property inside `generator_parameters.ttl`.

### 4.4 OCEL Event Integration
* Emit OCEL event logs (`Validate` and `Repair` activities) on diagnostic evaluation and Code Action execution to feed the process-mining loop.

---

## 5. Verification Method

1. **Verify Report Location**: Check that this report exists at `/Users/sac/rocket-craft/.agents/explorer_m1/handoff.md`.
2. **Review lsp-max server loop**: Run `cargo check --examples` in `/Users/sac/lsp-max` to confirm that all lsp-max types build successfully.
3. **Verify reference measurements path**: Verify `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/reference/reference_measurements.json` contains valid dimensions.
