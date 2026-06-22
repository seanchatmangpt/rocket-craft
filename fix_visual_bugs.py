import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# Fix VIS203: increase rotation multiplier from 20.0 to 30.0
content = content.replace("{{ ((i % 2) * 20.0) * -1.0 }}", "{{ ((i % 2) * 30.0) * -1.0 }}")
content = content.replace("{{ (i % 2) * 20.0 }}", "{{ (i % 2) * 30.0 }}")

# Fix USD307 and VIS205: ensure translation is negative for Left, positive for Right
blade_replacement = """                    double3 xformOp:scale = (2.0, 0.4, {{ 1.0 - (i * 0.15) }})
                    double3 xformOp:translate = ({% if is_right %}{{ i * 0.1 }}{% else %}{{ i * -0.1 }}{% endif %}, {{ i * 0.05 }}, 0.0)
                    double3 xformOp:rotateXYZ = (0, 0, {% if is_right %}-15.0{% else %}15.0{% endif %})"""

old_blade = """                    double3 xformOp:scale = (4.0, 0.2, {{ 1.0 - (i * 0.15) }})
                    double3 xformOp:translate = ({{ i * 0.1 }}, {{ i * 0.05 }}, 0.0)
                    double3 xformOp:rotateXYZ = (0, 0, {% if is_right %}-15.0{% else %}15.0{% endif %})"""

if old_blade in content:
    content = content.replace(old_blade, blade_replacement)
else:
    # Attempting an alternative regex replacement in case whitespace differs
    content = re.sub(r'double3 xformOp:scale = \(4\.0, 0\.2,[^\)]+\)\s+double3 xformOp:translate = \(\{\{ i \* 0\.1 \}\}, \{\{ i \* 0\.05 \}\}, 0\.0\)', 
                     r'double3 xformOp:scale = (2.0, 0.4, {{ 1.0 - (i * 0.15) }})\n                    double3 xformOp:translate = ({% if is_right %}{{ i * 0.1 }}{% else %}{{ i * -0.1 }}{% endif %}, {{ i * 0.05 }}, 0.0)', content)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
