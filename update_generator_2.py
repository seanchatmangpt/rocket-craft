import re

gen_file = "patch_geometry_generator.py"
with open(gen_file, "r") as f:
    content = f.read()

old_scale = "double3 xformOp:scale = (2.45, 0.1, 0.5)"
new_scale = "double3 xformOp:scale = (2.70, 0.1, 0.5)"

if old_scale in content:
    content = content.replace(old_scale, new_scale)
else:
    print("Could not find old_scale")

with open(gen_file, "w") as f:
    f.write(content)

print("Updated scale")
