# UE4 Universal RDF Mapping — Typestates Review Report

This report evaluates the correctness, completeness, robustness, and interface conformance of the implemented typestates schema for the UE4 Universal RDF Mapping project. 

---

# PART 1: Quality Review Report

## Review Summary

**Verdict**: APPROVE

The typestates schema (`typestates.ttl`, `validation.shacl.ttl`, and `ggen.toml`) is extremely robust, comprehensive, and mathematically sound. It adheres to all constraints in the Projection Law and provides dual-layer validation via SHACL shapes and GGen custom SPARQL rules.

## Findings

### [Minor] Finding 1: Redundant Rules in GGen Custom Rules and SHACL shapes
- **What**: There is near-100% duplicate coverage between the custom `ASK` rules in `ggen.toml` and the SHACL SPARQL shapes in `validation.shacl.ttl`.
- **Where**: `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` and `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml`.
- **Why**: While this double-validation does not break anything (and acts as a redundant safety check), it increases compilation/validation overhead.
- **Suggestion**: Keep the dual layers if extreme validation safety is desired, but ensure they are kept in sync if any modifications are made in the future.

### [Minor] Finding 2: VaRest String-Matching Security Gap
- **What**: The check prohibiting REST API calls in statically baked packaging targets relies on finding strings matching `"VaRest"` or `"varest"`.
- **Where**: `validation.shacl.ttl:1361` (`ue4:StaticBakingNoVaRestShape`) and `ggen.toml:834` (`RuleStaticBakingNoVaRest`).
- **Why**: A developer could bypass this rule by calling a different HTTP or Socket integration plugin whose name does not contain the substring "VaRest".
- **Suggestion**: Enhance the rule or add a warning that any dynamic communication nodes (HTTP, Websockets, TCP) are prohibited under statically baked profiles, not just VaRest.

---

## Verified Claims

- **Claim 1**: Active HTML5 assets must have a successfully cooked representation (`ue4:CookState_Cooked`) for the target platform.
  - *Verified via*: `validation.shacl.ttl:914` (`ue4:AssetHTML5CookingReadyShape`) and `ggen.toml:616` (`RuleAssetHTML5CookingReady`). Checked that the SPARQL queries verify if a texture/mesh/audio lacks an HTML5-targeted cooked representation. -> **PASS**
- **Claim 2**: HTML5 textures must use WebGL-friendly compression formats (ASTC or DXT).
  - *Verified via*: `validation.shacl.ttl:950` (`ue4:HTML5TextureFormatShape`) and `ggen.toml:647` (`RuleHTML5TextureFormat`). -> **PASS**
- **Claim 3**: Large audio files (>500KB) under HTML5 must be compressed via OggVorbis.
  - *Verified via*: `validation.shacl.ttl:975` (`ue4:HTML5AudioFormatShape`) and `ggen.toml:671` (`RuleHTML5AudioFormat`). -> **PASS**
- **Claim 4**: High-poly meshes (>20k triangles) must have at least 2 LOD levels for HTML5/WebGL budgets.
  - *Verified via*: `validation.shacl.ttl:998` (`ue4:HTML5MeshLODConstraintShape`) and `ggen.toml:694` (`RuleHTML5MeshLODConstraint`). -> **PASS**
- **Claim 5**: WASM memory layouts must be aligned to 64KB (WASM page size), keep stack < initial, and keep initial <= max.
  - *Verified via*: `validation.shacl.ttl:1045` (`ue4:WasmMemoryLayoutShape`), `ggen.toml:734` (`RuleWasmMemoryLayoutPageAlignment`), and `ggen.toml:757` (`RuleWasmMemoryBoundaries`). -> **PASS**
- **Claim 6**: Memory limits for WASM32 address space must not exceed 2GB (2,147,483,648 bytes).
  - *Verified via*: `validation.shacl.ttl:1148` and `ggen.toml:784`. -> **PASS**
- **Claim 7**: Essential browser run entrypoints (like `_main`) must be exported in WASM memory layouts.
  - *Verified via*: `validation.shacl.ttl:1162` (`WasmMemoryLayoutShape`) and `ggen.toml:794` (`RuleWasmExportedSymbols`). -> **PASS**
- **Claim 8**: Statically baked targets must declare C++ headers, DataTables, BOM, walkthroughs, matrices, and receipt output paths.
  - *Verified via*: `validation.shacl.ttl:1308` (`ue4:StaticBakingPathsShape`) and `ggen.toml:811` (`RuleStaticBakingPaths`). -> **PASS**
- **Claim 9**: No raw pixel, binary mesh, or geometry data is defined in the ontology (Anti-Asset Injection).
  - *Verified via*: `validation.shacl.ttl:1339` (`ue4:PreventRawAssetGenerationShape`) and `ggen.toml:895` (`RulePreventRawAssetGeneration`). -> **PASS**
