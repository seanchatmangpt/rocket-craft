import re
name = "prim_primary_wing_feathers_left_feather_blade_0"
base_name = re.sub(r'_(feather_blade|feather_tip|armor_plate|hardpoint|armor_piston|beveled_plate|blade_edge|tread_link|wheel|barrel_base|muzzle_brake|subframe_core|subframe_joint)_\d+$', '', name)
print(base_name)
