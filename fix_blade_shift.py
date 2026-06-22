import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# blade_edge
content = content.replace(
    'double3 xformOp:scale = (5.2, 0.2, 0.2)',
    'double3 xformOp:scale = (3.0, 0.5, 0.5)'
)
content = content.replace(
    'double3 xformOp:translate = ({% if is_right %}0.3{% else %}-0.3{% endif %}, {{ i * 0.05 }}, 2.0)',
    'double3 xformOp:translate:myOffset = ({% if is_right %}-1.5{% else %}1.5{% endif %}, {{ i * 0.05 }}, 2.0)'
)
content = content.replace(
    'uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:rotateXYZ", "xformOp:scale"]',
    'uniform token[] xformOpOrder = ["xformOp:translate:myOffset", "xformOp:rotateXYZ", "xformOp:scale"]'
)

# beveled_plate
content = content.replace(
    'double3 xformOp:scale = (5.2, 0.4, {{ 1.0 - (i * 0.15) }})',
    'double3 xformOp:scale = (3.0, 0.5, {{ 1.0 - (i * 0.15) }})'
)
content = content.replace(
    'double3 xformOp:translate = (0.0, {{ i * 0.05 }}, 2.0)',
    'double3 xformOp:translate:myOffset = ({% if is_right %}-1.5{% else %}1.5{% endif %}, {{ i * 0.05 }}, 2.0)'
)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
