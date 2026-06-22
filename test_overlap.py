import os
import re

ttl_content = ""
with open("ontology/all_merged.ttl", "r") as f:
    ttl_content = f.read()

prim_to_part = {}
blocks = re.findall(r'mud:(prim_[a-zA-Z0-9_]+)\s+rdf:type\s+mud:GeometryPrimitive\s*;([^.]+)\.', ttl_content, re.MULTILINE)
for prim_name, block in blocks:
    m_part = re.search(r'mud:belongsToPart\s+mud:([^\s;]+)', block)
    if m_part:
        prim_to_part[prim_name] = m_part.group(1).strip()

usd_dir = "generated/mech_assets/reference_fabric_001/usd"
pfp_wing_left = os.path.join(usd_dir, "SM_WingArray_Left.usda")
primary_scales = []
if os.path.exists(pfp_wing_left):
    with open(pfp_wing_left, "r") as f:
        content = f.read()
    mesh_blocks = re.findall(r'def Mesh "([^"]+)"\s*\{([^\}]+)\}', content, re.MULTILINE)
    for name, block in mesh_blocks:
        base_name = re.sub(r'_(feather_blade|feather_tip|armor_plate|hardpoint|armor_piston|beveled_plate|blade_edge|tread_link|wheel|barrel_base|muzzle_brake|subframe_core|subframe_joint)_\d+$', '', name)
        part_name = prim_to_part.get(base_name, prim_to_part.get(name, ""))
        if part_name == "primary_wing_feathers_left":
            m_trans = re.search(r'double3 xformOp:translate\s*=\s*\(([^)]+)\)', block)
            m_scale = re.search(r'double3 xformOp:scale\s*=\s*\(([^)]+)\)', block)
            if m_trans and m_scale:
                primary_scales.append((float(m_trans.group(1).split(",")[0]), float(m_scale.group(1).split(",")[0])))

overlap_count = 0
sorted_scales = sorted(primary_scales, key=lambda x: x[0])
if len(sorted_scales) > 1:
    for i in range(1, len(sorted_scales)):
        dx = abs(sorted_scales[i][0] - sorted_scales[i-1][0])
        if dx < sorted_scales[i-1][1] * 0.5:
            overlap_count += 1
    score = float(overlap_count / (len(sorted_scales) - 1))
    print("score:", score)
