export const meta = {
  name: 'reproducibility-fix',
  description: 'Make the mech convergence pipeline REPLAYABLE: re-root feather/cyan/mirror wins into source_law/*.ttl so they survive ggen sync, add a render-fresh verify wrapper (without editing the scorer), prove delete-and-resync replay yields identical metrics, then one convergence round on solid ground.',
  phases: [
    { title: 'Audit' },
    { title: 'Fix-Law' },
    { title: 'Fix-Freshness' },
    { title: 'Replay-Verify' },
    { title: 'Resume' },
  ],
}

const ROOT = '/Users/sac/rocket-craft'

const AUDIT = {
  type: 'object',
  required: ['feather_dependency', 'meshproof_root_plan', 'freshness_defect', 'wrapper_plan', 'sourcelaw_targets'],
  properties: {
    feather_dependency: { type: 'string', description: 'exactly how feather Mesh emission depends on meshProof: which ggen.toml SELECT references it, and where the 48 tags lived (all_merged.ttl, a derived file that ggen sync regenerates from source_law/*.ttl)' },
    meshproof_root_plan: { type: 'string', description: 'concrete plan to make meshProof survive ggen sync: either add the tags to the authoritative source_law/*.ttl that feeds all_merged.ttl, OR add a ggen inference rule that DERIVES meshProof from the feather prim graph each sync. Name the exact file and the rule/triples.' },
    freshness_defect: { type: 'string', description: 'confirm compare_reference_render.py reads pre-rendered PNGs from renders/ and does NOT render fresh (cite the lines). Confirm render_reference_fabric.py is the renderer.' },
    wrapper_plan: { type: 'string', description: 'plan for a NEW script (e.g. scripts/verify_asset.sh) that runs ggen sync -> render_reference_fabric.py -> compare_reference_render.py in lockstep, so metrics always reflect current source. Do NOT edit the scorer or renderer (preserve their HEAD integrity).' },
    sourcelaw_targets: { type: 'array', items: { type: 'string' }, description: 'abs paths of source_law TTL files that must change' },
  },
}

const FIX = {
  type: 'object',
  required: ['files_changed', 'syncs_clean', 'feathers_after_sync', 'summary'],
  properties: {
    files_changed: { type: 'array', items: { type: 'string' } },
    syncs_clean: { type: 'boolean' },
    feathers_after_sync: { type: 'number', description: 'wing_feather_count after a fresh ggen sync+render+compare (must be >0 and survive sync)' },
    summary: { type: 'string' },
  },
}

const REPLAY = {
  type: 'object',
  required: ['run1', 'run2', 'identical', 'feathers', 'cyan_renders', 'usd305_pass', 'modular_gate', 'scorer_unchanged', 'notes'],
  properties: {
    run1: { type: 'object', properties: { silhouette_iou: { type: 'number' }, edge_similarity: { type: 'number' }, color_palette_similarity: { type: 'number' } } },
    run2: { type: 'object', properties: { silhouette_iou: { type: 'number' }, edge_similarity: { type: 'number' }, color_palette_similarity: { type: 'number' } } },
    identical: { type: 'boolean', description: 'two clean delete-and-resync runs produced identical metrics (the replay law)' },
    feathers: { type: 'number' },
    cyan_renders: { type: 'boolean' },
    usd305_pass: { type: 'boolean' },
    modular_gate: { type: 'string' },
    scorer_unchanged: { type: 'boolean', description: 'compare_reference_render.py + render_reference_fabric.py byte-identical to git HEAD' },
    notes: { type: 'string' },
  },
}

phase('Audit')
const audit = await agent(
`Repo ${ROOT}. The mech convergence pipeline is NOT replayable. Two confirmed defects:
(A) Round 2 added 48 \`mud:meshProof\` tags to ontology/all_merged.ttl, which gated feather Mesh emission via OPTIONAL ?meshProof in the SM_WingArray SELECTs in ggen.toml. But ggen sync REGENERATES all_merged.ttl from ontology/source_law/*.ttl, wiping the tags -> feathers drop 48->0 on the next sync.
(B) scripts/compare_reference_render.py reads pre-rendered PNGs from generated/mech_assets/reference_fabric_001/renders/ instead of rendering fresh, so metrics can reflect stale renders.

READ ONLY. Determine: how all_merged.ttl is built from source_law (is there a merge step / which files); where feather prims + their belongsToPart live; exactly where to put meshProof so it survives sync (preferred: a ggen inference CONSTRUCT rule that derives meshProof for the 48 agreeing feather prims each sync, OR direct triples in the right source_law TTL); and confirm the freshness defect with line cites. Produce a precise fix spec. Edit nothing.`,
  { schema: AUDIT, phase: 'Audit' })

