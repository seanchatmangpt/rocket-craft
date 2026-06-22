import sys

with open('/Users/sac/rocket-craft/patch_geometry_generator.py', 'r') as f:
    content = f.read()

# Restore stack translate to dy=0.05, dx=0.045
old_translate = """double3 xformOp:translate = ({% if is_right %}0.9{% else %}-0.9{% endif %}, 0.0, 2.0)"""
new_translate = """double3 xformOp:translate = ({% if is_right %}{{ 0.9 + i * 0.045 }}{% else %}{{ -0.9 - i * 0.045 }}{% endif %}, {{ i * 0.05 }}, 2.0)"""
content = content.replace(old_translate, new_translate)

# Change scale_X to 2.14, scale_Y to 0.50
old_scale_bev = """double3 xformOp:scale = (2.4, 0.1, {{ 1.0 - (i * 0.15) }})"""
new_scale_bev = """double3 xformOp:scale = (2.14, 0.5, {{ 1.0 - (i * 0.15) }})"""
content = content.replace(old_scale_bev, new_scale_bev)

old_scale_edge = """double3 xformOp:scale = (2.4, 0.1, 0.5)"""
new_scale_edge = """double3 xformOp:scale = (2.14, 0.5, 0.5)"""
content = content.replace(old_scale_edge, new_scale_edge)

# Rotate is ALREADY correct from patch 5: (0, 0, {% if is_right %}-15.0{% else %}15.0{% endif %})
# Wait! In patch 5, I set it to:
# new_rotate = """double3 xformOp:rotateXYZ = (0, 0, {% if is_right %}-15.0{% else %}15.0{% endif %})"""
# So Right is -15.0, Left is 15.0!
# THIS IS WRONG! I just derived Right must be 15.0, Left must be -15.0!
# So I need to SWAP it back to what patch 4 had!
old_rotate = """double3 xformOp:rotateXYZ = (0, 0, {% if is_right %}-15.0{% else %}15.0{% endif %})"""
new_rotate = """double3 xformOp:rotateXYZ = (0, 0, {% if is_right %}15.0{% else %}-15.0{% endif %})"""
content = content.replace(old_rotate, new_rotate)

with open('/Users/sac/rocket-craft/patch_geometry_generator.py', 'w') as f:
    f.write(content)

print("Patched successfully!")
