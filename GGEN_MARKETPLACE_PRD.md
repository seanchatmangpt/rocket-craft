# Product Requirements Document (PRD)
## Ggen Marketplace: O-Crate (Ontology Crate) Extension

**Document Version:** 1.0
**Date:** June 2026
**Status:** Approved for Implementation
**Author:** Lead Architect / Ggen Team

---

### 1. Executive Summary
The `ggen` CLI framework operates on the Chatman Equation ($A = \mu(O^*)$), where artifacts ($A$) are deterministic projections of admitted ontologies ($O^*$). Currently, the `ggen` marketplace natively distributes "Packs" (templates and extraction queries, representing $\mu$). However, the raw ontologies ($O^*$) remain localized, fragmented files across disparate project directories.

This PRD outlines the requirements to "finish" the `ggen` marketplace by elevating raw ontologies to first-class citizens. By creating the **O-Crate (Ontology Crate)** standard, `ggen` will act as the centralized, cryptographically verifiable registry for the entire enterprise Semantic Universe.

### 2. Objectives & Goals
1.  **Total Ontological Closure:** Eliminate ad-hoc, untracked `.ttl` file dependencies across the ecosystem.
2.  **Centralized Discovery:** Allow developers and agents to discover, browse, and import standard enterprise ontologies (e.g., `cns`, `bytestar`, `dflss`) directly via `ggen search` and `ggen.toml`.
3.  **Cryptographic Integrity:** Ensure that resolving an O-Crate generates a BLAKE3 signature in `ggen.lock`, preventing unauthorized mutation of the fundamental laws governing the code generation.

### 3. Product Features & Requirements

#### 3.1. The `market:OntologyCrate` Primitive
- The marketplace's core definition file (`~/ggen/marketplace/ontology.ttl`) must be extended to include `market:OntologyCrate` as a peer to `market:Package`.
- O-Crates must define properties for: `market:id`, `market:version`, `market:ontologyCount` (number of `.ttl` files), `market:queryCount` (number of `.rq` files), and `market:blake3Hash`.

#### 3.2. Automated Indexing Engine
- A script (`index_o_crates.py`) must be provided within `~/ggen/marketplace/scripts/` to recursively scan the `ontology_catalogue` directory.
- The engine must treat top-level directories (e.g., `/cns`, `/chatmangpt`) as distinct O-Crate namespaces.
- The engine must automatically append the discovered O-Crates into the global `~/ggen/marketplace/registry/index.json`.

#### 3.3. Dependency Resolution in `ggen.toml`
- Downstream projects must be able to declare an O-Crate dependency instead of a local file path.
  *Current:* `source = "../path/to/cns.ttl"`
  *Target:* `source = "registry://cns@latest"`

### 4. Out of Scope (For V1)
- Automatic semantic merging of conflicting ontologies. (If a project imports two O-Crates that contradict each other, resolution is left to the downstream SPARQL query).
- Cloud-hosted federation (The V1 registry remains localized to the developer's filesystem at `~/ggen/marketplace`).

### 5. Success Metrics
- 100% of the 5,420 consolidated `.ttl` and `.rq` files are successfully indexed and searchable via the `ggen` marketplace JSON.
- Downstream projects (like `rocket-craft`) can successfully execute `ggen sync` relying entirely on an imported O-Crate dependency.
