import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# Fix the outdated part names in asset_tera
content = content.replace('SM_WingArray_Left.usda', 'SM_Wing_Left.usda')
content = content.replace('SM_WingArray_Right.usda', 'SM_Wing_Right.usda')
content = content.replace('SM_Limb_Left.usda', 'SM_Arm_Left.usda')
content = content.replace('SM_Limb_Right.usda', 'SM_Arm_Right.usda')

# We need to add Legs because they were split from Limbs!
if 'def Xform "Leg_Left"' not in content:
    legs = """
    def Xform "Leg_Left" (
        prepend references = @./SM_Leg_Left.usda@
    )
    {
    }

    def Xform "Leg_Right" (
        prepend references = @./SM_Leg_Right.usda@
    )
    {
    }
"""
    content = content.replace('    def Xform "Loadout"', legs + '    def Xform "Loadout"')

# Now add translations to aggressively compose the parts so their silhouettes overlap.
# Let's pull Left parts by +0.8 in X, and Right parts by -0.8 in X.
# Wait, let's just make it a python replacement.
def add_translate(match):
    name = match.group(1)
    usda = match.group(2)
    # determine offset
    tx, ty, tz = 0.0, 0.0, 0.0
    if "Left" in name:
        tx = 0.8
    elif "Right" in name:
        tx = -0.8
    
    if "Wing" in name:
        tx = 0.5 if "Left" in name else -0.5
        ty = -0.2
    
    if "Leg" in name:
        tx = 0.5 if "Left" in name else -0.5
        ty = 0.5
        
    if "Arm" in name:
        tx = 0.8 if "Left" in name else -0.8
        ty = 0.2
        
    if "Blade" in name:
        tx = 0.4 if "Left" in name else -0.4
        
    if tx == 0 and ty == 0 and tz == 0:
        return match.group(0)
        
    return f"""    def Xform "{name}" (
        prepend references = @{usda}@
    )
    {{
        double3 xformOp:translate = ({tx}, {ty}, {tz})
        uniform token[] xformOpOrder = ["xformOp:translate"]
    }}"""

# Find all Xform blocks in asset_tera
content = re.sub(r'    def Xform "([^"]+)" \(\n        prepend references = @([^@]+)@\n    \)\n    \{\n    \}', add_translate, content)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)

