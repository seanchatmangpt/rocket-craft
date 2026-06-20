# NON-HUMAN MECH GENERATION SPR

Core doctrine: no human creative surface owns standing.
The project is not artist-assisted procedural generation. It is total deterministic game manufacturing.
Every required mech must be generated from bounded law, not sculpted, painted, rigged, animated, imported, or approved by a human.

Human role: define laws, constraints, standards, acceptance gates, refusal rules, and verifier thresholds. Human does not author asset instances.

Forbidden control surfaces: Blender manual edit, Maya manual edit, Substance Painter UI, Unreal Editor clicking, manual sculpt, manual texture paint, manual rig, manual animation, hand-authored gameplay tables, artist approval, viewport inspection as authority, payload placeholders waiting for art team.

Correct pipeline: ideation → design-space enumeration → candidate selection → geometry generation → materials → textures → rig/skeleton/sockets → animation/motion → gameplay byte law → UE import/cook → OCEL/OTEL telemetry → replay → falsification → counterfactuals → receipts.

ggen is the deterministic manufacturing authority from ideation to gameplay.
ggen is not an ontology toy. ggen is not a document generator. ggen is the factory that manufactures the admissible game universe from bounded law.

Previous wrong path: fake private TTL namespaces, descriptive mech phenotype, random “seraphic” traits, private mesh vocabulary, USD assembly with missing payloads, block proxy, text-only proof, artist/DCC dependency.

Corrected law: no private geometry ontology until public standards are exhausted. Do not invent mesh ontology when USD already carries 3D scene truth.

USD/OpenUSD is the primary 3D scene/model/material/animation output authority. USDA is the deterministic text scene artifact. USD handles scene graph, mesh prims, transforms, variants, references, payloads, materials, skeletons, animation, physics, composition.

MaterialX/OpenPBR is the lookdev/material authority. Texture programs and texture manifests are generated headlessly.

RDF/TTL/SPARQL does not replace USD. RDF/SPARQL selects manufacturing rows, provenance, constraints, parameter domains, generator selection, and admission policies. Tera emits real USDA, MaterialX, texture programs/manifests, UE tables, Rust verifiers, telemetry dictionaries, reports.

No descriptive TTL. No “cool wing” ontology. No fake `rc:MechDesign` authority. Graph rows must lower into geometry files.

Public standards stack: USD/OpenUSD, MaterialX, OpenPBR, OCEL, OTEL, PROV-O, DCAT, DCTERMS, SHACL, SKOS, possibly X3D Ontology only for RDF-native 3D proof slices. ifcOWL is real but BIM-oriented, not primary for mechs.

Core manufacturing equation: DesignSpace × Constraints × FitnessFunction × Seed → CandidateSet → AdmittedVariant → USD/MaterialX/UE/gameplay artifacts.

Ideation without artists: deterministic search over bounded design space. ggen enumerates possible mech candidates. Constraints/refusal gates filter invalid candidates. Fitness functions rank candidates. Deterministic tie-break selects admitted candidate.

Candidate dimensions: body topology, silhouette class, role class, symmetry group, weapon topology, mobility class, material family, texture program, socket topology, motion family, gameplay byte class, LOD policy, support-role affordance.

Combinatorial Maximalism: enumerate bounded possibility, manufacture every admissible variant, refuse invalid artifacts, receipt admitted outputs. Not “generate something visually interesting.”

Reference image is observation, not asset source. Image can be measured, not copied by hand. It supplies visual target metrics: silhouette mask, edge map, dominant colors, white/dark/cyan/yellow/red ratios, aspect ratio, wing-span ratio, core-body ratio, symmetry score, cyan weapon regions, visor highlight region.

Deterministic visual convergence loop: reference image → measured visual targets → generated part grammar → USD geometry emission → headless render → computer-vision comparison → gap report → patch generator parameters → repeat until acceptance threshold.

Do not claim asset generation from file existence. Claim asset generation only after headless render similarity passes.

Proof ladder: USDA parses = syntax proof. USD renders = projection proof. Render matches target = visual-convergence proof. UE import/cook works = engine-consumption proof. OCEL/receipts/replay = manufacturing-standing proof.

Every generated mech must have visible proof: headless render, visual metrics, similarity report, gap report. Text-only USDA is insufficient.

