import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# 1. Fix VIS203: Feather panels
content = content.replace(
    'double3 xformOp:rotateXYZ = (0, {{ i * 10.0 }}, 0)',
    'double3 xformOp:rotateXYZ = (0, {% if is_right %}{{ ((i % 2) * 20.0) * -1.0 }}{% else %}{{ (i % 2) * 20.0 }}{% endif %}, 0)'
)

# 2. Fix VIS204: Torso Mass
content = content.replace(
    'double3 xformOp:scale = ({{ 1.0 - (i * 0.05) }}, {{ 1.0 - (i * 0.05) }}, 0.1)',
    'double3 xformOp:scale = ({{ 0.5 - (i * 0.05) }}, {{ 0.5 - (i * 0.05) }}, 0.1)'
)

# 3. Fix VIS205: Blade Placement
content = content.replace(
    'double3 xformOp:scale = (0.6, 0.2, {{ 1.0 - (i * 0.15) }})',
    'double3 xformOp:scale = (2.5, 0.2, {{ 1.0 - (i * 0.15) }})'
)
content = content.replace(
    'double3 xformOp:rotateXYZ = ({{ i * 5.0 }}, 0, 0)',
    'double3 xformOp:rotateXYZ = (0, 0, {% if is_right %}-15.0{% else %}15.0{% endif %})'
)

# 4. Inject Bipedal Chassis (arm, leg, core)
mesh_props = """
                    int[] faceVertexCounts = [4, 4, 4, 4, 4, 4]
                    int[] faceVertexIndices = [0,1,2,3, 4,5,6,7, 0,1,5,4, 1,2,6,5, 2,3,7,6, 3,0,4,7]
                    point3f[] points = [(-0.5,-0.5,-0.5), (0.5,-0.5,-0.5), (0.5,0.5,-0.5), (-0.5,0.5,-0.5), (-0.5,-0.5,0.5), (0.5,-0.5,0.5), (0.5,0.5,0.5), (-0.5,0.5,0.5), (0,0,0)]
                    uniform token subdivisionScheme = "none"
"""

geometry_insert = f"""
            {{% elif row.type == "arm" %}}
                def Mesh "arm_inner_frame"
                {{
{mesh_props}
                    double3 xformOp:scale = (0.3, 1.0, 0.3)
                    double3 xformOp:rotateXYZ = (90, 0, 0)
                    uniform token[] xformOpOrder = ["xformOp:rotateXYZ", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_DarkFrame>
                }}
                {{% for i in range(end=8) %}}
                def Mesh "arm_armor_shell_{{{{ i }}}}"
                {{
{mesh_props}
                    double3 xformOp:scale = (0.22, 0.8, 0.22)
                    double3 xformOp:translate = ({{% if i == 0 %}}0.12, 0.0, 0.0{{% elif i == 1 %}}-0.12, 0.0, 0.0{{% elif i == 2 %}}0.0, 0.0, 0.12{{% else %}}0.0, 0.0, -0.12{{% endif %}})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{{{ row.materialLocalName }}}}>
                }}
                def Mesh "arm_panel_split_{{{{ i }}}}"
                {{
{mesh_props}
                    double3 xformOp:scale = (0.01, 0.85, 0.01)
                    double3 xformOp:translate = ({{% if i == 0 %}}0.23, 0.0, 0.0{{% elif i == 1 %}}-0.23, 0.0, 0.0{{% elif i == 2 %}}0.0, 0.0, 0.23{{% else %}}0.0, 0.0, -0.23{{% endif %}})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_DarkFrame>
                }}
                {{% endfor %}}
            {{% elif row.type == "leg" %}}
                def Mesh "leg_inner_frame"
                {{
{mesh_props}
                    double3 xformOp:scale = (0.4, 1.2, 0.4)
                    double3 xformOp:rotateXYZ = (90, 0, 0)
                    uniform token[] xformOpOrder = ["xformOp:rotateXYZ", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_DarkFrame>
                }}
                {{% for i in range(end=8) %}}
                def Mesh "leg_armor_shell_{{{{ i }}}}"
                {{
{mesh_props}
                    double3 xformOp:scale = (0.3, 1.0, 0.3)
                    double3 xformOp:translate = ({{% if i == 0 %}}0.18, 0.0, 0.0{{% elif i == 1 %}}-0.18, 0.0, 0.0{{% elif i == 2 %}}0.0, 0.0, 0.18{{% else %}}0.0, 0.0, -0.18{{% endif %}})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{{{ row.materialLocalName }}}}>
                }}
                def Mesh "leg_panel_split_{{{{ i }}}}"
                {{
{mesh_props}
                    double3 xformOp:scale = (0.02, 1.05, 0.02)
                    double3 xformOp:translate = ({{% if i == 0 %}}0.33, 0.0, 0.0{{% elif i == 1 %}}-0.33, 0.0, 0.0{{% elif i == 2 %}}0.0, 0.0, 0.33{{% else %}}0.0, 0.0, -0.33{{% endif %}})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_DarkFrame>
                }}
                {{% endfor %}}
            {{% elif row.type == "core" %}}
                def Mesh "core_inner_frame"
                {{
{mesh_props}
                    double3 xformOp:scale = (0.9, 0.9, 0.9)
                    uniform token[] xformOpOrder = ["xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_DarkFrame>
                }}
                {{% for i in range(end=16) %}}
                def Mesh "core_armor_plate_{{{{ i }}}}"
                {{
{mesh_props}
                    double3 xformOp:scale = ({{% if i < 2 %}}0.8{{% else %}}1.05{{% endif %}}, {{% if i > 1 and i < 4 %}}0.8{{% else %}}1.05{{% endif %}}, {{% if i > 3 %}}0.8{{% else %}}1.05{{% endif %}})
                    double3 xformOp:translate = ({{% if i == 0 %}}0.1, 0, 0{{% elif i == 1 %}}-0.1, 0, 0{{% elif i == 2 %}}0, 0.1, 0{{% elif i == 3 %}}0, -0.1, 0{{% elif i == 4 %}}0, 0, 0.1{{% else %}}0, 0, -0.1{{% endif %}})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{{{ row.materialLocalName }}}}>
                }}
                {{% endfor %}}
"""

if "{% elif row.type == \"arm\" %}" not in content:
    content = content.replace('            {% else %}\n                {% for i in range(end=6) %}', geometry_insert + '\n            {% else %}\n                {% for i in range(end=6) %}')

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)

print("Applied changes successfully.")
