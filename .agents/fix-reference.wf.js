export const meta = {
  name: 'fix-reference',
  description: 'Fix the CORRUPT convergence target: re-segment the white winged mech from the light-gray gradient background in reference_original.jpg (crop caption bars, match render white-on-black polarity), recompute reference silhouette/edges/measurements/color_histogram from the FOREGROUND only, then re-baseline via verify_asset.sh under replay discipline. Until this lands, all silhouette/morphology/color metrics are measured against garbage.',
  phases: [
    { title: 'Fix-Extractor' },
    { title: 'Re-Baseline' },
  ],
}

const ROOT = '/Users/sac/rocket-craft'
const REF = 'generated/mech_assets/reference_fabric_001/reference'

const FIX = {
  type: 'object',
  required: ['mask_looks_correct', 'foreground_fill', 'caption_cropped', 'polarity_matches_render', 'files_changed', 'targets_regenerated', 'notes'],
  properties: {
    mask_looks_correct: { type: 'boolean', description: 'the regenerated reference_silhouette.png, viewed, depicts the winged mech as solid white-on-black (NOT the background, NOT caption text)' },
    foreground_fill: { type: 'number', description: 'fraction of frame that is mech foreground after fix (should be far below the broken 0.814 — the mech occupies maybe ~0.3-0.5 of frame)' },
    caption_cropped: { type: 'boolean' },
    polarity_matches_render: { type: 'boolean', description: 'white mech on black, same convention as renders/render_silhouette.png' },
    files_changed: { type: 'array', items: { type: 'string' } },
    targets_regenerated: { type: 'array', items: { type: 'string' }, description: 'reference_silhouette.png, reference_edges.png, reference_measurements.json, reference_color_histogram.json' },
    notes: { type: 'string' },
  },
}

const BASE = {
  type: 'object',
  required: ['silhouette_iou', 'edge_similarity', 'color_palette_similarity', 'replays', 'vis_errors', 'interpretation', 'notes'],
  properties: {
    silhouette_iou: { type: 'number' }, edge_similarity: { type: 'number' }, color_palette_similarity: { type: 'number' },
    replays: { type: 'boolean' },
    vis_errors: { type: 'array', items: { type: 'string' } },
    interpretation: { type: 'string', description: 'how the new (meaningful) baseline differs from the old garbage-target baseline, and whether adding wings should now HELP silhouette' },
    notes: { type: 'string' },
  },
}

phase('Fix-Extractor')
const fix = await agent(
`${ROOT}. The convergence TARGET is corrupt. scripts/extract_reference_visual_targets.py produced ${REF}/reference_silhouette.png by naive thresholding of ${REF}/reference_original.jpg — but the source is a WHITE winged Mecha on a LIGHT-GRAY GRADIENT background with Japanese caption bars at the bottom. The naive threshold grabbed the BACKGROUND (so the silhouette is ~81% filled, inverted) and included the caption text. The renders/render_silhouette.png convention is white-mech-on-BLACK; the reference is opposite. So silhouette_iou compares our mech against the reference BACKGROUND — garbage. The color_histogram is likewise background-contaminated (white=0.42 is the bg).

FIX scripts/extract_reference_visual_targets.py (this is the TARGET-extraction script, NOT the sacred scorer compare_reference_render.py or renderer — do not touch those):
1. CROP the bottom caption-text bars out of reference_original.jpg before segmentation (the white text on the lower region).
2. SEGMENT the white/off-white mech from the smooth light-gray gradient background. White-on-light-gray is low contrast — a single global threshold WILL fail. Use a robust method: e.g. GrabCut seeded with a center foreground rect + border background, or background-gradient modeling/subtraction, or combine edge density + local contrast + the colored regions (cyan sabers, yellow V-fin, red decals are definitely foreground). Iterate until the mask depicts the winged mech.
3. Output reference_silhouette.png as WHITE MECH ON BLACK (same polarity as renders/render_silhouette.png).
4. Recompute reference_edges.png, reference_measurements.json, and reference_color_histogram.json from the FOREGROUND MASK ONLY (so 'white' means white ARMOR, not background; expect white to remain high since the mech is white, but background pixels must be excluded; cyan/yellow/red small).
5. VISUALLY VERIFY: after regenerating, use your Read tool to VIEW ${REF}/reference_silhouette.png and confirm it depicts the winged mech as solid white-on-black (wings spread, twin sabers), not the background and not caption text. Iterate until it does.
Run the fixed extractor to regenerate all four reference targets. Report mask_looks_correct (only true if you visually confirmed), foreground_fill, and the files changed.`,
  { schema: FIX, phase: 'Fix-Extractor' })

phase('Re-Baseline')
const base = await agent(
`${ROOT}. The reference target was just corrected (re-segmented winged mech, caption cropped, foreground-only histogram, white-on-black polarity). Establish the NEW MEANINGFUL baseline:
1. Run \`bash scripts/verify_asset.sh\` (it deletes stale renders, re-renders fresh, scores against the corrected reference). Read generated/mech_assets/reference_fabric_001/reports/visual_gap_report.json.
2. Prove replay: delete renders+report, run again, confirm identical to 4 decimals.
3. Report silhouette_iou / edge_similarity / color_palette_similarity / vis_errors against the corrected target.
4. INTERPRET: the old baseline (0.68 silhouette) was measured against garbage (a compact blob overlapping the inverted background). Against the CORRECT winged target, the current featherless body should now score LOWER on silhouette (it lacks the wing spread the reference has) — and adding the proven wing grammar should now HELP rather than hurt. State whether the numbers confirm this directional flip. Do not touch the scorer/renderer. Honest numbers only.`,
  { schema: BASE, phase: 'Re-Baseline' })

return { fix, base }
