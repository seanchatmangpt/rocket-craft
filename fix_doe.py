import os
import re

with open("scripts/run_mecha_doe.py", "r") as f:
    content = f.read()

part_files_old = """    part_files = [
        "SM_Torso.usda",
        "SM_Head.usda",
        "SM_WingArray_Left.usda",
        "SM_WingArray_Right.usda",
        "SM_Blade_Left.usda",
        "SM_Blade_Right.usda"
    ]"""

part_files_new = """    part_files = [
        "SM_Torso.usda",
        "SM_Head.usda",
        "SM_Wing_Left.usda",
        "SM_Wing_Right.usda",
        "SM_Blade_Left.usda",
        "SM_Blade_Right.usda",
        "SM_Arm_Left.usda",
        "SM_Arm_Right.usda",
        "SM_Leg_Left.usda",
        "SM_Leg_Right.usda"
    ]"""

content = content.replace(part_files_old, part_files_new)

allowed_parts_old = """    allowed_parts = {
        "SM_Torso.usda": {"torso_core"},
        "SM_Head.usda": {"head_unit", "mecha_crown_left", "mecha_crown_right"},
        "SM_WingArray_Left.usda": {"wing_root_left", "primary_wing_feathers_left", "secondary_wing_feathers_left"},
        "SM_WingArray_Right.usda": {"wing_root_right", "primary_wing_feathers_right", "secondary_wing_feathers_right"},
        "SM_Blade_Left.usda": {"blade_left"},
        "SM_Blade_Right.usda": {"blade_right"}
    }"""

allowed_parts_new = """    allowed_parts = {
        "SM_Torso.usda": {"torso_core"},
        "SM_Head.usda": {"head_unit", "mecha_crown_left", "mecha_crown_right"},
        "SM_Wing_Left.usda": {"wing_root_left", "primary_wing_feathers_left", "secondary_wing_feathers_left"},
        "SM_Wing_Right.usda": {"wing_root_right", "primary_wing_feathers_right", "secondary_wing_feathers_right"},
        "SM_Blade_Left.usda": {"blade_left"},
        "SM_Blade_Right.usda": {"blade_right"},
        "SM_Arm_Left.usda": {"shoulder_left", "arm_left"},
        "SM_Arm_Right.usda": {"shoulder_right", "arm_right"},
        "SM_Leg_Left.usda": {"leg_left"},
        "SM_Leg_Right.usda": {"leg_right"}
    }"""

content = content.replace(allowed_parts_old, allowed_parts_new)

with open("scripts/run_mecha_doe.py", "w") as f:
    f.write(content)

print("Patched scripts/run_mecha_doe.py")
