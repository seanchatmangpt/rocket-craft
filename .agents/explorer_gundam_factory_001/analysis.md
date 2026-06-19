# Gap Analysis Report — GC-GUNDAM-FACTORY-001

## Status: PARTIAL
**Object under test:** Gundam Factory Walkthrough & Packaging Flow
**Observed evidence:** 
- `crates/rocket_preue4_verifier` contains hardcoded MechBirth dependencies.
- `ggen-validation-tests/ggen.toml` lacks generation rules for Gundam Factory deliverables.
- `verify_html5_pipeline.sh` is tailored specifically for the Brm game target.
- Playwright tests inside `pwa-staff/tests-e2e` do not yet target Gundam Factory.

---

## 1. Pre-UE4 Verifier Codebase (`crates/rocket_preue4_verifier`)

### Observations:
- **Cargo.toml:** Line 6 contains the description referencing the previous milestone:
  ```toml
  description = "GC-MECHBIRTH-002 Pre-UE4 Authority/SIMD/Prediction verifier layers"
  ```
- **CLI Binary (`src/bin/rocket_preue4_verify.rs`):**
  - Line 9: Imports `mechbirth_002_residuals`.
  - Line 17: CLI description matches `GC-MECHBIRTH-002`.
  - Lines 55–64: Hardcoded array of MechBirth steps (`SelectFrame`, `GenerateSocketTopology`, `GenerateArmorPanels`, etc.).
  - Lines 78–80: Fallback trace path points to `/Users/sac/powlv2lsp/samples/MechBirth.powl` and `/Users/sac/powlv2lsp/out.json`.
  - Lines 83–97: Hardcoded list of expected generated deliverables (`MechBirthSteps.h`, `MechBirthSteps.rs`, etc.).
  - Lines 100 & 118: Hardcoded milestone value `"GC-MECHBIRTH-002"`.
- **Replay Parser (`src/ocel.rs`):**
  - Line 64: Hardcoded case ID mapping:
    ```rust
    objects: vec!["case-mechbirth-001".into()],
    ```
- **Report & Residuals (`src/report.rs`):**
  - Lines 103–113: `mechbirth_002_residuals` defines static residuals specific to M2.
  - Lines 159 & 165: Test fixtures checking for `"GC-MECHBIRTH-002"`.
- **Tests (`tests/`):**
  - `tests/chaos_mechbirth.rs` and `tests/integration_mechbirth.rs` hardcode checks for the MechBirth 8-step pipeline, using the trace file `/Users/sac/powlv2lsp/out.json`.

### Gaps & Required Changes:
- **Parameterization:** Replace hardcoded `case-mechbirth-001` with an argument or parse it directly from the input OCEL trace file.
- **Refactoring CLI Entrypoint:** Modify `rocket_preue4_verify.rs` to support a `--milestone` or `--config` flag to load different steps and expected artifact arrays (e.g. Gundam Factory).
- **Gundam Factory Step Definitions:** Add the Gundam Factory walkthrough steps to the verifier (e.g., source admission, frame selection, physics handler validation, movement injection, visual projection, receipt emission).
- **New Integration Tests:** Create `tests/integration_gundam_factory.rs` and `tests/chaos_gundam_factory.rs` loading the new walkthrough's event trace instead of `out.json`.

---

## 2. procedurally Generating Gundam Factory Deliverables with `ggen`

### Observations:
- **Ontologies:** 
  - `ontology/gundam_nexus.ttl` contains mech structural primitives (`gn:Frame`, `gn:Mobility`, `gn:Power`, etc.) and specific mobile suit components (`gn:GundamFrame`, `gn:MinovskyUltracompact`).
  - `ggen-validation-tests/core.ttl` defines character instances (`gundam:MyGundam`), input/movement graph topology (`gundam:GundamInputGraph`), and typestates.
- **Configuration:** `ggen-validation-tests/ggen.toml` does not define generation rules for the Gundam Factory walkthrough files. It only contains validation rules (`R1`-`R4`, `RuleA`-`RuleM`) and a single stub `readme` generation rule.

