import os
import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

content = content.replace('{% set has_limb_left = false %}', '{% set has_arm_left = false %}\n{% set has_leg_left = false %}')
content = content.replace('{% set has_limb_right = false %}', '{% set has_arm_right = false %}\n{% set has_leg_right = false %}')

content = content.replace(
    '{% elif row.partLocalName == "shoulder_left" or row.partLocalName == "arm_left" or row.partLocalName == "leg_left" %}\n        {% set_global has_limb_left = true %}',
    '{% elif row.partLocalName == "shoulder_left" or row.partLocalName == "arm_left" %}\n        {% set_global has_arm_left = true %}\n    {% elif row.partLocalName == "leg_left" %}\n        {% set_global has_leg_left = true %}'
)

content = content.replace(
    '{% elif row.partLocalName == "shoulder_right" or row.partLocalName == "arm_right" or row.partLocalName == "leg_right" %}\n        {% set_global has_limb_right = true %}',
    '{% elif row.partLocalName == "shoulder_right" or row.partLocalName == "arm_right" %}\n        {% set_global has_arm_right = true %}\n    {% elif row.partLocalName == "leg_right" %}\n        {% set_global has_leg_right = true %}'
)

content = content.replace(
    'SM_WingArray_Left', 'SM_Wing_Left'
)
content = content.replace(
    'SM_WingArray_Right', 'SM_Wing_Right'
)

content = content.replace(
    '{% elif has_limb_left %}\n    defaultPrim = "SM_Limb_Left"',
    '{% elif has_arm_left %}\n    defaultPrim = "SM_Arm_Left"\n{% elif has_leg_left %}\n    defaultPrim = "SM_Leg_Left"'
)

content = content.replace(
    '{% elif has_limb_right %}\n    defaultPrim = "SM_Limb_Right"',
    '{% elif has_arm_right %}\n    defaultPrim = "SM_Arm_Right"\n{% elif has_leg_right %}\n    defaultPrim = "SM_Leg_Right"'
)


limb_left_xform = """{% if has_limb_left %}
def Xform "SM_Limb_Left"
{
    custom string owner_part_id = "SM_Limb_Left"
    {% for row in results %}
        {% if row.partLocalName == "shoulder_left" or row.partLocalName == "arm_left" or row.partLocalName == "leg_left" %}
            {{ self::render_primitive(row=row, is_right=false) }}
        {% endif %}
    {% endfor %}
}
{% endif %}"""

new_arm_leg_left = """{% if has_arm_left %}
def Xform "SM_Arm_Left"
{
    custom string owner_part_id = "SM_Arm_Left"
    {% for row in results %}
        {% if row.partLocalName == "shoulder_left" or row.partLocalName == "arm_left" %}
            {{ self::render_primitive(row=row, is_right=false) }}
        {% endif %}
    {% endfor %}
}
{% endif %}

{% if has_leg_left %}
def Xform "SM_Leg_Left"
{
    custom string owner_part_id = "SM_Leg_Left"
    {% for row in results %}
        {% if row.partLocalName == "leg_left" %}
            {{ self::render_primitive(row=row, is_right=false) }}
        {% endif %}
    {% endfor %}
}
{% endif %}"""

content = content.replace(limb_left_xform, new_arm_leg_left)


limb_right_xform = """{% if has_limb_right %}
def Xform "SM_Limb_Right"
{
    custom string owner_part_id = "SM_Limb_Right"
    {% for row in results %}
        {% if row.partLocalName == "shoulder_right" or row.partLocalName == "arm_right" or row.partLocalName == "leg_right" %}
            {{ self::render_primitive(row=row, is_right=true) }}
        {% endif %}
    {% endfor %}
}
{% endif %}"""

new_arm_leg_right = """{% if has_arm_right %}
def Xform "SM_Arm_Right"
{
    custom string owner_part_id = "SM_Arm_Right"
    {% for row in results %}
        {% if row.partLocalName == "shoulder_right" or row.partLocalName == "arm_right" %}
            {{ self::render_primitive(row=row, is_right=true) }}
        {% endif %}
    {% endfor %}
}
{% endif %}

{% if has_leg_right %}
def Xform "SM_Leg_Right"
{
    custom string owner_part_id = "SM_Leg_Right"
    {% for row in results %}
        {% if row.partLocalName == "leg_right" %}
            {{ self::render_primitive(row=row, is_right=true) }}
        {% endif %}
    {% endfor %}
}
{% endif %}"""

content = content.replace(limb_right_xform, new_arm_leg_right)


limb_left_usda = """    def Xform "Limb_Left" (
        prepend references = @./SM_Limb_Left.usda@
    )
    {
    }"""

new_arm_leg_left_usda = """    def Xform "Arm_Left" (
        prepend references = @./SM_Arm_Left.usda@
    )
    {
    }

    def Xform "Leg_Left" (
        prepend references = @./SM_Leg_Left.usda@
    )
    {
    }"""

content = content.replace(limb_left_usda, new_arm_leg_left_usda)


limb_right_usda = """    def Xform "Limb_Right" (
        prepend references = @./SM_Limb_Right.usda@
    )
    {
    }"""

new_arm_leg_right_usda = """    def Xform "Arm_Right" (
        prepend references = @./SM_Arm_Right.usda@
    )
    {
    }

    def Xform "Leg_Right" (
        prepend references = @./SM_Leg_Right.usda@
    )
    {
    }"""

content = content.replace(limb_right_usda, new_arm_leg_right_usda)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)

print("Patched patch_geometry_generator.py")

# Now modify validate_armor.py
if os.path.exists("validate_armor.py"):
    with open("validate_armor.py", "r") as f:
        val_content = f.read()
    
    val_content = val_content.replace("'final_mech_asset/SM_Limb_Left.usda',", "'final_mech_asset/SM_Arm_Left.usda',\n        'final_mech_asset/SM_Leg_Left.usda',")
    val_content = val_content.replace("'final_mech_asset/SM_Limb_Right.usda',", "'final_mech_asset/SM_Arm_Right.usda',\n        'final_mech_asset/SM_Leg_Right.usda',")
    val_content = val_content.replace("'final_mech_asset/SM_WingArray_Left.usda',", "'final_mech_asset/SM_Wing_Left.usda',")
    val_content = val_content.replace("'final_mech_asset/SM_WingArray_Right.usda',", "'final_mech_asset/SM_Wing_Right.usda',")

    with open("validate_armor.py", "w") as f:
        f.write(val_content)
    print("Patched validate_armor.py")
