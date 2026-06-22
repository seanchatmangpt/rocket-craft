export const meta = {
  name: 'pre-ue4-hero',
  description: 'Manufacture the pre-UE4 hero mech asset package from SOURCE LAW under strict replay discipline: clean baseline -> author wings + hard-surface detail + material/texture graphs in source_law -> IP-distance + OCEL + BLAKE3 receipts + falsification/counterfactual -> verifier-computed admission. claim=HOLD; keep only what survives delete-and-resync replay. No UE4.',
  phases: [
    { title: 'Baseline-Clean' },
    { title: 'Author-Wings' },
    { title: 'Hard-Surface' },
    { title: 'Materials-Textures' },
    { title: 'Evidence' },
    { title: 'Admission' },
  ],
}

const ROOT = '/Users/sac/rocket-craft'
const V = 'bash scripts/verify_asset.sh'   // sync(merge->ggen)->delete stale->render->compare

// Every authoring agent obeys this contract.
const LAW = `DISCIPLINE (non-negotiable):
- Edit ONLY authoritative source: ontology/source_law/*.ttl, the ggen-pack templates, or ggen.toml inference/SELECT rules. NEVER edit ontology/all_merged.ttl (regenerated), NEVER edit scripts/compare_reference_render.py or scripts/render_reference_fabric.py (scorer/renderer are sacred), NEVER hand-edit files under generated/.../usd/ or renders/ (ggen outputs).
- Baselines & gains are measured ONLY via \`${V}\` which deletes stale renders and renders fresh.
- A change is KEPT only if: (a) it survives delete-and-resync replay TWICE (two clean runs identical to 4 decimals), AND (b) it does not materially regress color_palette_similarity or silhouette_iou. Otherwise REVERT it fully and report the revert. No false standing.
- IP law: baselines (Mecha/tank/kit) are usable_for_metric_baseline only, usable_for_generation=false. Author original shape language; never copy protected silhouettes/marks/names.`

const BASELINE = {
  type: 'object',
  required: ['silhouette_iou', 'edge_similarity', 'color_palette_similarity', 'wing_feather_count', 'replays', 'cleaned', 'vis_errors', 'notes'],
  properties: {
    silhouette_iou: { type: 'number' }, edge_similarity: { type: 'number' }, color_palette_similarity: { type: 'number' },
    wing_feather_count: { type: 'number' }, blade_length_angle_delta: { type: 'number' }, core_compactness_delta: { type: 'number' },
    replays: { type: 'boolean', description: 'two clean runs identical to 4 decimals' },
    cleaned: { type: 'boolean', description: 'leftover mid-stop bad patch (cyan displayColor regressing color) removed from part_mesh.usda.tera if present' },
    vis_errors: { type: 'array', items: { type: 'string' } },
    notes: { type: 'string' },
  },
}

const STEP = {
  type: 'object',
  required: ['surface', 'before', 'after', 'kept', 'replayed', 'files_changed', 'vis_errors_after', 'notes'],
  properties: {
    surface: { type: 'string' },
    before: { type: 'object' }, after: { type: 'object' },
    kept: { type: 'boolean', description: 'true only if replayed AND no material color/silhouette regression' },
    replayed: { type: 'boolean' },
    files_changed: { type: 'array', items: { type: 'string' } },
    vis_errors_after: { type: 'array', items: { type: 'string' } },
    notes: { type: 'string' },
  },
}

const EVIDENCE = {
  type: 'object',
  required: ['ip_distance_report', 'ocel_events', 'receipt_chain', 'falsification', 'counterfactual', 'notes'],
  properties: {
    ip_distance_report: { type: 'string', description: 'path; each baseline marked usable_for_generation=false; any expressive-proximity flagged' },
    ocel_events: { type: 'string', description: 'path to OCEL manufacturing log' },
    receipt_chain: { type: 'string', description: 'path to BLAKE3 receipt chain linking graph rows -> USD -> renders -> metrics' },
    falsification: { type: 'string', description: 'falsification cases result (inject impossible facts, verify refusal)' },
    counterfactual: { type: 'string', description: 'counterfactual cases result' },
    notes: { type: 'string' },
  },
}

