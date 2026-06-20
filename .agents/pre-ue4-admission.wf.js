export const meta = {
  name: 'pre-ue4-admission',
  description: 'Manufacture the $5M pre-UE4 hero asset ADMISSION package under CONTINUE_REPAIR / CLAIM_HOLD. 10 agents in admission order: source-law replay gate -> triage+authoring -> modular identity -> POWL/wasm4pm admission -> IP-distance -> fresh render -> materials/textures -> residual-vector repair -> delete-and-resync replay (KEYSTONE) -> synthesis. Standing counts only when source-law-derived/fresh-rendered/residual-measured/bounded-repaired/receipt-linked/delete-and-resync-replayed. No false PASS. No UE4.',
  phases: [
    { title: 'R2a-Source-Law-Replay-Gate' },
    { title: 'R2b-Triage-and-Authoring' },
    { title: 'R1-Modular-Identity' },
    { title: 'R3-POWL-Admission' },
    { title: 'IP-Distance' },
    { title: 'R4-Fresh-Render' },
    { title: 'Materials-Textures' },
    { title: 'R5-Residual-Vector' },
    { title: 'R6-Delete-Resync-Replay' },
    { title: 'Synthesis-Admission' },
  ],
}

const ROOT = '/Users/sac/rocket-craft'
const V = 'bash scripts/verify_asset.sh'                 // merge->ggen sync->delete stale->render->compare
const WPM = '/Users/sac/wasm4pm/target/debug/wpm'        // installed wasm4pm CLI
const ASSET = 'generated/mech_assets/reference_fabric_001' // CANONICAL emit target

// Non-negotiable doctrine embedded in every agent prompt.
const LAW = `DOCTRINE — CONTINUE_REPAIR / CLAIM_HOLD (non-negotiable):
- You NEVER idle and NEVER ask whether to continue. You fix forward until the gate's evidence exists.
- Standing counts ONLY when evidence is source-law-derived / fresh-rendered / residual-measured / bounded-repaired / receipt-linked / delete-and-resync-replayed. File creation, syntax-valid, LSP-parsed, "renders exist", "script ran", and "metrics changed" do NOT count as standing.
- You MAY return standing:'ADMITTED' ONLY IF your report_emitted file exists on disk AND was produced by the named verifier (verify_asset.sh / the residual scorer / ip_distance_engine.py / the replay harness / wpm) — NOT by you asserting it. If the evidence file does not exist, return PARTIAL_ALIVE or UNKNOWN with next_action = the exact file/tool/gate that would produce it.
- EDIT authoritative source ONLY: ontology/source_law/*.ttl, the ggen-pack templates (.tera) / SPARQL (.rq) / ggen.toml, and (this run, EXPLICITLY ALLOWED) the named code-defect files. NEVER edit ontology/all_merged.ttl as source (it is regenerated). NEVER edit scripts/compare_reference_render.py or scripts/render_reference_fabric.py except the single allowed render_hash provenance stamp. NEVER hand-edit files under ${ASSET}/usd|renders (ggen outputs).
- Metrics measured ONLY via \`${V}\` (deletes stale renders, renders fresh). A gain is KEPT only if it survives delete-and-resync replay TWICE (two clean runs identical to 4 decimals) and does not materially regress color_palette_similarity or silhouette_iou — else REVERT fully and report it.
- ${ASSET}/ is the CANONICAL deliverable; final_mech_asset/ is only a packaging view (do not create competing ggen authority for it).
- Receipts are BLAKE3 everywhere.
- IP law: baselines (Mecha/BiotechMech/ArmorFrame/GlobalCorp/Studio/tank/kit) are usable_for_metric_baseline=true, usable_for_generation=false. Original shape language only; never copy protected silhouettes/marks/names.
- End with EXACTLY one standing verdict and a concrete next_action. No false standing.`

