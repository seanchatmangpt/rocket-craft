import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# blade_edge
content = content.replace(
    'double3 xformOp:scale = (3.0, 0.5, 0.5)',
    'double3 xformOp:scale = (1.2, 0.5, 0.5)'
)
# beveled_plate
content = content.replace(
    'double3 xformOp:scale = (3.0, 0.5, {{ 1.0 - (i * 0.15) }})',
    'double3 xformOp:scale = (1.2, 0.5, {{ 1.0 - (i * 0.15) }})'
)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
