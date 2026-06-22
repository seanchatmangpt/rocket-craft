import re

def generate_joint(name, pos, rot, radius, height):
    greebles = ""
    # Add dense mechanical greebles around the joint
    for i in range(4):
        offset_y = -height/3 + (i * height/4)
        # We will add complex geometry: cubes, cylinders
        greebles += f"""
        def Cube "{name}Greeble_{i}"
        {{
            double3 xformOp:scale = (0.1, 0.05, 0.1)
            double3 xformOp:translate = ({pos[0] + radius}, {pos[1] + offset_y}, {pos[2]})
            color3f[] primvars:displayColor = [(0.7, 0.4, 0.1)]
            uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:scale"]
        }}
        def Cylinder "{name}Piston_{i}"
        {{
            double radius = 0.05
            double height = {height * 0.8}
            double3 xformOp:translate = ({pos[0]}, {pos[1]}, {pos[2] + radius + 0.05})
            double3 xformOp:rotateXYZ = (0, 0, 0)
            color3f[] primvars:displayColor = [(0.4, 0.4, 0.4)]
            uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:rotateXYZ"]
        }}
        def Sphere "{name}Rotator_{i}"
        {{
            double radius = 0.08
            double3 xformOp:translate = ({pos[0] - radius - 0.05}, {pos[1] + offset_y}, {pos[2]})
            color3f[] primvars:displayColor = [(0.8, 0.6, 0.2)]
            uniform token[] xformOpOrder = ["xformOp:translate"]
        }}"""

    return f"""
        def Cylinder "{name}"
        {{
            double radius = {radius}
            double height = {height}
            double3 xformOp:translate = ({pos[0]}, {pos[1]}, {pos[2]})
            double3 xformOp:rotateXYZ = ({rot[0]}, {rot[1]}, {rot[2]})
            color3f[] primvars:displayColor = [(0.2, 0.2, 0.2)]
            uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:rotateXYZ"]
        }}{greebles}
"""

with open("winter_protocol_prelude_mecha.usda", "r") as f:
    content = f.read()

# 1. Torso Spine Joint
torso_joint = generate_joint("SpineJoint", (0, -1.0, 0), (0,0,0), 0.5, 1.5)
content = content.replace('def Cube "Chest"', torso_joint + '        def Cube "Chest"')

# 2. Head Neck Joint
head_joint = generate_joint("NeckJoint", (0, -0.5, 0), (0,0,0), 0.3, 0.8)
content = content.replace('def Cube "Cranium"', head_joint + '        def Cube "Cranium"')

# 3. Arms and Legs: we'll use regex to inject right after 'def Xform "LeftArm" (...) {'
def inject_into_xform(xform_name, injection, text):
    pattern = r'(def Xform "' + xform_name + r'" \(\s*kind = "component"\s*\)\s*\{[^\n]*\n(?:[^\{]*\{.*?\}\n|[^d]*)*?)(\s*def )'
    # Wait, simple string replacement by finding the first primitive in that xform might be safer.
    return text

# A better way is to replace the first primitive in each component
components = {
    "LeftArm": ('def Cube "ShoulderArmor"', generate_joint("ShoulderJoint", (0.5, 0, 0), (0,0,90), 0.3, 0.8) + generate_joint("ElbowJoint", (0, -2.6, 0), (90,0,0), 0.25, 0.6)),
    "RightArm": ('def Cube "ShoulderArmor"', generate_joint("ShoulderJoint", (-0.5, 0, 0), (0,0,90), 0.3, 0.8) + generate_joint("ElbowJoint", (0, -2.6, 0), (90,0,0), 0.25, 0.6)),
    "LeftLeg": ('def Cylinder "Thigh"', generate_joint("HipJoint", (0, 0, 0), (0,0,90), 0.4, 0.8) + generate_joint("KneeJoint", (0, -3.1, 0), (90,0,0), 0.35, 0.8) + generate_joint("AnkleJoint", (0, -7.1, 0), (90,0,0), 0.3, 0.6)),
    "RightLeg": ('def Cylinder "Thigh"', generate_joint("HipJoint", (0, 0, 0), (0,0,90), 0.4, 0.8) + generate_joint("KneeJoint", (0, -3.1, 0), (90,0,0), 0.35, 0.8) + generate_joint("AnkleJoint", (0, -7.1, 0), (90,0,0), 0.3, 0.6)),
}

for comp_name, (prim_str, injection) in components.items():
    # Split the content by the component name
    parts = content.split(f'def Xform "{comp_name}"')
    if len(parts) == 2:
        # replace the FIRST occurrence of prim_str in parts[1]
        parts[1] = parts[1].replace(prim_str, injection + '        ' + prim_str, 1)
        content = f'def Xform "{comp_name}"'.join(parts)

with open("winter_protocol_prelude_mecha.usda", "w") as f:
    f.write(content)