### Gaps & Required Changes:
Create a new configuration or extend `ggen-validation-tests/ggen.toml` with `[[generation.rules]]` to lower the ontology into the required artifacts:
- **`GundamFactorySteps.h` / `GundamFactorySteps.rs`:** SPARQL queries to extract the sequence of states/steps (e.g. `gundam:GundamInputGraph` node connections) and format them as C++ and Rust enums.
- **`GundamFactoryWalkthrough.csv`:** Extract pin connection values and movement vectors for input actuation from the RDF graph.
- **`GundamFactorySocketTopology.csv`:** Query components attached via `ue4:hasComponent` and output their mounting locations.
- **`GundamFactorySkinLayers.csv`:** Query materials and armor layers (`gn:Armor` subclasses/instances) and output their thickness and mass.
- **`GundamFactoryMotionFamilies.csv`:** Extract mobility profiles (`gn:Mobility` instances like walking, hover) and speed limits.
- **`GundamFactorySemanticLOD.csv`:** Query LOD allocations (`ue4:semantic_lod_class` properties).
- **`GundamFactoryAuthority.h`:** Map replicated fields (`ue4:bReplicates true` attributes on properties).
- **`GundamFactoryTypestates.h`:** Formulate valid state transitions based on `ue4:CookingTypestate` and `ue4:WasmPackagingTypestate` mappings.

---

## 3. UE4 HTML5 Packaging Flow

### Observations:
- **uproject Location:** The packaging script (`package-brm-html5.sh`) targets `/Users/sac/rocket-craft/versions/Brm427/Brm.uproject` (resolved via symlink as `Brm427` to avoid UHT macro failures with digit-starting directories).
- **RunUAT Arguments:**
  ```bash
  arch -x86_64 RunUAT.sh BuildCookRun \
      -project="$UPROJECT" -noP4 -platform=HTML5 -clientconfig=Development \
      -cook -build -stage -pak -package -archive -IgnoreCookErrors \
      -archivedirectory="$ARCHIVE_DIR"
  ```
- **Integration of Code and Assets:** 
  - `Brm.Build.cs` defines basic dependencies (`Core`, `CoreUObject`, `Engine`).
  - VaRest is redirected to `Brm` in `DefaultEngine.ini`. The C++ class wrappers (`VaRestRequestJSON.h`, etc.) are stubbed out because the client does not make actual network calls; instead, the static authority of the world is baked into the WASM binary and pak file during the UAT run.
  - Packaged files are outputted to `/tmp/brm-html5-archive/HTML5` (including `Brm.wasm`, `Brm.data`, `Brm.js`, `Brm.html`).

### Gaps & Required Changes:
- To support `GC-GUNDAM-FACTORY-001`, the generated files under `generated/gundam_factory/` must be copied to the `versions/v4_27_0/Source/` and `Content/` directories prior to building.
- The uproject map redirect (`DefaultEngine.ini`) must point the startup map to the Gundam Factory walkthrough level (e.g. `/Game/Maps/GundamFactory`).
- Extend `Html5Cook::run` in `tools/rocket-sdk/src/html5.rs` to support variable project builds so that a Gundam Factory build command can copy the correct generated artifacts automatically before UAT compiles them.

---

## 4. Playwright Test Setup

### Observations:
- **E2E Tests:** Run in `pwa-staff/` using `npx playwright test`.
- **Configuration:** `playwright.html5.config.ts` starts a server serving `pwa-staff/manufactured/` on port 8080.
- **WASM Load & Actuation in `tps-dflss.spec.ts`:**
  - Opens target URL (`process.env.TARGET_GAME_URL || '/Brm.html'`).
  - Polls `window.Module?.calledMain` and canvas width to detect engine readiness.
  - Performs a locator click on the canvas to obtain keyboard focus.
  - Injects keyboard actuation (`Space` and `W` keys down for 8 seconds, then up).
  - Takes screenshot before and after to check visual difference via `pixelmatch`.
  - Hashing the `.wasm` file and saving a BLAKE3-signed receipt to `pwa-staff/test-results/tps-dflss-receipt.json`.

### Gaps & Required Changes:
- **New Test Spec:** Implement `gundam_factory_walkthrough_projection.spec.ts` matching the actuation pattern for Gundam walkthrough (e.g., injecting `W` key to walk the Gundam forward).
- **Config Addition:** Add `gundam_factory_walkthrough_projection.spec.ts` to `playwright.html5.config.ts` matches list or pass it directly.
- **Receipt Writing:** Write the resulting receipt with `verdict`, `visualDelta`, and `output_hash` (WASM SHA256) to `pwa-staff/test-results/gundam-factory-receipt.json`.
- **WASM Mapping:** Package and host `GundamFactory.html`/`GundamFactory.wasm` in `pwa-staff/manufactured/`.

---

## Residuals & Assumptions
- **Assumption:** The SpeculativeCoder UE4.27 HTML5 ES3 compiler fork is fully operational on the build machine.
- **Assumption:** Rosetta (`arch -x86_64`) is present and working on the macOS system to run the Intel-only Mono compiler inside UAT.
- **Uninvestigated:** The exact coordinates of the walkthrough in the Unreal map are not yet known. The Playwright test assumes keyboard actuation is sufficient to generate a visual delta.
