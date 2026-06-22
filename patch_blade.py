import sys

with open('/Users/sac/rocket-craft/patch_geometry_generator.py', 'r') as f:
    content = f.read()

# Replace old translate and rotate for beveled_plate_i
old_translate_1 = """double3 xformOp:translate = ({% if is_right %}{{ 0.9 + i * 0.045 }}{% else %}{{ -0.9 - i * 0.045 }}{% endif %}, {{ i * 0.05 }}, 2.0)"""
new_translate_1 = """double3 xformOp:translate = ({% if is_right %}{{ 0.9 + i * 0.091 }}{% else %}{{ -0.9 - i * 0.091 }}{% endif %}, {{ i * 0.1 }}, 2.0)"""

old_rotate_1 = """double3 xformOp:rotateXYZ = (0, 0, {% if is_right %}-15.0{% else %}15.0{% endif %})"""
new_rotate_1 = """double3 xformOp:rotateXYZ = (0, 0, 0)"""

content = content.replace(old_translate_1, new_translate_1)
content = content.replace(old_rotate_1, new_rotate_1)

with open('/Users/sac/rocket-craft/patch_geometry_generator.py', 'w') as f:
    f.write(content)

print("Patched successfully!")
