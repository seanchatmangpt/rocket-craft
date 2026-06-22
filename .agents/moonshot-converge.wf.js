export const meta = {
  name: 'moonshot-converge',
  description: 'Converge mech morphology toward the winged reference via a SEQUENTIAL parameter-variant tournament on the live tree (snapshot -> ggen sync -> usdrecord -> IoU/edge -> restore), then graft the winning feature-graph parameters. No git worktrees (repo carries multi-GB UE4 binaries; disk-safe).',
  phases: [
    { title: 'Baseline' },
    { title: 'Explore' },
    { title: 'Graft' },
    { title: 'Verify' },
  ],
}

const ROOT = '/Users/sac/rocket-craft'
const BAK = '/tmp/moonshot_src_snapshot'

const METRIC = {
  type: 'object',
  required: ['silhouette_iou', 'edge_similarity', 'color_palette_similarity', 'snapshot_done'],
  properties: {
    silhouette_iou: { type: 'number' },
    edge_similarity: { type: 'number' },
    color_palette_similarity: { type: 'number' },
    wing_feather_count: { type: 'number' },
    snapshot_done: { type: 'boolean', description: 'authoritative source files copied to the snapshot dir' },
    notes: { type: 'string' },
  },
}

const VARIANT = {
  type: 'object',
  required: ['strategy', 'silhouette_iou', 'edge_similarity', 'color_palette_similarity', 'ran_clean', 'restored', 'diff', 'summary'],
  properties: {
    strategy: { type: 'string' },
    silhouette_iou: { type: 'number' },
    edge_similarity: { type: 'number' },
    color_palette_similarity: { type: 'number' },
    morphology_progress: { type: 'string', description: 'which morphology metrics (wing_feather_count, feather_panel_curvature, blade_length_angle_delta, symmetry_delta, USD305 mirror-proof) moved and by how much' },
    ran_clean: { type: 'boolean' },
    restored: { type: 'boolean', description: 'live tree restored from snapshot after measuring' },
    diff: { type: 'string', description: 'unified diff of the source-law TTL / Tera change' },
    summary: { type: 'string' },
  },
}

const FINAL = {
  type: 'object',
  required: ['silhouette_iou', 'edge_similarity', 'color_palette_similarity', 'thresholds_met', 'modular_gate', 'ran_clean', 'metrics_are_real', 'notes'],
  properties: {
    silhouette_iou: { type: 'number' },
    edge_similarity: { type: 'number' },
    color_palette_similarity: { type: 'number' },
    thresholds_met: { type: 'boolean' },
    modular_gate: { type: 'string' },
    ran_clean: { type: 'boolean' },
    metrics_are_real: { type: 'boolean', description: 'true iff a fresh independent compare re-run reproduced the numbers AND scorer/renderer are byte-identical to git HEAD' },
    notes: { type: 'string' },
  },
}

phase('Baseline')
const base = await agent(
`Repo ${ROOT}. (1) Snapshot the AUTHORITATIVE convergence source files so variants can restore cleanly: \`mkdir -p ${BAK} && cp generated/mech_assets/reference_fabric_001/templates/usd/*.tera ${BAK}/\` and also copy any ontology/source_law/*.ttl + ontology/ggen-packs files that the asset USD pipeline reads (identify them from ggen.toml). Record the exact file list to ${BAK}/manifest.txt.
(2) Establish the honest reproducible baseline: \`cd ${ROOT} && ggen sync && python3 scripts/render_reference_fabric.py && python3 scripts/compare_reference_render.py\`, then read generated/mech_assets/reference_fabric_001/reports/visual_gap_report.json. Report silhouette_iou / edge_similarity / color_palette_similarity / wing_feather_count. Do not hardcode. The known true baseline is ~0.322 / 0.101 / 0.876 with wing_feather_count=0 and thresholds_met=0 (morphology gate FAILS) — confirm or correct it.`,
  { schema: METRIC, phase: 'Baseline' })

phase('Explore')
// SEQUENTIAL on the live tree. Each variant restores from ${BAK} first, applies its change,
// measures, then restores again — so no two variants ever overlap and disk stays flat.
// Morphology is the real wall, so strategies target the morphology metrics, not just pose.
// ROUND 3 — baseline is the Round-2 graft (silhouette 0.374 / edge 0.093 / color 0.951, cyan renders,
// USD305 PASS). The REAL structural blocker is now exposed: foreground_component_count=22 (needs [1,5])
// — the silhouette is 22 disconnected blobs, not one connected mech. That same scatter drives
// VIS204 core_compactness (0.382, needs <=0.15). Fuse the parts -> both gates move together.
const STRATEGIES = [
  { name: 'connect-silhouette', detail: 'foreground_component_count=22, needs [1,5]. The rendered silhouette is many disconnected islands. In asset.usda.tera, aggressively compose the parts so their silhouettes OVERLAP into ONE connected mass: pull wings/blades/limbs/loadout inward toward the torso until adjacent parts touch/overlap in the front projection (the reference is one connected winged figure, not floating pieces). Target foreground_component_count <=5 WITHOUT dropping silhouette_iou below 0.35.' },
  { name: 'core-compactness-v2', detail: 'VIS204 core_compactness_delta=0.382 needs <=0.15. Round 2 found pose-scale alone could not move it without collapsing IoU. Try a DIFFERENT lever: tighten the TORSO/CORE shell massing in the grammar (reduce inter-prim gaps within the torso so the core reads as a solid compact mass) rather than scaling the whole assembly. Drive compactness down while keeping the wings spread.' },
  { name: 'blade-angle', detail: 'VIS205 blade_length_angle_delta=135 (needs <=15). Cyan now renders, so this is geometric: correct the blade mount ANGLE and LENGTH in the blade grammar / assembly pose so the twin beam-sabers extend at the reference angle. Aim to halve the delta at least.' },
  { name: 'merge-feathers', detail: 'The 48 feathers likely contribute many of the 22 silhouette components. Make the per-wing feathers overlap enough to read as ONE connected swept panel mass per wing (reduce inter-feather gaps) so each wing is 1 component, not ~20 islands. Helps foreground_component_count AND keeps curvature/overlap passing.' },
  { name: 'edge-push', detail: 'Push edge_similarity past 0.11 with crisper raised-ridge panel relief, without regressing VIS206/VIS207 or color. Build on the Round-2 groove reshape (width 0.015, depth 0.22, protrusion 0.16).' },
]

