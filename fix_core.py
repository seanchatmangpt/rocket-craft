import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# Fix core_armor_plate
content = content.replace(
    'double3 xformOp:scale = ({% if i < 2 %}0.8{% else %}1.05{% endif %}, {% if i > 1 and i < 4 %}0.8{% else %}1.05{% endif %}, {% if i > 3 %}0.8{% else %}1.05{% endif %})',
    'double3 xformOp:scale = (1.0, 1.0, 1.0)'
)
content = content.replace(
    'double3 xformOp:translate = ({% if i == 0 %}0.1, 0, 0{% elif i == 1 %}-0.1, 0, 0{% elif i == 2 %}0, 0.1, 0{% elif i == 3 %}0, -0.1, 0{% elif i == 4 %}0, 0, 0.1{% else %}0, 0, -0.1{% endif %})',
    'double3 xformOp:translate = ({% if i == 0 %}0.02, 0, 0{% elif i == 1 %}-0.02, 0, 0{% elif i == 2 %}0, 0.02, 0{% elif i == 3 %}0, -0.02, 0{% elif i == 4 %}0, 0, 0.02{% else %}0, 0, -0.02{% endif %})'
)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
