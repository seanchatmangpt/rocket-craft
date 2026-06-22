import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# blade_edge
content = content.replace(
    'double3 xformOp:scale = (2.6, 0.2, 0.2)',
    'double3 xformOp:scale = (5.2, 0.2, 0.2)'
)
# beveled_plate
content = content.replace(
    'double3 xformOp:scale = (2.6, 0.4, {{ 1.0 - (i * 0.15) }})',
    'double3 xformOp:scale = (5.2, 0.4, {{ 1.0 - (i * 0.15) }})'
)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
