# SPR: MODULAR USD IDENTITY & PART SCOPE CONFORMANCE

## Core Doctrine
Part files must contain part-local prims ONLY. Connected parts are NOT owned parts.
Sockets may point outward. Sockets may not smuggle geometry inward.
The factory does not inspect downstream quality (render, UE4 cook) until upstream ownership is correct.

## The Part Law Invariant
For any prim generated in a part file, the source law must enforce:
```
∀ prim ∈ PartFile(P):
  prim.owner_part_id == P
  OR (
    prim.kind == "socket"
    AND prim.has_no_mesh_payload == true
    AND prim.target_part_id != P
  )
```

## LSP Diagnostics (USD300 Series)
- `USD303 ERROR`: part-local file contains foreign component prims.
- `USD307 ERROR`: part bounding box exceeds declared component envelope.
- `USD308 ERROR`: part file contains assembly-level children.
- `USD309 ERROR`: socket emitted as attached geometry instead of mount declaration.
- `USD310 ERROR`: part-scope query returned nonlocal rows.
- `USD311 ERROR`: socket prim contains mesh payload.
- `USD312 ERROR`: part file references assembly root.

## Modular Identity Smoke Batch (Pre-Flight)
Before the 100-seed DOE is released, a 3-seed smoke batch must pass.

### Required Smoke Report Contract
`MODULAR_IDENTITY_SMOKE_REPORT.md` must contain exactly these sections:
1. `smoke_batch_summary`
2. `seed_results`
3. `part_scope_audit`
4. `socket_boundary_audit`
5. `part_bounds_audit`
6. `geometry_fingerprint_audit`
7. `negative_fixture_results`
8. `lsp_diagnostic_results`
9. `first_failure_station_results`
10. `disposition_summary`
11. `receipt_manifest`
12. `release_decision` (Must be binary: `DOE_RELEASED` or `DOE_HELD`)

### Anti-Cheat Negative Fixture Execution
The report must prove *execution* of the negative fixtures, not merely their existence. If any fixture is skipped, marked unknown, or not executed, `release_decision = DOE_HELD`.
Required results in section 7:
- `torso_contains_foreign_parts`: `REFUSE_MODULAR_USD`
- `socket_contains_mesh_payload`: `REFUSE_MODULAR_USD`
- `assembly_reference_inside_part_file`: `REFUSE_MODULAR_USD`
- `duplicate_part_fingerprint`: `REFUSE_MODULAR_USD`
- `missing_owner_part_id`: `REFUSE_MODULAR_USD`

### Formal Release Law
```
DOE_RELEASED ⇔
  smoke_seed_pass_count == 3
  ∧ negative_fixture_refusal_count == 5
  ∧ diagnostics_USD303_to_USD312_active == true
  ∧ no_modular_failure_reached_downstream == true
  ∧ all_dispositions_receipted == true
```
Everything else yields `DOE_HELD`.