const ADMISSION = {
  type: 'object',
  required: ['conditions', 'admitted', 'final_metrics', 'all_replayed', 'next_gap', 'claim'],
  properties: {
    conditions: { type: 'array', description: 'each pre-UE4 success condition with pass/fail computed from verifier output', items: { type: 'object', properties: { name: { type: 'string' }, pass: { type: 'boolean' }, evidence: { type: 'string' } } } },
    admitted: { type: 'boolean', description: 'PRE_UE4_HERO_ASSET_ADMITTED == all conditions pass AND all replayed' },
    final_metrics: { type: 'object' },
    all_replayed: { type: 'boolean' },
    next_gap: { type: 'string', description: 'the single highest-value remaining gap if not admitted' },
    claim: { type: 'string', enum: ['HOLD', 'PRE_UE4_HERO_ASSET_ADMITTED'] },
  },
}

phase('Baseline-Clean')
const base = await agent(
`${LAW}

Repo ${ROOT}. FIRST: \`rm -rf /tmp/preue4_*\` to clear stale per-surface snapshot dirs from prior runs (they have clobbered kept gains). The live tree contains a LOCKED, verified bilateral wing-mirror fix (part_mesh.usda.tera emits left wing at -X, right wing at +X; silhouette ~0.49) — DO NOT revert it; it is the baseline, not a bad patch. Establish a CLEAN replayable baseline. A stopped prior run may have left a bad half-patch in generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera (cyan displayColor on blades that regressed color_palette_similarity ~0.93 -> ~0.82 and added VIS203/VIS206).
1. Run \`${V}\` and read generated/mech_assets/reference_fabric_001/reports/visual_gap_report.json.
2. If color_palette_similarity < 0.90 OR VIS203/VIS206 present, the leftover cyan/displayColor blade patch is the cause: remove ONLY that leftover addition from part_mesh.usda.tera (the displayColor / cyan blade emission added by the stopped run) to recover the clean baseline (~silhouette 0.68, color ~0.93, blade_delta ~195). Do NOT remove owner_part_id stamping or legitimate structure. Re-run \`${V}\`.
3. Prove replay: delete renders+report, run \`${V}\`, delete again, run again — confirm identical to 4 decimals.
Report the clean baseline metrics, vis_errors, replays, and whether you cleaned a leftover patch.`,
  { schema: BASELINE, phase: 'Baseline-Clean' })

// KEYSTONE-FIRST ORDER: materials must render true colors BEFORE adding wing/detail mass, else
// every geometry addition renders gray and fights the color histogram (proven: wings cleared
// VIS202/203/USD305 and +silhouette but were reverted only because gray mass dropped color 0.875->0.585).
const SURFACES = [
  { phase: 'Author-Wings', surface: 'feather-plates-vis203',
    detail: `VIS203: feather panels read as line-primitives, want layered swept plates. PRIOR ATTEMPTS FAILED because broad plates with large CYAN tips cleared VIS203 but cost color and triggered VIS202. KEY: make the broad layered plates use the WHITE feather material (M_WhiteArmor / M_WingFeather), NOT cyan — so widening area does not move the cyan palette and does not trigger VIS202. In part_mesh.usda.tera feather_panel branch: widen each feather to a broad plate (real span+area, not scaleY~0.05 sliver) and layer ~3-5 overlapping rows, all WHITE. Goal: clear/relieve VIS203 with silhouette held (~0.53) AND color held (~0.91, white plates) AND no new VIS202. VIEW renders/render_silhouette.png. KEEP only if total vis_errors DROPS AND no color/silhouette regression AND it replays twice.` },
  { phase: 'Hard-Surface', surface: 'edge-density-vis207',
    detail: `VIS207: edge-density distribution mismatch — CORRECTED MEASUREMENT (scorer resolution bug now fixed). Edges are OVER-dense vs reference targets (left 0.12 / center 0.08 / right 0.12), NOT below. So REDUCE/redistribute edge density, do not add. Investigate which region overshoots by replicating the scorer's per-third density calc (edge>50 over silhouette>127 within bbox thirds) on the fresh render, then REDUCE edge-generating detail (excess panel grooves / thin sub-prims) in the overshooting region(s) in source_law/templates so dens_L->~0.12, dens_C->~0.08, dens_R->~0.12. Do not shrink the silhouette or regress color. VIEW the render. KEEP only if edge_density_distribution rises (toward >=0.60) / VIS207 relieves AND no aggregate regression AND it replays.` },
  { phase: 'Hard-Surface', surface: 'hard-surface-detail',
    detail: `Add an ORIGINAL hard-surface detail grammar in source_law: bevel hierarchy, panel breaks, armor plate steps, rim reinforcement, vents/intakes, bolts, greebles, surface-density bands — on torso/head/limb shells. Goal: lift edge_similarity and armor_shell_segmentation (VIS206/VIS207) toward hero quality WITHOUT regressing silhouette/color. Original shape language only (no protected silhouettes).` },
]

