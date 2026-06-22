import sys

with open('/Users/sac/rocket-craft/patch_geometry_generator.py', 'r') as f:
    content = f.read()

# Swap the rotate signs
old_rotate = """double3 xformOp:rotateXYZ = (0, 0, {% if is_right %}-15.0{% else %}15.0{% endif %})"""
new_rotate = """double3 xformOp:rotateXYZ = (0, 0, {% if is_right %}15.0{% else %}-15.0{% endif %})"""
content = content.replace(old_rotate, new_rotate)

with open('/Users/sac/rocket-craft/patch_geometry_generator.py', 'w') as f:
    f.write(content)

print("Patched successfully!")
