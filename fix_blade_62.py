import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

content = content.replace(
    'double3 xformOp:scale = (1.5, 0.1, 0.1)',
    'double3 xformOp:scale = (6.2, 0.5, 0.5)'
)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