// Shared structured-output schema. Every agent returns this shape.
const STANDING = {
  type: 'object',
  required: ['workstream', 'standing', 'next_action', 'evidence_paths', 'report_emitted', 'replay_verified', 'notes'],
  properties: {
    workstream: { type: 'string', description: 'R2a/R2b/R1/R3/IP/R4/Materials/R5/R6/Synthesis' },
    standing: { type: 'string', enum: ['ADMITTED', 'PARTIAL_ALIVE', 'REFUSED', 'UNKNOWN'] },
    next_action: { type: 'string', description: 'exact file/tool/gate to run or repair next' },
    evidence_paths: { type: 'array', items: { type: 'string' }, description: 'files that MUST exist on disk for this standing' },
    report_emitted: { type: 'string', description: 'path to the verifier report this agent produced (empty string if none)' },
    replay_verified: { type: 'boolean', description: 'gain reproduced across two clean delete-and-resync runs' },
    notes: { type: 'string' },
    metrics: { type: 'object', description: 'optional: relevant fresh metrics' },
  },
}

// ---------------------------------------------------------------- R2a (HARD GATE)
phase('R2a-Source-Law-Replay-Gate')
const r2a = await agent(
`${LAW}

Repo ${ROOT}. WORKSTREAM R2a — SOURCE-LAW REPLAY GATE (hard precondition for the whole run).
1. Author scripts/verify_source_law_replay.py (read-only proof; writes only SOURCE_LAW_REPLAY_REPORT.json + .md at repo root): snapshot b3sum of ontology/all_merged.ttl; capture HEAD via \`git show HEAD:ontology/all_merged.ttl\`; delete all_merged.ttl; run \`python3 scripts/merge_ontology.py\`; b3sum the fresh merge; assert every ontology/source_law/*.ttl appears exactly once as a \`# --- Source: <name> ---\` banner; contamination check = (HEAD has a banner per source file) AND (fresh merge byte-identical to HEAD) AND (no orphan triples tracing to no source file). Record per-source-file b3sum (the R2 receipt leaves).
2. Run it. Also confirm \`${V}\` exists and runs the full chain, run it twice from clean (delete ${ASSET}/renders/*.png + reports/visual_gap_report.json between), assert run1==run2 to 4 decimals. \`git hash-object scripts/compare_reference_render.py scripts/render_reference_fabric.py\` == HEAD.
KNOWN: HEAD all_merged.ttl is hand-merged (3823 lines, 0 banners); fresh merge ~1445 lines/104 banners — so contamination check WILL fail this run. That is correct: set standing=PARTIAL_ALIVE (not REFUSED — decision is auto-triage+continue), report_emitted=SOURCE_LAW_REPLAY_REPORT.json, next_action="R2b: triage the 3573-line HEAD delta into source_law/*.ttl then regenerate". Only set standing=REFUSED if verify_asset.sh itself is broken or the scorer/renderer differ from HEAD (a real hard stop).`,
  { schema: STANDING, phase: 'R2a-Source-Law-Replay-Gate', label: 'R2a-replay-gate' })

// Hard stop only if the spine itself is broken.
if (!r2a || r2a.standing === 'REFUSED') {
  log('R2a REFUSED — the verify spine itself is broken. Short-circuiting to Synthesis = HOLD.')
  const blocked = await agent(
`${LAW}

Repo ${ROOT}. WORKSTREAM Synthesis. The R2a replay gate REFUSED (the verify_asset.sh spine or the sacred scorer/renderer is broken): ${JSON.stringify(r2a)}. Write NEXT_GATE_STATUS.md with claim=HOLD and next_gap = the exact repair needed to make scripts/verify_asset.sh replay-safe. Do not claim any downstream standing.`,
    { schema: STANDING, phase: 'Synthesis-Admission', label: 'synthesis-blocked' })
  return { r2a, blocked, claim: 'HOLD', short_circuit: 'R2a' }
}

