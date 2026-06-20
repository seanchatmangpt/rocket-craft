export const meta = {
  name: 'moonshot-converge',
  description: 'Converge mech silhouette + surface detail toward the winged reference via a parallel parameter-variant tournament (worktree-isolated ggen sync -> usdrecord -> IoU/edge), then graft the winning feature-graph parameters onto the live tree',
  phases: [
    { title: 'Baseline' },
    { title: 'Explore' },
    { title: 'Graft' },
    { title: 'Verify' },
  ],
}

const ROOT = '/Users/sac/rocket-craft'

const METRIC = {
  type: 'object',
  required: ['silhouette_iou', 'edge_similarity', 'color_palette_similarity'],
  properties: {
    silhouette_iou: { type: 'number' },
    edge_similarity: { type: 'number' },
    color_palette_similarity: { type: 'number' },
  },
}

const VARIANT = {
  type: 'object',
  required: ['strategy', 'silhouette_iou', 'edge_similarity', 'color_palette_similarity', 'ran_clean', 'diff', 'summary'],
  properties: {
    strategy: { type: 'string' },
    silhouette_iou: { type: 'number' },
    edge_similarity: { type: 'number' },
    color_palette_similarity: { type: 'number' },
    ran_clean: { type: 'boolean', description: 'ggen sync + render + compare all exited 0 in the isolated worktree' },
    diff: { type: 'string', description: 'unified git diff of the source-law TTL / Tera changes that produced these metrics' },
    summary: { type: 'string' },
  },
}

const FINAL = {
  type: 'object',
  required: ['silhouette_iou', 'edge_similarity', 'color_palette_similarity', 'modular_gate', 'ran_clean', 'metrics_are_real', 'notes'],
  properties: {
    silhouette_iou: { type: 'number' },
    edge_similarity: { type: 'number' },
    color_palette_similarity: { type: 'number' },
    modular_gate: { type: 'string' },
    ran_clean: { type: 'boolean' },
    metrics_are_real: { type: 'boolean', description: 'true iff a fresh independent re-run of compare reproduced the numbers and the scorer is not hardcoded' },
    notes: { type: 'string' },
  },
}

phase('Baseline')
const base = await agent(
`Repo ${ROOT}. Establish the current convergence baseline. Run:
  cd ${ROOT} && ggen sync && python3 scripts/render_reference_fabric.py && python3 scripts/compare_reference_render.py
Then read generated/mech_assets/reference_fabric_001/reports/visual_gap_report.json and report silhouette_iou / edge_similarity / color_palette_similarity. Read scripts/compare_reference_render.py and scripts/render_reference_fabric.py enough to understand exactly which knobs (camera, threshold, alignment) affect the score so later variants don't game it. Do not edit anything.`,
  { schema: METRIC, phase: 'Baseline' })

phase('Explore')
// Each strategy attacks a different axis of the reference (winged Gundam: swept feather wings,
// posed silhouette, sharp hard-surface profiles, engraved surface detail). Worktree-isolated so
// each runs the full ggen sync -> render -> compare loop without colliding on generated/.
const STRATEGIES = [
  { name: 'wing-spine-arc', detail: 'Tune the feather-array placement spine + pitch sweep so the two wing arrays read as the large swept feather fans of the reference (wide, upward-back swept, ~18-22 feathers each, tip-scale falloff). Edit the pattern-along-curve rule + spine curve in the source-law TTL.' },
  { name: 'assembly-pose', detail: 'The parts currently render as an exploded scatter. Author the assembly composition transforms so part files compose into a single coherent posed mech silhouette (torso centered, head atop, wings rising behind, limbs/blades placed). This should be the biggest silhouette_iou win.' },
  { name: 'feather-profile', detail: 'Sharpen the feather/blade cross-section profile curve to the tapered pointed-feather profile of the reference instead of a blunt blade. Edit the loft profile sketch in the feather feature tree.' },
  { name: 'hardsurface-bevels', detail: 'Add beveled hard-surface plate detail + chamfers to torso/head/limb shells (047_angular_armor_shell_grammar / 048 beveled plate grammar) so profiles read as flagship hard-surface, lifting edge_similarity.' },
  { name: 'panel-line-density', detail: 'Increase engraved panel-line / vent density and depth across all parts so edge_similarity rises substantially. Build on whatever surface grammar exists; if absent, author it.' },
  { name: 'head-torso-silhouette', detail: 'Reshape head crest + torso chest profile toward the V-fin head and layered chest of the reference. Edit torso/head grammar profiles.' },
]

