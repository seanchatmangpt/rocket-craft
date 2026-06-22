import re

with open("ggen.toml", "r") as f:
    content = f.read()

content = content.replace('name = "SM_WingArray_Left"', 'name = "SM_Wing_Left"')
content = content.replace('output_file = "generated/mech_assets/reference_fabric_001/usd/SM_WingArray_Left.usda"', 'output_file = "generated/mech_assets/reference_fabric_001/usd/SM_Wing_Left.usda"')

content = content.replace('name = "SM_WingArray_Right"', 'name = "SM_Wing_Right"')
content = content.replace('output_file = "generated/mech_assets/reference_fabric_001/usd/SM_WingArray_Right.usda"', 'output_file = "generated/mech_assets/reference_fabric_001/usd/SM_Wing_Right.usda"')

# Remove SM_Limb_Left, SM_Limb_Right, SM_Loadout
content = re.sub(r'\[\[generation\.rules\]\]\nname = "SM_Limb_Left".*', '', content, flags=re.DOTALL)

# Now append the new rules for Arm, Leg, Loadout
new_rules = """
[[generation.rules]]
name = "SM_Arm_Left"
query = { inline = '''
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX mud: <https://rocket-craft.com/ontology/mud#>
SELECT ?prim ?part ?type ?translateX ?translateY ?translateZ ?scaleX ?scaleY ?scaleZ ?rotateX ?rotateY ?rotateZ ?materialBinding ?partLocalName ?materialLocalName ?primLocalName
WHERE {
  VALUES (?CURRENT_PART_ID) {
    (mud:shoulder_left) (mud:arm_left)
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
  FILTER (?part = ?CURRENT_PART_ID || (?type = "socket" && ?part != ?CURRENT_PART_ID))
}
ORDER BY ?prim
''' }
mode = "Overwrite"
template = { file = "generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera" }
output_file = "generated/mech_assets/reference_fabric_001/usd/SM_Arm_Left.usda"

[[generation.rules]]
name = "SM_Leg_Left"
query = { inline = '''
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX mud: <https://rocket-craft.com/ontology/mud#>
SELECT ?prim ?part ?type ?translateX ?translateY ?translateZ ?scaleX ?scaleY ?scaleZ ?rotateX ?rotateY ?rotateZ ?materialBinding ?partLocalName ?materialLocalName ?primLocalName
WHERE {
  VALUES (?CURRENT_PART_ID) {
    (mud:leg_left)
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
  FILTER (?part = ?CURRENT_PART_ID || (?type = "socket" && ?part != ?CURRENT_PART_ID))
}
ORDER BY ?prim
''' }
mode = "Overwrite"
template = { file = "generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera" }
output_file = "generated/mech_assets/reference_fabric_001/usd/SM_Leg_Left.usda"

[[generation.rules]]
name = "SM_Arm_Right"
query = { inline = '''
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX mud: <https://rocket-craft.com/ontology/mud#>
SELECT ?prim ?part ?type ?translateX ?translateY ?translateZ ?scaleX ?scaleY ?scaleZ ?rotateX ?rotateY ?rotateZ ?materialBinding ?partLocalName ?materialLocalName ?primLocalName
WHERE {
  VALUES (?CURRENT_PART_ID) {
    (mud:shoulder_right) (mud:arm_right)
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
  FILTER (?part = ?CURRENT_PART_ID || (?type = "socket" && ?part != ?CURRENT_PART_ID))
}
ORDER BY ?prim
''' }
mode = "Overwrite"
template = { file = "generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera" }
output_file = "generated/mech_assets/reference_fabric_001/usd/SM_Arm_Right.usda"

[[generation.rules]]
name = "SM_Leg_Right"
query = { inline = '''
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX mud: <https://rocket-craft.com/ontology/mud#>
SELECT ?prim ?part ?type ?translateX ?translateY ?translateZ ?scaleX ?scaleY ?scaleZ ?rotateX ?rotateY ?rotateZ ?materialBinding ?partLocalName ?materialLocalName ?primLocalName
WHERE {
  VALUES (?CURRENT_PART_ID) {
    (mud:leg_right)
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
  FILTER (?part = ?CURRENT_PART_ID || (?type = "socket" && ?part != ?CURRENT_PART_ID))
}
ORDER BY ?prim
''' }
mode = "Overwrite"
template = { file = "generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera" }
output_file = "generated/mech_assets/reference_fabric_001/usd/SM_Leg_Right.usda"

[[generation.rules]]
name = "SM_Loadout"
query = { inline = '''
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX mud: <https://rocket-craft.com/ontology/mud#>
SELECT ?prim ?part ?type ?translateX ?translateY ?translateZ ?scaleX ?scaleY ?scaleZ ?rotateX ?rotateY ?rotateZ ?materialBinding ?partLocalName ?materialLocalName ?primLocalName
WHERE {
  VALUES (?CURRENT_PART_ID) {
    (mud:backpack_core) (mud:thruster_cluster)
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
  FILTER (?part = ?CURRENT_PART_ID || (?type = "socket" && ?part != ?CURRENT_PART_ID))
}
ORDER BY ?prim
''' }
mode = "Overwrite"
template = { file = "generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera" }
output_file = "generated/mech_assets/reference_fabric_001/usd/SM_Loadout.usda"
"""

with open("ggen.toml", "w") as f:
    f.write(content + new_rules)
