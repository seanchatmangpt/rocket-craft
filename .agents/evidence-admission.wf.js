export const meta = {
  name: 'evidence-admission',
  description: 'Complete the pre-UE4 hero asset EVIDENCE package on top of the LOCKED replayable geometry (silhouette 0.536 / color 0.95, 4/7 morphology gates cleared) WITHOUT touching wing/template geometry (which clobbers). Build IP-distance + OCEL + BLAKE3 receipts + falsification + counterfactual, then a verifier-computed admission report with honest claim (HOLD + documented scorer-contradiction holdouts VIS203/VIS205, or ADMITTED only if all pass).',
  phases: [
    { title: 'Lock-Baseline' },
    { title: 'Evidence' },
    { title: 'Admission' },
  ],
}

const ROOT = '/Users/sac/rocket-craft'
const ASSET = 'generated/mech_assets/reference_fabric_001'

const LOCK = {
  type: 'object',
  required: ['silhouette_iou', 'color_palette_similarity', 'edge_density_distribution', 'replays', 'vis_errors', 'geometry_untouched'],
  properties: {
    silhouette_iou: { type: 'number' }, color_palette_similarity: { type: 'number' }, edge_density_distribution: { type: 'number' },
    replays: { type: 'boolean', description: 'two clean delete-and-resync runs identical to 4 decimals' },
    vis_errors: { type: 'array', items: { type: 'string' } },
    geometry_untouched: { type: 'boolean', description: 'no edits to 104_reference_fabric.ttl or part_mesh.usda.tera in this phase' },
  },
}

const EVID = {
  type: 'object',
  required: ['ip_distance_path', 'ocel_path', 'receipt_chain_path', 'falsification_pass', 'counterfactual_pass', 'hash_algo', 'notes'],
  properties: {
    ip_distance_path: { type: 'string' },
    ocel_path: { type: 'string' },
    receipt_chain_path: { type: 'string' },
    falsification_pass: { type: 'boolean', description: 'injected impossible fact was REFUSED, then reverted' },
    counterfactual_pass: { type: 'boolean', description: 'changing one bounded prior moved the metric as predicted, then reverted' },
    hash_algo: { type: 'string', description: 'blake3 if available else sha256' },
    notes: { type: 'string' },
  },
}

const ADMIT = {
  type: 'object',
  required: ['conditions', 'admitted', 'claim', 'final_metrics', 'all_replayed', 'holdouts', 'report_path'],
  properties: {
    conditions: { type: 'array', items: { type: 'object', properties: { name: { type: 'string' }, pass: { type: 'boolean' }, evidence: { type: 'string' } } } },
    admitted: { type: 'boolean' },
    claim: { type: 'string', enum: ['HOLD', 'PRE_UE4_HERO_ASSET_ADMITTED'] },
    final_metrics: { type: 'object' },
    all_replayed: { type: 'boolean' },
    holdouts: { type: 'array', items: { type: 'string' }, description: 'gates blocked by scorer contradiction with the specific conflict, e.g. VIS205: needs cyan blade but reference is 0.9% cyan' },
    report_path: { type: 'string', description: 'path to PRE_UE4_HERO_ASSET_ADMISSION_REPORT.{md,json} written under the asset dir' },
  },
}

phase('Lock-Baseline')
const lock = await agent(
`${ROOT}. Confirm the LOCKED replayable geometry baseline WITHOUT editing any geometry (do NOT touch ontology/source_law/104_reference_fabric.ttl or ${ASSET}/templates/usd/part_mesh.usda.tera or the scorer/renderer). Run \`bash scripts/verify_asset.sh\`, then delete renders+report and run again; confirm identical to 4 decimals. Report silhouette_iou, color_palette_similarity, edge_density_distribution, the vis_errors list, and geometry_untouched=true. Expected ~silhouette 0.536, color 0.95, vis_errors VIS203/VIS205/VIS208.`,
  { schema: LOCK, phase: 'Lock-Baseline' })

phase('Evidence')
const evid = await agent(
`${ROOT}. Build the pre-UE4 EVIDENCE package on top of the locked geometry. Do NOT edit geometry/scorer/renderer except the explicit, self-reverting falsification/counterfactual probes below. Generate the producing paths; do not hand-fake artifacts.
1. IP-DISTANCE report -> ${ASSET}/reports/ip_distance_report.json: list baselines mined (Mecha lore, Tiger-I armor ratios, kit decomposition) each marked usable_for_metric_baseline=true, usable_for_generation=false; assert emitted shape language is original (no protected silhouette/mark/name); include an expressive-distance statement and any REFUSE_EXPRESSIVE_PROXIMITY flags (expect none).
2. OCEL log -> ${ASSET}/ocel/manufacturing_log.ocel.json: object-centric events (candidate_created, source_law_resolved, artifact_emitted, gate_evaluated, candidate_refused/admitted, receipt_sealed, replay_completed) for this asset.
3. BLAKE3 receipt chain -> ${ASSET}/receipts/: hash source_law -> emitted USD -> renders -> metrics, each receipt chaining the previous hash. Use blake3 if available else sha256 (report which).
4. FALSIFICATION: inject ONE impossible fact (e.g. a part owning foreign geometry or a forbidden-band dimension), run verify/diagnostics, confirm it is REFUSED, then FULLY REVERT the injection and confirm baseline restored.
5. COUNTERFACTUAL: change ONE bounded prior within its band, confirm the metric moves as predicted via verify_asset.sh, then FULLY REVERT and confirm baseline restored.
Report artifact paths, falsification_pass, counterfactual_pass, hash_algo. After your probes, re-run verify_asset.sh and confirm the locked metrics are unchanged (geometry restored).`,
  { schema: EVID, phase: 'Evidence' })

phase('Admission')
const admit = await agent(
`${ROOT}. Compute the PRE_UE4_HERO_ASSET admission status from VERIFIER OUTPUT and write the report to ${ASSET}/reports/PRE_UE4_HERO_ASSET_ADMISSION_REPORT.md AND .json. Run \`bash scripts/verify_asset.sh\` twice from clean; confirm identical. Evaluate each condition with concrete evidence (path/metric), pass/fail:
- source law regenerates cleanly (merge_ontology.py + ggen sync exit 0)
- ggen sync from source only (no all_merged.ttl hand-edits)
- modular identity (run_mecha_doe.py --smoke = DOE_RELEASED; owner_part_id complete)
- USD parses; reference target correct (winged, foreground-only)
- color/silhouette thresholds (silhouette_iou>=0.25, color>=0.50) — both pass
- morphology gates: report which VIS pass/fail honestly (cleared VIS202/204/206/207; remaining VIS203/205/208)
- ip_distance_report exists; OCEL exists; BLAKE3 receipt chain exists
- falsification + counterfactual pass
- delete-and-resync replay reproduces metrics (all_replayed)
Set admitted=true / claim=PRE_UE4_HERO_ASSET_ADMITTED ONLY if EVERY condition passes AND all_replayed. Otherwise admitted=false, claim=HOLD, and list holdouts with the SPECIFIC scorer contradiction for each (VIS203: layered-plate area trips VIS202 + color regression; VIS205: blade metric scores cyan pixels but reference is only 0.9% cyan, so a measurable cyan blade regresses color_palette_similarity — a verifier-design conflict, not a generation failure; VIS208: cascades from 203/205). Report final_metrics and report_path. Do NOT touch UE4. Honest verifier-computed status only.`,
  { schema: ADMIT, phase: 'Admission' })

return { lock, evid, admit }
