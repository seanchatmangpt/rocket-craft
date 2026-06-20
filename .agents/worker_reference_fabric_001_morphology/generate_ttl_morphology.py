import os

output_path = "/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/graph/generator_parameters.ttl"

header = """@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix core: <https://ggen.io/ontology/core/> .
@prefix mud: <https://rocket-craft.com/ontology/mud#> .

mud:GeneratorParametersOntology a owl:Ontology ;
    rdfs:label "Generator Parameters Ontology" ;
    rdfs:comment "Defines materials, texture programs, and the geometry primitives for reference fabric mech assets." .

# ---------------------------------------------------------
# Materials
# ---------------------------------------------------------
mud:Material rdf:type owl:Class ;
    rdfs:subClassOf core:Entity ;
    rdfs:label "Material" .

mud:M_WhiteArmor rdf:type mud:Material ;
    mud:color "1.0 1.0 1.0" ;
    mud:roughness "0.4"^^xsd:float ;
    mud:emissive "0.0"^^xsd:float ;
    mud:metallic "0.0"^^xsd:float .

mud:M_CyanBlade rdf:type mud:Material ;
    mud:color "0.0 0.8 1.0" ;
    mud:roughness "0.15"^^xsd:float ;
    mud:emissive "0.8"^^xsd:float ;
    mud:metallic "0.9"^^xsd:float .

mud:M_DarkFrame rdf:type mud:Material ;
    mud:color "0.15 0.15 0.18" ;
    mud:roughness "0.6"^^xsd:float ;
    mud:emissive "0.0"^^xsd:float ;
    mud:metallic "0.5"^^xsd:float .

mud:M_GoldVisor rdf:type mud:Material ;
    mud:color "1.0 0.8 0.0" ;
    mud:roughness "0.1"^^xsd:float ;
    mud:emissive "0.5"^^xsd:float ;
    mud:metallic "0.9"^^xsd:float .

# ---------------------------------------------------------
# Texture Programs
# ---------------------------------------------------------
mud:TextureProgram rdf:type owl:Class ;
    rdfs:subClassOf core:Entity ;
    rdfs:label "Texture Program" .

mud:tex_prog_armor rdf:type mud:TextureProgram ;
    mud:targetMaterial mud:M_WhiteArmor ;
    mud:baseColorTex "T_WhiteArmor_BaseColor.png" ;
    mud:roughnessTex "T_WhiteArmor_Roughness.png" ;
    mud:normalTex "T_WhiteArmor_Normal.png" ;
    mud:resolution "2048"^^xsd:integer .

mud:tex_prog_blade rdf:type mud:TextureProgram ;
    mud:targetMaterial mud:M_CyanBlade ;
    mud:emissiveTex "T_CyanBlade_Emissive.png" ;
    mud:resolution "2048"^^xsd:integer .

# ---------------------------------------------------------
# Geometry Primitives
# ---------------------------------------------------------
mud:GeometryPrimitive rdf:type owl:Class ;
    rdfs:subClassOf core:Entity ;
    rdfs:label "Geometry Primitive" .
"""

primitives = []
count = 0

def add_prim(part, type_, tx, ty, tz, sx, sy, sz, rx, ry, rz, mat):
    global count
    count += 1
    prim_id = f"prim_{count:04d}"
    ttl = f"""
mud:{prim_id} rdf:type mud:GeometryPrimitive ;
    mud:belongsToPart mud:{part} ;
    mud:primitiveFamily "{type_}" ;
    mud:translateX "{tx:.4f}"^^xsd:float ;
    mud:translateY "{ty:.4f}"^^xsd:float ;
    mud:translateZ "{tz:.4f}"^^xsd:float ;
    mud:scaleX "{sx:.4f}"^^xsd:float ;
    mud:scaleY "{sy:.4f}"^^xsd:float ;
    mud:scaleZ "{sz:.4f}"^^xsd:float ;
    mud:rotateX "{rx:.4f}"^^xsd:float ;
    mud:rotateY "{ry:.4f}"^^xsd:float ;
    mud:rotateZ "{rz:.4f}"^^xsd:float ;
    mud:materialBinding mud:{mat} ."""
    primitives.append(ttl)