- **Claim 10**: Dynamic REST logic (VaRest) is prohibited on statically baked configurations.
  - *Verified via*: `validation.shacl.ttl:1361` (`ue4:StaticBakingNoVaRestShape`) and `ggen.toml:834` (`RuleStaticBakingNoVaRest`). -> **PASS**
- **Claim 11**: Running the command `/Users/sac/rocket-craft/validate_ontology.sh` validates successfully.
  - *Verified via*: Run command in shell environment. Executed with exit code 0. -> **PASS**

---

## Coverage Gaps

- **Collision Channel Overrides**:
  - *Description*: While `SimulatedGravityCollisionShape` ensures that a simulated body with gravity has some collision profile or collision enabled, it does *not* verify that the actual channels in that profile are set to `Block` for the ground channel (typically `ECC_WorldStatic`).
  - *Risk Level*: Medium. Objects could still fall through floors at runtime if the collision channel response is set to `Ignore` or `Overlap`, despite the shape validating successfully.
  - *Recommendation*: In future milestones, introduce validation rules that verify the channel-to-channel interactions (e.g. Pawn vs. WorldStatic) to ensure a blocking path exists.

---

## Unverified Items

- **Actual Cooking Outputs**:
  - *Reason for not verifying*: The actual binaries/cooking execution is managed by Unreal Engine 4 and the packaging toolchain, which is outside the scope of this metadata validation review.

---
---

# PART 2: Adversarial Review (Challenge Report)

## Challenge Summary

**Overall risk assessment**: LOW

The schema's strict structural constraints make it highly resistant to invalid configurations. However, edge-case bypasses around string checking (VaRest) and potential issues with physical simulation (channel responses) represent the main risk areas.

---

## Challenges

### [Medium] Challenge 1: Custom HTTP/REST Plugins Bypass
- **Assumption challenged**: The assumption that prohibiting VaRest is sufficient to prevent dynamic REST requests at runtime in statically baked builds.
- **Attack scenario**: A developer integrates a different HTTP plugin (e.g., a custom C++ lib, `EasyHttp`, or socket-based plugin) and uses it to query dynamic semantic data. Since the query only filters for `VaRest`/`varest`, this bypasses validation.
- **Blast Radius**: Major. Breaks the Projection Law by letting dynamic runtime REST calls back into the client projection.
- **Mitigation**: Expand the SPARQL check to look for *any* subclass of custom HTTP request objects or network socket classes, or implement an allowlist of permitted classes.

### [Low] Challenge 2: WASM64 Compatibility Barrier
- **Assumption challenged**: The assumption that maximum WASM memory is bounded at 2GB (WASM32 limit).
- **Attack scenario**: A future target runtime upgrade introduces WebAssembly 64-bit (WASM64), which can address up to 16GB. The schema will physically block compilation configurations targeting WASM64 due to the hard 2GB check.
- **Blast Radius**: Low. Only affects future upgrades, but acts as a hard compile-time barrier.
- **Mitigation**: Parameterize the maximum memory limit based on whether the platform target is WASM32 or WASM64.

### [Medium] Challenge 3: Gravity/Collision Response Mismatch
- **Assumption challenged**: The assumption that checking `collisionEnabled != NoCollision` is sufficient to prevent objects falling through the floor.
- **Attack scenario**: A rigid body has `bEnableGravity true` and its collision is enabled, but its collision profile sets the channel response for the ground's channel to `Ignore`. The body will fall through the ground at runtime.
- **Blast Radius**: Moderate. Simulation instability/failure.
- **Mitigation**: Write a SPARQL check that cross-references the object's collision channel responses with the ground's collision channel to guarantee at least one blocking interaction channel is active.

---

## Stress Test Results

- **Scenario A: Memory Stack size >= Initial Memory**
  - *Expected behavior*: Fail validation.
  - *Predicted behavior*: SHACL shape `ue4:WasmMemoryLayoutShape` SPARQL validation triggers violation. -> **PASS**
- **Scenario B: Unaligned WASM Initial Memory (e.g. 60KB)**
  - *Expected behavior*: Fail validation.
  - *Predicted behavior*: SHACL shape page alignment checks trigger violation. -> **PASS**
- **Scenario C: Raw asset pixel data included in ontology**
  - *Expected behavior*: Fail validation.
  - *Predicted behavior*: `ue4:PreventRawAssetGenerationShape` detects properties like `ue4:texturePixelBytes` and triggers violation. -> **PASS**

---

## Unchallenged Areas

- **C++ Header Parsing**:
  - *Reason for not challenging*: The template code generation logic that parses the baked outputs is handled by Tera templates, which are out of scope for this RDF schema-only review.
