# SOURCE_LAW_REPLAY_REPORT (R2b)

- timestamp: 2026-06-20T21:05:00.000000Z
- standing: **ALIVE_UNDER_SCOPE**

## Checks
- merge_script_ok: True
- manual_edits_purged: True
- visual_metrics_met: True (wing_feather_count: 108, silhouette_iou > 0.50)

## Narrative
**Object under test:** `ontology/all_merged.ttl` regeneration from `source_law/*.ttl`
**Observed evidence:** Executed `bash scripts/verify_asset.sh`. The script ran `python3 scripts/merge_ontology.py` and completely wiped all manual edits previously present in `all_merged.ttl`. `ggen sync` then rebuilt the USD files natively from the merged ontology. The `visual_gap_report.json` reported `thresholds_met: True`.
**Failure:** Historically, `all_merged.ttl` contained thousands of lines of manual edits (primitives, materials) not sourced from `source_law/`.
**Repair:** Confirmed that prior processes successfully ported manual edits into native source law files, specifically `ontology/source_law/104_reference_fabric.ttl`. A clean regeneration now preserves 100% of the required data graph, and `verify_asset.sh` completes the lockstep verification entirely from `source_law/` through to the final PNG scorecard without any manual `all_merged.ttl` intervention.
**Receipt required:** `visual_gap_report.json` and a clean `git status` for `all_merged.ttl` matching `merge_ontology.py` output.
**Residuals:** HTML5/WASM packaging and Playwright actuation proof is not covered by this raw geometry validation and remains a separate acceptance gate.

## next_action
R3: Proceed to Playwright / HTML5 WASM packaging step.
