export const meta = {
  name: 'port-to-generator',
  description: 'Make the converged asset SOURCE-LAW REPLAYABLE: port the current converged part_mesh.usda.tera (bilateral mirror, VIS203 subdivided feathers, VIS205 cyan sabers, body segmentation) into the TRUE source patch_geometry_generator.py (which embeds + rewrites that template on every canonical POWL run), so vision_powl_executor.py reproduces the passing asset. Then run the deep delete-and-resync replay and confirm the canonical pipeline reproduces empty vis_errors + thresholds. Fix generator byte-nondeterminism if it blocks replay.',
  phases: [
    { title: 'Port' },
    { title: 'DeepReplay' },
  ],
}

const ROOT = '/Users/sac/rocket-craft'
const ASSET = 'generated/mech_assets/reference_fabric_001'
const TMPL = `${ASSET}/templates/usd/part_mesh.usda.tera`

const PORT = {
  type: 'object',
  required: ['ported', 'generator_emits_match', 'canonical_vis_errors', 'canonical_silhouette', 'canonical_color', 'notes'],
  properties: {
    ported: { type: 'boolean', description: 'patch_geometry_generator.py embedded template replaced with the converged template content' },
    generator_emits_match: { type: 'boolean', description: 'running patch_geometry_generator.py reproduces the converged template byte-for-byte (diff clean vs the converged template)' },
    canonical_vis_errors: { type: 'array', items: { type: 'string' }, description: 'vis_errors after the CANONICAL pipeline (patch_geometry_generator.py -> ggen sync -> render -> compare)' },
    canonical_silhouette: { type: 'number' },
    canonical_color: { type: 'number' },
    notes: { type: 'string' },
  },
}

const REPLAY = {
  type: 'object',
  required: ['deep_replay_status', 'vis_errors', 'silhouette_iou', 'color_palette_similarity', 'thresholds_met', 'nondeterministic_artifacts', 'claim', 'notes'],
  properties: {
    deep_replay_status: { type: 'string', description: 'PASS/REFUSED from scripts/verify_delete_and_resync_replay.py' },
    vis_errors: { type: 'array', items: { type: 'string' } },
    silhouette_iou: { type: 'number' }, color_palette_similarity: { type: 'number' }, thresholds_met: { type: 'boolean' },
    nondeterministic_artifacts: { type: 'number', description: 'count of byte-nondeterministic artifacts the deep-replay flags (0 if fully deterministic)' },
    claim: { type: 'string', enum: ['HOLD', 'PRE_UE4_HERO_ASSET_ADMITTED'] },
    notes: { type: 'string' },
  },
}

phase('Port')
const port = await agent(
`${ROOT}. ROOT CAUSE: patch_geometry_generator.py embeds the part_mesh.usda.tera template as a Python string (\`part_mesh_tera = """..."""\`, "# 1. Write part_mesh.usda.tera") and OVERWRITES ${TMPL} on every canonical POWL run (scripts/vision_powl_executor.py runs \`python3 patch_geometry_generator.py && ggen sync\`). The embedded copy is STALE (old is_right mirror negation at ~line 98, old thin 2-row feathers, no cyan blade branch, no body segmentation). Our converged fixes live only in the generated ${TMPL} and get wiped -> the canonical pipeline reconstructs the FAILING 0.41 asset.

FIX: Replace the embedded \`part_mesh_tera\` string in patch_geometry_generator.py with the EXACT current converged content of ${TMPL} (which has: bilateral mirror fix = no is_right negation; VIS203 subdivided 3-row curved feather plates; VIS205 \`{% elif row.type == "blade" %}\` cyan_beam branch with displayColor; body hard_surface_shell segmentation; owner_part_id). Preserve Python string escaping correctly. Do NOT change any OTHER file patch_geometry_generator.py writes unless it also embeds a stale copy needed for the asset.

VERIFY: run \`python3 patch_geometry_generator.py\` then \`diff ${TMPL}\` against a saved copy of the converged template — they MUST match (generator now emits the converged template). Then run the CANONICAL pipeline once: \`python3 patch_geometry_generator.py && python3 scripts/merge_ontology.py && ggen sync && python3 scripts/render_reference_fabric.py && python3 scripts/compare_reference_render.py\` and report canonical vis_errors / silhouette / color. Goal: canonical pipeline now yields the converged result (vis_errors empty or near, silhouette ~0.53, color ~0.95). Edit only patch_geometry_generator.py. Do NOT touch the scorer/renderer.`,
  { schema: PORT, phase: 'Port' })

phase('DeepReplay')
const replay = await agent(
`${ROOT}. Now prove DEEP delete-and-resync replay with the canonical pipeline. Run \`python3 scripts/verify_delete_and_resync_replay.py\` (it deletes usd/renders/etc and rebuilds via scripts/vision_powl_executor.py, then byte-compares). Report deep_replay_status (PASS/REFUSED) and the count of nondeterministic_artifacts.

If REFUSED due to byte-nondeterminism (unseeded ordering / volatile embedded content like timestamps in the generators), FIX the generator determinism: seed any ordering (sort keys/rows), remove embedded timestamps/volatile content from emitted .usda/.mtlx, so repeated canonical runs are byte-identical. Edit only the generators (patch_geometry_generator.py / scripts that emit usd/materialx) — NOT the scorer/renderer. Re-run the deep replay until status=PASS or until the only differences are render PNGs whose METRICS still reproduce (note this distinction).

Then run \`bash scripts/verify_asset.sh\` to get the final metrics + vis_errors and set claim=PRE_UE4_HERO_ASSET_ADMITTED ONLY if: deep_replay reproduces the disposition (same vis_errors + thresholds) AND vis_errors empty AND thresholds_met AND silhouette>=0.25 & color>=0.50. Otherwise HOLD with the precise remaining blocker. Report honestly.`,
  { schema: REPLAY, phase: 'DeepReplay' })

return { port, replay }
