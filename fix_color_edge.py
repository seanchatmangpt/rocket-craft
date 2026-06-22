import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

edge_str = 'double3 xformOp:translate = (0.3, {{ i * 0.05 }}, 0.0)'
edge_color_str = 'color3f[] primvars:displayColor = [(0, 1, 1)]\n                    double3 xformOp:translate = ({% if is_right %}0.3{% else %}-0.3{% endif %}, {{ i * 0.05 }}, 0.0)'

if edge_str in content:
    content = content.replace(edge_str, edge_color_str)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