# --- 1. Torso Core (10 primitives) ---
# High angular armor shell hierarchy, torso/head/shoulder distinction
# Center Chest Plate
add_prim("torso_core", "tapered_box", 0.0, 2.0, 115.0, 14.0, 10.0, 12.0, 15.0, 0.0, 0.0, "M_WhiteArmor")
# Breast Plates left/right
add_prim("torso_core", "tapered_box", -6.0, 3.5, 120.0, 6.0, 4.0, 8.0, 15.0, 20.0, 10.0, "M_WhiteArmor")
add_prim("torso_core", "tapered_box", 6.0, 3.5, 120.0, 6.0, 4.0, 8.0, 15.0, -20.0, -10.0, "M_WhiteArmor")
# Upper Collar / Neck Guard
add_prim("torso_core", "tapered_box", 0.0, 4.0, 128.0, 10.0, 5.0, 4.0, -15.0, 0.0, 0.0, "M_WhiteArmor")
# Abdomen (lower torso frame)
add_prim("torso_core", "cylinder", 0.0, 0.0, 100.0, 8.0, 8.0, 10.0, 5.0, 0.0, 0.0, "M_DarkFrame")
# Lower Spine Segment
add_prim("torso_core", "tapered_box", 0.0, -3.0, 95.0, 6.0, 5.0, 10.0, -10.0, 0.0, 0.0, "M_DarkFrame")
# Back Booster Mount
add_prim("torso_core", "tapered_box", 0.0, -5.0, 112.0, 12.0, 6.0, 14.0, -20.0, 0.0, 0.0, "M_DarkFrame")
# Torso Side Flaps
add_prim("torso_core", "tapered_box", -8.0, 0.0, 105.0, 2.0, 8.0, 8.0, 0.0, 0.0, 15.0, "M_WhiteArmor")
add_prim("torso_core", "tapered_box", 8.0, 0.0, 105.0, 2.0, 8.0, 8.0, 0.0, 0.0, -15.0, "M_WhiteArmor")
# Torso Center Crest
add_prim("torso_core", "tapered_box", 0.0, 4.5, 118.0, 2.0, 4.0, 6.0, 30.0, 0.0, 0.0, "M_GoldVisor")

# --- 2. Head Unit (8 primitives) ---
for i in range(8):
    add_prim(
        part="head_unit",
        type_="tapered_box" if i < 6 else "cylinder",
        tx=0.0, ty=2.0, tz=135.0 + i * 1.5,
        sx=8.0, sy=8.0, sz=1.5,
        rx=0.0, ry=0.0, rz=0.0,
        mat="M_WhiteArmor" if i != 3 else "M_GoldVisor"
    )

# --- 3. V-Fin Left (4 primitives) ---
for i in range(4):
    add_prim(
        part="v_fin_left",
        type_="blade_prism" if i < 2 else "tapered_box",
        tx=-1.0 - i * 1.0, ty=6.0, tz=145.0 + i * 2.0,
        sx=1.0, sy=0.5, sz=3.0,
        rx=15.0, ry=30.0, rz=10.0,
        mat="M_GoldVisor" if i % 2 == 0 else "M_WhiteArmor"
    )

# --- 4. V-Fin Right (4 primitives) ---
for i in range(4):
    add_prim(
        part="v_fin_right",
        type_="blade_prism" if i < 2 else "tapered_box",
        tx=1.0 + i * 1.0, ty=6.0, tz=145.0 + i * 2.0,
        sx=1.0, sy=0.5, sz=3.0,
        rx=15.0, ry=-30.0, rz=-10.0,
        mat="M_GoldVisor" if i % 2 == 0 else "M_WhiteArmor"
    )

# --- 5. Wing Root Left (8 primitives) - Increased Density ---
for i in range(8):
    add_prim(
        part="wing_root_left",
        type_="tapered_box",
        tx=-15.0 - i * 1.0, ty=8.0, tz=120.0 + i * 1.0,
        sx=4.0, sy=4.0, sz=2.0,
        rx=0.0, ry=45.0, rz=20.0,
        mat="M_DarkFrame"
    )

