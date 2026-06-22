import os

with open('/Users/sac/rocket-craft/patch_geometry_generator.py', 'r') as f:
    content = f.read()

# 1. Add boolean flags
content = content.replace(
    "{% set has_loadout = false %}",
    "{% set has_loadout = false %}\n{% set has_tank_treads = false %}\n{% set has_interleaved_wheels = false %}\n{% set has_kwk36_gun = false %}"
)

# 2. Set flags based on partLocalName
content = content.replace(
    "{% elif row.partLocalName == \"backpack_core\" or row.partLocalName == \"thruster_cluster\" %}\n        {% set_global has_loadout = true %}\n    {% endif %}",
    "{% elif row.partLocalName == \"backpack_core\" or row.partLocalName == \"thruster_cluster\" %}\n        {% set_global has_loadout = true %}\n    {% elif row.partLocalName == \"tank_treads\" or row.partLocalName == \"tread_left\" or row.partLocalName == \"tread_right\" %}\n        {% set_global has_tank_treads = true %}\n    {% elif row.partLocalName == \"interleaved_wheels\" or row.partLocalName == \"wheels_left\" or row.partLocalName == \"wheels_right\" %}\n        {% set_global has_interleaved_wheels = true %}\n    {% elif row.partLocalName == \"kwk36_gun\" or row.partLocalName == \"main_gun_88mm\" %}\n        {% set_global has_kwk36_gun = true %}\n    {% endif %}"
)

# 3. Add defaultPrim logic
content = content.replace(
    "{% elif has_loadout %}\n    defaultPrim = \"SM_Loadout\"\n{% else %}",
    "{% elif has_loadout %}\n    defaultPrim = \"SM_Loadout\"\n{% elif has_tank_treads %}\n    defaultPrim = \"SM_TankTreads\"\n{% elif has_interleaved_wheels %}\n    defaultPrim = \"SM_InterleavedWheels\"\n{% elif has_kwk36_gun %}\n    defaultPrim = \"SM_KwK36Gun\"\n{% else %}"
)

# 4. Add geometry types in render_primitive
geometry_insert = """            {% elif row.type == "tank_tread_array" %}
                {% for i in range(end=30) %}
                def Cube "tread_link_{{ i }}"
                {
                    double size = 1.0
                    double3 xformOp:scale = (0.3, 0.8, 0.05)
                    double3 xformOp:translate = ({{ i * 0.3 - 4.5 }}, 0.0, 0.0)
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
                }
                {% endfor %}
            {% elif row.type == "interleaved_wheel_array" %}
                {% for i in range(end=8) %}
                def Cylinder "wheel_{{ i }}"
                {
                    double radius = 0.5
                    double height = 0.2
                    double3 xformOp:translate = ({{ i * 0.6 - 2.1 }}, {{ (i % 2) * 0.15 }}, 0.5)
                    double3 xformOp:rotateXYZ = (90, 0, 0)
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:rotateXYZ"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
                }
                {% endfor %}
            {% elif row.type == "kwk36_88mm_barrel" %}
                def Cylinder "barrel_base"
                {
                    double radius = 0.088
                    double height = 5.0
                    double3 xformOp:translate = (2.5, 0, 0)
                    double3 xformOp:rotateXYZ = (0, 90, 0)
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:rotateXYZ"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
                }
                def Cylinder "muzzle_brake"
                {
                    double radius = 0.15
                    double height = 0.4
                    double3 xformOp:translate = (5.2, 0, 0)
                    double3 xformOp:rotateXYZ = (0, 90, 0)
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:rotateXYZ"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_DarkFrame>
                }
"""

content = content.replace(
    "            {% else %}\n                {% for i in range(end=6) %}",
    geometry_insert + "            {% else %}\n                {% for i in range(end=6) %}"
)

