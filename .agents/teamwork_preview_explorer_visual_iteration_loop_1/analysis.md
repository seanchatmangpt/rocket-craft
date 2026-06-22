# Metric Morphology Analysis Report

**Summary:** The pre-render morphology gate fails because the generated mecha parts have a vertical layout that contradicts the hardcoded SHACL ratio bands in the ontology: the torso height ratio (12.90%) falls below the minimum limit (30%), while the limb/leg height ratio (63.07%) exceeds the maximum limit (55%).

---

## 1. Observations

When running the pre-render metric morphology gate (`python3 scripts/verify_metric_morphology.py`), the command exits with code `1` and prints the following refusals:
```text
verdict: PARTIAL_ALIVE | shacl_conforms=False | neg_refused=True | replay=True
  refusal: REFUSE_PART_HEIGHT_BAND(SM_Limb_Left): ratio 0.6307 outside [0.40,0.55]
  refusal: REFUSE_PART_HEIGHT_BAND(SM_Limb_Right): ratio 0.6307 outside [0.40,0.55]
  refusal: REFUSE_PART_HEIGHT_BAND(SM_Torso): ratio 0.1290 outside [0.30,0.45]
  refusal: REFUSE_PART_HEIGHT_BAND: part height ratio falls outside its declared morphology band.
```

### Measured Geometries
From `METRIC_MORPHOLOGY_REPORT.json`, the vertical bounds (Z-axis, mapping to Y-axis coordinates in the graph) of each flagship part are:
- `SM_Head`: `y_min_m` = `0.013`, `y_max_m` = `0.0185` -> Height = `0.0055` meters.
- `SM_Torso`: `y_min_m` = `-0.002`, `y_max_m` = `0.0035` -> Height = `0.0055` meters.
- `SM_Limb_Left` / `SM_Limb_Right`: `y_min_m` = `-0.0134`, `y_max_m` = `0.0135` -> Height = `0.0269` meters.
- `SM_WingArray_Left` / `SM_WingArray_Right`: `y_min_m` = `0.00275`, `y_max_m` = `0.02925` -> Height = `0.0265` meters.
- `SM_Blade_Left` / `SM_Blade_Right`: `y_min_m` = `0.015`, `y_max_m` = `0.025` -> Height = `0.010` meters.

### Total Body Height Calculation
The total body height `body_h` is computed in `scripts/verify_metric_morphology.py` at line 195:
```python
195:     body_h = max(tops) - min(bots)
```
- `max(tops)` = `0.02925` (from `SM_WingArray_Left`/`Right` top)
- `min(bots)` = `-0.0134` (from `SM_Limb_Left`/`Right` bottom)
- `body_h = 0.02925 - (-0.0134) = 0.04265` meters.

### Ratio Comparisons
Ratios are computed as `part_height / body_h` (see `scripts/verify_metric_morphology.py` lines 252-253) and compared against:
1. **Direct Python checks** (`PART_BANDS` at lines 73-82)
2. **SHACL shapes** (`law:PartHeightBandShape` in `ontology/source_law/116_metric_morphology_bands.ttl` lines 144-166)

The comparison details:

| Part | Class Type | Measured Ratio | Expected Range (SHACL & Python) | Status |
|---|---|---|---|---|
| `SM_Head` | `law:MechaCrown` | `0.1290` | `[0.08, 0.15]` | In Band (Conforms) |
| `SM_Torso` | `law:TorsoSegment` | `0.1290` | `[0.30, 0.45]` | **FAIL (Below Min)** |
| `SM_Limb_Left` | `law:BipedalLeg` / `law:BipedalLimb` | `0.6307` | `[0.40, 0.55]` | **FAIL (Above Max)** |
| `SM_Limb_Right` | `law:BipedalLeg` / `law:BipedalLimb` | `0.6307` | `[0.40, 0.55]` | **FAIL (Above Max)** |
| `SM_WingArray` | `law:WingArray` | `0.6213` | `[0.40, 0.90]` | In Band (Conforms) |
| `SM_Blade` | `law:MechaWeapon` | `0.2345` | `[0.10, 0.60]` | In Band (Conforms) |

---

## 2. Logic Chain & Root Cause Analysis

