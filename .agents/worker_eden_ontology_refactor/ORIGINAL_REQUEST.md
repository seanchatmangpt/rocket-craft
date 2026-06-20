## 2026-06-18T19:00:49-07:00

You are the Ontology Refactoring Worker (teamwork_preview_worker). Your working directory is /Users/sac/rocket-craft/.agents/worker_eden_ontology_refactor/.

Your task is to refactor the entire `eden_server` ontology registry to Level 5 Combinatorial Maximalist graphs with OWL 2 DL restrictions, metadata alignment, and native SHACL validation shapes. You also need to wire the validation harness into the master `ggen.toml` manifest.

### Requirements:
1. **Refactor Core Ontology Graphs**:
   - Refactor `pack.ttl`, `deltas.ttl`, `bandai_tps.ttl`, `egp_racing.ttl`, `mars_market.ttl` located in `/Users/sac/.ggen/packs/eden_server/ontology/`.
   - Ensure they are valid Turtle syntax and adhere to strict OWL 2 DL restrictions (fully type all classes, object properties, datatype properties, and align metadata using Dublin Core, SKOS, or RDFS labels/comments).
   - Bind all telemetry state properties (damageClass, stressClass, heatClass, fatigueClass, gripClass, riskClass, proofClass) to `xsd:unsignedByte` (value range [0, 255]).
   - Implement deep `owl:equivalentProperty` mappings or subproperties to public standard ontologies (FIBO, SOSA, QUDT, PROV-O) as currently declared in the import headers.
   - Add structural OWL cardinalities (e.g. `eden:MechRoot` has exactly one `tps:ScaleGrade`).

2. **Implement SHACL Validation Shapes**:
   - Write explicit SHACL shapes in `/Users/sac/.ggen/packs/eden_server/ontology/validation_shapes.ttl`.
   - Enforce byte-class typestate boundaries (values of properties `eden:damageClass`, `eden:stressClass`, `eden:heatClass`, `eden:fatigueClass`, `egp:gripClass`, `egp:heatClass`, `mars:riskClass`, `mars:proofClass` must be of type `xsd:unsignedByte` and between 0 and 255 inclusive).
   - Enforce structural constraints: A racing vehicle root/chassis must have exactly 4 tires (`egp:Tire`) connected via `eden:hasSocket` and `eden:plugsInto` property paths. Specifically, verify that for any `egp:VehicleRoot` (or chassis), the path `(eden:hasSocket/^eden:plugsInto)` connects to exactly 4 tires.
   - Enforce that a `mars:DimensionalAsset` must have at least one cryptographic receipt chain proof state (`mars:proofClass`).

3. **Wire ggen.toml Validation Harness**:
   - Write/configure `/Users/sac/.ggen/packs/eden_server/ggen.toml` to declare the main ontology (`pack.ttl`), import all other ttl files, set `strict_mode = true`, and include SHACL validation paths.
   - Add SPARQL ASK validation rules matching the class hierarchy and disjointness requirements.
   - **CRITICAL prefix check bypass**: To prevent the compiler's prefix check from failing, write all SPARQL ASK queries in `ggen.toml` starting directly with `ASK` (no `PREFIX` blocks) and using inline full IRIs.
   - Add SPARQL CONSTRUCT inference rules (e.g. `infer-socket-hosts-component`) to extract typestate relations.

4. **Verify and Validate**:
   - Run `rapper` or `riot` syntax validation on all ontology Turtle files to ensure zero syntax errors and valid import resolution.
   - Run `/Users/sac/.local/bin/ggen sync --validate-only true` inside `/Users/sac/.ggen/packs/eden_server/` to ensure the ggen compiler successfully compiles the manifest and executes validations.
   - Propose a negative test/SHACL execution: inject a temporary validation violation (e.g. out-of-bounds `riskClass` or missing `proofClass` on a `mars:DimensionalAsset`) and verify that it is successfully caught and rejected by `ggen sync`.
   - Log all commands run, verify output, and write detailed progress to `/Users/sac/rocket-craft/.agents/worker_eden_ontology_refactor/progress.md` and handoff report to `/Users/sac/rocket-craft/.agents/worker_eden_ontology_refactor/handoff.md`.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.
