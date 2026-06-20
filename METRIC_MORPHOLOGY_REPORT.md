# METRIC_MORPHOLOGY_REPORT (pre-render graph law)

- timestamp: 2026-06-20T23:09:52.976409Z
- verdict: **PARTIAL_ALIVE**
- body_height_m: 0.04265 (vertical axis Z, metersPerUnit 0.01)
- shacl_conforms: False
- replay_verified: True
- receipt_blake3: `blake3:eada4bcc5cc966f7358921abdb8bacf8cd219c7d17e2ba116c753e1ac7f26b2b`

## Per-part envelopes
| part | y_min_m | y_max_m | height_m | ratio | band | in_band |
|---|---|---|---|---|---|---|
| SM_Blade_Left | -0.0025 | 0.0025 | 0.005 | 0.1172 | [0.1, 0.6] | True |
| SM_Blade_Right | -0.0025 | 0.0025 | 0.005 | 0.1172 | [0.1, 0.6] | True |
| SM_Head | 0.0105 | 0.0195 | 0.009 | 0.211 | [0.08, 0.15] | False |
| SM_Limb_Left | -0.0134 | 0.0145 | 0.0279 | 0.6542 | [0.4, 0.55] | False |
| SM_Limb_Right | -0.0134 | 0.0145 | 0.0279 | 0.6542 | [0.4, 0.55] | False |
| SM_Torso | -0.0045 | 0.0045 | 0.009 | 0.211 | [0.3, 0.45] | False |
| SM_WingArray_Left | 0.00275 | 0.02925 | 0.0265 | 0.6213 | [0.4, 0.9] | True |
| SM_WingArray_Right | 0.00275 | 0.02925 | 0.0265 | 0.6213 | [0.4, 0.9] | True |

## Refusals
- REFUSE_PART_HEIGHT_BAND(SM_Head): ratio 0.2110 outside [0.08,0.15]
- REFUSE_PART_HEIGHT_BAND(SM_Limb_Left): ratio 0.6542 outside [0.40,0.55]
- REFUSE_PART_HEIGHT_BAND(SM_Limb_Right): ratio 0.6542 outside [0.40,0.55]
- REFUSE_PART_HEIGHT_BAND(SM_Torso): ratio 0.2110 outside [0.30,0.45]
- REFUSE_PART_HEIGHT_BAND: part height ratio falls outside its declared morphology band.

## Negative fixture (UFO disc)
- refused: True
- codes: ANATOMY_PARADOX, REFUSE_DEFAULT_SHIELD_PROPORTION
