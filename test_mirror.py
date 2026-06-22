import re
import os

usd_dir = "generated/mech_assets/reference_fabric_001/usd"
left_feathers = []
right_feathers = []

for pf in ["SM_WingArray_Left.usda", "SM_WingArray_Right.usda"]:
    pfp = os.path.join(usd_dir, pf)
    if os.path.exists(pfp):
        with open(pfp, "r") as f:
            content = f.read()
        mesh_blocks = re.findall(r'def Mesh "([^"]+)"\s*\{([^\}]+)\}', content, re.MULTILINE)
        print(f"Found {len(mesh_blocks)} mesh blocks in {pf}")
        for name, block in mesh_blocks:
            m_trans = re.search(r'double3 xformOp:translate\s*=\s*\(([^)]+)\)', block)
            if m_trans:
                trans = [float(x.strip()) for x in m_trans.group(1).split(",")]
                base_name = re.sub(r'_(feather_blade|feather_tip|armor_plate|hardpoint|armor_piston|beveled_plate|blade_edge|tread_link|wheel|barrel_base|muzzle_brake|subframe_core|subframe_joint)_\d+$', '', name)
                # Let's just print base_name
                print(f"Mesh: {name}, base: {base_name}, trans: {trans}")

