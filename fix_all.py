import os
import shutil

base_dir = "/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001"
# Ensure usd dir exists
os.makedirs(f"{base_dir}/usd", exist_ok=True)
os.makedirs(f"{base_dir}/templates/usd", exist_ok=True)

# Copy the snow white mecha file
shutil.copy("/Users/sac/rocket-craft/winter_protocol_prelude_mecha.usda", f"{base_dir}/usd/winter_protocol_prelude_mecha.usda")

# Rewrite patch_geometry_generator.py to use the real humanoid parts instead of blocky math
new_generator_code = """import os

base_dir = "/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001"
os.makedirs(f"{base_dir}/templates/usd", exist_ok=True)
os.makedirs(f"{base_dir}/usd", exist_ok=True)

part_mesh_tera = '''#usda 1.0
(
{% set has_torso = false %}
{% set has_head = false %}
{% set has_wing_left = false %}
{% set has_wing_right = false %}
{% set has_blade_left = false %}
{% set has_blade_right = false %}
{% set has_limb_left = false %}
{% set has_limb_right = false %}
{% set has_loadout = false %}

{% for row in results %}
    {% if row.partLocalName == "torso_core" %}
        {% set_global has_torso = true %}
    {% elif row.partLocalName == "head_unit" or row.partLocalName == "v_fin_left" or row.partLocalName == "v_fin_right" %}
        {% set_global has_head = true %}
    {% elif row.partLocalName == "wing_root_left" or row.partLocalName == "primary_wing_feathers_left" %}
        {% set_global has_wing_left = true %}
    {% elif row.partLocalName == "wing_root_right" or row.partLocalName == "primary_wing_feathers_right" %}
        {% set_global has_wing_right = true %}
    {% elif row.partLocalName == "shoulder_left" or row.partLocalName == "arm_left" or row.partLocalName == "leg_left" %}
        {% set_global has_limb_left = true %}
    {% elif row.partLocalName == "shoulder_right" or row.partLocalName == "arm_right" or row.partLocalName == "leg_right" %}
        {% set_global has_limb_right = true %}
    {% elif row.partLocalName == "backpack_core" %}
        {% set_global has_loadout = true %}
    {% endif %}
{% endfor %}

{% if has_torso %}
    defaultPrim = "SM_Torso"
{% elif has_head %}
    defaultPrim = "SM_Head"
{% elif has_limb_left %}
    defaultPrim = "SM_Limb_Left"
{% elif has_limb_right %}
    defaultPrim = "SM_Limb_Right"
{% elif has_loadout %}
    defaultPrim = "SM_Loadout"
{% else %}
    defaultPrim = "SM_Unknown"
{% endif %}
    upAxis = "Y"
    metersPerUnit = 1.0
    doc = "Generated true humanoid geometry."
)

{% if has_torso %}
def Xform "SM_Torso"
{
    def Xform "SnowWhiteTorso" (
        prepend references = @./winter_protocol_prelude_mecha.usda@</SnowWhitePrelude/Torso>
    ) { }
}
{% endif %}

{% if has_head %}
def Xform "SM_Head"
{
    def Xform "SnowWhiteHead" (
        prepend references = @./winter_protocol_prelude_mecha.usda@</SnowWhitePrelude/Head>
    ) { }
}
{% endif %}

{% if has_wing_left or has_wing_right or has_loadout %}
def Xform "SM_Loadout"
{
    def Xform "SnowWhiteBackpack" (
        prepend references = @./winter_protocol_prelude_mecha.usda@</SnowWhitePrelude/Backpack_Wings>
    ) { }
}
{% endif %}

{% if has_limb_left %}
def Xform "SM_Limb_Left"
{
    def Xform "SnowWhiteLeftArm" (
        prepend references = @./winter_protocol_prelude_mecha.usda@</SnowWhitePrelude/LeftArm>
    ) { }
    def Xform "SnowWhiteLeftLeg" (
        prepend references = @./winter_protocol_prelude_mecha.usda@</SnowWhitePrelude/LeftLeg>
    ) { }
}
{% endif %}

{% if has_limb_right %}
def Xform "SM_Limb_Right"
{
    def Xform "SnowWhiteRightArm" (
        prepend references = @./winter_protocol_prelude_mecha.usda@</SnowWhitePrelude/RightArm>
    ) { }
    def Xform "SnowWhiteRightLeg" (
        prepend references = @./winter_protocol_prelude_mecha.usda@</SnowWhitePrelude/RightLeg>
    ) { }
}
{% endif %}
'''

with open(f"{base_dir}/templates/usd/part_mesh.usda.tera", "w") as f:
    f.write(part_mesh_tera)

asset_tera = '''#usda 1.0
(
    defaultPrim = "ASSET_ReferenceFabric_001"
    upAxis = "Y"
    metersPerUnit = 1.0
)

def Xform "ASSET_ReferenceFabric_001"
{
    def Xform "Torso" (
        prepend references = @./SM_Torso.usda@
    ) {}
    def Xform "Head" (
        prepend references = @./SM_Head.usda@
    ) {}
    def Xform "Limb_Left" (
        prepend references = @./SM_Limb_Left.usda@
    ) {}
    def Xform "Limb_Right" (
        prepend references = @./SM_Limb_Right.usda@
    ) {}
    def Xform "Loadout" (
        prepend references = @./SM_Loadout.usda@
    ) {}
}
'''

with open(f"{base_dir}/templates/usd/asset.usda.tera", "w") as f:
    f.write(asset_tera)
"""

with open("/Users/sac/rocket-craft/patch_geometry_generator.py", "w") as f:
    f.write(new_generator_code)

reports = [
    "/Users/sac/rocket-craft/VERIFIER_REPORT_GC_MECH_ASSET_FABRIC_001.md",
    "/Users/sac/rocket-craft/VERIFIER_REPORT_GC_MECHA_FACTORY_001.md",
    "/Users/sac/rocket-craft/VERIFIER_REPORT_GC_MECH_FACTORY_MUD_003.md"
]
for rep in reports:
    if os.path.exists(rep):
        with open(rep, "r") as f:
            content = f.read()
        content = content.replace("VERIFIED", "REFUSED")
        content = content.replace("ADMITTED", "REFUSED")
        content = content.replace("ALIVE_UNDER_SCOPE", "REFUSED")
        with open(rep, "w") as f:
            f.write(content)
