# METRIC_MORPHOLOGY_REPORT (pre-render graph law)

- timestamp: 2026-06-21T14:17:35.278485Z
- verdict: **PARTIAL_ALIVE**
- body_height_m: 0.18 (vertical axis Z, metersPerUnit 0.01)
- shacl_conforms: False
- replay_verified: True
- receipt_blake3: `blake3:22fa645468f41ff5956b8494d781400c97c72a93695bd5f2ccc2b08aa17420cb`

## Per-part envelopes
| part | y_min_m | y_max_m | height_m | ratio | band | in_band |
|---|---|---|---|---|---|---|
| SM_Blade_Left | -0.002 | 0.006 | 0.008 | 0.0444 | [0.1, 0.6] | False |
| SM_Blade_Right | -0.002 | 0.006 | 0.008 | 0.0444 | [0.1, 0.6] | False |
| SM_Head | -0.02 | 0.0625 | 0.0825 | 0.4583 | [0.08, 0.15] | False |
| SM_Limb_Left | -0.035 | 0.04 | 0.075 | 0.4167 | [0.55, 0.85] | False |
| SM_Limb_Right | -0.035 | 0.04 | 0.075 | 0.4167 | [0.55, 0.85] | False |
| SM_Torso | -0.08 | 0.07 | 0.15 | 0.8333 | [0.3, 0.45] | False |
| SM_WingArray_Left | 0.019938 | 0.1 | 0.080063 | 0.4448 | [0.4, 0.9] | True |
| SM_WingArray_Right | 0.019938 | 0.1 | 0.080063 | 0.4448 | [0.4, 0.9] | True |

## Refusals
- ANATOMY_PARADOX: The Head Y_MIN must be structurally higher than the Torso Y_MAX.
- REFUSE_PART_HEIGHT_BAND(SM_Blade_Left): ratio 0.0444 outside [0.10,0.60]
- REFUSE_PART_HEIGHT_BAND(SM_Blade_Right): ratio 0.0444 outside [0.10,0.60]
- REFUSE_PART_HEIGHT_BAND(SM_Head): ratio 0.4583 outside [0.08,0.15]
- REFUSE_PART_HEIGHT_BAND(SM_Limb_Left): ratio 0.4167 outside [0.55,0.85]
- REFUSE_PART_HEIGHT_BAND(SM_Limb_Right): ratio 0.4167 outside [0.55,0.85]
- REFUSE_PART_HEIGHT_BAND(SM_Torso): ratio 0.8333 outside [0.30,0.45]
- REFUSE_PART_HEIGHT_BAND: part height ratio falls outside its declared morphology band.

## Negative fixture (UFO disc)
- refused: True
- codes: ANATOMY_PARADOX, REFUSE_DEFAULT_SHIELD_PROPORTION, REFUSE_PROVENANCE_VIOLATION
