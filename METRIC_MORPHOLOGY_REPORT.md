# METRIC_MORPHOLOGY_REPORT (pre-render graph law)

- timestamp: 2026-06-20T23:36:50.745156Z
- verdict: **PARTIAL_ALIVE**
- body_height_m: 0.2092 (vertical axis Z, metersPerUnit 0.01)
- shacl_conforms: False
- replay_verified: True
- receipt_blake3: `blake3:f3e3569d2020efac49ca745c1bca982f7ad55b2704fd0f27549e8585b32f8144`

## Per-part envelopes
| part | y_min_m | y_max_m | height_m | ratio | band | in_band |
|---|---|---|---|---|---|---|
| SM_Blade_Left | 0.077 | 0.1092 | 0.0322 | 0.1539 | [0.2245, 0.2445] | False |
| SM_Blade_Right | 0.077 | 0.1092 | 0.0322 | 0.1539 | [0.2245, 0.2445] | False |
| SM_Head | -0.01 | 0.01 | 0.02 | 0.0956 | [0.119, 0.139] | False |
| SM_Limb_Left | -0.006 | 0.062 | 0.068 | 0.325 | [0.6207, 0.6407] | False |
| SM_Limb_Right | -0.006 | 0.062 | 0.068 | 0.325 | [0.6207, 0.6407] | False |
| SM_Torso | 0.062 | 0.075 | 0.013 | 0.0621 | [0.119, 0.139] | False |
| SM_WingArray_Left | -0.0075 | 0.1992 | 0.2067 | 0.988 | [0.6113, 0.6313] | False |
| SM_WingArray_Right | -0.0075 | 0.1992 | 0.2067 | 0.988 | [0.6113, 0.6313] | False |

## Refusals
- ANATOMY_PARADOX: The Head Y_MIN must be structurally higher than the Torso Y_MAX.
- REFUSE_PART_HEIGHT_BAND(SM_Blade_Left): ratio 0.1539 outside [0.22,0.24]
- REFUSE_PART_HEIGHT_BAND(SM_Blade_Right): ratio 0.1539 outside [0.22,0.24]
- REFUSE_PART_HEIGHT_BAND(SM_Head): ratio 0.0956 outside [0.12,0.14]
- REFUSE_PART_HEIGHT_BAND(SM_Limb_Left): ratio 0.3250 outside [0.62,0.64]
- REFUSE_PART_HEIGHT_BAND(SM_Limb_Right): ratio 0.3250 outside [0.62,0.64]
- REFUSE_PART_HEIGHT_BAND(SM_Torso): ratio 0.0621 outside [0.12,0.14]
- REFUSE_PART_HEIGHT_BAND(SM_WingArray_Left): ratio 0.9880 outside [0.61,0.63]
- REFUSE_PART_HEIGHT_BAND(SM_WingArray_Right): ratio 0.9880 outside [0.61,0.63]
- REFUSE_PART_HEIGHT_BAND: part height ratio falls outside its declared morphology band.

## Negative fixture (UFO disc)
- refused: True
- codes: ANATOMY_PARADOX, REFUSE_DEFAULT_SHIELD_PROPORTION
