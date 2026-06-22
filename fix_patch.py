import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# Replace has_limb_left with has_arm_left etc.
content = content.replace('{% elif row.partLocalName == "shoulder_left" or row.partLocalName == "arm_left" or row.partLocalName == "leg_left" %}\n        {% set_global has_limb_left = true %}',
'''{% elif row.partLocalName == "shoulder_left" or row.partLocalName == "arm_left" %}
        {% set_global has_arm_left = true %}
    {% elif row.partLocalName == "leg_left" %}
        {% set_global has_leg_left = true %}''')

content = content.replace('{% elif row.partLocalName == "shoulder_right" or row.partLocalName == "arm_right" or row.partLocalName == "leg_right" %}\n        {% set_global has_limb_right = true %}',
'''{% elif row.partLocalName == "shoulder_right" or row.partLocalName == "arm_right" %}
        {% set_global has_arm_right = true %}
    {% elif row.partLocalName == "leg_right" %}
        {% set_global has_leg_right = true %}''')

content = content.replace('{% set has_limb_left = false %}\n{% set has_limb_right = false %}',
'''{% set has_arm_left = false %}
{% set has_arm_right = false %}
{% set has_leg_left = false %}
{% set has_leg_right = false %}''')

# Now replace the def Xform SM_Limb_Left with Arm and Leg
content = re.sub(r'\{% if has_limb_left %}.*?\{% endif %}',
'''{% if has_arm_left %}
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
{% endif %}''', content, flags=re.DOTALL)

content = re.sub(r'\{% if has_limb_right %}.*?\{% endif %}',
'''{% if has_arm_right %}
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
{% endif %}''', content, flags=re.DOTALL)


content = content.replace('{% elif has_limb_left %}\n    defaultPrim = "SM_Limb_Left"\n{% elif has_limb_right %}\n    defaultPrim = "SM_Limb_Right"',
'''{% elif has_arm_left %}
    defaultPrim = "SM_Arm_Left"
{% elif has_arm_right %}
    defaultPrim = "SM_Arm_Right"
{% elif has_leg_left %}
    defaultPrim = "SM_Leg_Left"
{% elif has_leg_right %}
    defaultPrim = "SM_Leg_Right"''')

content = content.replace('def Xform "Limb_Left" (\n        prepend references = @./SM_Limb_Left.usda@\n    )\n    {\n    }\n\n    def Xform "Limb_Right" (\n        prepend references = @./SM_Limb_Right.usda@\n    )\n    {\n    }',
'''def Xform "Arm_Left" (
        prepend references = @./SM_Arm_Left.usda@
    )
    {
    }

    def Xform "Arm_Right" (
        prepend references = @./SM_Arm_Right.usda@
    )
    {
    }

    def Xform "Leg_Left" (
        prepend references = @./SM_Leg_Left.usda@
    )
    {
    }

    def Xform "Leg_Right" (
        prepend references = @./SM_Leg_Right.usda@
    )
    {
    }''')

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
