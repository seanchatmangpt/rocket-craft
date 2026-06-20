# Handoff Report — GC-GUNDAM-FACTORY-001

## 1. Observation

Direct observations made on the workspace:
* The pre-UE4 verifier codebase contains hardcoded dependencies to the M2 milestone. In `crates/rocket_preue4_verifier/src/bin/rocket_preue4_verify.rs`:
  ```rust
  let steps = [
      "SelectFrame",
      "GenerateSocketTopology",
      ...
  ];
  let artifacts = vec![
      "MechBirthSteps.h".into(),
      ...
  ];
  ```
* In `crates/rocket_preue4_verifier/src/ocel.rs` at line 64:
  ```rust
  objects: vec!["case-mechbirth-001".into()],
  ```
* In `ggen-validation-tests/ggen.toml`, the `[generation]` section contains no rules to map the ontologies `gundam_nexus.ttl` or `core.ttl` into the deliverables under `generated/gundam_factory/`:
  ```toml
  [generation]
  rules = [
    { name = "readme", query = { inline = "SELECT * WHERE { ?s ?p ?o } ORDER BY ?s LIMIT 1" }, template = { inline = "# UE4 Ontology\n" }, output_file = "README.md", mode = "Overwrite" }
  ]
  ```
* In `package-brm-html5.sh`, the project target is statically set to `Brm.uproject` (line 10):
  ```bash
  UPROJECT="/Users/sac/rocket-craft/versions/Brm427/Brm.uproject"
  ```
* In `pwa-staff/tests-e2e/tps-dflss.spec.ts` at line 13, the target URL defaults to `Brm.html`:
  ```typescript
  const targetUrl = process.env.TARGET_GAME_URL || '/Brm.html';
  ```

---

## 2. Logic Chain

1. Since `crates/rocket_preue4_verifier` hardcodes MechBirth steps, artifact lists, and the case ID, running the verifier binary on a Gundam Factory walkthrough trace will yield validation errors and incorrect report serialization.
2. Since `ggen-validation-tests/ggen.toml` has no generation rules for Gundam Factory walkthrough artifacts, running `ggen sync` will not procedurally output the required C++ headers, CSVs, or JSON manifests.
3. Since the packaging script and Playwright tests default to the `Brm` project and its corresponding URL, executing them directly will verify the old Brm vehicle simulation rather than the new Gundam mobile suit simulation.
4. Therefore, modifications are needed across: the pre-UE4 Rust verifier (to parameterize steps/artifacts/case IDs), the ggen config (to add SPARQL templates/queries for code generation), the packaging flow (to inject generated files before compilation), and the E2E Playwright test (to focus and actuate the Gundam suit and verify visual motion).

---

## 3. Caveats

- We assume the local compiler toolchain (SpeculativeCoder's UE4.27 fork and Emscripten) is functional and Rosetta is active on the macOS system.
- We did not compile or run the full UE4 packaging flow because of resource constraints and the read-only scope of this exploration.
- The exact coordinate sequence of the walkthrough within the Gundam level has not been verified; it is assumed that general movement actuation is sufficient to compute a non-zero visual delta.

---

## 4. Conclusion

The pipeline is currently in a **PARTIAL** status. The tools exist and are functional, but their target data, steps, and validation constraints are locked to the previous `GC-MECHBIRTH-002` milestone. Reconfiguring the verifier, creating the `ggen.toml` rules, updating the project mappings, and implementing the new Playwright spec is required to successfully implement the `GC-GUNDAM-FACTORY-001` walkthrough.

---

## 5. Verification Method

To verify the upcoming implementations:
1. Run `cargo test -p rocket-preue4-verifier` to verify that all Rust unit, integration, and chaos tests pass.
2. Run `ggen sync --manifest ggen-validation-tests/ggen.toml` and confirm the SHACL/SPARQL rules pass and the files under `generated/gundam_factory/` are successfully generated.
3. Serve the staged directory and run the E2E proof:
   ```bash
   TARGET_GAME_URL="/GundamFactory.html" npx playwright test tests-e2e/gundam_factory_walkthrough_projection.spec.ts --config playwright.html5.config.ts
   ```
4. Verify that `pwa-staff/test-results/gundam-factory-receipt.json` is created with a `verdict: "PASS"`.

---

## 6. Remaining Work

1. **Rust Verifier Parameterization:**
   - Modify `crates/rocket_preue4_verifier` to support a milestone configuration file or command line flags for loading step lists and case IDs.
   - Refactor `OcelLog::from_powlv2lsp_trace` to dynamically extract the case ID.
2. **`ggen.toml` Extensions:**
   - Define generation rules for all 13 Gundam Factory deliverables.
   - Author SPARQL queries matching the schemas in `core.ttl`/`gundam_nexus.ttl`.
   - Write Tera templates for `.h`, `.rs`, `.csv`, and `.json` outputs.
3. **Packaging Flow Updates:**
   - Create a build script that copies generated files to `versions/v4_27_0/Source/` and `Content/` before triggering UAT.
4. **Playwright Spec Implementation:**
   - Create `gundam_factory_walkthrough_projection.spec.ts` modeling the canvas click, W-key injection, screenshot matching, and receipt hashing.
