# PRD: Gundam Nexus Manufacturing Facility (GMF)

**Document Version:** 1.0
**Target State:** WebGL2 Walkable Mech Factory (Correct-by-Construction)
**Status:** Mandatory for Swarm Implementation

---

## 1. Mission
To manufacture a persistent civilization universe from bounded primitives. The GMF (Gundam Nexus Manufacturing Facility) is the physical instantiation of the `A = \mu(O^*)` equation, demonstrating how game realities (Mechs, Worlds, Civilizations) are projected from ontologies, not "created" through manual asset importation.

## 2. Pipeline-as-Factory Architecture
The GMF maps the `ggen` pipeline ($\mu_1$-$\mu_5$) directly into a spatial, walkable environment:

| Zone | Stage | Function |
| :--- | :--- | :--- |
| **Foundry** | $\mu_1$ | Normalize Ontology; manufacture raw bone/armor primitives. |
| **Runner Wall** | $\mu_2$ | Extract part bindings from SPARQL results. |
| **Gantry** | $\mu_3$ | Assembly projection based on Tera templates. |
| **Fit/Collision Bay**| $\mu_4$ | Canonical validation (Structural/Collision/Socket fit). |
| **Proving Ground** | $\mu_5$ | Actuation and receipting (The "Car Drives" Gate). |
| **Reveal Platform** | Output | Present artifact with cryptographic proof. |

## 3. Manufacturing Requirements
- **No Manual Assets:** The pipeline MUST NOT import pre-existing GLB/FBX files. Parts, sockets, hardpoints, and collision volumes must be generated from ontological primitives.
- **Branchless Logic:** Part-to-socket attachment must use the `BranchlessPartsGenerator` logic; invalid fits must trigger an immediate *Jidoka* (line halt).
- **Correct-by-Construction:** Assembly validation must rely on structural typestate proofs. An assembly that violates clearance or mass bounds cannot physically manifest in the bay.
- **The Visual Language:** All parts must project the "Divine Silhouette": White/Gold/Cyan aesthetics, high-mobility, layered feather-plates.

## 4. Acceptance Criteria
- **Receipt Proof:** Every finished mech must manifest with an unforgeable BLAKE3 receipt proving the assembly path ($O^* \rightarrow A$).
- **Visual Delta Validation:** The mech must demonstrate functional motion (Walk/Boost/Land/Kneel) in the Proving Ground, with Playwright calculating a positive visual delta.
- **Structural Integrity:** The assembly bay must mathematically reject any collision volume collision, socket mismatch, or mass overload.
- **Replayability:** The process must be 100% deterministic; the same seed in the ontology must produce the exact same mech artifact on re-projection.
