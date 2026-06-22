import os

gen_file = "patch_geometry_generator.py"
with open(gen_file, "r") as f:
    content = f.read()

content = content.replace("double3 xformOp:scale = (1.8, 0.67,", "double3 xformOp:scale = (1.8, 0.68,")

with open(gen_file, "w") as f:
    f.write(content)

print("Patched blade 7")
