import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

content = content.replace("{{ -(0.4 - (i * 0.02)) }}", "{{ (0.4 - (i * 0.02)) * -1.0 }}")
content = content.replace("{{ -(-0.4 + (i * 0.02)) }}", "{{ (-0.4 + (i * 0.02)) * -1.0 }}")
content = content.replace("{{ -(i * 0.1) }}", "{{ (i * 0.1) * -1.0 }}")
content = content.replace("{{ -(i * 0.3 - 4.5) }}", "{{ (i * 0.3 - 4.5) * -1.0 }}")
content = content.replace("{{ -(i * 0.6 - 2.1) }}", "{{ (i * 0.6 - 2.1) * -1.0 }}")
content = content.replace("{{ -(i * 10.0) }}", "{{ (i * 10.0) * -1.0 }}")

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)

