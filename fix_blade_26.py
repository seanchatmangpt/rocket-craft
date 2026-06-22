import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# blade_edge
content = content.replace(
    'double3 xformOp:scale = (6.2, 0.5, 0.5)',
    'double3 xformOp:scale = (2.6, 0.5, 0.5)'
)
# beveled_plate (which is currently 1.5)
content = content.replace(
    'double3 xformOp:scale = (1.5, 0.4, {{ 1.0 - (i * 0.15) }})',
    'double3 xformOp:scale = (2.6, 0.4, {{ 1.0 - (i * 0.15) }})'
)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
