# The BIG BANG $\mu$-Pipeline: The Cleanroom & The 5 Surfaces of Determinism

**Iteration:** 14 (Cron Iteration 09)
**Target Ecosystem:** `~/ggen` (Core Manufacturing Pipeline)
**Source Files Analyzed:** 
- `crates/ggen-core/src/cleanroom/surfaces.rs`
- `crates/ggen-core/src/cleanroom/attestation.rs`

## Prologue
The 5-minute schedule loop has executed. We have seen how the AGI's generated code is constrained by the Constitution, the World Contract, and the compiler itself. But what stops the AGI from writing non-deterministic code that relies on the system clock, random number generators, or external network calls? How does the cybernetic loop ensure identical executions every single time?

## First Principles Extracted: The Cybernetic Cleanroom

### 1. The 5 Surfaces of Determinism (`surfaces.rs`)
The `ggen` pipeline executes the AGI's instructions inside a mathematically locked-down "Cleanroom". It identifies exactly 5 surfaces that can introduce non-determinism, and it completely overrides them during the generation and test execution phase:
1. **Time (`TimeMode`):** The AGI cannot read the real system clock. The clock is `Frozen(seed)` or `Stepped(seed, step_ms)`.
2. **RNG (`RngMode`):** True randomness is disabled. The random number generator is forced to `Seeded(seed)` using an xorshift64 algorithm.
3. **FileSystem (`FsMode`):** The execution is run in an `Ephemeral` tmpfs sandbox to prevent silent state pollution.
4. **Network (`NetMode`):** Set to `Offline`. The AGI cannot reach out to external APIs during execution.
5. **Process (`ProcMode`):** Capabilities are dropped (`CAP_NET_ADMIN`, `CAP_SYS_ADMIN`), and it runs as `NonRoot`.

### 2. Programmable Attestation (`attestation.rs`)
Because the environment is 100% deterministic, the pipeline can issue an unforgeable `Attestation` receipt for every generation cycle.
The `Attestation` proves the structural integrity of the run. It records the exact seeds used for Time and RNG, lists the SBOM (Software Bill of Materials) and dependencies, and emits a SLSA-like Provenance document. It also calculates a `determinism_score` (1.0 meaning mathematically reproducible).

## Conclusion
The AGI is not just bound by static code analysis; it is bound by the environment. By overriding the fundamental physics of the runtime (Time, Entropy, IO), the Cleanroom prevents the AGI from introducing any hidden variables into the physical parts it manufactures. The equation $A = \mu(O^*)$ holds true because $\mu$ (the pipeline execution) is physically forced to be a pure function.
