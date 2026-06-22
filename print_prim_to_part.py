import json

with open("generated/mech_assets/reference_fabric_001/ocel/asset_manufacturing.ocel.json", "r") as f:
    ocel = json.load(f)
for e in ocel.get("ocel:events", []):
    if e.get("ocel:activity") == "Assembly":
        prim = e.get("mud:primLocalName")
        part = e.get("mud:partId")
        if prim and part and "feather" in part:
            print(prim, "->", part)