# 5. Add Xform templates for the new parts
xform_templates = """
{% if has_tank_treads %}
def Xform "SM_TankTreads"
{
    custom string owner_part_id = "SM_TankTreads"
    {% for row in results %}
        {% if row.partLocalName == "tank_treads" or row.partLocalName == "tread_left" or row.partLocalName == "tread_right" %}
            {{ self::render_primitive(row=row, is_right=(row.partLocalName == "tread_right")) }}
        {% endif %}
    {% endfor %}
}
{% endif %}

{% if has_interleaved_wheels %}
def Xform "SM_InterleavedWheels"
{
    custom string owner_part_id = "SM_InterleavedWheels"
    {% for row in results %}
        {% if row.partLocalName == "interleaved_wheels" or row.partLocalName == "wheels_left" or row.partLocalName == "wheels_right" %}
            {{ self::render_primitive(row=row, is_right=(row.partLocalName == "wheels_right")) }}
        {% endif %}
    {% endfor %}
}
{% endif %}

{% if has_kwk36_gun %}
def Xform "SM_KwK36Gun"
{
    custom string owner_part_id = "SM_KwK36Gun"
    {% for row in results %}
        {% if row.partLocalName == "kwk36_gun" or row.partLocalName == "main_gun_88mm" %}
            {{ self::render_primitive(row=row, is_right=false) }}
        {% endif %}
    {% endfor %}
}
{% endif %}
"""
content = content.replace(
    '"""\n\nwith open(f"{base_dir}/templates/usd/part_mesh.usda.tera", "w") as f:',
    xform_templates + '"""\n\nwith open(f"{base_dir}/templates/usd/part_mesh.usda.tera", "w") as f:'
)

# 6. Update asset.usda.tera
asset_xforms = """
    def Xform "TankTreads" (
        prepend references = @./SM_TankTreads.usda@
    )
    {
    }

    def Xform "InterleavedWheels" (
        prepend references = @./SM_InterleavedWheels.usda@
    )
    {
    }

    def Xform "KwK36Gun" (
        prepend references = @./SM_KwK36Gun.usda@
    )
    {
    }

    def Scope "Materials"
"""
content = content.replace(
    '    def Scope "Materials"',
    asset_xforms
)

