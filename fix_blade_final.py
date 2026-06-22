import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# Remove displayColor from beveled_plate
content = content.replace(
    'color3f[] primvars:displayColor = [(0, 1, 1)]\n                    double3 xformOp:scale = (3.2, 0.4, {{ 1.0 - (i * 0.15) }})',
    'double3 xformOp:scale = (1.5, 0.4, {{ 1.0 - (i * 0.15) }})'
)

# Update blade_edge scale to 1.5, keep displayColor
content = content.replace(
    'double3 xformOp:scale = (3.2, 0.1, 0.1)',
    'double3 xformOp:scale = (1.5, 0.1, 0.1)'
)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