const variants = await parallel(STRATEGIES.map((s) => () => agent(
`You are in an ISOLATED git worktree copy of ${ROOT} — edit and run freely, you cannot affect other agents. Pipeline: \`ggen sync\` renders Tera templates from the RDF source-law graph (ontology/source_law/*.ttl + ontology/ggen-packs) into generated/mech_assets/reference_fabric_001/usd/*.usda; scripts/render_reference_fabric.py runs usdrecord (Metal); scripts/compare_reference_render.py writes visual_gap_report.json with silhouette_iou/edge_similarity/color_palette_similarity. The reference is a white winged Gundam (swept feather wings, V-fin head, layered chest, twin beam swords).

Baseline metrics: ${JSON.stringify(base)}.

Your strategy "${s.name}": ${s.detail}

Author the change in the AUTHORITATIVE source-law TTL / ggen-pack template (never edit files under generated/ — they are ggen sync outputs). Then run:
  cd . && ggen sync && python3 scripts/render_reference_fabric.py && python3 scripts/compare_reference_render.py
Read the report. Iterate up to 3 times to maximize improvement on YOUR axis without regressing the others badly. Do NOT touch the scorer scripts and do NOT hardcode metrics — that is cheating and will be caught. Return your final metrics and \`git diff\` of your source changes.`,
  { schema: VARIANT, isolation: 'worktree', phase: 'Explore', label: s.name })))

const ranked = variants.filter(Boolean).filter((v) => v.ran_clean)
  .map((v) => ({ ...v, score: (v.silhouette_iou - base.silhouette_iou) + 2 * (v.edge_similarity - base.edge_similarity) }))
  .filter((v) => v.score > 0)
  .sort((a, b) => b.score - a.score)

log(`Explore done: ${ranked.length}/${variants.length} variants improved on baseline. Top: ${ranked.slice(0, 3).map((v) => `${v.strategy}(iou=${v.silhouette_iou?.toFixed(3)},edge=${v.edge_similarity?.toFixed(3)})`).join(', ')}`)

if (!ranked.length) {
  return { base, result: 'NO_VARIANT_IMPROVED', variants: variants.filter(Boolean) }
}

phase('Graft')
const graft = await agent(
`Repo ${ROOT} (the LIVE working tree, which already contains the finish-runner foundation: owner_part_id stamping + base surface grammar). Below are the winning source-law diffs from worktree-isolated tournament variants, ranked best-first. Apply the BEST COMPATIBLE COMBINATION onto the live tree — prefer assembly-pose + wing-spine + the strongest surface/profile wins; resolve overlaps sensibly; skip any diff that conflicts destructively with a better one.

${ranked.slice(0, 5).map((v, i) => `### #${i + 1} ${v.strategy} (silhouette_iou=${v.silhouette_iou}, edge_similarity=${v.edge_similarity})\n${v.summary}\n\n\`\`\`diff\n${(v.diff || '').slice(0, 6000)}\n\`\`\``).join('\n\n')}

After applying, run: cd ${ROOT} && ggen sync && python3 scripts/render_reference_fabric.py && python3 scripts/compare_reference_render.py. Report the combined metrics honestly. If the combination regressed below the best single variant, back out the conflicting piece and re-run. Do not edit the scorer.`,
  { schema: FINAL, phase: 'Graft' })

phase('Verify')
const final = await agent(
`Repo ${ROOT}. Adversarially verify the grafted result is REAL, not gamed:
1. Re-run scripts/compare_reference_render.py fresh and confirm the numbers reproduce.
2. Diff scripts/compare_reference_render.py and scripts/render_reference_fabric.py against git HEAD — confirm the scorer/renderer were NOT modified (any change there invalidates the result).
3. Confirm owner_part_id is still complete on emitted USD and the modular-identity gate is still DOE_RELEASED (run ggen-asset-lsp diagnostics / run_mecha_doe.py smoke).
4. Report final silhouette_iou / edge_similarity / color vs baseline ${JSON.stringify(base)}, and set metrics_are_real honestly.`,
  { schema: FINAL, phase: 'Verify' })

return { base, grafted: graft, verified: final, top_variants: ranked.slice(0, 3) }
