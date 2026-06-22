# The BIG BANG $\mu$-Pipeline: Genesis & Cryptographic Provenance

**Iteration:** 08 (Cron Iteration 03)
**Target Ecosystem:** `~/ggen` (Core Manufacturing Pipeline)
**Source Files Analyzed:** 
- `crates/ggen-core/src/genesis.rs`
- `crates/ggen-core/src/pki.rs`

## Prologue
The 5-minute schedule loop has triggered. Previous iterations established the physical Poka-Yoke of the `Λ_CD` gate (Iteration 05) and the Lean Six Sigma DMAIC validation of the generative manifest (Iteration 07). This iteration drills down into the absolute lowest layer of the cybernetic loop: the `genesis` primitives. 

## First Principles Extracted: The Heap-Free Law
At the bottom of `ggen` lies `genesis.rs`, which enforces the structural reality of the AGI's causal influence. The foundational principle of the Genesis layer is **Bounded Determinism**. It operates entirely heap-free, rejecting dynamic allocations that could introduce memory-level non-determinism into the execution trace.

### 1. The `Construct8` Kinetic Delta
The AGI does not merely "write code." It generates kinetic deltas that move from graph truth into execution. This is represented by `Construct8`, an explicitly bounded primitive where every field is exactly 8 bytes (`[u8; 8]`):
- `Subject`, `Predicate`, `Object`, `Graph` (The Semantic Intent)
- `Mask` (Capability)
- `Provenance` (Origin tracking)
- `Admission` (Gate validator token)
- `ReceiptHint` (Cryptographic salt)

**The Principle:** An AGI's action must fit into exactly 64 bytes of bounded state. If the operation cannot be modeled within this deterministic `Construct8` boundary, it cannot be safely replayed, and therefore it is refused.

### 2. The Cryptographic Execution `Receipt`
In the Post-Chatman ecosystem, code is meaningless without a receipt. `genesis.rs` defines the `Receipt` struct, which is the undeniable physical proof of generative work. 
- It hashes the *Observed Inputs*.
- It hashes the *Produced Outputs*.
- It causally links to the *Previous Receipt Hash* (forming an unbroken chain of custody).
- It applies an Ed25519 signature binding the AGI's action to the cryptographic state.

### 3. The `Refusal` Concrete
When the AGI attempts an illegal operation, the system does not throw a generic error. It emits a concrete, heap-free `Refusal` containing the exact `RefusalCode` (e.g., `ExpectedOCELMissing`, `BoundaryEvidenceMissing`, `CausalInconsistency`), the 32-byte hash of the failed operation, and the exact 64-byte evidence state at the moment of failure.

### 4. Public Key Infrastructure (`pki.rs`)
The Receipts are strictly governed by `pki.rs`. The AGI cannot simply sign its own work with an arbitrary key. The PKI Manager maintains a `.ggen/trusted-keys.toml` store. An execution receipt is only valid if its Ed25519 signature maps to a trusted public key authorized for `receipt-verification`.

## Conclusion
The BIG BANG code generation pipeline is a zero-trust cryptographic engine. The AGI's actions are decomposed into bounded 64-byte `Construct8` deltas. Upon successful execution, the AGI does not simply output text; it is issued a cryptographically signed `Receipt` linking its inputs to its outputs. If it hallucinates or acts out of bounds, the pipeline emits a mathematically undeniable `Refusal`. The AGI is fully constrained by physics and cryptography, preventing any form of narrative hallucination.
