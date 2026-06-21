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

## Principle

Collapse to single-source where it can be verified safely; **guard** where the fix is
risky; defer ontology-design-grade refactors to attended passes rather than risk a
broken generator. The drift guards ensure the remaining mirrors cannot diverge unnoticed.
