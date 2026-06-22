# VERIFIER REPORT — GC-MECH-ASSET-FABRIC-001

---

## Milestone

**GC-MECH-ASSET-FABRIC-001**  
**Scoped Status: REFUSED**  
**Final Status: REFUSED**  

---

## Scope

This report covers the end-to-end verification of GC-MECH-ASSET-FABRIC-001:
- Procedural generation of white armor base color, roughness, normal, and cyan emissive blade textures.
- Headless rendering of front and angled views of the assembly USD model using `/usr/bin/usdrecord`.
- Image processing and generation of silhouette masks and edge maps.
- Similarity comparisons against reference targets (Silhouette IoU, Edge Cosine, Color Palette, and Bounding Box IoUs).
- Conformance validation to thresholds (`silhouette_iou >= 0.25` and `color_palette_similarity >= 0.50`).
- Generation of the Object-Centric Event Log (OCEL) and cryptographic receipts chain.
- Bounded morphology and modularity check constraints.

---

## Repository Boundaries

- `generated/mech_assets/reference_fabric_001/textures/` ← procedurally generated textures
- `generated/mech_assets/reference_fabric_001/renders/` ← USD renders and masks
- `generated/mech_assets/reference_fabric_001/reports/` ← gap and verifier reports
- `generated/mech_assets/reference_fabric_001/ocel/` ← Object-Centric Event Log
- `generated/mech_assets/reference_fabric_001/receipts/` ← cryptographic receipts chain

---

## Metric Verification Details

| Metric | Target | Actual | Verdict |
|---|---|---|---|
| Silhouette IoU | >= 0.25 | 0.5698 | **PASS** |
| Color Palette Similarity | >= 0.50 | 0.4854 | **FAIL** |
| Part Graph Similarity | >= 0.90 | 1.0000 | **PASS** |
| Wing Layer Count Delta | <= 1.0 | 0.0000 | **PASS** |
| Feather Panel Curvature Score | >= 0.10 | 0.1381 | **PASS** |
| Feather Overlap Depth Score | >= 0.10 | 1.0000 | **PASS** |
| Core Compactness Delta | <= 0.15 | 0.1235 | **PASS** |
| Head to Torso Ratio Delta | <= 0.15 | 0.0167 | **PASS** |
| Blade Length/Angle Delta | <= 15.0 | 155.7950 | **FAIL** |
| Armor Shell Segmentation Score | >= 0.04 | 0.1178 | **PASS** |
| Edge Density Distribution Similarity | >= 0.60 | 0.9383 | **PASS** |
| Foreground Component Count | [1, 5] | 3 | **PASS** |
| Edge Similarity | N/A | 0.0961 | **INFO** |
| Cyan Region Similarity | N/A | 0.0390 | **INFO** |
| Symmetry Delta | N/A | 0.2347 | **INFO** |
| Wing Span Delta | N/A | 492.0000 px | **INFO** |
| Body Mass Delta | N/A | 0.0848 | **INFO** |

### Diagnostics & Errors

- **Modularity Errors**: None
- **Visual Morphology Errors**: VIS205 ERROR: blade placement/angle mismatch, VIS208 ERROR: candidate passed coarse silhouette but failed morphology gate

---

## Receipt Chain

The final verifier JSON `verifier_report.json` contains 12 verified sync receipts.
Latest receipt registered: `c1b2d7debc96a9f479977c170fc9e2eb788e496662f0a0e5ec7ed728dbae81ac`

---

## Residuals

No residuals.

---

## Final Status

**Overall Verdict: REFUSED**