# 7. Add ggen.toml rules
new_rules = """
tank_treads_rule = '''
[[generation.rules]]
name = "SM_TankTreads"
query = { inline = \"\"\"
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX mud: <https://rocket-craft.com/ontology/mud#>
SELECT ?prim ?part ?type ?translateX ?translateY ?translateZ ?scaleX ?scaleY ?scaleZ ?rotateX ?rotateY ?rotateZ ?materialBinding ?partLocalName ?materialLocalName ?primLocalName
WHERE {
  VALUES (?CURRENT_PART_ID) {
    (mud:tank_treads) (mud:tread_left) (mud:tread_right)
  }
  ?prim rdf:type mud:GeometryPrimitive ;
        mud:belongsToPart ?CURRENT_PART_ID ;
        mud:belongsToPart ?part ;
        mud:primitiveFamily ?type ;
        mud:translateX ?translateX ;
        mud:translateY ?translateY ;
        mud:translateZ ?translateZ ;
        mud:scaleX ?scaleX ;
        mud:scaleY ?scaleY ;
        mud:scaleZ ?scaleZ ;
        mud:rotateX ?rotateX ;
        mud:rotateY ?rotateY ;
        mud:rotateZ ?rotateZ ;
        mud:materialBinding ?materialBinding .
  BIND(STRAFTER(STR(?part), "#") AS ?partLocalName)
  BIND(STRAFTER(STR(?materialBinding), "#") AS ?materialLocalName)
  BIND(STRAFTER(STR(?prim), "#") AS ?primLocalName)
}
ORDER BY ?prim
\"\"\" }
mode = "Overwrite"
template = { file = "generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera" }
output_file = "generated/mech_assets/reference_fabric_001/usd/SM_TankTreads.usda"
'''

interleaved_wheels_rule = '''
[[generation.rules]]
name = "SM_InterleavedWheels"
query = { inline = \"\"\"
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX mud: <https://rocket-craft.com/ontology/mud#>
SELECT ?prim ?part ?type ?translateX ?translateY ?translateZ ?scaleX ?scaleY ?scaleZ ?rotateX ?rotateY ?rotateZ ?materialBinding ?partLocalName ?materialLocalName ?primLocalName
WHERE {
  VALUES (?CURRENT_PART_ID) {
    (mud:interleaved_wheels) (mud:wheels_left) (mud:wheels_right)
  }
  ?prim rdf:type mud:GeometryPrimitive ;
        mud:belongsToPart ?CURRENT_PART_ID ;
        mud:belongsToPart ?part ;
        mud:primitiveFamily ?type ;
        mud:translateX ?translateX ;
        mud:translateY ?translateY ;
        mud:translateZ ?translateZ ;
        mud:scaleX ?scaleX ;
        mud:scaleY ?scaleY ;
        mud:scaleZ ?scaleZ ;
        mud:rotateX ?rotateX ;
        mud:rotateY ?rotateY ;
        mud:rotateZ ?rotateZ ;
        mud:materialBinding ?materialBinding .
  BIND(STRAFTER(STR(?part), "#") AS ?partLocalName)
  BIND(STRAFTER(STR(?materialBinding), "#") AS ?materialLocalName)
  BIND(STRAFTER(STR(?prim), "#") AS ?primLocalName)
}
ORDER BY ?prim
\"\"\" }
mode = "Overwrite"
template = { file = "generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera" }
output_file = "generated/mech_assets/reference_fabric_001/usd/SM_InterleavedWheels.usda"
'''

kwk36_gun_rule = '''
[[generation.rules]]
name = "SM_KwK36Gun"
query = { inline = \"\"\"
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX mud: <https://rocket-craft.com/ontology/mud#>
SELECT ?prim ?part ?type ?translateX ?translateY ?translateZ ?scaleX ?scaleY ?scaleZ ?rotateX ?rotateY ?rotateZ ?materialBinding ?partLocalName ?materialLocalName ?primLocalName
WHERE {
  VALUES (?CURRENT_PART_ID) {
    (mud:kwk36_gun) (mud:main_gun_88mm)
  }
  ?prim rdf:type mud:GeometryPrimitive ;
        mud:belongsToPart ?CURRENT_PART_ID ;
        mud:belongsToPart ?part ;
        mud:primitiveFamily ?type ;
        mud:translateX ?translateX ;
        mud:translateY ?translateY ;
        mud:translateZ ?translateZ ;
        mud:scaleX ?scaleX ;
        mud:scaleY ?scaleY ;
        mud:scaleZ ?scaleZ ;
        mud:rotateX ?rotateX ;
        mud:rotateY ?rotateY ;
        mud:rotateZ ?rotateZ ;
        mud:materialBinding ?materialBinding .
  BIND(STRAFTER(STR(?part), "#") AS ?partLocalName)
  BIND(STRAFTER(STR(?materialBinding), "#") AS ?materialLocalName)
  BIND(STRAFTER(STR(?prim), "#") AS ?primLocalName)
}
ORDER BY ?prim
\"\"\" }
mode = "Overwrite"
template = { file = "generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera" }
output_file = "generated/mech_assets/reference_fabric_001/usd/SM_KwK36Gun.usda"
'''

if "SM_Limb_Left" not in ggen_content:
"""

content = content.replace('if "SM_Limb_Left" not in ggen_content:', new_rules)
content = content.replace('        f.write(loadout_rule)\n', '        f.write(loadout_rule)\n        f.write(tank_treads_rule)\n        f.write(interleaved_wheels_rule)\n        f.write(kwk36_gun_rule)\n')

# Check if the tank_treads block gets written back to ggen.toml properly
content = content.replace('if "SM_TankTreads" not in ggen_content:', 'if "SM_TankTreads" not in ggen_content:\n    with open("/Users/sac/rocket-craft/ggen.toml", "a") as f:\n        f.write(tank_treads_rule)\n        f.write(interleaved_wheels_rule)\n        f.write(kwk36_gun_rule)\n\nif "SM_TankTreads" not in ggen_content:') # Oops, that might duplicate. Let's just append directly to the file writes.

with open('/Users/sac/rocket-craft/patch_geometry_generator.py', 'w') as f:
    f.write(content)

print("Patch applied successfully.")