First serious milestone: GC-MECH-ASSET-FABRIC-001: REFERENCE_IMAGE_TO_GENERATED_USD_VISUAL_CONVERGENCE.

Required output root: generated/mech_assets/reference_fabric_001/.
Required subdirs: reference, graph, queries, templates, usd, materialx, textures, renders, reports, ocel, receipts.
Required reference artifacts: reference_original.jpg, reference_silhouette.png, reference_edges.png, reference_color_histogram.json, reference_measurements.json.
Required graph artifacts: asset_fabric.ttl, visual_targets.ttl, generator_parameters.ttl.
Required queries: candidate_parts.rq, usd_prims.rq, materials.rq, texture_programs.rq, verifier_expectations.rq.
Required templates: asset.usda.tera, part_mesh.usda.tera, materials.mtlx.tera, texture_program.rs.tera, visual_gap_report.md.tera.
Required USD outputs: ASSET_ReferenceFabric_001.usda, SM_Torso.usda, SM_Head.usda, SM_WingArray_Left.usda, SM_WingArray_Right.usda, SM_Blade_Left.usda, SM_Blade_Right.usda.
Required MaterialX outputs: M_WhiteArmor.mtlx, M_CyanBlade.mtlx, M_DarkFrame.mtlx, M_GoldVisor.mtlx.
Required texture outputs: T_WhiteArmor_BaseColor.png, T_WhiteArmor_Roughness.png, T_WhiteArmor_Normal.png, T_CyanBlade_Emissive.png, texture_manifest.json.
Required render outputs: render_front.png, render_angled.png, render_silhouette.png, render_edges.png.
Required reports: visual_gap_report.json, visual_gap_report.md, verifier_report.json, verifier_report.md, gap_closure_report.json, gap_closure_report.md.
Required evidence: asset_manufacturing.ocel.json, asset_receipts.jsonl.

Minimum generated part families: torso_core, head_unit, v_fin_left, v_fin_right, shoulder_left, shoulder_right, arm_left, arm_right, leg_left, leg_right, wing_root_left, wing_root_right, primary_wing_feathers_left, primary_wing_feathers_right, secondary_wing_feathers_left, secondary_wing_feathers_right, blade_left, blade_right, backpack_core, thruster_cluster.
Minimum geometry primitive families: tapered_box, beveled_panel, triangular_fin, feather_panel, wing_binder, cylinder_joint, sphere_joint, blade_prism, armor_shell, greeble_panel.
Minimum generated USD prim count: >= 120.
Minimum wing-feather panels: >= 48.
Minimum material bindings: >= 4.
Required generated colors/materials: white armor, dark mechanical frame, cyan emissive blade/energy, yellow/gold visor or trim, red micro accent.

Textures without artists: generated base color, normal, roughness, metallic, emissive, opacity, wear mask, panel-line mask, decal placement, damage mask, heat discoloration. Texture source is generated program + manifest + seed + parameter rows. PNGs are projections.
Geometry without artists: generated point sets, curves, extrusions, sweeps, lathes, panels, sockets, joints, wing arrays, armor shells, greebles. Every generator has id, seed, input parameters, expected point/face counts, material binding, socket binding.
Rig without artists: generated skeleton hierarchy, sockets, joint limits, IK targets, attachment points, clearance constraints, UsdSkel-ready structures.
Animation without artists: generated motion families, animation curves, state transitions, IK constraints, foot clearance, weapon clearance, wing clearance, center-of-mass bounds, socket constraints.
Gameplay without designers: generated byte transition tables. part class → authority bytes. socket damage → capability degradation. heat/stress/grip/socket_health → failure/firing/repair law. support role → repair/shield/supply actions. motion state → allowed action table.
Balance without designers: generated domains + simulation tests + fitness scoring + refusal rules. Values admitted by deterministic simulation, not manual tuning.
UE4 without manual import: generated UE import/cook tables, command-line import proof, cooked artifact verification. Unreal owns pixels/projection only. Rust/ggen/wasm4pm own meaning and admission.

No human interaction surface means no tool UI owns standing. USD may describe the asset. MaterialX may describe look. Unreal may consume the result. ggen + validators + Rust receipts decide admission.

