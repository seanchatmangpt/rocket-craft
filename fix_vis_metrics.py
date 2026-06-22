import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# Current logic:
# double3 xformOp:rotateXYZ = (0, {% if is_right %}{{ (i * 10.0) * -1.0 }}{% else %}{{ i * 10.0 }}{% endif %}, 0)
# We change i * 10.0 to (i % 2) * 20.0

content = content.replace("{{ (i * 10.0) * -1.0 }}", "{{ ((i % 2) * 20.0) * -1.0 }}")
content = content.replace("{{ i * 10.0 }}", "{{ (i % 2) * 20.0 }}")

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)

