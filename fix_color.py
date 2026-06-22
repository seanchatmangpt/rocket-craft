import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# Add displayColor to blade
blade_str = 'double3 xformOp:scale = (2.0, 0.4, {{ 1.0 - (i * 0.15) }})'
blade_color_str = 'color3f[] primvars:displayColor = [(0, 1, 1)]\n                    double3 xformOp:scale = (2.0, 0.4, {{ 1.0 - (i * 0.15) }})'

if blade_str in content and 'displayColor' not in content:
    content = content.replace(blade_str, blade_color_str)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
