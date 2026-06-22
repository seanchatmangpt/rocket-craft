import os
import re
import json

usd_dir = "generated/mech_assets/reference_fabric_001/usd"
pfp_wing_left = os.path.join(usd_dir, "SM_WingArray_Left.usda")

# load prim_to_part
with open("generated/mech_assets/reference_fabric_001/ocel/asset_manufacturing.ocel.json", "r") as f:
    ocel = json.load(f)
prim_to_part = {}
for e in ocel.get("ocel:events", []):
    if e.get("ocel:activity") == "Assembly":
        prim = e.get("mud:primLocalName")
        part = e.get("mud:partId")
        if prim and part:
            prim_to_part[prim] = part

primary_rys = []
primary_txs = []
if os.path.exists(pfp_wing_left):
    with open(pfp_wing_left, "r") as f:
        content = f.read()
    mesh_blocks = re.findall(r'def Mesh "([^"]+)"\s*\{([^\}]+)\}', content, re.MULTILINE)
    for name, block in mesh_blocks:
        base_name = re.sub(r'_(feather_blade|feather_tip|armor_plate|hardpoint|armor_piston|beveled_plate|blade_edge|tread_link|wheel|barrel_base|muzzle_brake|subframe_core|subframe_joint)_\d+$', '', name)
        part_name = prim_to_part.get(base_name, "")
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
    print("diffs", diffs)
    feather_panel_curvature_score = float(min(1.0, (sum(diffs) / (len(sorted_feathers) - 1)) / 10.0))
    print("score", feather_panel_curvature_score)
