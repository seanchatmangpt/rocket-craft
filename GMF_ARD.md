# ARD: Mech Manufacturing via Procedural Assembly

**Date:** June 17, 2026
**Status:** Accepted
**Context:** GMF (Gundam Nexus Manufacturing Facility) Implementation

---

### 1. Context & Problem Statement
The requirement is to demonstrate a walkable AAA-quality mech factory in WebGL2 that showcases end-to-end procedural manufacturing. Standard game industry practice involves importing pre-authored 3D mesh files (GLBs/FBXs). This approach is rejected as "Red Ocean" technical debt because it decouples the geometric artifact from the semantic law, preventing formal verification and prohibiting combinatorial variation.

### 2. Decision: Ontological Assembly
We will manufacture mechs using procedural assembly of primitive components (voxels/procedural meshes) mapped directly to a **Typestate Kernel (`Machine<L, P>`)**. 

- **Ontological Source:** All components (Frame, Joint, Mobility, Armor, Weapon) are defined as `MechPrimitiveCategory` in the `nexus-ostar.ttl` ontology.
- **Assembly Logic:** `ggen` templates project these primitives into Rust code that enforces strict socket/attachment rules at compile time.
- **Constraint Enforcement:** Collision volumes, mass balance, and motion envelopes are calculated as a projection of the state machine, not as post-hoc runtime checks.

### 3. Architectural Implications
#### Positive Consequences:
- **Zero-Mesh Bottleneck:** By assembling mechs from generated parts, we can manufacture an infinite variety of silhouettes from a minimal set of primitives.
- **Combinatorial Validity:** Because assembly logic is a projection of the `ostar` ontology, it is mathematically impossible to "build" a mech that violates structural, collision, or mechanical boundaries. 
- **Determinism:** The assembly process is 100% replayable and verifiable through the resulting BLAKE3 receipt, essential for the "manufacturing" audit trail.

#### Negative Consequences & Risks:
- **Geometric Complexity:** Procedural assembly may struggle to achieve the fidelity of hand-modeled meshes. *Mitigation:* Focus on the "Divine Silhouette" abstraction (high-mobility, layered plates) which lends itself to procedural generation rather than realistic organic surfaces.
- **Performance:** Procedural assembly in-browser requires efficient vertex buffers. *Mitigation:* Use the `wasm4pm-compat` type foundry for high-performance memory layout management and `branchless` logic.

### 4. Alternatives Considered
- **Importing GLB Models:** Rejected. It breaks the $A = \mu(O^*)$ mandate, introduces non-verifiable binary blobs, and prevents the "Combinatorial Maximalism" objective.
- **Runtime If/Else logic:** Rejected. Violates the Law of the Chip (branchlessness) and introduces state-explosion bugs. The typestate kernel (`Machine<L,P>`) enforces legal assembly by construction.
