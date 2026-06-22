import re

gen_file = "patch_geometry_generator.py"
with open(gen_file, "r") as f:
    content = f.read()

content = content.replace("i // 4", "(i - (i % 4)) / 4")

with open(gen_file, "w") as f:
    f.write(content)

print("Fixed tera syntax")
