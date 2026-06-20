# SPR: AI VISION JUDGE CELL (TPS/DfLSS CORRECTED)

## Core Doctrine
The AI Vision Judge does not assign crown scores or subjective vibe ratings. It classifies visual nonconformance against flagship CTQs. Any critical visual defect refuses the asset.
A scalar score is not an operational definition. A defect class is.

The AI Vision Judge never owns final admission alone. It owns one station: visual flagship conformance. The verifier aggregator owns final disposition.

## The Rubric (VJ-CRIT Namespace)
- `VJ-CRIT-001`: silhouette lacks flagship authority
- `VJ-CRIT-002`: hard-surface detail below production threshold
- `VJ-CRIT-003`: material response not cinematic/PBR-rich
- `VJ-CRIT-004`: part hierarchy reads as primitive/proxy
- `VJ-CRIT-005`: destruction/loadout integration absent or toy-like
- `VJ-CRIT-006`: UE4 presentation fails flagship standard

## Disposition Rules (No Scalar Averaging)
The `ai_vision_judge_report.json` provides only the visual disposition:
- `PASS_VISUAL_FLAGSHIP`: All visual CTQs pass; no unresolved visual defects
- `REFUSE_NON_FLAGSHIP`: Fails cinematic/AAA/F1 visual bar (e.g. looks like Hot Wheels, proxy, blob)
- `HOLD_FOR_ROOT_CAUSE`: Measurement conflict or unclear defect source
- `REPLAY_REQUIRED`: Looks acceptable but process proof missing

## Aggregate Final Admission
Final admission (`PASS_FLAGSHIP`) occurs in `candidate_disposition.jsonl` and requires:
`visual_pass ∧ modular_usd_pass ∧ pbr_pass ∧ rig_socket_pass ∧ ue4_cook_pass ∧ ip_distance_pass ∧ receipts_replay_pass`

## Factory Cybernetic Loop & JSON Report
The AI Vision Judge JSON must strictly match:
```json
{
  "candidate_id": "seed_...",
  "station": "AI_VISION_JUDGE",
  "disposition": "REFUSE_NON_FLAGSHIP",
  "critical_defects": [ ... ],
  "major_defects": [],
  "minor_defects": [],
  "admission": false
}
```
If the disposition is `REFUSE_NON_FLAGSHIP`, the line stops, the defect is recorded, and the repair route is dispatched.
