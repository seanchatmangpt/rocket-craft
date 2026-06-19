## 2026-06-19T04:34:49Z
Perform Milestone 2 (Ontology & DL Refactoring) to enrich and align the RDF ontologies in `/Users/sac/.ggen/packs/eden_server/` and `/Users/sac/.ggen/packs/ue4_ontology/` as detailed in `/Users/sac/rocket-craft/.agents/orchestrator_swarm_audit/m1_gap_report.md`.

You must implement:
1. All 12 gameplay cells: Manufacturing, Repair, Race, Trade, Insurance, Prediction, Resource Collection, Infrastructure, Defense, Exploration, Discovery, Research. Define them as classes/subclasses or individuals under a common `eden:GameplayCell` concept.
2. All 8 states of resolution: Global, Regional, Zone, Facility, Assembly, Subassembly, Part, Socket. Define them as classes or individuals.
3. All 5 semantic importance LOD classes: CROWN, PRIMARY, SECONDARY, TERTIARY, BACKGROUND.
4. All dynamic rendering parameters: LOD class, material class, instancing class, semantic importance class, silhouette importance class, and interaction distance class. Map them to properties on `eden:AssemblyComponent` and/or `ue4:USceneComponent`.
5. Walkthrough closure details: Locations, Exits, Routes, Zones, Interactables, and Facilities (e.g., manufacturing stations, repair stations, race facilities, market facilities). Ensure these can represent a connected topological graph.
6. All 12 authority state dimensions represented as byte-classes: Damage, Heat, Stress, Fatigue, Grip, Energy, Resource, Market Condition, Risk, Provenance, Conformance, Standing. Add the missing 5 dimensions (Energy, Resource, Market Condition, Conformance, Standing) as datatype properties with `xsd:unsignedByte` ranges.
7. Integrate the new shapes into `/Users/sac/.ggen/packs/eden_server/ontology/validation_shapes.ttl` and `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`. Ensure every new constraint is covered by SHACL validation (especially byte-class bounds [0, 255] and structural limits).
8. Ensure all new SPARQL validation/select queries contain an explicit `ORDER BY` clause.
9. Verify that both packs compile and validate successfully using `ggen sync --validate-only true`.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Your working directory is `/Users/sac/rocket-craft/.agents/worker_m2_refactor/`. Your identity is worker_m2_refactor.
Send a message back to the orchestrator when you are done, listing the modified files and validation results.
