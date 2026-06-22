export const meta = {
  name: 'finish-admit',
  description: 'Re-apply the PROVEN VIS205 cyan beam-saber fix (relieves VIS205: blade_delta->~14.9<=15, cyan_region->~0.62) on top of the live VIS203-cleared geometry, KEEPING it under the correct admission criterion (all metrics pass THRESHOLDS silhouette>=0.25 & color>=0.50, not zero-regression). If all vis_errors clear AND replay holds, refresh evidence + write PRE_UE4_HERO_ASSET_ADMISSION_REPORT with claim=PRE_UE4_HERO_ASSET_ADMITTED. Honest verifier-computed only.',
  phases: [
    { title: 'Apply-VIS205' },
    { title: 'Admit' },
  ],
}

const ROOT = '/Users/sac/rocket-craft'
const ASSET = 'generated/mech_assets/reference_fabric_001'
const V = 'bash scripts/verify_asset.sh'

const M = {
  type: 'object',
  required: ['silhouette_iou', 'color_palette_similarity', 'cyan_region_similarity', 'blade_length_angle_delta', 'vis_errors', 'morphology_ok', 'thresholds_met', 'kept', 'replayed', 'notes'],
  properties: {
    silhouette_iou: { type: 'number' }, color_palette_similarity: { type: 'number' },
    cyan_region_similarity: { type: 'number' }, blade_length_angle_delta: { type: 'number' },
    feather_panel_curvature_score: { type: 'number' },
    vis_errors: { type: 'array', items: { type: 'string' } },
    morphology_ok: { type: 'boolean' }, thresholds_met: { type: 'boolean' },
    kept: { type: 'boolean' }, replayed: { type: 'boolean' }, notes: { type: 'string' },
  },
}

const ADMIT = {
  type: 'object',
  required: ['claim', 'admitted', 'all_vis_clear', 'thresholds_met', 'all_replayed', 'final_metrics', 'report_path', 'notes'],
  properties: {
    claim: { type: 'string', enum: ['HOLD', 'PRE_UE4_HERO_ASSET_ADMITTED'] },
    admitted: { type: 'boolean' }, all_vis_clear: { type: 'boolean' }, thresholds_met: { type: 'boolean' }, all_replayed: { type: 'boolean' },
    final_metrics: { type: 'object' }, report_path: { type: 'string' }, notes: { type: 'string' },
  },
}

phase('Apply-VIS205')
const v205 = await agent(
`${ROOT}. Re-apply the PROVEN VIS205 cyan beam-saber fix and KEEP it under the CORRECT criterion. The current live tree already has VIS203 cleared (feather subdivide; feather_panel_curvature_score ~0.68) and bilateral wings + VIS207 fix locked. Remaining vis_errors: VIS205, VIS208.

Edit ONLY ontology/source_law/104_reference_fabric.ttl and ${ASSET}/templates/usd/part_mesh.usda.tera (snapshot both to /tmp/finish_admit_snap first). Re-create the proven blade fix:
- In part_mesh.usda.tera add/ensure a \`{% elif row.type == "blade" %}\` branch emitting one elongated thin Mesh "cyan_beam" with \`color3f[] primvars:displayColor = [(0,0.85,1)]\`, rotated ~15deg about Z (image plane) so the scorer's per-half line fit reads a ~15deg saber; beam scale ~ (2.0, 0.058, 0.058).
- In 104_reference_fabric.ttl position the blades to match the reference saber bboxes: left blade translateX ~ -3.0, translateY ~ +1.5 (upper-left); right blade translateX ~ +1.5, translateY ~ -1.0 (lower-middle).
Tune scale/thickness so blade_length_angle_delta <= 15 (VIS205 relieved) and cyan stays modest (reference cyan ~2.0%).

KEEP CRITERION (CORRECTED — this is admission, not zero-regression): keep the change iff after \`${V}\`: VIS205 is gone AND VIS208 is gone (morphology_ok true / vis_errors empty) AND silhouette_iou >= 0.25 AND color_palette_similarity >= 0.50 AND it REPLAYS twice (delete renders between, metrics stable to 4 decimals). The small dip (silhouette ~0.534, color ~0.942) is acceptable because both stay far above threshold and 2 gates clear. If VIS205 will not relieve, restore snapshot and report kept=false honestly. Report final vis_errors, morphology_ok, thresholds_met.`,
  { schema: M, phase: 'Apply-VIS205' })

phase('Admit')
const admit = await agent(
`${ROOT}. Compute final admission from VERIFIER OUTPUT and write ${ASSET}/reports/PRE_UE4_HERO_ASSET_ADMISSION_REPORT.md and .json. Run \`${V}\` twice from clean; confirm identical to 4 decimals. Refresh the evidence layer if a builder exists (scripts/build_evidence_layer.py) so receipts cover the final geometry. Evaluate every pre-UE4 condition (source-law regen clean; ggen sync source-only; modular identity DOE_RELEASED + owner_part_id; USD parses; reference target correct; thresholds silhouette>=0.25 & color>=0.50; ALL morphology gates VIS202-208 clear; ip_distance ADMIT_ORIGINAL; OCEL present; BLAKE3 chain intact; falsification + counterfactual pass; delete-and-resync replay reproduces). Set claim=PRE_UE4_HERO_ASSET_ADMITTED ONLY if vis_errors is EMPTY AND thresholds_met=true AND all_replayed=true AND every evidence artifact present. Otherwise HOLD with the precise remaining holdout. Report final_metrics and report_path honestly. Do NOT touch UE4.`,
  { schema: ADMIT, phase: 'Admit' })

return { v205, admit }
