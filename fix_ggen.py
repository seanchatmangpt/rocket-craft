import re

with open("ggen.toml", "r") as f:
    content = f.read()

new_content = content.replace(
    'BIND(STRAFTER(STR(?prim), "#") AS ?primLocalName)',
    'BIND(STRAFTER(STR(?prim), "#") AS ?primLocalName)\n  FILTER (?part = ?CURRENT_PART_ID || (?type = "socket" && ?part != ?CURRENT_PART_ID))'
)

with open("ggen.toml", "w") as f:
    f.write(new_content)
