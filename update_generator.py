import re

gen_file = "patch_geometry_generator.py"
with open(gen_file, "r") as f:
    content = f.read()

# 1. Fix blade_edge_i scale and translate
old_blade = """                def Mesh "{{ row.primLocalName }}_blade_edge_{{ i }}"
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

new_blade = """                def Mesh "{{ row.primLocalName }}_blade_edge_{{ i }}"
                {

                    int[] faceVertexCounts = [4, 4, 4, 4, 4, 4]
                    int[] faceVertexIndices = [0,1,2,3, 4,5,6,7, 0,1,5,4, 1,2,6,5, 2,3,7,6, 3,0,4,7]
                    point3f[] points = [(-0.5,-0.5,-0.5), (0.5,-0.5,-0.5), (0.5,0.5,-0.5), (-0.5,0.5,-0.5), (-0.5,-0.5,0.5), (0.5,-0.5,0.5), (0.5,0.5,0.5), (-0.5,0.5,0.5), (0,0,0)]
                    uniform token subdivisionScheme = "none"
                    color3f[] primvars:displayColor = [(0, 1, 1)]
                    double3 xformOp:scale = (2.45, 0.1, 0.5)
                    double3 xformOp:translate = ({% if is_right %}0.9{% else %}-0.9{% endif %}, 0.0, 2.0)
                    double3 xformOp:rotateXYZ = (0, 0, {% if is_right %}4.5{% else %}-4.5{% endif %})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:rotateXYZ", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_CyanBlade>
                }"""

if old_blade in content:
    content = content.replace(old_blade, new_blade)
else:
    print("Could not find old_blade in patch_geometry_generator.py")

with open(gen_file, "w") as f:
    f.write(content)

print("Updated patch_geometry_generator.py")
