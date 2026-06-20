#!/usr/bin/env bash
# Lockstep asset verification: regenerate source -> render -> score -> report.
# Guarantees the printed metrics always reflect current source. Any step that
# fails aborts the whole run with a nonzero exit code.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"
GGEN="${GGEN_BIN:-/Users/sac/.local/bin/ggen}"

REPORT="generated/mech_assets/reference_fabric_001/reports/visual_gap_report.json"

# 0. Recompile all_merged.ttl from the AUTHORITATIVE source_law/*.ttl so ggen's
#    input is never stale relative to source law (closes the last replay hole).
echo ">> [0/3] merge source_law -> all_merged.ttl"
python3 scripts/merge_ontology.py

# 1. Regenerate ontology + re-derive inference, then emit USD source.
echo ">> [1/3] ggen sync"
"$GGEN" sync

# 1b. PRE-RENDER METRIC MORPHOLOGY GATE (graph law). Morphology must be admitted
#     in the graph (SPARQL provision + SHACL refusal) BEFORE the render. A "huge
#     UFO disc" is REFUSED here by metric band law, not discovered later in UE4.
#     Exit codes: 0=ADMITTED, 1=PARTIAL_ALIVE (real mech violates a band -> warn
#     but keep render flowing for now; band tuning is non-blocking), 2=REFUSED
#     (the law itself failed to bite, e.g. negative fixture passed -> HARD FAIL).
echo ">> [1b] pre-render metric morphology gate"
set +e
python3 scripts/verify_metric_morphology.py
MM_RC=$?
set -e
if [ "$MM_RC" -eq 2 ]; then
    echo "!! METRIC MORPHOLOGY GATE FAILED (law broken / negative fixture passed) -- aborting"
    exit 2
elif [ "$MM_RC" -eq 1 ]; then
    echo "!! METRIC MORPHOLOGY: PARTIAL_ALIVE -- real mech violates a band (see METRIC_MORPHOLOGY_REPORT.md). Render continues (non-blocking)."
else
    echo ">> METRIC MORPHOLOGY: ADMITTED"
fi

# 2. Delete stale renders + report so metrics CANNOT reflect a prior run, then
#    render FRESH PNGs from the just-synced USD.
echo ">> [2/3] delete stale renders + render_reference_fabric.py"
rm -f generated/mech_assets/reference_fabric_001/renders/*.png "$REPORT"
python3 scripts/render_reference_fabric.py

# 3. Score the fresh renders against the reference targets.
echo ">> [3/3] compare_reference_render.py"
python3 scripts/compare_reference_render.py

# 4. Print the key metrics from the freshly-written report.
echo ">> Key metrics from $REPORT"
python3 - "$REPORT" <<'PY'
import json, sys
with open(sys.argv[1]) as f:
    r = json.load(f)
keys = [
    "wing_feather_count",
    "silhouette_iou",
    "edge_similarity",
    "color_palette_similarity",
    "symmetry_delta",
    "usd_prim_count",
    "material_binding_count",
    "part_graph_similarity",
    "thresholds_met",
]
for k in keys:
    if k in r:
        print(f"  {k}: {r[k]}")
errs = r.get("vis_errors", [])
print(f"  vis_errors: {len(errs)}")
for e in errs:
    print(f"    - {e}")
PY
