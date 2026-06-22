import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# blade_edge
content = content.replace(
    'double3 xformOp:scale = (4.0, 0.2, 0.2)',
    'double3 xformOp:scale = (2.6, 0.2, 0.2)'
)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
