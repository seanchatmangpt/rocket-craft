import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

content = content.replace("((i % 2) * 20.0)", "((i % 2) * 40.0)")

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)

