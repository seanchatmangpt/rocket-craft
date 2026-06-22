import os
import json

root = "generated/mech_assets/tiger_tank"
os.makedirs(f"{root}/usd", exist_ok=True)
os.makedirs(f"{root}/materialx", exist_ok=True)
os.makedirs(f"{root}/receipts", exist_ok=True)

# 1. mtlx file
with open(f"{root}/materialx/materials.mtlx", "w") as f:
    f.write("""<?xml version="1.0"?>
<materialx version="1.38">
  <surfacematerial name="M_TigerArmor" type="material" />
</materialx>""")

# 2. receipts file
with open(f"{root}/receipts/TigerMesh.json", "w") as f:
    f.write("{}")

# 3. SM_TigerHull.usda
usda_content = """#usda 1.0
(
    defaultPrim = "SM_TigerHull"
    metersPerUnit = 0.01
    upAxis = "Y"
)

def Xform "SM_TigerHull"
{
        float3[] extents = [(-50, -50, -50), (50, 50, 50)]

    def Mesh "TigerMesh"
    {
        payload = @./some_payload.usda@
        rel material:binding = </ASSET_TigerTank/Materials/M_TigerArmor>
    }

    def Xform "socket_tiger_coaxial_mg34"
    {
        double3 xformOp:translate = (45.0, 20.0, 180.0)
        uniform token[] xformOpOrder = ["xformOp:translate"]
    }

    def Xform "socket_tiger_hull_mg34"
    {
        double3 xformOp:translate = (85.0, -30.0, 130.0)
        uniform token[] xformOpOrder = ["xformOp:translate"]
    }
}
"""

with open(f"{root}/usd/SM_TigerHull.usda", "w") as f:
    f.write(usda_content)

print("Tiger Tank USD asset and dependencies generated.")
