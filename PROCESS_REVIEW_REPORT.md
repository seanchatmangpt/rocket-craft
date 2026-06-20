# Pre-UE4 Hero-Asset Admission — Executive Process Review

**Process under review:** the $5M pre-UE4 hero-asset ADMISSION pipeline.
**Doctrine:** CONTINUE_REPAIR / CLAIM_HOLD — standing may rest only on real verifier
evidence (source-law-derived, fresh-rendered, residual-measured, bounded-repaired,
receipt-linked, delete-and-resync-replayed).
**Synthesis of four independent reviews:** doctrine-admission (NEEDS_REPAIR),
replay-receipts (NEEDS_REPAIR), metric-morphology (SOUND_WITH_GAPS),
gaps-risks (SOUND_WITH_GAPS).

**Overall verdict: CLAIM_HOLD — NEEDS_REPAIR.** The top-line claim
`PRE_UE4_HERO_ASSET_ADMITTED` is not earned. The honest, evidence-backed state of
the flagship asset is **PARTIAL_ALIVE**, and at least one standing was upgraded to
ADMITTED without on-disk verifier backing.

---

## 1. What is genuinely admitted (real evidence stands)

These hold up under independent recomputation and adversarial reading:

- **R6 receipt-chain cryptography.** All 162 BLAKE3 receipts recompute as
  `blake3(hash || prev_hash)` over hex with 0/162 mismatches; genesis = 64 zeros,
  head/tail bind to first/last entries, prev_hash linkage is error-free. The chain
  is internally self-consistent and end-to-end reproducible against the documented
  algorithm — *not* self-asserted.
- **Honest verifier scoping at R6.** `verify_delete_and_resync_replay.py` computes
  `passed = generator_artifacts_byte_identical AND disposition_replays`. VERIFIED
  means "replay is deterministic," **not** "asset passed quality." `thresholds_met=false`
  is carried honestly through replay; no faked pass.
- **Metric-morphology verifier honesty (the focus law).** Emits
  `verdict=PARTIAL_ALIVE`, `shacl_conforms=false`, 3 live `REFUSE_PART_HEIGHT_BAND`
  refusals (Torso 0.129 vs [0.30,0.45]; both Limbs 0.6307 vs [0.40,0.55]). The
  negative UFO-disc fixture is genuinely REFUSED (ANATOMY_PARADOX +
  REFUSE_DEFAULT_SHIELD_PROPORTION), and the law-self-test is wired to the verdict
  (a dead law fails the whole gate, exit 2).
- **Ratio-of-body-height band design.** Unit-free, so the ~400× absolute-scale error
  in the geometry does not corrupt the band logic. This is the correct invariant to
  enforce pre-render.
- **Source-law regenerability (R2).** A true delete + re-merge bijection: removes
  `all_merged.ttl`, re-runs `merge_ontology.py`, checks byte-identity plus
  banner↔source bijection, and restores the working tree (read-only contract honored).
- **IP non-confusion.** `IP_DISTANCE_EVIDENCE.json` = ADMIT_ORIGINAL, refused=0,
  present and consistent.

## 2. What is PARTIAL_ALIVE — and why that is the honest state

The flagship mech does **not** pass. The verifier says so; the synthesis layer hides it.

- **Band refusals are live.** Torso and both limbs fall outside their declared
  height bands. `METRIC_MORPHOLOGY_REPORT.json` carries 4 unrepaired refusals and
  `shacl_conforms=false`.
- **A real orientation defect is unrepaired.** Parts stack along **Z** while the USD
  declares `upAxis=Y` (`orientation_finding.agree=false`). `usdrecord` will render
  mis-oriented. This is currently a non-blocking *note*, not a refusal code.
- **Geometry is still placeholder.** R6 disposition shows 20 USD400/USD401 errors —
  "abstract blocky math," "8-vertex mock mesh" for all core parts. These are not hero
  meshes yet.
- **Internal anatomy incoherence the gate does not catch.** SM_Torso and SM_Head
  report the *identical* ratio 0.129 (a torso the same height as the head). Head
  passes [0.08,0.15]; only the too-small torso is flagged. The head-above-torso law
  collapses to a single generic SHACL message.