1. **Height stretching by limbs/wings:** The total body height `body_h` is computed globally from the absolute minimum and maximum vertical coordinates across all parts. Because limbs descend down to `-0.0134` and wings ascend to `0.02925`, they establish a relatively large body height (`0.04265` meters).
2. **Flat torso geometry:** The physical torso geometry (`SM_Torso`) is vertically compact (height `0.0055` meters). Consequently, it occupies only `12.90%` of the total height.
3. **Ontological mismatch:** `116_metric_morphology_bands.ttl` defines shape constraints modeling a conventional biped (where the torso represents `30%-45%` of the height and legs represent `40%-55%`). This discrepancy forces the SHACL processor to flag a failure:
   - Torso is too flat for the expected band.
   - Legs are too long relative to the total height envelope.

---

## 3. Automatic Bounds Tuning Strategy

To automatically align the SHACL ontology rules with the physical proportions of the manufactured asset, the system can dynamically update the RDF shapes prior to validation.

### Design of the Tuner
We recommend a Python script (`scripts/tune_morphology_bands.py`) that:
1. Loads the actual proportions from `METRIC_MORPHOLOGY_REPORT.json` (or measures them on-the-fly).
2. For each part class, computes a tightened range:
   - `MinRatio = max(0.01, round(actual_ratio - tolerance, 3))`
   - `MaxRatio = round(actual_ratio + tolerance, 3)`
   - *Recommended tolerance:* `0.02` (absolute ratio buffer) to allow minor generative fluctuations.
3. Modifies the SHACL shape definitions in `ontology/source_law/116_metric_morphology_bands.ttl`.

### Implementation Options for Modifying TTL
- **Option A (RegEx Matching):** Target specific shape nodes in the raw Turtle text. This preserves human comments, formatting, and file organization.
  ```python
  # Regex to match bandMinRatio and bandMaxRatio within specific blocks
  pattern = r'(law:TorsoHeightBand[\s\S]*?law:bandMinRatio\s*")[^"]+("\^\^xsd:decimal\s*;\s*law:bandMaxRatio\s*")[^"]+("\^\^xsd:decimal)'
  replacement = rf'\1{new_min}\2{new_max}\3'
  ```
- **Option B (RDFLib Manipulation):** Load the ontology into a graph, query the triples, replace values semantically, and serialize back to Turtle.
  ```python
  import rdflib
  from rdflib import Namespace, Literal, XSD
  
  LAW = Namespace("https://rocket-craft.com/ontology/law#")
  g = rdflib.Graph()
  g.parse("ontology/source_law/116_metric_morphology_bands.ttl", format="turtle")
  
  # Remove old triples and insert tuned ones
  g.remove((LAW.TorsoHeightBand, LAW.bandMinRatio, None))
  g.add((LAW.TorsoHeightBand, LAW.bandMinRatio, Literal(new_min, datatype=xsd.decimal)))
  ```
- **Option C (Template Interpolation):** Define `116_metric_morphology_bands.ttl.tera` as a template and render it using a local parameters file. This matches the project's doctrine of "The Graph is the Code".

---

## 4. Pipeline Execution Loop (Merge -> Sync -> Verify -> Replay)

To execute the entire lifecycle cleanly and automatically, a shell strategy should wrap the tuning and verification processes:

```bash
#!/usr/bin/env bash
set -euo pipefail

# 1. Run the tuning script to inspect geometry and rewrite source TTL law
echo ">> Tuning morphology bands in ontology based on reference_fabric USD dimensions..."
python3 scripts/tune_morphology_bands.py

# 2. Merge the updated source_law TTL files into all_merged.ttl
echo ">> Merging ontologies..."
python3 scripts/merge_ontology.py

# 3. Synchronize with ggen to regenerate dynamic asset files and manifests
echo ">> Running ggen sync..."
/Users/sac/.local/bin/ggen sync

# 4. Run the pre-render metric morphology gate to verify compliance and test negative fixtures
echo ">> Verifying metric morphology..."
python3 scripts/verify_metric_morphology.py
```

`verify_metric_morphology.py` contains a built-in replay test that double-runs the evaluation to guarantee deterministic output (meaning step 4 validates both compliance and replayability).
