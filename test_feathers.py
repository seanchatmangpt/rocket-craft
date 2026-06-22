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
left_feathers = []
pfp = os.path.join(usd_dir, "SM_WingArray_Left.usda")
if os.path.exists(pfp):
    with open(pfp, "r") as f:
        content = f.read()
    mesh_blocks = re.findall(r'def Mesh "([^"]+)"\s*\{([^\}]+)\}', content, re.MULTILINE)
    print("Found Meshes:", [name for name, _ in mesh_blocks][:5])
    for name, block in mesh_blocks:
        m_trans = re.search(r'double3 xformOp:translate\s*=\s*\(([^)]+)\)', block)
        if m_trans:
            trans = [float(x.strip()) for x in m_trans.group(1).split(",")]
            base_name = re.sub(r'_(feather|armor|subframe|tread|wheel|barrel|tip|blade).*', '', name)
            part_name = prim_to_part.get(base_name, prim_to_part.get(name, ""))
            if part_name in ["wing_root_left", "primary_wing_feathers_left", "secondary_wing_feathers_left"]:
                left_feathers.append(trans)

print("Left feathers count:", len(left_feathers))
