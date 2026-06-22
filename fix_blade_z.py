import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# blade_edge
content = content.replace(
    'double3 xformOp:translate = ({% if is_right %}0.3{% else %}-0.3{% endif %}, {{ i * 0.05 }}, 0.0)',
    'double3 xformOp:translate = ({% if is_right %}0.3{% else %}-0.3{% endif %}, {{ i * 0.05 }}, 2.0)'
)
# beveled_plate
content = content.replace(
    'double3 xformOp:translate = (0.0, {{ i * 0.05 }}, 0.0)',
    'double3 xformOp:translate = (0.0, {{ i * 0.05 }}, 2.0)'
)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
