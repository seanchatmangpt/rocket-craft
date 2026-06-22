export const meta = {
  name: 'vis203-205-push',
  description: 'Close the last two morphology gates on the locked winged baseline (0.536/0.95): VIS205 by rendering cyan beam-sabers matching the reference saber bbox at ~1% cyan area (so cyan_region_similarity rises + blade_delta drops WITHOUT palette regression), and VIS203 by SUBDIVIDING feathers into more layered plate rows at smaller footprint (raise curvature past 0.10 WITHOUT enlarging the wing => no VIS202). Each surface: snapshot live -> edit source law -> verify -> double-clean replay -> KEEP only if its gate relieves AND no aggregate regression. Then re-measure. claim=HOLD.',
  phases: [
    { title: 'Baseline' },
    { title: 'VIS205-cyan-sabers' },
    { title: 'VIS203-subdivide' },
    { title: 'Final' },
  ],
}

const ROOT = '/Users/sac/rocket-craft'
const ASSET = 'generated/mech_assets/reference_fabric_001'
const V = 'bash scripts/verify_asset.sh'
const RULE = `Edit ONLY ontology/source_law/104_reference_fabric.ttl and/or ${ASSET}/templates/usd/part_mesh.usda.tera. NEVER edit all_merged.ttl, the scorer (scripts/compare_reference_render.py), or the renderer. Snapshot the live files you touch to a FRESH dir (rm -rf it first) so the LOCKED bilateral-wing + VIS207 gains are preserved on revert. Measure ONLY via ${V}. KEEP only if your target gate relieves AND silhouette_iou & color_palette_similarity do not regress AND it replays twice (identical to 4 decimals); else restore your snapshot and report kept=false.`

const M = {
  type: 'object',
  required: ['silhouette_iou', 'color_palette_similarity', 'kept', 'replayed', 'gate_relieved', 'vis_errors', 'notes'],
  properties: {
    silhouette_iou: { type: 'number' }, color_palette_similarity: { type: 'number' },
    cyan_region_similarity: { type: 'number' }, blade_length_angle_delta: { type: 'number' },
    feather_panel_curvature_score: { type: 'number' },
    kept: { type: 'boolean' }, replayed: { type: 'boolean' },
    gate_relieved: { type: 'boolean', description: 'the targeted VIS gate cleared/relieved' },
    vis_errors: { type: 'array', items: { type: 'string' } },
    notes: { type: 'string' },
  },
}

phase('Baseline')
const base = await agent(
`${ROOT}. Confirm the locked baseline via ${V} (twice, delete renders between). Report silhouette_iou, color_palette_similarity, cyan_region_similarity, blade_length_angle_delta, feather_panel_curvature_score, vis_errors. Expected ~0.536/0.95, vis_errors VIS203/VIS205/VIS208, cyan_region_similarity 0, curvature ~0.026. Do not edit anything.`,
  { schema: M, phase: 'Baseline' })

phase('VIS205-cyan-sabers')
const v205 = await agent(
`${ROOT}. ${RULE}
Target VIS205. The scorer fits blades from CYAN pixels (needs ~180px length, ~15deg). Reference cyan = ~1.09% of pixels, total_cyan_bbox [46,211,928,546] in ref coords (1200x1002): a saber from upper-left diagonal + one across the middle. Currently cyan_region_similarity=0 (our blades render gray; blade_length_angle_delta=195). Make the LEFT and RIGHT blade prims render as CYAN beam sabers (bind/displayColor cyan, M_CyanBlade) positioned + angled to match the reference saber bbox/extent, sized so total cyan stays ~1% (do NOT over-paint — reference is only ~1% cyan). Goal: cyan_region_similarity rises and blade_length_angle_delta drops toward <=15, WITHOUT color_palette_similarity regressing (since reference also has ~1% cyan at those positions, correct placement should NOT hurt palette). Read ${ASSET}/reference/reference_original.jpg and reference_measurements.json for saber geometry. KEEP per ${RULE}.`,
  { schema: M, phase: 'VIS205-cyan-sabers' })

phase('VIS203-subdivide')
const v203 = await agent(
`${ROOT}. ${RULE}
Target VIS203 (feather_panel_curvature_score=0.026 needs >=0.10; panels read as line-primitives). DO NOT widen/enlarge the wing (that trips VIS202 + color, proven 3x). Instead SUBDIVIDE: split each existing feather's volume into MORE layered plate rows at SMALLER per-plate footprint, with progressive curvature (vary rotateX/translate per layer) so the layered-swept-plate + curvature detection passes, while the overall wing envelope/span is unchanged. Keep feathers WHITE (no cyan). Goal: feather_panel_curvature_score >= 0.10 / VIS203 relieves, silhouette held (~0.536), color held (~0.95), no new VIS202. KEEP per ${RULE}.`,
  { schema: M, phase: 'VIS203-subdivide' })

phase('Final')
const fin = await agent(
`${ROOT}. Final replay-verified state. Run ${V} twice from clean; confirm identical. Report silhouette_iou, color_palette_similarity, cyan_region_similarity, blade_length_angle_delta, feather_panel_curvature_score, the vis_errors list, whether morphology_ok / thresholds_met now true, and replayed. Then re-run the evidence/admission build if present (scripts/build_evidence_layer.py) to refresh receipts, and state the honest claim (PRE_UE4_HERO_ASSET_ADMITTED only if vis_errors empty AND all conditions pass AND replays; else HOLD with remaining holdouts). Do not edit geometry in this phase.`,
  { schema: M, phase: 'Final' })

return { base, v205, v203, fin }
