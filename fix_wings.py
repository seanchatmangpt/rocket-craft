import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

content = content.replace('SM_WingArray_Left', 'SM_Wing_Left')
content = content.replace('SM_WingArray_Right', 'SM_Wing_Right')

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
