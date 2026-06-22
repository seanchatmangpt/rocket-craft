import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# I want to add arm, leg, and core panel logic to render_primitive
geometry_insert = """
            {% elif row.type == "arm" %}
                def Cylinder "arm_inner_frame"
                {
                    double radius = 0.15
                    double height = 1.0
                    double3 xformOp:rotateXYZ = (90, 0, 0)
                    uniform token[] xformOpOrder = ["xformOp:rotateXYZ"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_DarkFrame>
                }
                {% for i in range(end=4) %}
                def Cube "arm_armor_shell_{{ i }}"
                {
                    double size = 1.0
                    double3 xformOp:scale = (0.22, 0.8, 0.22)
                    double3 xformOp:translate = ({% if i == 0 %}0.12, 0.0, 0.0{% elif i == 1 %}-0.12, 0.0, 0.0{% elif i == 2 %}0.0, 0.0, 0.12{% else %}0.0, 0.0, -0.12{% endif %})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
                }
                def Cube "arm_panel_split_{{ i }}"
                {
                    double size = 1.0
                    double3 xformOp:scale = (0.01, 0.85, 0.01)
                    double3 xformOp:translate = ({% if i == 0 %}0.23, 0.0, 0.0{% elif i == 1 %}-0.23, 0.0, 0.0{% elif i == 2 %}0.0, 0.0, 0.23{% else %}0.0, 0.0, -0.23{% endif %})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_DarkFrame>
                }
                {% endfor %}
                def Sphere "arm_joint_socket"
                {
                    double radius = 0.25
                    double3 xformOp:translate = (0.0, 0.5, 0.0)
                    uniform token[] xformOpOrder = ["xformOp:translate"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
                }
                def Sphere "arm_joint_core"
                {
                    double radius = 0.20
                    double3 xformOp:translate = (0.0, 0.5, 0.0)
                    uniform token[] xformOpOrder = ["xformOp:translate"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_DarkFrame>
                }
            {% elif row.type == "leg" %}
                def Cylinder "leg_inner_frame"
                {
                    double radius = 0.2
                    double height = 1.2
                    double3 xformOp:rotateXYZ = (90, 0, 0)
                    uniform token[] xformOpOrder = ["xformOp:rotateXYZ"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_DarkFrame>
                }
                {% for i in range(end=4) %}
                def Cube "leg_armor_shell_{{ i }}"
                {
                    double size = 1.0
                    double3 xformOp:scale = (0.3, 1.0, 0.3)
                    double3 xformOp:translate = ({% if i == 0 %}0.18, 0.0, 0.0{% elif i == 1 %}-0.18, 0.0, 0.0{% elif i == 2 %}0.0, 0.0, 0.18{% else %}0.0, 0.0, -0.18{% endif %})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
                }
                def Cube "leg_panel_split_{{ i }}"
                {
                    double size = 1.0
                    double3 xformOp:scale = (0.02, 1.05, 0.02)
                    double3 xformOp:translate = ({% if i == 0 %}0.33, 0.0, 0.0{% elif i == 1 %}-0.33, 0.0, 0.0{% elif i == 2 %}0.0, 0.0, 0.33{% else %}0.0, 0.0, -0.33{% endif %})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_DarkFrame>
                }
                {% endfor %}
                def Sphere "leg_joint_socket"
                {
                    double radius = 0.3
                    double3 xformOp:translate = (0.0, 0.6, 0.0)
                    uniform token[] xformOpOrder = ["xformOp:translate"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
                }
                def Sphere "leg_joint_core"
                {
                    double radius = 0.25
                    double3 xformOp:translate = (0.0, 0.6, 0.0)
                    uniform token[] xformOpOrder = ["xformOp:translate"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_DarkFrame>
                }
            {% elif row.type == "core" %}
                def Cube "core_inner_frame"
                {
                    double size = 1.0
                    double3 xformOp:scale = (0.9, 0.9, 0.9)
                    uniform token[] xformOpOrder = ["xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_DarkFrame>
                }
                {% for i in range(end=6) %}
                def Cube "core_armor_plate_{{ i }}"
                {
                    double size = 1.0
                    double3 xformOp:scale = ({% if i < 2 %}0.8{% else %}1.05{% endif %}, {% if i > 1 and i < 4 %}0.8{% else %}1.05{% endif %}, {% if i > 3 %}0.8{% else %}1.05{% endif %})
                    double3 xformOp:translate = ({% if i == 0 %}0.1, 0, 0{% elif i == 1 %}-0.1, 0, 0{% elif i == 2 %}0, 0.1, 0{% elif i == 3 %}0, -0.1, 0{% elif i == 4 %}0, 0, 0.1{% else %}0, 0, -0.1{% endif %})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
                }
                def Cube "core_panel_gap_{{ i }}"
                {
                    double size = 1.0
                    double3 xformOp:scale = ({% if i < 2 %}0.02{% else %}1.1{% endif %}, {% if i > 1 and i < 4 %}0.02{% else %}1.1{% endif %}, {% if i > 3 %}0.02{% else %}1.1{% endif %})
                    double3 xformOp:translate = ({% if i == 0 %}0.5, 0, 0{% elif i == 1 %}-0.5, 0, 0{% elif i == 2 %}0, 0.5, 0{% elif i == 3 %}0, -0.5, 0{% elif i == 4 %}0, 0, 0.5{% else %}0, 0, -0.5{% endif %})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_DarkFrame>
                }
                {% endfor %}
"""

# add it before the `{% else %}` branch
if "{% elif row.type == \"arm\" %}" not in content:
    content = content.replace("            {% else %}\n                {% for i in range(end=6) %}", geometry_insert + "            {% else %}\n                {% for i in range(end=6) %}")

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
