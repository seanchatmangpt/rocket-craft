# Handoff Report — explorer_geom_2

## 1. Observation
- The mecha offline tests are declared in `/Users/sac/rocket-craft/pwa-staff/mecha_offline.test.ts`. Specifically, lines 244–276 check Feature 4: Rigging and Sockets:
  ```typescript
  expect(torsoContent).toContain('def Mesh');
  expect(assemblyContent).toContain('def Xform "Blade_Left"');
  expect(assemblyContent).toContain('def Xform "Torso"');
  ```
- The geometry primitive instances are loaded from `/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/graph/generator_parameters.ttl`. For example, `prim_0043` through `prim_0046` (lines 657–700) represent `feather_panel` primitives for `primary_wing_feathers_left`:
  ```turtle
  mud:prim_0043 rdf:type mud:GeometryPrimitive ;
      mud:belongsToPart mud:primary_wing_feathers_left ;
      mud:primitiveFamily "feather_panel" ;
      mud:translateX "-15.0000"^^xsd:float ;
  ```
- The asset verification pipeline is coordinated by `/Users/sac/rocket-craft/verify_mecha_pipeline.sh`, which runs Vitest, stages files, waits for the local HTTP server, and runs Playwright mecha walkthrough E2E tests:
  ```bash
  (cd "$CWD/pwa-staff" && npx vitest run mecha_offline.test.ts)
  npx playwright test tests-e2e/mecha_walkthrough.spec.ts
  ```
- The test suite readiness is certified in `/Users/sac/rocket-craft/TEST_READY.md`. Lines 55–69 map the 13 CTQ F1 gates (e.g. `CTQ-F1-008` for destruction-state coverage).
- Visual nonconformance guidelines are defined in `/Users/sac/rocket-craft/.agents/SPR_AI_VISION_JUDGE.md`, detailing rubrics `VJ-CRIT-001` through `VJ-CRIT-006`.

---

## 2. Logic Chain
1. **Geometric Sweep Implementation**: Since the test suite expects `wing_feather_count >= 48` and mirrored coordinates (opposite translation signs on X-axis and opposite Y/Z rotation signs), the wing layout can be modeled using a quadratic sweep function mapped inside the `generator_parameters.ttl` graph.
2. **Armor and Frame Hierarchy**: Since `VJ-CRIT-002` (hard-surface detail) and `VJ-CRIT-004` (part hierarchy) restrict primitive or proxy massings, the torso and head units should be designed with layered internal structural steel (cylinders) overlaid by outer white angular armor panels (tapered boxes).
3. **Variant-Based Destruction**: Since `CTQ-F1-008` and `VJ-CRIT-005` mandate dedicated battle-damaged states (broken armor, exposed frames, and VFX sockets), we can implement these using OpenUSD `VariantSets` prepended to the prim definition in `part_mesh.usda.tera`. This lets the engine toggle between an `intact` variant and a `damaged` variant containing fractured geometries, frame cylinders, and custom spark sockets.
4. **Locomotion State Mapping**: Since `CTQ-F1-007` requires heavy locomotion cycles (`idle`, `walk`, `deploy`), these must be semantically mapped as animation instances in the ontology and linked to walkthrough route nodes (`mud:RouteNode`) for the digital twin replay loops.

---

## 3. Caveats
- Direct execution of the pipeline requires local Node.js and Playwright dependencies to be properly configured.
- Supabase database telemetry calls are mocked or bypassed in the E2E test runs when the local emulator is offline.

---

## 4. Conclusion
We have detailed a concrete, F1-compliant geometry, morphology, destruction, and animation strategy for `FLAGSHIP_UE4_MECH_PLANT_001`. The proposal uses OpenUSD `VariantSets` for battle-damage, quadratic sweep formulas for feather panels, and semantic locomotion mappings. The recommendations are recorded in `/Users/sac/rocket-craft/.agents/sub_orch_implementation_aaa_ue4_mech_pack_001/explorer_geom_2_report.md`.

---

## 5. Verification Method
1. Verify the schema validation by running:
   ```bash
   /Users/sac/rocket-craft/validate_ontology.sh
   ```
2. Run the offline test suite:
   ```bash
   cd /Users/sac/rocket-craft/pwa-staff && npx vitest run mecha_offline.test.ts
   ```
3. Run the full mecha pipeline verification:
   ```bash
   /Users/sac/rocket-craft/verify_mecha_pipeline.sh
   ```
