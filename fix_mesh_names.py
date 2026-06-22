import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# Prefix all Mesh definitions with the primLocalName so the verifier can map them back
content = re.sub(r'def\s+Mesh\s+"([^"]+)"', r'def Mesh "{{ row.primLocalName }}_\1"', content)

# Remove the fake point3f[] points = [(0, 0, 0)] that triggers USD401
content = content.replace('            point3f[] points = [(0, 0, 0)]\n', '')

# Fix blade length to 4.0
content = content.replace(
    'double3 xformOp:scale = (2.5, 0.2, {{ 1.0 - (i * 0.15) }})',
    'double3 xformOp:scale = (4.0, 0.2, {{ 1.0 - (i * 0.15) }})'
)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
