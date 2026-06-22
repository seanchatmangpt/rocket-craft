import re

prim_to_part = {}
ttl_path = "ontology/all_merged.ttl"
with open(ttl_path, "r") as f:
    ttl_content = f.read()
blocks = re.findall(r'mud:(prim_[a-zA-Z0-9_]+)\s+rdf:type\s+mud:GeometryPrimitive\s*;([^.]+)\.', ttl_content, re.MULTILINE)
for prim_name, block in blocks:
    m_part = re.search(r'mud:belongsToPart\s+mud:([^\s;]+)', block)
    if m_part:
        prim_to_part[prim_name] = m_part.group(1).strip()

print("Found", len(prim_to_part), "prims mapped.")

usd_dir = "generated/mech_assets/reference_fabric_001/usd"
pfp_wing_left = usd_dir + "/SM_WingArray_Left.usda"

primary_rys = []
primary_txs = []
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

sorted_feathers = sorted(zip(primary_txs, primary_rys), key=lambda x: x[0])
print(sorted_feathers)
if len(sorted_feathers) > 1:
    diffs = [abs(sorted_feathers[i][1] - sorted_feathers[i-1][1]) for i in range(1, len(sorted_feathers))]
    print("diffs sum:", sum(diffs), "len:", len(diffs))
    feather_panel_curvature_score = float(min(1.0, (sum(diffs) / (len(sorted_feathers) - 1)) / 10.0))
    print("score:", feather_panel_curvature_score)
