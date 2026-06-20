import os
import re

input_file = "snow_white_prelude_mecha.usda"

with open(input_file, "r") as f:
    lines = f.readlines()

# We'll split by `def Xform "Name" (`
part_names = ["Torso", "Head", "LeftArm", "RightArm", "LeftLeg", "RightLeg", "Backpack_Wings"]

def extract_part(part_name):
    part_content = f"""#usda 1.0
(
    defaultPrim = "{part_name}"
    upAxis = "Y"
    metersPerUnit = 1.0
)

def Xform "{part_name}" (
    kind = "component"
)
{{
"""
    inside = False
    brace_level = 0
    
    for i, line in enumerate(lines):
        if not inside:
            if f'def Xform "{part_name}"' in line:
                inside = True
                brace_level = 0
                if "{" in line:
                    brace_level += line.count("{") - line.count("}")
                continue
        else:
            brace_level += line.count("{") - line.count("}")
            if brace_level < 0 or (brace_level == 0 and "}" in line and line.strip() == "}"):
                part_content += "}\n"
                break
            
            # Remove an indent level
            if line.startswith("    "):
                part_content += line[4:]
            else:
                part_content += line
                
    return part_content

for part in part_names:
    content = extract_part(part)
    with open(f"pipeline_demo/SM_{part}.usda", "w") as f:
        f.write(content)

assembly_content = """#usda 1.0
(
    defaultPrim = "SnowWhitePrelude"
    metersPerUnit = 1.0
    upAxis = "Y"
    doc = "Structural USDA representation for the Snow White Prelude mecha, metric scale (meters)."
)

def Xform "SnowWhitePrelude" (
    kind = "assembly"
)
{
    double3 xformOp:translate = (0, 15.0, 0)
    uniform token[] xformOpOrder = ["xformOp:translate"]
"""
for part in part_names:
    assembly_content += f"""
    def Xform "{part}" (
        references = @./SM_{part}.usda@</{part}>
    )
    {{}}
"""
assembly_content += "}\n"

with open("pipeline_demo/ASSET_SnowWhite_Prelude.usda", "w") as f:
    f.write(assembly_content)

print("Split successful.")
