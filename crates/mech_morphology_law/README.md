# mech_morphology_law

Pre-UE4 hero-asset **metric-morphology GRAPH LAW** as a `Machine<Law, Phase>`
typestate. Pure, deterministic, `no_std`-capable core that enforces ontology laws
**110 / 116 / 117** *before* any render — so a "huge UFO disc" is refused by metric
law, not discovered later in UE4/Playwright.

## Doctrine — CONTINUE_REPAIR / CLAIM_HOLD

Admission is never granted by exclusion. A mech is `Admitted` only when
`Standing::Admitted` is **computed** by the law with zero live refusals and backed
by a fresh, non-stale chained receipt. Any live refusal yields `PartialAlive` and a
runtime `ClaimHold` — the honest "keep repairing upstream" verdict.

## Laws encoded

- **110** anatomy ordering — head bbox-min above torso bbox-max; legs below pelvis
  along the law-chosen vertical axis. Violation → `ANATOMY_PARADOX`.
- **116** per-part ratio-of-body-height bands (mirrored exactly from the
  authoritative `law:*HeightBand` literals: `MechaCrown 0.1190–0.1390`,
  `TorsoSegment 0.1190–0.1390`, `BipedalLeg 0.6207–0.6407`,
  `WingArray 0.6113–0.6313`, `MechaWeapon 0.2245–0.2445`) + shield tiers
  (`Normal 0.25–0.35`, `Large 0.35–0.55`,
  `Tower 0.55–0.85`, `Exception 0.85–1.10`). The exception tier **requires** an
  explicit `ArchetypeException`; absent → `REFUSE_DEFAULT_SHIELD_PROPORTION`.
- **117** body-height denominator = skeletal head-top-to-leg-bottom reference,
  **not** the accessory-inclusive union.
- Orientation — declared up-axis must equal actual stacking axis; the Z-vs-Y
  disagreement surfaces as `ORIENTATION_DEFECT`, never hidden.
- Measurability — a rotated (`has_rotation`) or `Unclassified` part yields
  `UNMEASURABLE_PART` and blocks admission; no silent drop.

## Typestate

```
Machine<L, Measured>  --validate()-->  Machine<L, Validated>  --admit()-->  Result<Machine<L, Admitted>, ClaimHold>
```

`admit()` does not exist for `Measured`, and `validate()` does not exist for
`Admitted` — illegal transitions are compile errors (absent impl blocks).

## Features

- `default = ["std", "serde", "blake3hash"]`
- `--no-default-features` builds the `core + alloc` metric/law/machine core
  (`no_std`), without serde, thiserror, or blake3.

## Test

```
cargo test -p mech_morphology_law
cargo build -p mech_morphology_law --no-default-features   # no_std core
```

Mirrors `scripts/verify_metric_morphology.py`: the UFO-disc fixture is `Refused`
with both `ANATOMY_PARADOX` and `REFUSE_DEFAULT_SHIELD_PROPORTION`.
