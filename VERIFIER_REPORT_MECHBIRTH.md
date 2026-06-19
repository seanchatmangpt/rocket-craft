# GC-MECHBIRTH-001: POWL v2 Grammar-to-Verification Birth Trace

**Status:** PARTIAL_ALIVE_CANDIDATE
**Scoped Admitted Sub-status:** POWL_GRAMMAR_TRACE_ALIVE_UNDER_SCOPE

**Object under test:** POWL v2 Grammar → Law → Verification Pipeline
**Observed evidence:** 
- `powlv2lsp` AST validation and Jidoka Refusal (`/samples/MechBirth.powl`)
- `wasm4pm-compat` Canonical Shape definition (`PowlNodeKind::ChoiceGraph`)
- `wasm4pm` Test Fixture (`/playground/scenarios/14-mech-birth-conformance.test.ts`)

---

## 1. Authoring Grammar (`powlv2lsp`)
The `MechBirth.powl` model was authored using the `powlv2lsp` declarative DSL.
It represents the procedural generation flow for a mech, coordinating partial orders, validation loops, and branching choices.

**Agent Jidoka Fired:** The initial run was REFUSED because the `ChoiceGraph` had an unreachable `RefuseMechVariant` node. This proves the authoring layer has actual refusal behavior, not just happy-path validation. After publishing the residual and applying the repair (adding `ResStart`), the graph was re-evaluated and admitted.

**Result:** Grammar passed validation and emitted a tamper-evident sequential receipt chain representing the birth trace.

## 2. Structural Law (`wasm4pm-compat`)
The model shape descends into `wasm4pm-compat` via `PowlNodeKind::ChoiceGraph`, which replaces block-structured XORs and Loops with a mathematically canonical DAG.
**Result:** The witness pattern (`PhantomData<W>`) mathematically guarantees the structure of the AST at compile time. Status: STRUCTURALLY_ADMITTED.

## 3. Runtime Verification (`wasm4pm`)
The test fixture `14-mech-birth-conformance.test.ts` lowers the DSL into the WASM conformance engine.
**Result:** The engine successfully parsed the choice graph and generated a mapping to a Petri Net, proving the structural integrity of the execution model. Status: PARTIAL_ALIVE.

---

## The Crown Residuals & Next Falsifier

Why is this `PARTIAL_ALIVE_CANDIDATE` and not fully `ALIVE`?

1. **ggen Residual:** POWL trace exists, but `ggen` does not yet lower it into UE4-facing artifacts.
2. **UE4 Projection Residual:** Receipt chain exists, but no rendered mech surface consumes it yet.

### Next Falsifier: Grammar → Law → Verification → Manufacturing
Can `MechBirth.powl` produce one UE4-consumable projection package through `ggen`, and can that package prove which POWL step manufactured each projected surface?

Minimum successful output required:
`/Generated/MechBirth/`
- `MechBirthSteps.h`
- `MechBirthProjectionRows.csv`
- `MechBirthReceiptManifest.json`
- `MechBirthSocketTopology.csv`
- `MechBirthSkinLayers.csv`
- `MechBirthMotionFamilies.csv`
- `verifier_report.json`

---

## Final Law
**Procedural generation is no longer a random act of creation.**
**It is now a process-conformant manufacturing trace with refusal, repair, receipt, and replay.**
**MechBirth proves that a generated asset can have a birth certificate before it has pixels.**
