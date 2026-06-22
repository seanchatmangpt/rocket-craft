import sys

with open('/Users/sac/rocket-craft/patch_geometry_generator.py', 'r') as f:
    content = f.read()

# Revert previous translations to 0.045 and 0.05
old_translate = """double3 xformOp:translate = ({% if is_right %}{{ 0.9 + i * 0.091 }}{% else %}{{ -0.9 - i * 0.091 }}{% endif %}, {{ i * 0.1 }}, 2.0)"""
new_translate = """double3 xformOp:translate = ({% if is_right %}{{ 0.9 + i * 0.045 }}{% else %}{{ -0.9 - i * 0.045 }}{% endif %}, {{ i * 0.05 }}, 2.0)"""
content = content.replace(old_translate, new_translate)

# Scale for beveled_plate_i
old_scale_bev = """double3 xformOp:scale = (0.8, 0.5, {{ 1.0 - (i * 0.15) }})"""
new_scale_bev = """double3 xformOp:scale = (0.2, 0.5, {{ 1.0 - (i * 0.15) }})"""
content = content.replace(old_scale_bev, new_scale_bev)

# Scale for blade_edge_i
old_scale_edge = """double3 xformOp:scale = (1.15, 0.5, 0.5)"""
new_scale_edge = """double3 xformOp:scale = (0.2, 0.5, 0.5)"""
content = content.replace(old_scale_edge, new_scale_edge)

with open('/Users/sac/rocket-craft/patch_geometry_generator.py', 'w') as f:
    f.write(content)

print("Patched successfully!")
