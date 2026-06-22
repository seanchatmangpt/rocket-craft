import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# Replace beveled_plate scale
content = content.replace(
    'double3 xformOp:scale = (2.0, 0.4, {{ 1.0 - (i * 0.15) }})',
    'double3 xformOp:scale = (3.2, 0.4, {{ 1.0 - (i * 0.15) }})'
)

# Add scale to blade_edge
blade_edge_str = 'double3 xformOp:translate = ({% if is_right %}0.3{% else %}-0.3{% endif %}, {{ i * 0.05 }}, 0.0)'
blade_edge_new_str = 'double3 xformOp:scale = (3.2, 0.1, 0.1)\n                    double3 xformOp:translate = ({% if is_right %}0.3{% else %}-0.3{% endif %}, {{ i * 0.05 }}, 0.0)'

if blade_edge_str in content and 'scale = (3.2, 0.1' not in content:
    content = content.replace(blade_edge_str, blade_edge_new_str)

# Also update the xformOpOrder to include scale for blade_edge
content = content.replace(
    'uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:rotateXYZ"]',
    'uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:rotateXYZ", "xformOp:scale"]'
)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