phase('Fix-Law')
const fixLaw = await agent(
`Repo ${ROOT}. Implement the meshProof re-rooting per this spec so feathers survive ggen sync:

${JSON.stringify(audit, null, 2)}

Make meshProof part of the LAW: either add a ggen inference/CONSTRUCT rule in ggen.toml that derives mud:meshProof for the feather prims whose belongsToPart agrees between the ontology and generator_parameters each sync, OR add the triples to the authoritative source_law/*.ttl that feeds all_merged.ttl. Do NOT edit all_merged.ttl directly (it is regenerated). Then run \`cd ${ROOT} && ggen sync && python3 scripts/render_reference_fabric.py && python3 scripts/compare_reference_render.py\` and confirm wing_feather_count > 0. Then run ggen sync a SECOND time and re-render/compare and confirm feathers are STILL present (survives regeneration). Report feathers_after_sync from the second clean cycle. If the second sync drops feathers, the fix is wrong — keep iterating until feathers survive repeated syncs.`,
  { schema: FIX, phase: 'Fix-Law' })

phase('Fix-Freshness')
const fixFresh = await agent(
`Repo ${ROOT}. Create scripts/verify_asset.sh (new file, chmod +x) that runs, in lockstep and exiting nonzero on any failure: (1) ggen sync, (2) python3 scripts/render_reference_fabric.py, (3) python3 scripts/compare_reference_render.py, then prints the key metrics from generated/mech_assets/reference_fabric_001/reports/visual_gap_report.json. This guarantees metrics always reflect current source. Do NOT edit compare_reference_render.py or render_reference_fabric.py (they must stay byte-identical to git HEAD — verify with git diff). Run scripts/verify_asset.sh once and confirm it works end-to-end. Report files_changed and that the scorer/renderer are unchanged.`,
  { schema: FIX, phase: 'Fix-Freshness' })

phase('Replay-Verify')
const replay = await agent(
`Repo ${ROOT}. Prove the pipeline is REPLAYABLE (your NFR-002 deletion replay):
1. Delete the rendered artifacts + report: \`rm -f generated/mech_assets/reference_fabric_001/renders/*.png generated/mech_assets/reference_fabric_001/reports/visual_gap_report.json\`
2. Run scripts/verify_asset.sh -> capture run1 metrics (silhouette_iou, edge_similarity, color_palette_similarity), wing_feather_count, cyan_region, USD305 status.
3. Delete the rendered artifacts again, run scripts/verify_asset.sh a SECOND time -> capture run2 metrics.
4. Confirm run1 == run2 to 4 decimals (identical = replayable). Confirm feathers > 0 and survive, cyan renders (cyan_region > 0), USD305 passes.
5. Confirm scripts/compare_reference_render.py + render_reference_fabric.py are byte-identical to git HEAD (git hash-object vs HEAD).
6. Run run_mecha_doe.py --smoke and report modular_gate.
Report all fields honestly. identical=false if the two clean runs diverge.`,
  { schema: REPLAY, phase: 'Replay-Verify' })

phase('Resume')
// Only resume convergence if the foundation is now replayable.
let resume = null
if (replay && replay.identical && replay.feathers > 0) {
  resume = await agent(
`Repo ${ROOT}. The pipeline is now replayable. Establish the TRUE replay-verified baseline via scripts/verify_asset.sh, then make ONE conservative convergence improvement that SURVIVES replay: pick the single highest-value lever (e.g. assembly-pose tightening for silhouette_iou, or panel-line relief for edge) authored in source_law/*.ttl or the templates (never in all_merged.ttl). After applying, run scripts/verify_asset.sh TWICE from clean (delete renders between) and confirm the gain reproduces identically both times. Back out the change if it does not reproduce or regresses. Report the replay-verified before/after metrics and confirm reproducibility. Leave the surviving improvement in place.`,
    { schema: REPLAY, phase: 'Resume' })
} else {
  log('Foundation not yet replayable — skipping convergence resume; reproducibility must pass first.')
}

return { audit, fixLaw, fixFresh, replay, resume }
