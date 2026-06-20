# Architecture Decision Record (ARD)
## ADR-042: Elevating Ontologies to First-Class Dependencies (O-Crates)

**Date:** June 2026
**Status:** Accepted
**Context:** Ggen Marketplace Architecture

---

### 1. Context and Problem Statement
The `ggen` architecture relies on a 5-stage transformation pipeline ($\mu_1$-$\mu_5$) to project RDF ontologies into executable code. While the `ggen` marketplace successfully versions and distributes the extraction logic (SPARQL) and projection logic (Tera templates) via "Packs," the actual Source of Truth—the RDF ontologies—are managed out-of-band. 

Relying on out-of-band `.ttl` files breaks the "Big Bang 80/20 Specification Closure" rule. If an ontology is modified locally without a version bump or cryptographic seal, the resulting generated artifacts ($A$) will drift silently, invalidating the foundational proof: $R_B \vdash A = \mu(O^*_B)$.

### 2. Decision
We will expand the `ggen` marketplace to natively index and distribute **O-Crates (Ontology Crates)**. 
1. We have centralized the entire semantic firmament (5,420 files) into `~/ggen/ontology_catalogue/`.
2. We will inject a new class `market:OntologyCrate` into the marketplace ontology.
3. We will execute an automated indexing script to map the physical catalogue into the logical marketplace registry (`index.json`).

### 3. Architectural Implications & Consequences

#### Positive Consequences:
- **Provable Provenance:** Every piece of generated code on the machine can now trace its origin back to a cryptographically sealed, versioned ontology in the registry.
- **Ecosystem Standardization:** High-value domains (like `cns`, `bytestar`, `dflss`) become reusable, standardized libraries. A new project can instantly adopt the enterprise's exact definition of "Compliance" by importing `registry://cns/dflss`.
- **Mechanical Intelligence Enablement:** LLM agents and the `tower-lsp-max` observer can now confidently parse the dependencies of a `ggen` project, knowing that the imported ontologies are immutable and universally resolved.

#### Negative Consequences & Risks:
- **Registry Bloat:** The marketplace `index.json` and internal SQLite caches will grow significantly to track the metadata of 78+ new top-level O-Crates encompassing thousands of files.
- **File System Coupling:** Until a cloud/remote federation protocol is implemented, the registry heavily relies on the exact directory structure of `~/ggen/ontology_catalogue`. 

### 4. Implementation Constraints (The Law of the Chip)
The indexing engine must remain deterministic and bounded. It will not perform deep SHACL validation during the indexing phase (which would exceed reasonable time bounds for 5,400 files). Indexing simply hashes the bytes and extracts the structural metadata (counts and namespaces). Full SHACL validation remains the responsibility of the $\mu_1$ phase during `ggen sync`.