# --- 6. Wing Root Right (8 primitives) - Increased Density ---
for i in range(8):
    add_prim(
        part="wing_root_right",
        type_="tapered_box",
        tx=15.0 + i * 1.0, ty=8.0, tz=120.0 + i * 1.0,
        sx=4.0, sy=4.0, sz=2.0,
        rx=0.0, ry=-45.0, rz=-20.0,
        mat="M_DarkFrame"
    )

# --- 7. Primary Wing Feathers Left (24 primitives - layered swept panels) ---
# Tapered curved plates with thickness, bevel, and overlap
for i in range(24):
    t = i / 23.0
    tx = -15.0 - t * 35.0
    ty = 6.0 - t * 4.0
    tz = 120.0 + t * 25.0 - (t * t) * 45.0
    
    sz = 12.0 + 16.0 * (1.0 - t)
    sx = 3.5 - t * 1.5
    sy = 0.5 - t * 0.25
    
    rx = 8.0
    ry = 35.0 + t * 40.0
    rz = 15.0 + t * 35.0
    
    mat = "M_WhiteArmor" if i % 4 != 0 else "M_CyanBlade"
    add_prim(
        part="primary_wing_feathers_left",
        type_="feather_panel",
        tx=tx, ty=ty, tz=tz,
        sx=sx, sy=sy, sz=sz,
        rx=rx, ry=ry, rz=rz,
        mat=mat
    )

# --- 8. Primary Wing Feathers Right (24 primitives - layered swept panels) ---
# Tapered curved plates with thickness, bevel, and overlap
for i in range(24):
    t = i / 23.0
    tx = 15.0 + t * 35.0
    ty = 6.0 - t * 4.0
    tz = 120.0 + t * 25.0 - (t * t) * 45.0
    
    sz = 12.0 + 16.0 * (1.0 - t)
    sx = 3.5 - t * 1.5
    sy = 0.5 - t * 0.25
    
    rx = 8.0
    ry = -35.0 - t * 40.0
    rz = -15.0 - t * 35.0
    
    mat = "M_WhiteArmor" if i % 4 != 0 else "M_CyanBlade"
    add_prim(
        part="primary_wing_feathers_right",
        type_="feather_panel",
        tx=tx, ty=ty, tz=tz,
        sx=sx, sy=sy, sz=sz,
        rx=rx, ry=ry, rz=rz,
        mat=mat
    )

# --- 9. Secondary Wing Feathers Left (8 primitives) ---
for i in range(8):
    add_prim(
        part="secondary_wing_feathers_left",
        type_="feather_panel",
        tx=-15.0 - i * 1.2, ty=6.0, tz=110.0 - i * 1.5,
        sx=1.5, sy=0.3, sz=10.0,
        rx=5.0, ry=80.0, rz=45.0,
        mat="M_WhiteArmor"
    )

# --- 10. Secondary Wing Feathers Right (8 primitives) ---
for i in range(8):
    add_prim(
        part="secondary_wing_feathers_right",
        type_="feather_panel",
        tx=15.0 + i * 1.2, ty=6.0, tz=110.0 - i * 1.5,
        sx=1.5, sy=0.3, sz=10.0,
        rx=5.0, ry=-80.0, rz=-45.0,
        mat="M_WhiteArmor"
    )

# --- 11. Blade Left (1 long cyan rod/blade) ---
add_prim(
    part="blade_left",
    type_="blade_prism",
    tx=-25.0, ty=0.0, tz=75.0,
    sx=2.0, sy=2.0, sz=40.0,
    rx=0.0, ry=0.0, rz=-15.0,
    mat="M_CyanBlade"
)

# --- 12. Blade Right (1 long cyan rod/blade) ---
add_prim(
    part="blade_right",
    type_="blade_prism",
    tx=25.0, ty=0.0, tz=75.0,
    sx=2.0, sy=2.0, sz=40.0,
    rx=0.0, ry=0.0, rz=15.0,
    mat="M_CyanBlade"
)