const steps = []
for (const s of SURFACES) {
  phase(s.phase)
  const r = await agent(
`${LAW}

Repo ${ROOT}. Clean baseline just established: ${JSON.stringify({ s: base.silhouette_iou, e: base.edge_similarity, c: base.color_palette_similarity, ve: base.vis_errors })}.
SURFACE: ${s.surface}
TASK: ${s.detail}
Procedure: take a FRESH snapshot of the CURRENT LIVE state of the files you will touch — \`rm -rf /tmp/preue4_${s.surface} && mkdir -p /tmp/preue4_${s.surface}\` then cp the live files into it (NEVER reuse a stale snapshot dir from a prior run — stale snapshots have clobbered earlier kept gains). The current live tree already contains prior KEPT gains (e.g. the bilateral wing-mirror fix) — your snapshot must capture them so a revert preserves them. Apply your change in AUTHORITATIVE source law/templates; run \`${V}\`; capture metrics; REPLAY (delete renders, run \`${V}\` again) and confirm identical to 4 decimals; KEEP only if it replays AND color_palette_similarity & silhouette_iou do not materially regress vs baseline — otherwise restore ONLY your touched files from /tmp/preue4_${s.surface} (do not touch files you did not edit) and report kept=false. Report before/after metrics, vis_errors_after, files_changed (must be source-law/templates only), and kept/replayed honestly.`,
    { schema: STEP, phase: s.phase, label: s.surface })
  if (r) steps.push(r)
}

phase('Evidence')
const evidence = await agent(
`${LAW}

Repo ${ROOT}. Produce the pre-UE4 EVIDENCE layer (generate the producing paths if missing; do not hand-fake):
1. IP-DISTANCE report: enumerate baselines used (Mecha/tank/kit) each marked usable_for_metric_baseline=true, usable_for_generation=false; assert the emitted shape language is original (no protected silhouette/mark/name); flag any expressive-proximity as REFUSE_EXPRESSIVE_PROXIMITY. Write generated/mech_assets/reference_fabric_001/reports/ip_distance_report.json.
2. OCEL manufacturing log: object-centric events (candidate_created, source_law_resolved, artifact_emitted, gate_evaluated, receipt_sealed, replay_completed) — write to generated/mech_assets/reference_fabric_001/ocel/.
3. BLAKE3 receipt chain linking source_law hash -> emitted USD hashes -> render hashes -> metrics, each receipt chaining previous hash. Write generated/mech_assets/reference_fabric_001/receipts/.
4. FALSIFICATION: inject an impossible fact (e.g. a part owning foreign geometry, or a forbidden-band dimension) and verify the pipeline/diagnostics REFUSE it; then revert the injection.
5. COUNTERFACTUAL: change one bounded prior within its band and confirm the metric moves as predicted, then revert.
Report the artifact paths and pass/fail for falsification & counterfactual. Use blake3 if available (else sha256, note which).`,
  { schema: EVIDENCE, phase: 'Evidence' })

phase('Admission')
const admission = await agent(
`${LAW}

Repo ${ROOT}. Compute the PRE_UE4_HERO_ASSET admission status from VERIFIER OUTPUT, not prose. Run \`${V}\` twice from clean and confirm identical. Then evaluate each condition with concrete evidence (path/metric), pass/fail:
- source law regenerates cleanly (merge_ontology.py + ggen sync, exit 0)
- ggen sync from source only (no all_merged.ttl hand-edits; grep clean)
- modular identity passes (run_mecha_doe.py --smoke = DOE_RELEASED; owner_part_id complete)
- USD parses; MaterialX/OpenPBR outputs exist and bind in-scope (no cross-scope binding warnings)
- texture manifest exists
- fresh renders exist; visual_gap_report.json present
- residual vector + bounded repair history exist (this run's STEP reports)
- ip_distance_report exists
- OCEL evidence exists; BLAKE3 receipt chain exists
- falsification + counterfactual pass
- delete-and-resync replay reproduces metrics (all_replayed)
Set admitted=true (claim=PRE_UE4_HERO_ASSET_ADMITTED) ONLY if every condition passes AND all_replayed. Otherwise admitted=false, claim=HOLD, and give the single highest-value next_gap. Report final_metrics. Do NOT touch UE4.`,
  { schema: ADMISSION, phase: 'Admission' })

return { base, steps, evidence, admission }
