import os
import re

usd_dir = "generated/mech_assets/reference_fabric_001/usd"
left_wing_rys = []
pfp_wing_left = os.path.join(usd_dir, "SM_WingArray_Left.usda")
if os.path.exists(pfp_wing_left):
    with open(pfp_wing_left, "r") as f:
        content = f.read()
    mesh_blocks = re.findall(r'def Mesh "([^"]+)"\s*\{([^\}]+)\}', content, re.MULTILINE)
    for name, block in mesh_blocks:
        m_rot = re.search(r'double3 xformOp:rotateXYZ\s*=\s*\(([^)]+)\)', block)
        if m_rot:
            left_wing_rys.append(float(m_rot.group(1).split(",")[1]))

ry_clusters = []
for ry in left_wing_rys:
    added = False
    for c in ry_clusters:
        if abs(c[0] - ry) < 5.0:
            c.append(ry)
            added = True
            break
    if not added:
        ry_clusters.append([ry])

wing_layer_count = len(ry_clusters)
print("wing_layer_count:", wing_layer_count)
print("ry_clusters:", ry_clusters)
