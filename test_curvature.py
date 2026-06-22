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
primary_rys = []
primary_txs = []
if os.path.exists(pfp_wing_left):
    with open(pfp_wing_left, "r") as f:
        content = f.read()
    mesh_blocks = re.findall(r'def Mesh "([^"]+)"\s*\{([^\}]+)\}', content, re.MULTILINE)
    for name, block in mesh_blocks:
        base_name = re.sub(r'_(feather_blade|feather_tip|armor_plate|hardpoint|armor_piston|beveled_plate|blade_edge|tread_link|wheel|barrel_base|muzzle_brake|subframe_core|subframe_joint)_\d+$', '', name)
        part_name = prim_to_part.get(base_name, prim_to_part.get(name, ""))
        if part_name == "primary_wing_feathers_left":
            m_rot = re.search(r'double3 xformOp:rotateXYZ\s*=\s*\(([^)]+)\)', block)
            m_trans = re.search(r'double3 xformOp:translate\s*=\s*\(([^)]+)\)', block)
            if m_rot and m_trans:
                primary_rys.append(float(m_rot.group(1).split(",")[1]))
                primary_txs.append(float(m_trans.group(1).split(",")[0]))

print("Length of primary_txs:", len(primary_txs))
if len(primary_txs) > 1:
    sorted_feathers = sorted(zip(primary_txs, primary_rys), key=lambda x: x[0])
    diffs = [abs(sorted_feathers[i][1] - sorted_feathers[i-1][1]) for i in range(1, len(sorted_feathers))]
    print("sum(diffs):", sum(diffs))
    print("score:", float(min(1.0, (sum(diffs) / (len(sorted_feathers) - 1)) / 10.0)))
