import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

prims = [
    "feather_blade", "feather_tip", "armor_plate", "hardpoint", "armor_piston",
    "beveled_plate", "blade_edge", "tread_link", "wheel", "barrel_base",
    "muzzle_brake", "subframe_core", "subframe_joint", "armor_piston2"
]

for p in prims:
    # We want to replace `"p_` with `"{{ row.primLocalName }}_p_`
    # and `"p"` with `"{{ row.primLocalName }}_p"` if it doesn't have an index
    content = re.sub(rf'"({p}_[^{{]*)', r'"{{ row.primLocalName }}_\1', content)
    content = re.sub(rf'"({p})"', r'"{{ row.primLocalName }}_\1"', content)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
