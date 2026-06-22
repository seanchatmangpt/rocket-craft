export const meta = {
  name: 'residual-repair',
  description: 'REPLAY-DISCIPLINED convergence. Precondition: replay is real (verify_asset.sh). Encode narrow QUDT-typed prior bands for the 3 failing gates, then replace blind variant search with a residual-vector repair loop: measure residual -> bounded operator -> patch SOURCE LAW -> double-clean replay -> accept only if residual drops AND metrics replay. Output ONLY the four reports.',
  phases: [
    { title: 'Replay-Gate' },
    { title: 'Prior-Bands' },
    { title: 'Residual-Loop' },
    { title: 'Gate-Status' },
  ],
}

const ROOT = '/Users/sac/rocket-craft'
// The three failing morphology gates this run is allowed to touch.
const GATES = ['foreground_component_count', 'core_compactness_delta', 'blade_length_angle_delta']

const REPLAY_REPORT = {
  type: 'object',
  required: ['wrapper_exists', 'run1', 'run2', 'identical', 'scorer_unchanged', 'no_derived_edits', 'verdict'],
  properties: {
    wrapper_exists: { type: 'boolean', description: 'scripts/verify_asset.sh exists and does ggen sync -> delete stale renders -> render -> compare -> emit report' },
    run1: { type: 'object', description: 'metrics from a clean delete-and-resync run' },
    run2: { type: 'object', description: 'metrics from a second clean delete-and-resync run' },
    identical: { type: 'boolean', description: 'run1 == run2 to 4 decimals (replay law holds)' },
    scorer_unchanged: { type: 'boolean', description: 'compare_reference_render.py + render_reference_fabric.py byte-identical to git HEAD' },
    no_derived_edits: { type: 'boolean', description: 'no progress depends on edits to all_merged.ttl or other regenerated artifacts' },
    verdict: { type: 'string', enum: ['REPLAY_OK', 'REPLAY_FAIL'] },
  },
}

const PRIOR_REPORT = {
  type: 'object',
  required: ['priors', 'authored_in', 'survives_sync'],
  properties: {
    priors: {
      type: 'array',
      description: 'one entry per failing gate',
      items: {
        type: 'object',
        required: ['gate', 'unit', 'target_band', 'preferred_band', 'forbidden_band', 'exception_class', 'evidence_tier', 'repair_operators'],
        properties: {
          gate: { type: 'string' },
          unit: { type: 'string', description: 'QUDT-style unit/quantity (ratio, count, degree, meter)' },
          target_band: { type: 'string' },
          preferred_band: { type: 'string' },
          forbidden_band: { type: 'string' },
          exception_class: { type: 'string', description: 'archetype that permits the forbidden band, or "none"' },
          evidence_tier: { type: 'string', enum: ['T0_measured', 'T1_lore', 'T2_derived', 'T3_archetype_estimate', 'T4_negative_bound', 'T5_exception'] },
          repair_operators: { type: 'array', items: { type: 'string' }, description: 'named bounded operators that move this gate toward its band' },
        },
      },
    },
    authored_in: { type: 'array', items: { type: 'string' }, description: 'authoritative source_law/template files the bands were written into (NOT all_merged.ttl)' },
    survives_sync: { type: 'boolean', description: 'bands present after two clean ggen syncs' },
  },
}

const RESIDUAL_REPORT = {
  type: 'object',
  required: ['steps', 'accepted_count', 'reverted_count', 'notes'],
  properties: {
    steps: {
      type: 'array',
      items: {
        type: 'object',
        required: ['gate', 'residual_before', 'operator', 'residual_after', 'replayed', 'accepted'],
        properties: {
          gate: { type: 'string' },
          residual_before: { type: 'number' },
          operator: { type: 'string' },
          residual_after: { type: 'number' },
          replayed: { type: 'boolean', description: 'gain reproduced across two clean delete-and-resync runs' },
          accepted: { type: 'boolean', description: 'kept only if residual dropped AND replayed' },
        },
      },
    },
    accepted_count: { type: 'number' },
    reverted_count: { type: 'number' },
    notes: { type: 'string' },
  },
}

const GATE_STATUS = {
  type: 'object',
  required: ['gates', 'morphology_ok', 'thresholds_met', 'aggregate_metrics', 'all_replayed'],
  properties: {
    gates: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          gate: { type: 'string' },
          value: { type: 'number' },
          in_band: { type: 'boolean' },
        },
      },
    },
    morphology_ok: { type: 'boolean' },
    thresholds_met: { type: 'boolean' },
    aggregate_metrics: { type: 'object', description: 'replay-verified silhouette_iou / edge_similarity / color_palette_similarity' },
    all_replayed: { type: 'boolean' },
  },
}

phase('Replay-Gate')
const replay = await agent(
`Repo ${ROOT}. HARD PRECONDITION. Confirm the pipeline is replay-safe before ANY convergence work:
1. Confirm scripts/verify_asset.sh exists and runs: ggen sync -> delete stale renders (generated/mech_assets/reference_fabric_001/renders/*.png + reports/visual_gap_report.json) -> python3 scripts/render_reference_fabric.py -> python3 scripts/compare_reference_render.py -> print metrics. If it does NOT exist or is incomplete, STOP and return verdict=REPLAY_FAIL (do not create it — that was the prior workflow's job; report the gap).
2. Run it once (run1), delete renders+report, run it again (run2). Confirm run1==run2 to 4 decimals.
3. git hash-object scripts/compare_reference_render.py scripts/render_reference_fabric.py == HEAD (unchanged).
4. Confirm no current progress depends on edits to ontology/all_merged.ttl or other regenerated artifacts (grep for stray meshProof tags in all_merged.ttl that are NOT derived from source_law/inference).
Return verdict=REPLAY_OK only if identical AND scorer_unchanged AND no_derived_edits. Otherwise REPLAY_FAIL.`,
  { schema: REPLAY_REPORT, phase: 'Replay-Gate' })