// ---------------------------------------------------------------- R2b triage + authoring
phase('R2b-Triage-and-Authoring')
const r2b = await agent(
`${LAW}

Repo ${ROOT}. WORKSTREAM R2b — SOURCE-LAW TRIAGE + AUTHORING (decision: auto-triage+continue).
1. TRIAGE the ~3573-line delta that exists only in HEAD ontology/all_merged.ttl (diff \`git show HEAD:ontology/all_merged.ttl\` vs a fresh \`python3 scripts/merge_ontology.py\`). For each block of content present only in HEAD: classify as REAL-LAW (lift it verbatim into a NEW appropriately-numbered ontology/source_law/NNN_*.ttl with a real namespace) vs STALE (drop, recording what was dropped in the report). After lifting, re-run merge_ontology.py and confirm the fresh merge now contains the lifted law (banner present) and re-run \`python3 scripts/verify_source_law_replay.py\` — contamination diff should now PASS (fresh == regenerable). Do NOT re-commit the hand-merged blob.
2. IDEMPOTENCY FIX: scripts/sync_source_law.py currently REWRITES 052 and 054-065 back into 1-line stubs every run from .agents/SPR_SOURCE_LAW_MANIFEST.md. Make it idempotent — do NOT clobber files that already contain real RDF (e.g. only stub a file if it is missing/empty; or remove 052/054-065 from the clobber set in the manifest). Prove: author a marker triple in one of those TTLs, run sync_source_law.py, confirm the marker survives.
3. AUTHOR the IP source laws as REAL RDF (BLAKE3): ontology/source_law/097_external_expressive_distance_policy.ttl (ProtectedCluster per franchise w/ tradeDressThreshold + signatures, MechaCommons, OriginalAxis, CandidateAdmissionShape, BaselineUsage usableForMetricBaseline=true/usableForGeneration=false), 098_non_confusion_policy.ttl (verdict individuals incl REFUSE_EXPRESSIVE_PROXIMITY, prohibitedProvenance, non-confusion SHACL shape), 099_ip_distance_report_schema.ttl (SHACL shapes for IP_DISTANCE_REPORT.json/NON_CONFUSION_REPORT.json/ADMISSION_RECEIPT.jsonl requiring BLAKE3 hash + baseline_usage). Mirror ip_policy_packs/mecha_external_corpus.policy.json. Use the rich http://rocket-craft.com/ontology/ip-distance# namespace style (like 103_proportion_priors.ttl), not the legacy example.org stub style.
4. Prove the merge still regenerates cleanly TWICE (two \`merge_ontology.py\` + \`ggen sync\` runs, exit 0, no GGEN-* andon). Emit SOURCE_LAW_AUTHORING_REPORT.json at repo root.
standing=ADMITTED only if the fresh merge is regenerable (R2 replay now passes) AND the authored laws survive two syncs; else PARTIAL_ALIVE with next_action.`,
  { schema: STANDING, phase: 'R2b-Triage-and-Authoring', label: 'R2b-triage-authoring' })

// ---------------------------------------------------------------- R1 modular identity
phase('R1-Modular-Identity')
const r1 = await agent(
`${LAW}

Repo ${ROOT}. WORKSTREAM R1 — MODULAR IDENTITY (already passing; prove it FRESH, do not rebuild).
Known: 13 unique SM_*.usda, negative fixtures (seeds 201-205) refuse with USD301-USD312, smoke seeds 101-103 PASS_FLAGSHIP, release_decision=DOE_RELEASED. Static evidence lives in .agents/sub_orch_implementation_aaa_ue4_mech_pack_001/{MODULAR_IDENTITY_SMOKE_REPORT,NEGATIVE_FIXTURE_RESULTS,GEOMETRY_FINGERPRINTS}.json.
1. FIRST locate the runnable regenerator that produced those fixtures (search scripts/ and the sub_orch dir for run_mecha_doe.py or the geometry generator + USD30x validator). If none is runnable, that is the gap — report it and emit from the existing static reports, marking replay_verified=false.
2. If runnable: re-run negative fixtures (assert each REFUSE_MODULAR_USD with USD301-USD312 firing) + smoke seeds (assert PASS_FLAGSHIP, 13 unique fingerprints cross-checked vs GEOMETRY_FINGERPRINTS.json, owner_part_id complete, no socket mesh payload, no torso foreign geometry).
3. Emit MODULAR_IDENTITY_REPORT.json + .md at repo root (drop-in superset of the existing smoke report).
standing=ADMITTED only if fresh re-run reproduces DOE_RELEASED with all refusals + unique fingerprints; PARTIAL_ALIVE if only static evidence (no runnable regenerator) — with next_action naming the generator to wire.`,
  { schema: STANDING, phase: 'R1-Modular-Identity', label: 'R1-modular-identity' })

