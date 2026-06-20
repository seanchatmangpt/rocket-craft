# Gap Closure Report — GC-MECH-ASSET-FABRIC-001

**Status**: VERIFIED
**Scoped Status**: REFERENCE_TO_USD_RENDER_VERIFIED_UNDER_SCOPE

## Milestone Admission Status

All 19 checks were evaluated. Passed: 19, Failed: 0.

### Verification Matrix

| Gap ID | Description | Expected | Actual | Status |
|---|---|---|---|---|
| REFERENCE_MEASUREMENTS_EXIST | reference_measurements.json exists | True | True | PASSED |
| GGEN_SYNC_PASSES | ggen sync runs and outputs files | True | True | PASSED |
| USD_ASSEMBLY_EXISTS | ASSET_ReferenceFabric_001.usda exists | True | True | PASSED |
| USD_MESH_FILES_EXIST | Part USD files exist | True | True | PASSED |
| USD_PARSES | USD files are valid and parseable | True | True | PASSED |
| USD_PRIM_COUNT_GE_120 | usd_prim_count >= 120 | >=120 | 1072 | PASSED |
| WING_FEATHER_COUNT_GE_48 | wing_feather_count >= 48 | >=48 | 340 | PASSED |
| MATERIAL_BINDINGS_GE_4 | material_binding_count >= 4 | >=4 | 1020 | PASSED |
| MATERIALX_FILES_GE_4 | 4 MaterialX files exist | True | True | PASSED |
| TEXTURE_MANIFEST_EXISTS | texture_manifest.json exists | True | True | PASSED |
| RENDER_FRONT_EXISTS | render_front.png exists | True | True | PASSED |
| RENDER_ANGLED_EXISTS | render_angled.png exists | True | True | PASSED |
| SILHOUETTE_IOU_GE_025 | silhouette_iou >= 0.25 | >=0.25 | 0.2820123667781105 | PASSED |
| COLOR_PALETTE_SIMILARITY_GE_050 | color_palette_similarity >= 0.50 | >=0.50 | 0.7730427124151055 | PASSED |
| FALSIFICATION_CASES_GE_8_PASS | At least 8 falsification cases successfully pass | True | True | PASSED |
| COUNTERFACTUAL_CASES_GE_8_PASS | At least 8 counterfactual cases successfully pass | True | True | PASSED |
| OCEL_EXISTS | asset_manufacturing.ocel.json exists | True | True | PASSED |
| RECEIPTS_EXIST | asset_receipts.jsonl exists | True | True | PASSED |
| REPORTS_UPDATED | Markdown/JSON verifier reports are present | True | True | PASSED |

## Falsification Suite

The verifier logic was subjected to 8 physical file mutations to verify rejection bounds:

| Case | Expected | Actual | Verdict |
|---|---|---|---|
| MISSING_WING_ARRAY | REFUSED (MISSING_WING_ARRAY) | REFUSED (MISSING_WING_ARRAY) | PASSED |
| ZERO_POINT_MESH | REFUSED (ZERO_POINT_MESH) | REFUSED (ZERO_POINT_MESH) | PASSED |
| MISSING_MATERIAL_BINDING | REFUSED (MISSING_MATERIAL_BINDING) | REFUSED (MISSING_MATERIAL_BINDING) | PASSED |
| RENDER_NOT_CREATED | REFUSED (RENDER_NOT_CREATED) | REFUSED (RENDER_NOT_CREATED) | PASSED |
| LOW_PRIM_COUNT | REFUSED (LOW_PRIM_COUNT) | REFUSED (LOW_PRIM_COUNT) | PASSED |
| LOW_FEATHER_COUNT | REFUSED (LOW_FEATHER_COUNT) | REFUSED (LOW_FEATHER_COUNT) | PASSED |
| MISSING_TEXTURE_MANIFEST | REFUSED (MISSING_TEXTURE_MANIFEST) | REFUSED (MISSING_TEXTURE_MANIFEST) | PASSED |
| MISSING_REFERENCE_MEASUREMENTS | REFUSED (MISSING_REFERENCE_MEASUREMENTS) | REFUSED (MISSING_REFERENCE_MEASUREMENTS) | PASSED |

## Counterfactual Suite

8 hypothetical mecha design modifications were simulated with the following metric deltas:

| Case | Description | Deltas |
|---|---|---|
| DOUBLE_WING_FEATHERS | Double the mecha wing feathers to increase thrust capacity | wing_feather_count: +340, usd_prim_count: +340, material_binding_count: +340 |
| HALF_WING_FEATHERS | Reduce mecha wing feathers by half to decrease drag | wing_feather_count: -170, usd_prim_count: -170, material_binding_count: -170 |
| REMOVE_CYAN_BLADES | Remove cyan blade assemblies to lower heat signature | usd_prim_count: -340, material_binding_count: -340 |
| INCREASE_WHITE_ARMOR_RATIO | Increase white armor plating coverage ratio from 40% to 60% | color_palette_similarity: +0.017000000000000015 |
| DECREASE_CORE_BODY_WIDTH | Decrease mecha body core torso width by 10% | body_mass_delta: -0.10000000000000003 |
| INCREASE_WING_SPAN | Increase mecha wingspan to 1195px to match reference target span | wing_span_delta: -150.0 |
| REMOVE_GOLD_VISOR | Remove gold visor sensor array, replacing with dark frame armor | material_binding_count: -170 |
| ADD_RED_MICRO_DECALS | Add micro decals to torso and leg joints | material_binding_count: +12 |
