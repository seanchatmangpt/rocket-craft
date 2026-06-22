import re
import os

ttl_path = "ontology/all_merged.ttl"
prim_to_part = {}
if os.path.exists(ttl_path):
    with open(ttl_path, "r") as f:
        ttl_content = f.read()
    blocks = re.findall(r'mud:(prim_[a-zA-Z0-9_]+)\s+rdf:type\s+mud:GeometryPrimitive\s*;([^.]+)\.', ttl_content, re.MULTILINE)
    for prim_name, block in blocks:
        m_part = re.search(r'mud:belongsToPart\s+mud:([^\s;]+)', block)
        if m_part:
            prim_to_part[prim_name] = m_part.group(1).strip()

print(f"Loaded {len(prim_to_part)} prims")
print(f"prim_pwf_left_00 -> {prim_to_part.get('prim_pwf_left_00')}")

content = ""
with open("generated/mech_assets/reference_fabric_001/usd/SM_WingArray_Left.usda", "r") as f:
    content = f.read()

left_feathers = []
mesh_blocks = re.findall(r'def Mesh "([^"]+)"\s*\{([^\}]+)\}', content, re.MULTILINE)
for name, block in mesh_blocks:
    m_trans = re.search(r'double3 xformOp:translate\s*=\s*\(([^)]+)\)', block)
    if m_trans:
        trans = [float(x.strip()) for x in m_trans.group(1).split(",")]
        base_name = re.sub(r'_(feather_blade|feather_tip|armor_plate|hardpoint|armor_piston|beveled_plate|blade_edge|tread_link|wheel|barrel_base|muzzle_brake|subframe_core|subframe_joint)_\d+$', '', name)
        part_name = prim_to_part.get(base_name, prim_to_part.get(name, ""))
        if part_name in ["wing_root_left", "primary_wing_feathers_left", "secondary_wing_feathers_left"]:
            left_feathers.append(trans)

print(f"Found {len(left_feathers)} left feathers")

content_r = ""
with open("generated/mech_assets/reference_fabric_001/usd/SM_WingArray_Right.usda", "r") as f:
    content_r = f.read()

right_feathers = []
mesh_blocks_r = re.findall(r'def Mesh "([^"]+)"\s*\{([^\}]+)\}', content_r, re.MULTILINE)
for name, block in mesh_blocks_r:
    m_trans = re.search(r'double3 xformOp:translate\s*=\s*\(([^)]+)\)', block)
    if m_trans:
        trans = [float(x.strip()) for x in m_trans.group(1).split(",")]
        base_name = re.sub(r'_(feather_blade|feather_tip|armor_plate|hardpoint|armor_piston|beveled_plate|blade_edge|tread_link|wheel|barrel_base|muzzle_brake|subframe_core|subframe_joint)_\d+$', '', name)
        part_name = prim_to_part.get(base_name, prim_to_part.get(name, ""))
        if part_name in ["wing_root_right", "primary_wing_feathers_right", "secondary_wing_feathers_right"]:
            right_feathers.append(trans)

print(f"Found {len(right_feathers)} right feathers")

mirror_failures = 0
for l_t in left_feathers:
    matched = False
    for r_t in right_feathers:
        if abs(r_t[0] + l_t[0]) < 1.0 and abs(r_t[1] - l_t[1]) < 1.0 and abs(r_t[2] - l_t[2]) < 1.0:
            matched = True
            break
    if not matched:
        mirror_failures += 1

print(f"Mirror failures: {mirror_failures}")