# --- 13. Backpack Core (8 primitives) ---
for i in range(8):
    add_prim(
        part="backpack_core",
        type_="tapered_box",
        tx=0.0, ty=-10.0, tz=105.0 + i * 2.0,
        sx=10.0, sy=6.0, sz=2.0,
        rx=-10.0, ry=0.0, rz=0.0,
        mat="M_DarkFrame"
    )

# --- 14. Thruster Cluster (8 primitives) ---
for i in range(8):
    add_prim(
        part="thruster_cluster",
        type_="cylinder",
        tx=-4.0 if i < 4 else 4.0, ty=-14.0, tz=95.0 + (i % 4) * 2.0,
        sx=3.0, sy=3.0, sz=2.0,
        rx=-30.0, ry=0.0, rz=0.0,
        mat="M_DarkFrame" if i % 2 == 0 else "M_GoldVisor"
    )

# --- 15. Shoulder Left (8 primitives) - Increased Density ---
for i in range(8):
    add_prim(
        part="shoulder_left",
        type_="tapered_box",
        tx=-15.0 - i * 0.4, ty=0.0 + (i % 2) * 0.5, tz=120.0 + (i % 3) * 0.5,
        sx=5.0, sy=7.0, sz=5.0,
        rx=0.0, ry=5.0 * (i % 2), rz=5.0 * (i % 3),
        mat="M_WhiteArmor"
    )

# --- 16. Shoulder Right (8 primitives) - Increased Density ---
for i in range(8):
    add_prim(
        part="shoulder_right",
        type_="tapered_box",
        tx=15.0 + i * 0.4, ty=0.0 + (i % 2) * 0.5, tz=120.0 + (i % 3) * 0.5,
        sx=5.0, sy=7.0, sz=5.0,
        rx=0.0, ry=-5.0 * (i % 2), rz=-5.0 * (i % 3),
        mat="M_WhiteArmor"
    )

# --- 17. Arm Left (12 primitives) - Increased Density on Forearms ---
for i in range(12):
    add_prim(
        part="arm_left",
        type_="tapered_box" if i % 2 == 0 else "cylinder",
        tx=-20.0, ty=0.0, tz=110.0 - i * 2.2,
        sx=3.0, sy=3.0, sz=2.2,
        rx=0.0, ry=0.0, rz=0.0,
        mat="M_DarkFrame" if i < 6 else "M_WhiteArmor"
    )

# --- 18. Arm Right (12 primitives) - Increased Density on Forearms ---
for i in range(12):
    add_prim(
        part="arm_right",
        type_="tapered_box" if i % 2 == 0 else "cylinder",
        tx=20.0, ty=0.0, tz=110.0 - i * 2.2,
        sx=3.0, sy=3.0, sz=2.2,
        rx=0.0, ry=0.0, rz=0.0,
        mat="M_DarkFrame" if i < 6 else "M_WhiteArmor"
    )

# --- 19. Leg Left (14 primitives) - Increased Density on Lower Assembly ---
for i in range(14):
    add_prim(
        part="leg_left",
        type_="tapered_box" if i % 2 == 0 else "cylinder",
        tx=-8.0, ty=-1.0, tz=85.0 - i * 3.8,
        sx=4.5 - i * 0.08, sy=4.5 - i * 0.08, sz=3.8,
        rx=5.0, ry=0.0, rz=0.0,
        mat="M_WhiteArmor" if i % 3 != 0 else "M_DarkFrame"
    )

# --- 20. Leg Right (14 primitives) - Increased Density on Lower Assembly ---
for i in range(14):
    add_prim(
        part="leg_right",
        type_="tapered_box" if i % 2 == 0 else "cylinder",
        tx=8.0, ty=-1.0, tz=85.0 - i * 3.8,
        sx=4.5 - i * 0.08, sy=4.5 - i * 0.08, sz=3.8,
        rx=5.0, ry=0.0, rz=0.0,
        mat="M_WhiteArmor" if i % 3 != 0 else "M_DarkFrame"
    )

print(f"Total primitives added: {count}")

# Ensure the output directory exists
os.makedirs(os.path.dirname(output_path), exist_ok=True)

with open(output_path, "w") as f:
    f.write(header)
    for p in primitives:
        f.write(p + "\n")