const variants = []
for (const s of STRATEGIES) {
  const v = await agent(
`Repo ${ROOT}. SEQUENTIAL tournament variant — you run on the LIVE tree, so you MUST isolate yourself:
  STEP 1 restore clean baseline: \`cp ${BAK}/*.tera generated/mech_assets/reference_fabric_001/templates/usd/ \` and restore any TTLs per ${BAK}/manifest.txt.
  STEP 2 apply ONLY your strategy "${s.name}": ${s.detail}
         Edit the AUTHORITATIVE source (ggen-pack template / source_law TTL), never files that ggen sync overwrites blindly.
  STEP 3 measure: \`cd ${ROOT} && ggen sync && python3 scripts/render_reference_fabric.py && python3 scripts/compare_reference_render.py\`; read visual_gap_report.json.
  STEP 4 capture \`git diff\` (and diff vs ${BAK} for the .tera) of your change.
  STEP 5 RESTORE the baseline again from ${BAK} and confirm the working tree matches the snapshot (so the next variant starts clean).
Baseline metrics: ${JSON.stringify({ s: base.silhouette_iou, e: base.edge_similarity, c: base.color_palette_similarity })}. Do NOT edit scripts/compare_reference_render.py or scripts/render_reference_fabric.py and do NOT hardcode metrics — verify will diff them against HEAD. Report your metrics, morphology progress, the diff, and restored=true.`,
    { schema: VARIANT, phase: 'Explore', label: s.name })
  if (v) variants.push(v)
}

const ranked = variants.filter((v) => v.ran_clean)
  .map((v) => ({ ...v, score: (v.silhouette_iou - base.silhouette_iou) + 2 * (v.edge_similarity - base.edge_similarity) }))
  .sort((a, b) => b.score - a.score)

log(`Explore: ${ranked.length}/${variants.length} ran clean. Top: ${ranked.slice(0, 3).map((v) => `${v.strategy}(iou=${v.silhouette_iou?.toFixed(3)},edge=${v.edge_similarity?.toFixed(3)})`).join(', ')}`)

phase('Graft')
const graft = await agent(
`Repo ${ROOT}. Restore the clean baseline from ${BAK} first. Then apply the BEST COMPATIBLE COMBINATION of these tournament winners — prioritize morphology wins (wing-feathers, mirror-proof, assembly-pose-v2) since the gate is blocked on morphology, not pose alone:

${ranked.slice(0, 5).map((v, i) => `### #${i + 1} ${v.strategy} (iou=${v.silhouette_iou}, edge=${v.edge_similarity}) — ${v.morphology_progress || ''}\n${v.summary}\n\`\`\`diff\n${(v.diff || '').slice(0, 5000)}\n\`\`\``).join('\n\n')}

Apply, then \`ggen sync && python3 scripts/render_reference_fabric.py && python3 scripts/compare_reference_render.py\`. If a combined piece regresses below the best single variant, back it out and re-run. Report combined metrics + whether thresholds_met / morphology gate improved. Leave the winning combination IN PLACE on the live tree (do not restore). Do not touch the scorer.`,
  { schema: FINAL, phase: 'Graft' })

phase('Verify')
const final = await agent(
`Repo ${ROOT}. Adversarially verify the grafted result is REAL:
1. Re-run python3 scripts/compare_reference_render.py fresh; confirm numbers reproduce exactly.
2. \`git hash-object scripts/compare_reference_render.py scripts/render_reference_fabric.py\` vs HEAD — confirm scorer+renderer UNCHANGED (any change invalidates the result).
3. Confirm owner_part_id complete on emitted USD and modular gate via run_mecha_doe.py smoke = DOE_RELEASED.
4. Report final metrics vs baseline ${JSON.stringify({ s: base.silhouette_iou, e: base.edge_similarity, c: base.color_palette_similarity })}, thresholds_met honestly, and set metrics_are_real truthfully (false if anything fails to reproduce).`,
  { schema: FINAL, phase: 'Verify' })

return { base, grafted: graft, verified: final, top_variants: ranked.slice(0, 3) }
