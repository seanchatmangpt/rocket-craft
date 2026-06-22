import sys

with open('/Users/sac/rocket-craft/patch_geometry_generator.py', 'r') as f:
    content = f.read()

# Scale for beveled_plate_i
old_scale_bev = """double3 xformOp:scale = (0.2, 0.5, {{ 1.0 - (i * 0.15) }})"""
new_scale_bev = """double3 xformOp:scale = (0.8, 0.5, {{ 1.0 - (i * 0.15) }})"""
content = content.replace(old_scale_bev, new_scale_bev)

# Scale for blade_edge_i
old_scale_edge = """double3 xformOp:scale = (0.2, 0.5, 0.5)"""
new_scale_edge = """double3 xformOp:scale = (1.15, 0.5, 0.5)"""
content = content.replace(old_scale_edge, new_scale_edge)

# Translate is ALREADY X = -0.9 - i * 0.045 and dy = 0.05!
# Wait! In patch_blade2.py I used:
# new_translate = "double3 xformOp:translate = ({% if is_right %}{{ 0.9 + i * 0.045 }}{% else %}{{ -0.9 - i * 0.045 }}{% endif %}, {{ i * 0.05 }}, 2.0)"
# So it is ALREADY correct!

# Rotate should be restored to 15.0
old_rotate = """double3 xformOp:rotateXYZ = (0, 0, 0)"""
new_rotate = """double3 xformOp:rotateXYZ = (0, 0, {% if is_right %}-15.0{% else %}15.0{% endif %})"""
content = content.replace(old_rotate, new_rotate)

with open('/Users/sac/rocket-craft/patch_geometry_generator.py', 'w') as f:
    f.write(content)

print("Patched successfully!")
