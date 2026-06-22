import os

gen_file = "patch_geometry_generator.py"
with open(gen_file, "r") as f:
    content = f.read()

# Replace the beveled_plate block
old_beveled = """                def Mesh "{{ row.primLocalName }}_beveled_plate_{{ i }}"
                {

                    int[] faceVertexCounts = [4, 4, 4, 4, 4, 4]
                    int[] faceVertexIndices = [0,1,2,3, 4,5,6,7, 0,1,5,4, 1,2,6,5, 2,3,7,6, 3,0,4,7]
                    point3f[] points = [(-0.5,-0.5,-0.5), (0.5,-0.5,-0.5), (0.5,0.5,-0.5), (-0.5,0.5,-0.5), (-0.5,-0.5,0.5), (0.5,-0.5,0.5), (0.5,0.5,0.5), (-0.5,0.5,0.5), (0,0,0)]
                    uniform token subdivisionScheme = "none"
                    double3 xformOp:scale = (2.14, 0.5, {{ 1.0 - (i * 0.15) }})
                    double3 xformOp:translate = ({% if is_right %}{{ 0.9 + i * 0.045 }}{% else %}{{ -0.9 - i * 0.045 }}{% endif %}, {{ i * 0.05 }}, 2.0)
                    double3 xformOp:rotateXYZ = (0, 0, {% if is_right %}15.0{% else %}-15.0{% endif %})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:rotateXYZ", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
                }"""

new_beveled = """                def Mesh "{{ row.primLocalName }}_beveled_plate_{{ i }}"
                {

                    int[] faceVertexCounts = [4, 4, 4, 4, 4, 4]
                    int[] faceVertexIndices = [0,1,2,3, 4,5,6,7, 0,1,5,4, 1,2,6,5, 2,3,7,6, 3,0,4,7]
                    point3f[] points = [(-0.5,-0.5,-0.5), (0.5,-0.5,-0.5), (0.5,0.5,-0.5), (-0.5,0.5,-0.5), (-0.5,-0.5,0.5), (0.5,-0.5,0.5), (0.5,0.5,0.5), (-0.5,0.5,0.5), (0,0,0)]
                    uniform token subdivisionScheme = "none"
                    double3 xformOp:scale = (1.8, 0.67, {{ 1.0 - (i * 0.15) }})
                    double3 xformOp:translate = ({% if is_right %}{{ 0.9 + i * 0.045 }}{% else %}{{ -0.9 - i * 0.045 }}{% endif %}, {{ i * 0.0035 }}, 2.0)
                    double3 xformOp:rotateXYZ = (0, 0, {% if is_right %}4.5{% else %}-4.5{% endif %})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:rotateXYZ", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
                }"""

# Replace the blade_edge block
old_edge = """                def Mesh "{{ row.primLocalName }}_blade_edge_{{ i }}"
                {

                    int[] faceVertexCounts = [4, 4, 4, 4, 4, 4]
                    int[] faceVertexIndices = [0,1,2,3, 4,5,6,7, 0,1,5,4, 1,2,6,5, 2,3,7,6, 3,0,4,7]
                    point3f[] points = [(-0.5,-0.5,-0.5), (0.5,-0.5,-0.5), (0.5,0.5,-0.5), (-0.5,0.5,-0.5), (-0.5,-0.5,0.5), (0.5,-0.5,0.5), (0.5,0.5,0.5), (-0.5,0.5,0.5), (0,0,0)]
                    uniform token subdivisionScheme = "none"
                    color3f[] primvars:displayColor = [(0, 1, 1)]
                    double3 xformOp:scale = (2.14, 0.5, 0.5)
                    double3 xformOp:translate = ({% if is_right %}{{ 0.9 + i * 0.045 }}{% else %}{{ -0.9 - i * 0.045 }}{% endif %}, {{ i * 0.05 }}, 2.0)
                    double3 xformOp:rotateXYZ = (0, 0, {% if is_right %}15.0{% else %}-15.0{% endif %})
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
                    double3 xformOp:scale = (1.8, 0.67, 0.5)
                    double3 xformOp:translate = ({% if is_right %}{{ 0.9 + i * 0.045 }}{% else %}{{ -0.9 - i * 0.045 }}{% endif %}, {{ i * 0.0035 }}, 2.0)
                    double3 xformOp:rotateXYZ = (0, 0, {% if is_right %}4.5{% else %}-4.5{% endif %})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:rotateXYZ", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_CyanBlade>
                }"""

content = content.replace(old_beveled, new_beveled)
content = content.replace(old_edge, new_edge)

with open(gen_file, "w") as f:
    f.write(content)

print("Patched blade 6")