// ---------------------------------------------------------------- R3 POWL admission
phase('R3-POWL-Admission')
const r3 = await agent(
`${LAW}

Repo ${ROOT}. WORKSTREAM R3 — VISION POWL LOOP ADMISSION (wasm4pm process evidence, not syntax).
Two REAL code defects to FIX FORWARD (explicitly allowed this run):
  (a) crates/rocket_preue4_verifier/src/bin/rocket_preue4_verify.rs hardcodes an all-Admitted ReceiptChain and NEVER parses --trace. Repair it to read the actual XES/OCEL trace from --trace, derive per-activity AdmissionStatus (a missing/out-of-order/skipped POWL activity => NOT Admitted), so scoped_status() can legitimately yield PARTIAL_ALIVE_CANDIDATE. (report.rs/verifier.rs from_pipeline PARTIAL logic already exists — feed it real per-step status.)
  (b) scripts/vision_powl_executor.py calls modelless \`${WPM} audit\` which ALWAYS returns DECEPTIVE/fitness 0. Repair it to: derive a DECLARED reference log from ontology/source_law/VisionSnapLoop.powl (lawful happy-path StartVisionLoop->GenerateGeometry->RenderProjection->ExtractTargets->MeasureGap->RepairLoop(SelectOperator->PatchLaw->Regenerate)->VerifyEngine->EmitReceipt), \`${WPM} mining discover <declared.xes> --algo inductive\` -> declared model, then \`${WPM} mining conformance vision_trace.xes <declared_model>\` for the real fitness/precision verdict.
Then:
1. Run \`python3 scripts/vision_powl_executor.py\` to execute the loop and emit vision_trace.xes (+ OCEL seed). Record per-activity executed/order_ok.
2. Run the conformance path (b). Parse fitness/precision/deviations.
3. cargo run --bin rocket-preue4-verify -- --milestone GC-VISION-SNAP-001 --powl ontology/source_law/VisionSnapLoop.powl --trace vision_trace.xes --report VISION_VERIFIER_REPORT.json (after repair (a)).
4. Emit VISION_POWL_LOOP_ADMISSION_REPORT.json + .md (powl hash, trace hash, wpm conformance verdict, powl_step_status[], verifier final_status/scoped_status, BLAKE3 receipt). Mapping: ALL activities present+lawful order AND fitness==1.0 AND no DECEPTIVE/VARIANCE/PARTIAL AND verifier ALIVE_UNDER_SCOPE => ADMITTED; any skip/extra/out-of-order or fitness<1.0 or PARTIAL verifier => PARTIAL_ALIVE (NEVER PASS); no trace at all => REFUSED.
5. Delete vision_trace.xes + generated/vision_snap/* and re-run to confirm the admission verdict + receipt reproduce (replay_verified).
standing per the mapping; report_emitted=VISION_POWL_LOOP_ADMISSION_REPORT.json.`,
  { schema: STANDING, phase: 'R3-POWL-Admission', label: 'R3-powl-admission' })

// ---------------------------------------------------------------- IP distance
phase('IP-Distance')
const ip = await agent(
`${LAW}

Repo ${ROOT}. WORKSTREAM IP-DISTANCE (GlobalCorp/Studio + Mecha/BiotechMech/ArmorFrame non-confusion).
Depends on R2b having authored 097/098/099.
1. Upgrade scripts/ip_distance_engine.py to emit BLAKE3 (it currently uses blake2b) and to include a per-candidate baseline_usage block (usable_for_metric_baseline=true, usable_for_generation=false) required by the 099 schema.
2. Run \`python3 scripts/merge_ontology.py\` then \`${V}\` (fresh candidate generated/**/*.ttl), then \`python3 scripts/ip_distance_engine.py\`.
3. Validate the three emitted artifacts (generated/ip_distance_engine/IP_DISTANCE_REPORT.json, NON_CONFUSION_REPORT.json, ADMISSION_RECEIPT.jsonl) against the 099 SHACL shapes. Flag any candidate with proximity_score >= cluster.trade_dress_threshold (or >= 0.7 admission rule) as REFUSE_EXPRESSIVE_PROXIMITY.
4. Emit IP_DISTANCE_EVIDENCE.json at repo root summarizing verdict.
standing=ADMITTED only if NON_CONFUSION_REPORT.json exists with refused==0 for the hero candidate AND baseline_usage flags present AND BLAKE3 receipts; REFUSED if any expressive-proximity flag fires for the hero; PARTIAL_ALIVE if 097-099 not yet real or engine fields missing (next_action names the fix).`,
  { schema: STANDING, phase: 'IP-Distance', label: 'IP-distance' })

