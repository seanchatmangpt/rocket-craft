## 2026-06-18T17:06:37Z
You are the teamwork_preview_worker subagent.
Your working directory is `/Users/sac/rocket-craft/.agents/worker_ontology_2`.
Please apply fixes in `/Users/sac/.ggen/packs/eden_server` to resolve the quality review findings from Reviewer 2.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Objectives:
1. Fix PROV-O Disjointness Violation:
   Deltas are subclasses of `prov:Entity`, which is disjoint with `prov:Activity`. The property `prov:wasAssociatedWith` has domain `prov:Activity`.
   Change all occurrences of `prov:wasAssociatedWith` to `prov:wasAttributedTo` to associate delta entities with agents in:
   - `/Users/sac/.ggen/packs/eden_server/queries/extract_authority_deltas.rq`
   - `/Users/sac/.ggen/packs/eden_server/queries/extract_assembly_deltas.rq`
   - `/Users/sac/.ggen/packs/eden_server/queries/extract_receipt_deltas.rq`
   - `/Users/sac/.ggen/packs/eden_server/verify.py` (in the `build_mock_data` method for `eden:mockAuthDelta`, `eden:mockAssyDelta`, and `eden:mockReceiptDelta`).

2. Fix Domain Mismatch on Telemetry Properties:
   In `/Users/sac/.ggen/packs/eden_server/ontology/pack.ttl`, change the domain of the datatype properties `eden:damageClass`, `eden:stressClass`, `eden:heatClass`, and `eden:fatigueClass` from `eden:AssemblyComponent` to the union of `eden:AssemblyComponent` and `eden:AuthorityDelta` to allow their assertion on both components and authority delta overrides without disjointness/class inference issues.
   Format:
   ```turtle
   rdfs:domain [ a owl:Class ; owl:unionOf (eden:AssemblyComponent eden:AuthorityDelta) ] ;
   ```

3. Generalize substrate.rq:
   In `/Users/sac/.ggen/packs/eden_server/queries/substrate.rq`, remove the constraint:
   `FILTER(?childType IN (eden:SubAssembly, eden:Part))`
   to make it robust to any custom or future subclasses of `eden:AssemblyComponent`.

4. Execute Validation Suite:
   - Run `/Users/sac/.ggen/packs/eden_server/verify.py` and verify all tests pass.
   - Run Raptor `rapper` tool checks on `/Users/sac/.ggen/packs/eden_server/ontology/pack.ttl` and `/Users/sac/.ggen/packs/eden_server/ontology/deltas.ttl` to confirm syntax validity.

5. Record all execution logs, validation output, and results in your handoff report at `/Users/sac/rocket-craft/.agents/worker_ontology_2/handoff.md`.
