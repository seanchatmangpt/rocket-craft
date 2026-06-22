import re

gen_file = "patch_geometry_generator.py"
with open(gen_file, "r") as f:
    content = f.read()

# Replace arm armor
old_arm = """                {% for i in range(end=8) %}
                def Mesh "{{ row.primLocalName }}_arm_armor_shell_{{ i }}"
                {

                    int[] faceVertexCounts = [4, 4, 4, 4, 4, 4]
                    int[] faceVertexIndices = [0,1,2,3, 4,5,6,7, 0,1,5,4, 1,2,6,5, 2,3,7,6, 3,0,4,7]
                    point3f[] points = [(-0.5,-0.5,-0.5), (0.5,-0.5,-0.5), (0.5,0.5,-0.5), (-0.5,0.5,-0.5), (-0.5,-0.5,0.5), (0.5,-0.5,0.5), (0.5,0.5,0.5), (-0.5,0.5,0.5), (0,0,0)]
                    uniform token subdivisionScheme = "none"

                    double3 xformOp:scale = (0.22, 0.8, 0.22)
                    double3 xformOp:translate = ({% if i == 0 %}0.12, 0.0, 0.0{% elif i == 1 %}-0.12, 0.0, 0.0{% elif i == 2 %}0.0, 0.0, 0.12{% else %}0.0, 0.0, -0.12{% endif %})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
                }
                {% endfor %}"""

new_arm = """                {% for i in range(end=32) %}
                def Mesh "{{ row.primLocalName }}_arm_armor_shell_{{ i }}"
                {

                    int[] faceVertexCounts = [4, 4, 4, 4, 4, 4]
                    int[] faceVertexIndices = [0,1,2,3, 4,5,6,7, 0,1,5,4, 1,2,6,5, 2,3,7,6, 3,0,4,7]
                    point3f[] points = [(-0.5,-0.5,-0.5), (0.5,-0.5,-0.5), (0.5,0.5,-0.5), (-0.5,0.5,-0.5), (-0.5,-0.5,0.5), (0.5,-0.5,0.5), (0.5,0.5,0.5), (-0.5,0.5,0.5), (0,0,0)]
                    uniform token subdivisionScheme = "none"

                    double3 xformOp:scale = (0.22, 0.08, 0.22)
                    double3 xformOp:translate = ({% if i % 4 == 0 %}0.12{% elif i % 4 == 1 %}-0.12{% else %}0.0{% endif %}, {{ ((i // 4) - 3) * 0.11 }}, {% if i % 4 == 2 %}0.12{% elif i % 4 == 3 %}-0.12{% else %}0.0{% endif %})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
                }
                {% endfor %}"""

# Replace leg armor
old_leg = """                {% for i in range(end=8) %}
                def Mesh "{{ row.primLocalName }}_leg_armor_shell_{{ i }}"
                {

                    int[] faceVertexCounts = [4, 4, 4, 4, 4, 4]
                    int[] faceVertexIndices = [0,1,2,3, 4,5,6,7, 0,1,5,4, 1,2,6,5, 2,3,7,6, 3,0,4,7]
                    point3f[] points = [(-0.5,-0.5,-0.5), (0.5,-0.5,-0.5), (0.5,0.5,-0.5), (-0.5,0.5,-0.5), (-0.5,-0.5,0.5), (0.5,-0.5,0.5), (0.5,0.5,0.5), (-0.5,0.5,0.5), (0,0,0)]
                    uniform token subdivisionScheme = "none"

                    double3 xformOp:scale = (0.3, 1.0, 0.3)
                    double3 xformOp:translate = ({% if i == 0 %}0.18, 0.0, 0.0{% elif i == 1 %}-0.18, 0.0, 0.0{% elif i == 2 %}0.0, 0.0, 0.18{% else %}0.0, 0.0, -0.18{% endif %})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
                }
                {% endfor %}"""

new_leg = """                {% for i in range(end=32) %}
                def Mesh "{{ row.primLocalName }}_leg_armor_shell_{{ i }}"
                {

                    int[] faceVertexCounts = [4, 4, 4, 4, 4, 4]
                    int[] faceVertexIndices = [0,1,2,3, 4,5,6,7, 0,1,5,4, 1,2,6,5, 2,3,7,6, 3,0,4,7]
                    point3f[] points = [(-0.5,-0.5,-0.5), (0.5,-0.5,-0.5), (0.5,0.5,-0.5), (-0.5,0.5,-0.5), (-0.5,-0.5,0.5), (0.5,-0.5,0.5), (0.5,0.5,0.5), (-0.5,0.5,0.5), (0,0,0)]
                    uniform token subdivisionScheme = "none"

                    double3 xformOp:scale = (0.3, 0.10, 0.3)
                    double3 xformOp:translate = ({% if i % 4 == 0 %}0.18{% elif i % 4 == 1 %}-0.18{% else %}0.0{% endif %}, {{ ((i // 4) - 3) * 0.13 }}, {% if i % 4 == 2 %}0.18{% elif i % 4 == 3 %}-0.18{% else %}0.0{% endif %})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
                }
                {% endfor %}"""

if old_arm in content:
    content = content.replace(old_arm, new_arm)
else:
    print("Could not find old_arm")

if old_leg in content:
    content = content.replace(old_leg, new_leg)
else:
    print("Could not find old_leg")

with open(gen_file, "w") as f:
    f.write(content)

print("Updated leg and arm armor")