// ---------------------------------------------------------------- R4 fresh render
phase('R4-Fresh-Render')
const r4 = await agent(
`${LAW}

Repo ${ROOT}. WORKSTREAM R4 — FRESH RENDER VISUAL MEASUREMENT (resolve the stale 9-vs-35 split).
Known: gap_closure_report.json (foreground_component_count=35, blade_delta=195 SENTINEL, feather_curvature=0.0256) is STALE; the fresh visual_gap_report.json (thresholds_met=true, fc_count=9, blade_delta~11.6) is real. blade_delta=195 is the degenerate sentinel fit_blade returns when cyan pixels<10.
1. Author scripts/fresh_render_verify.sh wrapping \`${V}\`: pre-hash existing ${ASSET}/renders/*.png + report mtime (stale baseline); run \`${V}\` (run1) -> BLAKE3 each of the 4 fresh PNGs + report + utc timestamp + metrics; run \`${V}\` again (run2) -> hashes+metrics; assert run1==run2 to 4dp (REPLAY); STALE-REFUSAL FIXTURE: if a report's render_hash != the just-written run1 hashes, exit STALE_RENDER_REFUSED.
2. Add the single ALLOWED scorer change: stamp render_hash into compare_reference_render.py's emit dict so visual_gap_report.json self-certifies its render provenance (provenance only; never alter a metric).
3. Mark gap_closure_report.json superseded (write gap_closure_superseded:true into the report) citing the 195-sentinel + fc=35-stale mechanism.
4. Emit FRESH_RENDER_VERIFICATION_REPORT.json + .md (render_blake3 map, report_blake3, timestamp, run1/run2 metrics, identical_4dp, scorer_git_unchanged, stale_refusal_fixture, gap_closure_superseded, resolved_truth, verdict in {FRESH_VERIFIED,REPLAY_FAIL,STALE_RENDER_REFUSED}).
standing=ADMITTED iff verdict==FRESH_VERIFIED (renders provably deleted-then-regenerated, replays); else PARTIAL_ALIVE/REFUSED with next_action.`,
  { schema: STANDING, phase: 'R4-Fresh-Render', label: 'R4-fresh-render' })

// ---------------------------------------------------------------- Materials / textures
phase('Materials-Textures')
const materials = await agent(
`${LAW}

Repo ${ROOT}. WORKSTREAM MATERIALS/TEXTURES (real OpenPBR from source law; ${ASSET} canonical).
Root cause of the 123-byte stub .mtlx: ${ASSET}/templates/materialx/materials.mtlx.tera is a 6-line stub that ignores every SPARQL row, and queries/materials.rq projects too few channels.
1. DATA: add per-material channels to ontology/source_law/104_reference_fabric.ttl (mud:materialClass + coat/anisotropy/emission_color + mud:*Tex texture filename literals matching texture_manifest.json output_maps) and one mud:TextureProgram per material. (104 is NOT in the SPR clobber manifest, so it survives — confirm R2b made sync_source_law idempotent first.)
2. SPARQL: extend ${ASSET}/queries/materials.rq (and the paired ggen.toml material rules) with OPTIONAL projections for every channel; use the inline filtered-query pattern already used by SM_Torso/SM_Head (FILTER on material local name).
3. TEMPLATE: rewrite materials.mtlx.tera to branch on materialClass and emit real <open_pbr_surface> networks — clearcoat armor, anodized frame, emissive blade/visor — plus the Agent07_Mecha_Materials.mtlx NG_AdvancedArmorCompositing wear+damage mix graph. Material names MUST equal the USD binding targets (M_WhiteArmor/M_DarkFrame/M_CyanBlade/M_GoldVisor). Author texture programs for panel-line/wear/damage/normal/roughness/metallic/emissive.
4. Run \`${V}\`; VERIFY via the ggen-asset-lsp binding check (crates/ggen-asset-lsp): zero missing/invalid-material-binding across all mesh prims, zero stub .mtlx (each must contain <open_pbr_surface> + a named <surfacematerial> and exceed stub size). Confirm color_palette_similarity does NOT regress (it should hold/improve as parts get true colors).
5. Emit MATERIALS_TEXTURE_EVIDENCE.json at repo root + BLAKE3 receipt lines into ${ASSET}/receipts/asset_receipts.jsonl (material_binding_count, openpbr_node_validity, texture_manifest_completeness, stubs=0; each with output_hash/run_id/replay_pointer).
standing=ADMITTED only if a FRESH render shows zero binding errors, zero stubs, and no color regression that replays; else PARTIAL_ALIVE with next_action.`,
  { schema: STANDING, phase: 'Materials-Textures', label: 'materials-textures' })

