import os
import re

with open("/Users/sac/rocket-craft/patch_geometry_generator.py", "r") as f:
    code = f.read()

new_macro = """{% macro render_primitive(row) %}
    {% if row.type == "socket" %}
        def Xform "{{ row.primLocalName }}"
        {
            double3 xformOp:translate = ({{ row.translateX }}, {{ row.translateY }}, {{ row.translateZ }})
            uniform token[] xformOpOrder = ["xformOp:translate"]
        }
    {% else %}
        def Xform "{{ row.primLocalName }}_group"
        {
            double3 xformOp:translate = ({{ row.translateX }}, {{ row.translateY }}, {{ row.translateZ }})
            double3 xformOp:scale = ({{ row.scaleX }}, {{ row.scaleY }}, {{ row.scaleZ }})
            double3 xformOp:rotateXYZ = ({{ row.rotateX }}, {{ row.rotateY }}, {{ row.rotateZ }})
            uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:rotateXYZ", "xformOp:scale"]

            {% if row.type == "angular_armor_shell" or row.type == "tapered_box" %}
                {% for i in range(end=8) %}
                def Cube "armor_plate_{{ i }}"
                {
                    double size = 1.0
                    double3 xformOp:scale = (1.0 - ({{ i }} * 0.05), 1.0 - ({{ i }} * 0.05), 0.1)
                    double3 xformOp:translate = (0.0, 0.0, {{ i }} * 0.08)
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
                }
                def Xform "hardpoint_{{ i }}"
                {
                    double3 xformOp:translate = (0.0, 0.0, {{ i }} * 0.08)
                    uniform token[] xformOpOrder = ["xformOp:translate"]
                }
                def Cylinder "armor_piston_{{ i }}"
                {
                    double radius = 0.05
                    double height = 1.2
                    double3 xformOp:translate = (0.4, 0.4, {{ i }} * 0.08)
                    uniform token[] xformOpOrder = ["xformOp:translate"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_DarkFrame>
                }
                def Cylinder "armor_piston2_{{ i }}"
                {
                    double radius = 0.05
                    double height = 1.2
                    double3 xformOp:translate = (-0.4, -0.4, {{ i }} * 0.08)
                    uniform token[] xformOpOrder = ["xformOp:translate"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_DarkFrame>
                }
                {% endfor %}
            {% elif row.type == "layered_swept_feather_panel" or row.type == "feather_panel" %}
                {% for i in range(end=12) %}
                def Cube "feather_blade_{{ i }}"
                {
                    double size = 1.0
                    double3 xformOp:scale = (0.8 - ({{ i }} * 0.05), 0.05, 1.5 - ({{ i }} * 0.1))
                    double3 xformOp:translate = ({{ i }} * 0.1, {{ i }} * 0.05, 0.0)
                    double3 xformOp:rotateXYZ = (0, {{ i }} * 10.0, 0)
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:rotateXYZ", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
                }
                def Cone "feather_tip_{{ i }}"
                {
                    double radius = 0.2
                    double height = 0.5
                    double3 xformOp:translate = ({{ i }} * 0.1, {{ i }} * 0.05, 0.8)
                    uniform token[] xformOpOrder = ["xformOp:translate"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_CyanBlade>
                }
                {% endfor %}
            {% elif row.type == "beveled_hard_surface_plate" or row.type == "blade_prism" %}
                {% for i in range(end=5) %}
                def Cube "beveled_plate_{{ i }}"
                {
                    double size = 1.0
                    double3 xformOp:scale = (0.6, 0.2, 1.0 - ({{ i }} * 0.15))
                    double3 xformOp:translate = (0.0, {{ i }} * 0.05, 0.0)
                    double3 xformOp:rotateXYZ = ({{ i }} * 5.0, 0, 0)
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:rotateXYZ", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
                }
                def Capsule "blade_edge_{{ i }}"
                {
                    double radius = 0.02
                    double height = 1.2
                    double3 xformOp:translate = (0.3, {{ i }} * 0.05, 0.0)
                    uniform token[] xformOpOrder = ["xformOp:translate"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_CyanBlade>
                }
                {% endfor %}
            {% else %}
                {% for i in range(end=6) %}
                def Cylinder "subframe_core_{{ i }}"
                {
                    double radius = 0.3 - ({{ i }} * 0.04)
                    double height = 0.8 + ({{ i }} * 0.1)
                    double3 xformOp:rotateXYZ = (90, 0, 0)
                    uniform token[] xformOpOrder = ["xformOp:rotateXYZ"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_DarkFrame>
                }
                def Sphere "subframe_joint_{{ i }}"
                {
                    double radius = 0.35
                    double3 xformOp:scale = (1.0, 0.5, 1.0)
                    double3 xformOp:translate = (0.0, 0.4 + ({{ i }} * 0.05), 0.0)
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
                }
                {% endfor %}
            {% endif %}
        }
    {% endif %}
{% endmacro %}"""

start_str = '{% macro render_primitive(row) %}'
end_str = '{% endmacro %}'
start_idx = code.find(start_str)
end_idx = code.find(end_str) + len(end_str)

new_code = code[:start_idx] + new_macro + code[end_idx:]

with open("/Users/sac/rocket-craft/patch_geometry_generator.py", "w") as f:
    f.write(new_code)
