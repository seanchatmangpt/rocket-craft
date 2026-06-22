import os
import re

filepath = "/Users/sac/rocket-craft/ontology/source_law/104_reference_fabric.ttl"
with open(filepath, "r") as f:
    content = f.read()

# We need to find blocks of GeometryPrimitive and update translateY/scaleY.
# We'll split the file by blank lines to get blocks.
blocks = content.split("\n\n")
updated_blocks = []

rules = {
    "mud:torso_core": (6.35, 0.65),
    "mud:head_unit": (7.65, 0.65),
    "mud:shoulder_left": (2.60, 3.20),
    "mud:shoulder_right": (2.60, 3.20),
    "mud:primary_wing_feathers_left": (2.39, 3.00),
    "mud:primary_wing_feathers_right": (2.39, 3.00),
    "mud:blade_left": (8.193, 3.41),
    "mud:blade_right": (8.193, 3.41)
}

updated_count = 0

for block in blocks:
    if "rdf:type mud:GeometryPrimitive" in block:
        # Find belongsToPart
        part_match = re.search(r"mud:belongsToPart\s+(mud:\w+)", block)
        if part_match:
            part_name = part_match.group(1)
            if part_name in rules:
                ty_val, sy_val = rules[part_name]
                # Replace translateY and scaleY
                # translateY is like: mud:translateY 0.0
                block = re.sub(r"mud:translateY\s+[-+]?\d*\.?\d+(?:[eE][-+]?\d+)?", f"mud:translateY {ty_val}", block)
                block = re.sub(r"mud:scaleY\s+[-+]?\d*\.?\d+(?:[eE][-+]?\d+)?", f"mud:scaleY {sy_val}", block)
                updated_count += 1
    updated_blocks.append(block)

new_content = "\n\n".join(updated_blocks)
with open(filepath, "w") as f:
    f.write(new_content)

print(f"Successfully patched {updated_count} primitives in 104_reference_fabric.ttl")