// ---------------------------------------------------------------- R5 residual repair
phase('R5-Residual-Vector')
const r5 = await agent(
`${LAW}

Repo ${ROOT}. WORKSTREAM R5 — RESIDUAL-VECTOR REPAIR (bounded operators patch SOURCE LAW only; no blind tournament).
1. Extend ontology/source_law/103_proportion_priors.ttl so each prior:RepairOperator carries machine-checkable bounds: prior:boundsStepMax, prior:preservesMetric (the floor, e.g. metrics:silhouetteIoU / metrics:colorPaletteSimilarity), prior:patchesTriple (the exact source-law subject+predicate it may mutate, e.g. mud:prim_torso_core mud:scaleX), prior:requiresOwnerProvenance true. Confirm bands survive two ggen syncs.
2. Read the R4 fresh truth (FRESH_RENDER_VERIFICATION_REPORT.json) for the residual vector. Registry / order:
   - blade color-provision FIRST: provision_blade_color_from_source_law — make displayColor in ${ASSET}/templates/usd/part_mesh.usda.tera DERIVE (r,g,b) from mud:color via the paired SPARQL SELECT (closes the latent GGEN-TPL-001 static-literal desync). Without cyan, blade_length_angle_delta is the 195 sentinel and unmeasurable.
   - then set_blade_mount_angle_within_band -> set_blade_length_ratio_within_band (mud:rotate*/scale on blade prims in 104), band <=15 pref <=8 forbidden >60.
   - tighten_core_mass_inside_envelope (mud:scale on mud:prim_torso_core), target compactness 0.42.
   - compose_adjacent_parts_within_overlap_band / merge_feather_components_preserving_wingspan for foreground_component_count toward [1,5], preserving owner_part_id + wing_span_delta.
3. ACCEPT/REVERT loop per operator: residual_before (from fresh report) -> apply ONE bounded operator within boundsStepMax to its patchesTriple source-law target -> re-run fresh_render_verify.sh (R4 wrapper) -> ACCEPT iff residual_after<residual_before AND identical_4dp AND no preservesMetric floor regressed; else REVERT that specific TTL/tera patch. Record every step honestly.
4. Emit RESIDUAL_VECTOR_REPORT.json (per-gate target_band/current/residual/in_band), REPAIR_OPERATOR_SELECTION_REPORT.json (steps[] residual_before/after/replayed/accepted), SOURCE_LAW_PATCH_REPORT.json (exact diffs, touched_generated_artifacts:false, touched_scorer:false) at repo root.
standing=ADMITTED for the set of gates whose accepted gains replayed; PARTIAL_ALIVE if any target gate still outside band — next_action names the gate+operator.`,
  { schema: STANDING, phase: 'R5-Residual-Vector', label: 'R5-residual-vector' })