OCEL manufacturing log: object-centric evidence of generated candidate, geometry primitives, USD prims, materials, texture programs, renders, validators, receipts, UE import/cook.
OCEL8/OTEL8 telemetry: UE4 emits bytes. Rust restores meaning. Game telemetry is generated byte-code dictionaries, not string logging. Event/object/field/status codes are generated u8/u16 dictionaries. Canonical strings appear only at export boundary.
wasm4pm role: process evidence authority. Verifies manufacturing sequence, OCEL object-event links, replay, conformance, falsification, counterfactuals, receipts.
Receipts: tamper-evident sequential receipt chain. Hash every emitted artifact. Prove graph rows == emitted USD facts == parsed USD facts == render metrics. Do not say unforgeable.

Falsification required: missing wing array refuses, zero-point mesh refuses, missing material binding refuses, render not created refuses, low prim count refuses, low feather count refuses, missing texture manifest refuses, missing reference measurements refuses, invalid face index refuses, nondeterministic texture seed refuses, broken socket refuses.
Counterfactuals required: double wing feathers, half wing feathers, remove cyan blades, increase white armor ratio, decrease core body width, increase wing span, remove gold visor, add red micro decals. Each reports metric deltas.
Visual metrics required: silhouette_iou, edge_similarity, color_palette_similarity, cyan_region_similarity, symmetry_delta, wing_span_delta, body_mass_delta, usd_prim_count, material_binding_count, wing_feather_count, status, residuals.

Initial visual acceptance threshold: USD parses; render exists; no missing payloads; usd_prim_count >= 120; wing_feather_count >= 48; material_binding_count >= 4; silhouette_iou >= 0.25; color_palette_similarity >= 0.50; falsification >= 8 PASS; counterfactual >= 8 PASS; OCEL exists; receipts exist.
Headless render allowed tools: usdrecord, Blender background mode, Unreal command-line render, Python offscreen renderer, software fallback renderer if it consumes generated USD/geometry. Manual screenshot is not authority.

Agent failure mode observed: produced pipeline-looking files, claimed pipeline readiness, generated missing payload references, showed text instead of visual proof, then claimed LLM cannot generate geometry. This is refused. Correct response is not excuse; correct response is deterministic visual-convergence manufacturing loop.

Pipeline TD framing: serious asset people care about USD, MaterialX/OpenPBR, render consistency, asset handoff, validation, automated import, provenance, lookdev consistency. But in this doctrine, even the artist is a control surface; so the system must generate all asset instances itself.
Eric/Pipeline TD lesson: do not pitch ontology. Pitch deterministic USD/MaterialX manufacturing with verifiable studio handoff. But final doctrine removes artist control surface: ggen must generate all assets, not traffic-control human-made assets.

Required agent law: No “artist will fill it in.” No “drop payloads into meshes folder.” No “open Blender manually.” No “open Unreal manually.” No “LLM cannot generate geometry.” No “pipeline-ready without render.” No “look in viewport.” Generated render is the first visual proof.

Gap checker required: scripts/asset_fabric_gap_check.py. Computes status from files and metrics. Writes reports/gap_closure_report.json/md.
Gap IDs: REFERENCE_MEASUREMENTS_EXIST, GGEN_SYNC_PASSES, USD_ASSEMBLY_EXISTS, USD_MESH_FILES_EXIST, USD_PARSES, USD_PRIM_COUNT_GE_120, WING_FEATHER_COUNT_GE_48, MATERIAL_BINDINGS_GE_4, MATERIALX_FILES_GE_4, TEXTURE_MANIFEST_EXISTS, RENDER_FRONT_EXISTS, RENDER_ANGLED_EXISTS, SILHOUETTE_IOU_GE_025, COLOR_PALETTE_SIMILARITY_GE_050, FALSIFICATION_CASES_GE_8_PASS, COUNTERFACTUAL_CASES_GE_8_PASS, OCEL_EXISTS, RECEIPTS_EXIST, REPORTS_UPDATED.

Executable conscience law: status must be computed by gap checker/verifier, not model prose. next_gap → patch → verify → repeat. Audit findings are not progress until converted into failing checks, repaired, and re-verified.

Active future sequence: GC-MECH-ASSET-FABRIC-001A armor panel; 001B wing feather array; 001C limb segment; 001D socket hierarchy; 001E full static mech; 001F rigged mech; 001G animated mech; 001H gameplay-ready mech.

Final doctrine: from ideation to gameplay, every artifact is either generated, verified, and receipted — or refused.
