import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

content = content.replace(
    '{% elif row.type == "core" %}',
    '{% elif row.type == "core" or row.type == "hard_surface_shell" %}'
)
content = content.replace(
    '{% elif row.type == "beveled_hard_surface_plate" or row.type == "blade_prism" %}',
    '{% elif row.type == "beveled_hard_surface_plate" or row.type == "blade_prism" or row.type == "blade" %}'
)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