// ---------------------------------------------------------------- R6 KEYSTONE
phase('R6-Delete-Resync-Replay')
const r6 = await agent(
`${LAW}

Repo ${ROOT}. WORKSTREAM R6 — DELETE-AND-RESYNC REPLAY (KEYSTONE GATE — the crown success signal).
1. Gate on R2: read SOURCE_LAW_REPLAY_REPORT.json; if the source law is not regenerable-clean after R2b, set standing=REFUSED with next_action=R2b (a generated-replay proof is meaningless on contaminated source law).
2. Author scripts/verify_r6_delete_resync_replay.py wrapping the PROVEN scripts/verify_delete_and_resync_replay.py (do NOT reimplement its GPU carve-out: byte-compare deterministic .usda/.mtlx/textures, disposition-compare GPU PNGs to 4dp). It runs two full delete->merge->ggen sync->fresh render->compare rebuilds and compares rebuild1 vs rebuild2.
3. Build the unified BLAKE3_RECEIPT_CHAIN.json linking source(each source_law/*.ttl + all_merged.ttl) -> USD/material/texture byte hashes -> render hashes (class:gpu, comparison:disposition — never chain-breaking) -> metric disposition hash (sorted JSON, metrics@4dp) -> report hash, each prev_hash-linked (genesis = 64 zeros), receipt = blake3(hash||prev_hash). Mirror the existing ${ASSET}/receipts/asset_receipts.jsonl format.
4. Emit the keystone DELETE_RESYNC_REPLAY_REPORT.json + .md (both rebuild summaries, R2 gate result, chain head/tail receipts, generator_artifacts_byte_identical, disposition_replays, verdict). Flag any ggen nondeterminism (SPARQL SELECT feeding a template without total ORDER BY) if rebuild1!=rebuild2.
standing=ADMITTED iff generator_artifacts_byte_identical AND disposition_replays AND R2 regenerable AND chain linkage valid end-to-end; else REFUSED/PARTIAL_ALIVE with the exact divergent artifact as next_action.`,
  { schema: STANDING, phase: 'R6-Delete-Resync-Replay', label: 'R6-delete-resync-keystone' })

// ---------------------------------------------------------------- Synthesis (independent re-stat)
phase('Synthesis-Admission')
const upstream = { r2a, r2b, r1, r3, ip, r4, materials, r5, r6 }
const synthesis = await agent(
`${LAW}

Repo ${ROOT}. WORKSTREAM SYNTHESIS — FINAL ADMISSION (anti-false-PASS backstop).
Upstream verdicts (DO NOT trust their standing strings — re-verify):
${JSON.stringify(upstream, null, 2)}

1. For EVERY upstream agent: independently \`ls\`/stat each cited evidence_paths[] and report_emitted. A missing file downgrades that workstream to UNKNOWN regardless of what it claimed. A standing of ADMITTED whose report_emitted does not exist on disk is a FALSE CLAIM — strike it.
2. Verify the BLAKE3 receipt chain (BLAKE3_RECEIPT_CHAIN.json) prev_hash linkage end-to-end.
3. Confirm the keystone DELETE_RESYNC_REPLAY_REPORT.json exists with verdict ADMITTED.
4. Confirm IP NON_CONFUSION_REPORT.json has no expressive-proximity refusal for the hero.
5. Set claim=PRE_UE4_HERO_ASSET_ADMITTED in NEXT_GATE_STATUS.md ONLY IF: keystone exists & ADMITTED, AND every upstream workstream re-verifies as ADMITTED with existing evidence, AND IP non-confusion clean, AND chain verifies. OTHERWISE claim=HOLD with the SINGLE highest-value next_gap (the gate blocking admission, with its exact next_action).
6. Write NEXT_GATE_STATUS.md (per-workstream re-verified standing table + claim + next_gap).
standing reflects the OVERALL admission (ADMITTED only if claim==PRE_UE4_HERO_ASSET_ADMITTED); report_emitted=NEXT_GATE_STATUS.md. HOLD on the first run is an honest, expected outcome — do not inflate it.`,
  { schema: STANDING, phase: 'Synthesis-Admission', label: 'synthesis-admission' })

return {
  R2a_source_law_replay: r2a,
  R2b_triage_authoring: r2b,
  R1_modular_identity: r1,
  R3_powl_admission: r3,
  IP_distance: ip,
  R4_fresh_render: r4,
  Materials_textures: materials,
  R5_residual_vector: r5,
  R6_delete_resync_replay_KEYSTONE: r6,
  Synthesis: synthesis,
  claim: synthesis && synthesis.standing === 'ADMITTED' ? 'PRE_UE4_HERO_ASSET_ADMITTED' : 'HOLD',
}
