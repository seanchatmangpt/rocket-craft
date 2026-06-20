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
| Silhouette IoU | >= 0.25 | 0.3358 | **PASS** |
| Color Palette Similarity | >= 0.50 | 0.9242 | **PASS** |
| Part Graph Similarity | >= 0.90 | 1.0000 | **PASS** |
| Wing Layer Count Delta | <= 1.0 | 2.0000 | **FAIL** |
| Feather Panel Curvature Score | >= 0.10 | 0.0000 | **FAIL** |
| Feather Overlap Depth Score | >= 0.10 | 0.0000 | **FAIL** |
| Core Compactness Delta | <= 0.15 | 0.3545 | **FAIL** |
| Head to Torso Ratio Delta | <= 0.15 | 0.0167 | **PASS** |
| Blade Length/Angle Delta | <= 15.0 | 195.0000 | **FAIL** |
| Armor Shell Segmentation Score | >= 0.04 | 0.2340 | **PASS** |
| Edge Density Distribution Similarity | >= 0.60 | 0.0000 | **FAIL** |
| Foreground Component Count | [1, 5] | 143 | **FAIL** |
| Edge Similarity | N/A | 0.0932 | **INFO** |
| Cyan Region Similarity | N/A | 0.0000 | **INFO** |
| Symmetry Delta | N/A | 0.0156 | **INFO** |
| Wing Span Delta | N/A | 237.5000 px | **INFO** |
| Body Mass Delta | N/A | 0.2414 | **INFO** |

### Diagnostics & Errors

- **Modularity Errors**: USD305 ERROR: mirrored part lacks mirror transform proof
- **Visual Morphology Errors**: VIS202 ERROR: wing morphology mismatch, VIS203 ERROR: generated wing panels are line-primitives, expected layered swept plates, VIS204 ERROR: core body massing exceeds compactness bound, VIS205 ERROR: blade placement/angle mismatch, VIS207 ERROR: edge-density distribution mismatch, VIS208 ERROR: candidate passed coarse silhouette but failed morphology gate

---

## Receipt Chain

The final verifier JSON `verifier_report.json` contains 12 verified sync receipts.
Latest receipt registered: `210f0d4ba76ec9b83e7cfd418b70b2c1e97a033a14490bf7f4f41fecda4ea738`

---

## Residuals

No residuals.

---

## Final Status

**Overall Verdict: REFUSED (VERIFIED)**