if (!replay || replay.verdict !== 'REPLAY_OK') {
  log('Replay gate FAILED — refusing to do convergence on a non-replayable pipeline. Returning REPLAY_VERIFICATION_REPORT only.')
  return { REPLAY_VERIFICATION_REPORT: replay, PRIOR_BAND_REPORT: null, RESIDUAL_OPERATOR_REPORT: null, NEXT_GATE_STATUS: { blocked_on: 'REPLAY_FAIL' } }
}

phase('Prior-Bands')
const priors = await agent(
`Repo ${ROOT}. Replay is verified. Encode a NARROW prior-band layer for ONLY these three failing gates: ${GATES.join(', ')}. Author them in AUTHORITATIVE source law (a new ontology/source_law/NNN_proportion_priors.ttl and/or the relevant grammar TTL) — NEVER in all_merged.ttl. Use public-ontology patterns (QUDT for unit/quantity, SHACL-style min/preferred/max bands, an exception class where justified).

For EACH gate define: unit (count / ratio / degree), target_band, preferred_band, forbidden_band, exception_class (or none), evidence_tier (T0_measured..T5_exception — these are T3_archetype_estimate / T4_negative_bound, back-of-napkin bounded), and a list of named bounded repair_operators (e.g. for foreground_component_count: compose_adjacent_parts_within_overlap_band, merge_feather_components_preserving_wingspan; for core_compactness_delta: tighten_core_mass_inside_envelope; for blade_length_angle_delta: set_blade_mount_angle_within_band, set_blade_length_ratio_within_band).

Reasonable starting bands (refine from the reference): foreground_component_count target [1,5] preferred [1,3] forbidden >8; core_compactness_delta target <=0.15 preferred <=0.10 forbidden >0.30; blade_length_angle_delta target <=15 preferred <=8 forbidden >60. Then run scripts/verify_asset.sh and a SECOND ggen sync to confirm the bands survive regeneration. Return survives_sync honestly.`,
  { schema: PRIOR_REPORT, phase: 'Prior-Bands' })

phase('Residual-Loop')
// Residual-vector repair, NOT blind tournament. Per gate: measure -> pick bounded operator from its
// prior -> patch SOURCE LAW -> double-clean replay -> accept only if residual drops AND replays.
const loop = await agent(
`Repo ${ROOT}. Run the RESIDUAL-VECTOR REPAIR LOOP for the three gates ${GATES.join(', ')} using the prior bands just authored:

${JSON.stringify(priors, null, 2)}

For each gate, repeat up to 3 steps:
  a. Measure current value via scripts/verify_asset.sh; compute residual = distance outside the target band (0 if in band).
  b. If residual==0, gate is done — skip.
  c. Select ONE bounded repair operator from that gate's repair_operators. Apply it by patching AUTHORITATIVE SOURCE LAW / templates ONLY (never all_merged.ttl, never the scorer/renderer). Stay within the operator's band — do NOT overshoot (overshoot is exactly what collapsed the prior blind tournament).
     AUTHORIZED COLOR OPERATOR (blade_length_angle_delta ONLY): this gate is currently UNRESPONSIVE to geometry because the scorer measures the blade from CYAN pixels and blades render gray — materials.mtlx.tera emits an empty <nodegraph> so material color never reaches usdrecord. The cyan IS in source law (104_reference_fabric.ttl: mud:color "#00FFFF" on the blade material). FIX FORWARD: emit \`color3f[] primvars:displayColor = [(r,g,b)]\` on each Mesh prim in generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera, deriving (r,g,b) from the prim's mud:color (project it through the SPARQL SELECT if not already available). This is PROVEN to make usdrecord render cyan. Apply this FIRST for the blade gate so cyan renders; THEN the geometric operators (mount angle / length ratio) become measurable. Watch color_palette_similarity for regression (giving all parts their true colors should help, not hurt) and revert if it drops materially or fails to replay.
  d. REPLAY-VERIFY: run scripts/verify_asset.sh, delete renders, run it AGAIN. Accept the patch ONLY if (residual_after < residual_before) AND the two clean runs are identical to 4 decimals AND no aggregate metric (silhouette_iou/color) regressed materially. Otherwise REVERT the patch.
Record every step (gate, residual_before, operator, residual_after, replayed, accepted). Do not claim a gain that does not replay. Report accepted_count / reverted_count honestly.`,
  { schema: RESIDUAL_REPORT, phase: 'Residual-Loop' })

phase('Gate-Status')
const status = await agent(
`Repo ${ROOT}. Final REPLAY-VERIFIED status. Run scripts/verify_asset.sh twice from clean (delete renders between) and confirm identical. Report each of the three gates (${GATES.join(', ')}) with its current value and whether it is in its target band; morphology_ok; thresholds_met; the replay-verified aggregate metrics (silhouette_iou/edge_similarity/color_palette_similarity); and all_replayed (true iff the two clean runs matched). Honest only — a gate is in_band only if it replays.`,
  { schema: GATE_STATUS, phase: 'Gate-Status' })

return {
  REPLAY_VERIFICATION_REPORT: replay,
  PRIOR_BAND_REPORT: priors,
  RESIDUAL_OPERATOR_REPORT: loop,
  NEXT_GATE_STATUS: status,
}