Per doctrine, an active SHACL refusal plus an acknowledged orientation defect is a
**CLAIM_HOLD**, not ADMITTED.

## 3. Top risks, ranked

| # | Risk | Severity | Where |
|---|------|----------|-------|
| 1 | **Top-line claim is unsound.** `NEXT_GATE_STATUS.md` asserts `PRE_UE4_HERO_ASSET_ADMITTED` while the dedicated morphology law stands at PARTIAL_ALIVE with 4 refusals. | HIGH | NEXT_GATE_STATUS.md:4,47-50 vs METRIC_MORPHOLOGY_REPORT.json:135-143 |
| 2 | **Failing gate omitted from the census.** The morphology report is never referenced in NEXT_GATE_STATUS; the only PARTIAL_ALIVE crate is left out of the 9-workstream "all ADMITTED" table. Admission reached by exclusion. | HIGH | NEXT_GATE_STATUS.md:12-24,46-50 |
| 3 | **R3 standing fabricated vs its generator.** `generate_all_reports.py:340` hardcodes `standing='PARTIAL_ALIVE'`, yet committed `VISION_POWL_LOOP_ADMISSION_REPORT.json` says ADMITTED with all verifier evidence fields = null. Not regenerable; an upgrade with no on-disk verifier backing. | HIGH | generate_all_reports.py:340 vs VISION_POWL_LOOP_ADMISSION_REPORT.json |
| 4 | **Receipt chain is STALE vs the working tree.** 11/156 byte-leaves no longer match disk — including keystone `ontology/all_merged.ttl` and 9 `.usda` geometry files (SM_Head alone: 359 ins / 3250 del). chain_valid=True/ADMITTED persists over artifacts that no longer exist. **No freshness gate re-hashes live artifacts.** | HIGH | BLAKE3_RECEIPT_CHAIN.json vs on-disk; verify_r6_*.py:267-268 |
| 5 | **GPU "determinism" overstated.** Disposition replay compares two back-to-back rebuilds in *one* process on *one* Apple-M3/Metal session. No cross-machine / cross-OS / CI-runner evidence. "GPU determinism handled" is a same-session claim. | HIGH | verify_delete_and_resync_replay.py:128-150,171-194 |
| 6 | **Rotation silently dropped from bbox measurement.** `measure_part` composes only translate+scale; `rotateXYZ` is never parsed. SM_Blade_L/R carry real rotations (0,0,20.0)/(0,0,18.0) — measured in the unrotated frame, wrong. An out-of-band tilt is invisible to the gate. `xformOpOrder` is ignored; TRS order hardcoded. | HIGH | verify_metric_morphology.py:96-119 |
| 7 | **The Rust focus crate is dead weight.** `rocket_preue4_verifier` (edition 2024 typestate spine) has ZERO invocations in scripts/Justfile/*.sh. The real keystone is Python. Its compile-time guarantees gate nothing. | HIGH | crates/rocket_preue4_verifier; absent in Justfile/scripts |
| 8 | **Dangling receipt-linked reference.** NEXT_GATE_STATUS cites `NON_CONFUSION_REPORT.json` at root twice; the file exists only at `generated/ip_distance_engine/NON_CONFUSION_REPORT.json`. A receipt-linked claim points at the wrong path. | MED | NEXT_GATE_STATUS.md:19-20 |
| 9 | **Body-height denominator is a script invention.** "Union of ALL part envelopes" (incl. blades/wings) means core-part band thresholds shift with weapon loadout. The denominator should be a canonical skeletal reference defined in THE LAW. | MED | verify_metric_morphology.py:245-264 |
| 10 | **Double tolerance widens the carve-out.** Metrics are 4dp-rounded, then compared at 1e-3 on top; the chained disposition hash is the rounded value, so two rebuilds can hash differently yet both be `disposition_replays=True`. | MED | verify_delete_and_resync_replay.py:116-150 |
| 11 | **Merge gate never parse-validates.** `merge_ontology.py` byte-concatenates raw files, never checks `all_merged.ttl` is valid Turtle. A broken source or prefix collision yields a "successful" merge and green replay. 373 @prefix lines with collisions; duplicate 106_/110_ numbering is a latent merge-order trap. | MED | merge_ontology.py:16-26 |
| 12 | **Silent geometry loss + b3sum fallback.** Regex USD parsing drops a part that fails to match (no error → shifts every ratio). Receipt hashing silently degrades to SHA-256 when `b3sum` is absent, distinguished only by a string prefix. | MED/LOW | verify_metric_morphology.py:84-119,122-126 |
| 13 | **Declared-but-dead X-span band.** 116 declares span bands (0.30-1.20); no SHACL shape or Python check enforces them. | LOW | 116_metric_morphology_bands.ttl:62-63,120-121 |

## 4. Recommendation — what `crates/mech_morphology_law` must own

The new Rust crate must become the **first-class, on-the-evidence-path** morphology
law, replacing the brittle Python/regex measurement layer and wiring the dormant
typestate spine into actual admission. It should own:

1. **Correct USD transform evaluation.** Parse `xformOpOrder` and compose
   translate/`rotateXYZ`/`orient`/scale via a real 4×4 affine matrix at both mesh and
   group level. No hardcoded TRS shortcut. Rotated parts must measure correctly.
2. **Schema-driven geometry parsing.** Replace depth-coupled regex with a real USD
   reader. Any FLAGSHIP_PART that yields 0 points is a **FATAL refusal**
   (`REFUSE_PART_UNMEASURABLE`), never a silent drop.
3. **Up-axis as a first-class refusal.** Derive the vertical axis from declared
   `upAxis`; raise `REFUSE_UP_AXIS_DISAGREEMENT` when the de-facto layout axis
   disagrees (the current Z-vs-Y defect), instead of a non-blocking note. Stop
   silently following the buggy Z layout via hardcoded `VERT=2`.
4. **Canonical skeletal denominator in THE LAW.** Define body-height reference
   (e.g. crown-to-foot of core anatomy parts only) in the graph law, so band
   thresholds are loadout-independent and not a script implementation detail.
5. **Per-part anatomy attribution.** Head-above-torso and leg-below-pelvis must emit
   distinct, per-part refusal codes — not collapse to one generic SHACL message — and
   must catch the torso==head incoherence.
6. **Enforce the declared-but-dead bands.** Wire X-span band checks (and any other
   declared band with no enforcer) so no law is dead.
7. **A freshness gate and honest aggregate.** On every run, re-hash the chained
   byte-class artifacts against the live working tree and **FAIL** (`STALE_CHAIN`) on
   divergence. Replace hand-authored `NEXT_GATE_STATUS.md` with an executable
   re-stat that reads each verifier's verdict field and computes the aggregate —
   so the census cannot omit a failing gate.
8. **Typestate that makes ADMITTED unreachable while PARTIAL_ALIVE.**
   `Machine<Law, Phase>` where `Admitted` simply has no constructor from a phase
   carrying refusals or `shacl_conforms=false`. Wire the crate into the Justfile so
   the admission gate physically routes through it — the compile-time guarantees must
   protect the real evidence path, not sit unreferenced.
9. **Single, hard receipt hashing.** BLAKE3 mandatory (no silent SHA-256 fallback);
   chain rebuild#1 *and* the rebuild1==rebuild2 byte-identity claim into linked
   receipts, with a shared run_id binding the chain to its report.

### Required immediate repairs before any ADMITTED claim
- Downgrade `NEXT_GATE_STATUS.md` to **CLAIM_HOLD / PARTIAL_ALIVE**; cite
  `METRIC_MORPHOLOGY_REPORT.json` and the orientation defect.
- Regenerate `VISION_POWL_LOOP_ADMISSION_REPORT.json` from its generator (it will
  produce PARTIAL_ALIVE) — remove the unbacked ADMITTED.
- Fix the dangling `NON_CONFUSION_REPORT.json` path reference.
- Re-run the delete-resync-replay so the chain rebinds to live `all_merged.ttl` and
  the 9 mutated `.usda` files.
