# ASSEMBLY_COHERENCE_REPORT (kinematic connectivity)

- timestamp: 2026-06-21T06:52:25.274702Z
- verdict: **ADMITTED**
- root_link: SM_Torso
- foreground_component_count: 3 (source: visual_gap_report.json, ADMITTED ceiling 3)
- shacl_conforms (121): True
- replay_verified: True
- receipt_blake3: `blake3:accdd1bef334ed610ffb02ff898d6f1ea5841f107f2bca56ead0060ff4cbcf65`

## Reachability (every part must reach the Torso root via joints)
| part | reaches_root | chain |
|---|---|---|
| SM_Blade_Left | True | SM_Torso -> SM_Limb_Left -> SM_Blade_Left |
| SM_Blade_Right | True | SM_Torso -> SM_Limb_Right -> SM_Blade_Right |
| SM_Head | True | SM_Torso -> SM_Head |
| SM_Limb_Left | True | SM_Torso -> SM_Limb_Left |
| SM_Limb_Right | True | SM_Torso -> SM_Limb_Right |
| SM_Torso | True | SM_Torso |
| SM_WingArray_Left | True | SM_Torso -> SM_WingArray_Left |
| SM_WingArray_Right | True | SM_Torso -> SM_WingArray_Right |

## Per-joint contact
| joint | parent | child | in_contact | worst_gap_m |
|---|---|---|---|---|
| joint_neck | SM_Torso | SM_Head | True | 0.0002 |
| joint_shoulder_left | SM_Torso | SM_Limb_Left | True | -0.007 |
| joint_shoulder_right | SM_Torso | SM_Limb_Right | True | -0.007 |
| joint_wing_mount_left | SM_Torso | SM_WingArray_Left | True | -0.0028 |
| joint_wing_mount_right | SM_Torso | SM_WingArray_Right | True | -0.0028 |
| joint_weapon_left | SM_Limb_Left | SM_Blade_Left | True | -0.01 |
| joint_weapon_right | SM_Limb_Right | SM_Blade_Right | True | -0.01 |

## Refusals
- (none)

## Negative fixture (Blade_Left translated far away)
- refused: True
- disconnected_joints: joint_neck, joint_shoulder_left, joint_shoulder_right, joint_weapon_left, joint_weapon_right, joint_wing_mount_left, joint_wing_mount_right
