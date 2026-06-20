# MODULAR IDENTITY SMOKE REPORT

## 1. smoke_batch_summary
- Total seeds evaluated: 8
- Passing smoke seeds: 3 / 3
- Refused negative fixtures: 5 / 5

## 2. seed_results
- **Seed 101**: PASS_FLAGSHIP (Passed: True)
- **Seed 102**: PASS_FLAGSHIP (Passed: True)
- **Seed 103**: PASS_FLAGSHIP (Passed: True)

## 3. part_scope_audit
Verified that all parts contain only local primitives. Head, wings, and blades are isolated from torso.

## 4. socket_boundary_audit
Verified that sockets contain no mesh payloads and are purely Xforms.

## 5. part_bounds_audit
Verified that all bounding boxes conform to envelopes.

## 6. geometry_fingerprint_audit
Verified that each part USD file generates a unique fingerprint.

## 7. negative_fixture_results
- **Fixture torso_contains_foreign_parts** (Seed 201): REFUSE_MODULAR_USD (Expected: REFUSE_MODULAR_USD) -> **PASS**
- **Fixture socket_contains_mesh_payload** (Seed 202): REFUSE_MODULAR_USD (Expected: REFUSE_MODULAR_USD) -> **PASS**
- **Fixture assembly_reference_inside_part_file** (Seed 203): REFUSE_MODULAR_USD (Expected: REFUSE_MODULAR_USD) -> **PASS**
- **Fixture duplicate_part_fingerprint** (Seed 204): REFUSE_MODULAR_USD (Expected: REFUSE_MODULAR_USD) -> **PASS**
- **Fixture missing_owner_part_id** (Seed 205): REFUSE_MODULAR_USD (Expected: REFUSE_MODULAR_USD) -> **PASS**

## 8. lsp_diagnostic_results
LSP diagnostics active: USD301, USD302, USD303, USD304, USD305, USD306, USD307, USD308, USD309, USD310, USD311, USD312. Proved active on all mutated templates.

## 9. first_failure_station_results
- MODULAR_USD: 5 failures
- Downstream: 0 failures

## 10. disposition_summary
- PASS_FLAGSHIP: 3
- REFUSE_MODULAR_USD: 5

## 11. receipt_manifest
Receipt manifest contains signed hashes of all assets generated during smoke run.

## 12. release_decision
**DOE_RELEASED**
