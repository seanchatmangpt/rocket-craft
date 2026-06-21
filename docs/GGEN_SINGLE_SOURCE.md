# ggen Single-Source Refactor Campaign

Doctrine: the ontology graph is the single source of truth. Code/templates should
**derive** from it, never **re-state** it. Every hand-mirror of ontology data is a
drift hazard (a copy that silently diverges and then admits/refuses the wrong thing).

This file tracks the campaign to collapse those mirrors. Status as of 2026-06-21.

## Done (collapsed to single source, all verified non-regressing)

| Mirror | Was | Now | Commit |
|---|---|---|---|
| Metric-gate part-height bands | hardcoded `PART_BANDS` dict in `verify_metric_morphology.py` | `part_bands()` derives from graph: `117` part-class typing ⋈ `116` `MorphologyBand` (SPARQL) | `5d1a9d51` |
| Metric-gate flagship roster | hardcoded `FLAGSHIP_PARTS` dict | `flagship_parts()` derives from `117 eng:hasPart` | `090217ed` |
| Crate band literals | `mech_morphology_law::class_band` hardcoded (DRIFTED to stale shrink-wrapped values) | re-synced to `116` archetype bands; fixtures re-derived; 27 tests green | `d203b781` |

## Guards (mirrors that remain, made non-silent)

| Guard | Detects | Commit |
|---|---|---|
| `scripts/check_crate_band_sync.py` | crate `class_band` literals diverging from `116` | `7e54945c` |
| `just verify-morphology` / `just check-band-sync` | runs the guard + graph gates as a visible gate | `dc2a3f89` |

The assembly-coherence gate (`verify_assembly_coherence.py`) already derives its
joint tree, root, and link set from `121` — no mirror to collapse there.

## Remaining candidates (attended-grade — need ontology design + iteration, NOT safe unattended one-shots)

1. **`asset.usda.tera` hardcoded part references.** The template loops graph-queried
   *materials* but hardcodes the 9 part `prepend references` lines, including an
   `xform-name → file` mapping (`"Wing_Left"` → `SM_WingArray_Left.usda`) and
   `SM_Loadout` (not in the `117` flagship roster). To collapse: declare an assembly
   composition in the ontology (per assembled part: display/xform name + ref file),
   add an assembly-parts SPARQL `SELECT ... ORDER BY`, and loop it in the template.
   Risk: generated USD must stay valid and gate-passing; needs build-check iteration.

2. **`ggen.toml` `VALUES (?CURRENT_PART_ID) { ... }` compositions.** Each `SM_*.usda`
   rule hardcodes which `mud:` subparts it aggregates (e.g. `SM_Limb_Left` =
   `shoulder_left` + `arm_left` + `leg_left`). To collapse: declare an
   `eng:aggregates` relation in the ontology and query it instead of `VALUES`, across
   ~13 rules. Risk: must produce byte-identical (or gate-equivalent) USD for all parts.

3. **`ggen-asset-lsp` `expected_roots`.** Hardcoded alias branches per part. LOW
   priority: it already has a sane generic fallback (file stem), so it is not a real
   drift hazard — the explicit branches only add legacy-name tolerance.

## morphology gate — geometry format reconciled; PLACEMENT is the open layer

Status 2026-06-21 (updated): `measure_part` is **fixed** to parse the current native
USD shape format and the gate is **functional + honest** again. The open issue moved
down a layer: part PLACEMENT.

What happened: `part_mesh.usda.tera` was changed (parallel swarm) to emit native USD
shapes — `def Cube "armor_plate_N"` + `def Cylinder "armor_piston_N"` inside
`def Xform "prim_NNNN_group"` — instead of `def Mesh` + `point3f[] points`. The gate's
`measure_part` only parsed the old format, so it measured ZERO parts and the on-disk
`ADMITTED` reports were stale.

FIXED (commit-tracked): `measure_part` now extracts AABB corners from `def Cube`
(`±size/2` × cube scale+translate) and `def Cylinder` (`radius` in X/Y, `height/2` in
Z), and runs them through the SAME group scale+translate the legacy code used
(rotateXYZ intentionally ignored, matching the original — keeps ratios comparable).
Hand-verified: SM_Torso `prim_0001` group (scaleY 10, translateY 2, cube halfY 0.5) →
`Ymax = 0.07m` exactly. Negative fixture still refuses; replay holds. The gate now
returns an honest `PARTIAL_ALIVE` instead of a broken FATAL or a stale ADMITTED.

OPEN (attended) — PLACEMENT: the recomputed ratios (head 0.46, torso 0.83, overlapping)
show the parts are measured in their OWN local frames. The vertical placement that made
the bipedal ratios meaningful (head-high / torso-mid / legs-low, set in `104` group
translates) was RESET when the swarm regenerated the part files. For the bipedal
proportions to mean anything, either (a) restore per-part vertical placement in the
geometry source so each `SM_*.usda` sits at its assembly Y, or (b) measure the PLACED
assembly (`ASSET_ReferenceFabric_001.usda` with references resolved) rather than each
part in isolation. (a) keeps the per-part gate; (b) is more robust but needs USD ref
resolution (pxr unavailable — would need a manual reference-flatten). Decide which.

Note: candidate #1 (assembly part-list → graph-driven) was verified to produce
IDENTICAL output at the ggen level and is ready to land independently.

## Principle

Collapse to single-source where it can be verified safely; **guard** where the fix is
risky; defer ontology-design-grade refactors to attended passes rather than risk a
broken generator. The drift guards ensure the remaining mirrors cannot diverge unnoticed.
A broken gate that reports stale ADMITTED is more dangerous than one that visibly fails —
prefer the loud failure.
