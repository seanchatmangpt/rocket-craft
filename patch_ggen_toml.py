import os

toml_path = "/Users/sac/rocket-craft/ggen.toml"

if not os.path.exists(toml_path):
    print(f"Error: {toml_path} not found")
    exit(1)

with open(toml_path, "r") as f:
    content = f.read()

# Prefix replacement
target_prefixes = 'PREFIX mud: <https://rocket-craft.com/ontology/mud#>'
replacement_prefixes = 'PREFIX mud: <https://rocket-craft.com/ontology/mud#>\nPREFIX law: <https://rocket-craft.com/ontology/law#>'

content = content.replace(target_prefixes, replacement_prefixes)

# SELECT clause replacement
target_select = 'SELECT ?prim ?part ?type ?translateX ?translateY ?translateZ ?scaleX ?scaleY ?scaleZ ?rotateX ?rotateY ?rotateZ ?materialBinding ?partLocalName ?materialLocalName ?primLocalName'
replacement_select = 'SELECT ?prim ?part ?type ?translateX ?translateY ?translateZ ?scaleX ?scaleY ?scaleZ ?rotateX ?rotateY ?rotateZ ?materialBinding ?partLocalName ?materialLocalName ?primLocalName ?density ?matZoneLocalName ?scaleMin ?scaleMax ?edgeCount ?socket ?sweep ?armorBand'

content = content.replace(target_select, replacement_select)

# Primitives pattern replacement
target_where = """  ?prim rdf:type mud:GeometryPrimitive ;
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
  BIND(STRAFTER(STR(?prim), "#") AS ?primLocalName)"""

replacement_where = """  ?prim rdf:type mud:GeometryPrimitive ;
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
        mud:materialBinding ?materialBinding ;
        law:hasSubdivisionDensity ?density ;
        law:hasMaterialZoneBinding ?matZone ;
        law:hasBladeScaleMin ?scaleMin ;
        law:hasBladeScaleMax ?scaleMax ;
        law:hasEdgeCount ?edgeCount ;
        law:hasSocketAttachment ?socket ;
        law:hasCurvatureSweepClass ?sweep ;
        law:hasArmorDensityBand ?armorBand .
  BIND(STRAFTER(STR(?part), "#") AS ?partLocalName)
  BIND(STRAFTER(STR(?materialBinding), "#") AS ?materialLocalName)
  BIND(STRAFTER(STR(?prim), "#") AS ?primLocalName)
  BIND(STRAFTER(STR(?matZone), "#") AS ?matZoneLocalName)"""

# Handle single vs double quote strings and line ending variations
content = content.replace(target_where, replacement_where)
content = content.replace(target_where.replace("'", '"'), replacement_where.replace("'", '"'))

# Also handle single quoted inline strings or other variations
target_where_single = target_where.replace('"', "'")
replacement_where_single = replacement_where.replace('"', "'")
content = content.replace(target_where_single, replacement_where_single)

with open(toml_path, "w") as f:
    f.write(content)

print("Successfully updated ggen.toml!")
