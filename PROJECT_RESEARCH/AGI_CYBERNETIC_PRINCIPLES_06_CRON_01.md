# The BIG BANG $\mu$-Pipeline: Precondition Poka-Yoke

**Iteration:** 06 (Cron Iteration 01)
**Target Ecosystem:** `~/ggen` (Core Manufacturing Pipeline)
**Source File Analyzed:** `crates/ggen-core/src/domain/sync_profile.rs`

## Prologue: The First Cybernetic Loop Cron
This document is synthesized as the first automated response from the 15-minute background cron loop targeting `~/ggen`. The user instructed the AGI to perform a deep scan of the research-level architectures and extract first principles of the BIG BANG 80/20 code generation pipeline. Having previously established that `ggen` is physically governed by the LSP `Λ_CD` gate (Iteration 05), this iteration pierces into the actual execution pipeline of `ggen-core`.

## The Sync Profile Precondition Layer
When an AGI or human attempts to run the $\mu$-pipeline (`ggen sync`), the system does not immediately parse ontologies or emit artifacts. It first hits the **Sync Profile Precondition Layer**. This layer enforces the exact environmental constraints necessary for truth to be generated.

### The Three Profiles of Governance
1. **EnterpriseStrict**: Requires a `.ggen/packs.lock` file and absolutely forbids unsigned execution packs. This is the zero-trust ceiling.
2. **Permissive**: Relaxes the lockfile and signature constraints, allowing for organic ontology exploration.
3. **Development**: An alias for Permissive, but specifically tailored to emit debug-friendly messages.

## First Principles Extracted: The Sabotage Contract
The most profound discovery in `sync_profile.rs` is how the codebase physically maps to the **Sabotage Suite** outlined in the earlier GALL documentation. The Rust code itself explicitly names the sabotage conditions it is designed to survive.

### 1. The Separation of Concerns (Presence vs. Content)
*Sabotage §5 row 2: Writing garbage to packs.lock then calling `--locked`.*
An AGI might assume that the precondition layer should immediately crash if the lockfile contains malformed JSON. The codebase rejects this. The precondition layer (`validate_sync_preconditions`) evaluates **presence** and **integrity field completeness**, not structural JSON validity. 
- A corrupt-but-present lockfile *passes* the precondition gate.
- It is caught and rejected by the downstream parser.

**The Cybernetic Principle:** In a highly distributed agentic architecture, validation must be strictly bounded. The precondition layer guarantees *causal sufficiency* (the file exists). The execution layer guarantees *semantic truth* (the file is mathematically valid). Entangling the two leads to cascading failures.

### 2. The Hard-Fail on Missing Integrity
If the `--locked` flag is passed, every single pack inside the lockfile must possess a cryptographic `integrity` digest. If an AGI attempts to inject a pack without a digest, the precondition layer hard-fails immediately.

**The Cybernetic Principle:** Cryptographic provenance cannot be circumvented. Without a digest, there is no BLAKE3 hash. Without a hash, there is no Receipt. Without a Receipt, the artifact does not legally exist in the CodeManufactory.

## Conclusion
The BIG BANG 80/20 pipeline is guarded by a Poka-Yoke (mistake-proofing) layer that strictly enforces environmental determinism. An AGI cannot "trick" the pipeline into running unsigned or un-hashed execution packs when `--locked` or `EnterpriseStrict` is active. The Sabotage Suite is not just a theoretical testing concept; it is literally codified into the unit tests of the sync pipeline.
