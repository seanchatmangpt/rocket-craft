import os
import re

ttl_path = "/Users/sac/rocket-craft/ontology/source_law/104_reference_fabric.ttl"

if not os.path.exists(ttl_path):
    print(f"Error: {ttl_path} not found")
    exit(1)

with open(ttl_path, "r") as f:
    content = f.read()

# Add prefix to top if not present
if "@prefix law:" not in content:
    content = "@prefix law: <https://rocket-craft.com/ontology/law#> .\n" + content

blocks = content.split("\n\n")
new_blocks = []

for block in blocks:
    if "rdf:type mud:GeometryPrimitive" in block:
        # Find primitive family
        fam_match = re.search(r'mud:primitiveFamily\s+"([^"]+)"', block)
        family = fam_match.group(1) if fam_match else "unknown"
        
        # Find material binding
        mat_match = re.search(r'mud:materialBinding\s+(mud:\w+)', block)
        mat = mat_match.group(1) if mat_match else "mud:M_WhiteArmor"
        
        # Choose density
        if family == "socket":
            density = 1
            edge_count = 0
            socket = "mud:Socket_None" # We can customize this if it attaches to something
            sweep = "law:LinearSweep"
            armor_band = "law:LowDensityArmor"
        elif family in ["tapered_box", "angular_armor_shell"]:
            density = 8
            edge_count = 4
            socket = "mud:Socket_None"
            sweep = "law:LinearSweep"
            armor_band = "law:MediumDensityArmor"
        elif family in ["feather_panel", "layered_swept_feather_panel"]:
            density = 12
            edge_count = 1
            socket = "mud:Socket_None"
            sweep = "law:QuadraticSweep"
            armor_band = "law:LowDensityArmor"
        elif family in ["blade_prism", "blade"]:
            density = 5
            edge_count = 2
            socket = "mud:Socket_None"
            sweep = "law:LinearSweep"
            armor_band = "law:LowDensityArmor"
        elif family in ["chest_armor", "abdomen"]:
            density = 4
            edge_count = 4
            socket = "mud:Socket_None"
            sweep = "law:LinearSweep"
            armor_band = "law:HighDensityArmor"
        elif family in ["waist", "helmet_armor", "pauldron", "shield_plate"]:
            density = 3
            edge_count = 4
            socket = "mud:Socket_None"
            sweep = "law:LinearSweep"
            armor_band = "law:HighDensityArmor"
        elif family in ["core", "hard_surface_shell", "arm", "leg"]:
            density = 32
            edge_count = 8
            socket = "mud:Socket_None"
            sweep = "law:LinearSweep"
            armor_band = "law:HighDensityArmor"
        else:
            density = 6
            edge_count = 4
            socket = "mud:Socket_None"
            sweep = "law:LinearSweep"
            armor_band = "law:MediumDensityArmor"
            
        scale_min = 0.0100
        scale_max = 50.0000
        
        # Clean up block end - replace trailing dot with semicolon and new properties
        block_clean = block.strip()
        if block_clean.endswith("."):
            block_clean = block_clean[:-1].strip()
            
        extra = f""";
    law:hasSubdivisionDensity "{density}"^^xsd:integer ;
    law:hasMaterialZoneBinding {mat} ;
    law:hasBladeScaleMin "{scale_min:.4f}"^^xsd:decimal ;
    law:hasBladeScaleMax "{scale_max:.4f}"^^xsd:decimal ;
    law:hasEdgeCount "{edge_count}"^^xsd:integer ;
    law:hasSocketAttachment {socket} ;
    law:hasCurvatureSweepClass "{sweep}" ;
    law:hasArmorDensityBand "{armor_band}" ."""
        
        block = block_clean + extra
        
    new_blocks.append(block)

final_content = "\n\n".join(new_blocks)

with open(ttl_path, "w") as f:
    f.write(final_content)

print("Successfully patched 104_reference_fabric.ttl!")
