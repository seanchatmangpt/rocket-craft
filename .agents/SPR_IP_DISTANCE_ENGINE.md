# SPR: MECHA IP DISTANCE ENGINE AND AFC ADAPTATION

## Core Formulation
By semantically modeling the mecha genre and the protected signature clusters of major mech franchises, we can mathematically identify an admissible design region: inside the generic mecha commons, outside protected expressive/trademark/trade-dress clusters, and positively anchored in our own original design grammar.

Admit(x) = (∀i, d(x, 𝒫_i) > τ_i) ∧ (x ∈ 𝒢_mecha) ∧ (x ∈ 𝒩_new)

## The Idea/Expression Fence
Do not delete the whole genre. Delete or avoid the protected expressive clusters.
- **Genre Commons (𝒢_mecha)**: Giant robot, piloted war machine, armor panels, cockpit, mechanical joints, weapon sockets, repair class. (Admissible)
- **Protected Clusters (𝒫_i)**: Specific franchise V-fins, heroic proportions linked to specific IPs, iconic faction color blocking, franchise-specific naming conventions, explicit logo/faction marks. (Danger Zone - Must Subtract)
- **Original Axes (𝒩_new)**: Support roles as first-class heroes, process-intelligence visual grammar, receipt-bearing parts, logistics-as-combat, byte-law gameplay authority, visible asset provenance.

## Operational Model: The Mecha IP Feature Graph
1. Extract generic mech grammar (limb articulation, cockpit affordance).
2. Filter protected signatures (specific heads, faction colors, specific weapon clusters).
3. Add original axes (support-first mechanics, receipt-bearing visual language).

Feature Admission Classes:
- `ADMIT_COMMON`: Genre/mechanical commons (cockpit, thruster).
- `ADMIT_FUNCTIONAL`: Dictated by function (knee joint).
- `ADMIT_TRANSFORMED`: Usable only after transformation.
- `CAUTION_CLUSTER`: Risky in combination (shield + rifle + saber + crest).
- `REFUSE_SIGNATURE`: Protected/commercial signature (official names, logos).
- `REFUSE_SOURCE`: Official source asset/prompt/reference.

## The IP Distance Engine
The pipeline must output an IP distance and non-confusion report for every generated mech.
Outputs:
- `IP_FEATURE_GRAPH.ttl`
- `MECHA_COMMONS_GRAPH.ttl`
- `PROTECTED_SIGNATURES_GRAPH.ttl`
- `ORIGINAL_DESIGN_AXES.ttl`
- `IP_DISTANCE_REPORT.json`
- `NON_CONFUSION_REPORT.json`
- `ADMISSION_RECEIPT.jsonl`

## Falsification Gates
Refuse candidate and patch generator law if:
1. Neutral viewer identifies it as "basically Gundam/Eva/MechWarrior."
2. Source-identifying terms, marks, or confusing names are present.
3. Visual classifier places candidate closer to a franchise cluster than original cluster.
4. Candidate is a collage of recognizable protected signatures.
5. Generated asset used official assets, screenshots, scans, names, or marks as source inputs.
6. Marketing/metadata creates affiliation confusion.

## LSP Diagnostic Abstraction Law
The LSP must emit generic admission diagnostics. Specific protected-cluster identities belong to the IP Distance Engine, policy packs, and verifier reports, not to the LSP diagnostic namespace.

### Correct Separation
- **asset-lsp-core**: Generic diagnostics only (e.g., `IP103 WARN: head-form proximity elevated`)
- **ip-distance-engine**: Computes proximity to protected clusters
- **ip-policy-pack**: Private/public policy configuration (`ip_policy_packs/mecha_external_corpus.policy.json`)
- **verifier-report**: Detailed scores and cluster labels

### Code Action Naming
LSP code actions must target the source law without naming the franchise:
- GOOD: `Regenerate head topology from support-role axis.`
- GOOD: `Shift silhouette away from external cluster centroid.`
- BAD: `Move away from Gundam V-fin.`
