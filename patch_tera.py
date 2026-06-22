import os

gen_file = "generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera"
with open(gen_file, "r") as f:
    content = f.read()

# Replace the blade_edge block
old_edge = """                def Mesh "{{ row.primLocalName }}_blade_edge_{{ i }}"
                {

                    int[] faceVertexCounts = [4, 4, 4, 4, 4, 4]
                    int[] faceVertexIndices = [0,1,2,3, 4,5,6,7, 0,1,5,4, 1,2,6,5, 2,3,7,6, 3,0,4,7]
                    point3f[] points = [(-0.5,-0.5,-0.5), (0.5,-0.5,-0.5), (0.5,0.5,-0.5), (-0.5,0.5,-0.5), (-0.5,-0.5,0.5), (0.5,-0.5,0.5), (0.5,0.5,0.5), (-0.5,0.5,0.5), (0,0,0)]
                    uniform token subdivisionScheme = "none"
                    color3f[] primvars:displayColor = [(0, 1, 1)]
                    double3 xformOp:scale = (1.8, 0.68, 0.5)
                    double3 xformOp:translate = ({% if is_right %}{{ 0.9 + i * 0.045 }}{% else %}{{ -0.9 - i * 0.045 }}{% endif %}, {{ i * 0.0035 }}, 2.0)
                    double3 xformOp:rotateXYZ = (0, 0, {% if is_right %}4.5{% else %}-4.5{% endif %})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:rotateXYZ", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_CyanBlade>
                }"""

new_edge = """                def Mesh "{{ row.primLocalName }}_blade_edge_{{ i }}"
                {

                    int[] faceVertexCounts = [4, 4, 4, 4, 4, 4]
                    int[] faceVertexIndices = [0,1,2,3, 4,5,6,7, 0,1,5,4, 1,2,6,5, 2,3,7,6, 3,0,4,7]
                    point3f[] points = [(-0.5,-0.5,-0.5), (0.5,-0.5,-0.5), (0.5,0.5,-0.5), (-0.5,0.5,-0.5), (-0.5,-0.5,0.5), (0.5,-0.5,0.5), (0.5,0.5,0.5), (-0.5,0.5,0.5), (0,0,0)]
                    uniform token subdivisionScheme = "none"
                    color3f[] primvars:displayColor = [(0, 1, 1)]
                    double3 xformOp:scale = (1.8, 0.1, 0.5)
                    double3 xformOp:translate = ({% if is_right %}{{ 0.9 + i * 0.045 }}{% else %}{{ -0.9 - i * 0.045 }}{% endif %}, {{ i * 0.0035 }}, 2.0)
                    double3 xformOp:rotateXYZ = (0, 0, {% if is_right %}4.5{% else %}-4.5{% endif %})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:rotateXYZ", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_CyanBlade>
                }"""

if old_edge in content:
    content = content.replace(old_edge, new_edge)
    with open(gen_file, "w") as f:
        f.write(content)
    print("Patched tera template successfully.")
else:
    print("Could not find old_edge in tera template. Printing the file around blade_edge:")
    os.system("grep -B 5 -A 10 blade_edge " + gen_file)
