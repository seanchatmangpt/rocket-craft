import re

gen_file = "generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera"
with open(gen_file, "r") as f:
    content = f.read()

# We want to replace the entire block from `{% elif row.type == "beveled_hard_surface_plate"...`
# to `{% endfor %}` right before `{% else %}`.

pattern = r'\{% elif row\.type == "beveled_hard_surface_plate" or row\.type == "blade_prism" %\}.*?\{% endfor %\}'

replacement = """{% elif row.type == "beveled_hard_surface_plate" or row.type == "blade_prism" %}
                {% for i in range(end=5) %}
                def Mesh "{{ row.primLocalName }}_beveled_plate_{{ i }}"
                {
                    int[] faceVertexCounts = [4, 4, 4, 4, 4, 4]
                    int[] faceVertexIndices = [0,1,2,3, 4,5,6,7, 0,1,5,4, 1,2,6,5, 2,3,7,6, 3,0,4,7]
                    point3f[] points = [(-0.5,-0.5,-0.5), (0.5,-0.5,-0.5), (0.5,0.5,-0.5), (-0.5,0.5,-0.5), (-0.5,-0.5,0.5), (0.5,-0.5,0.5), (0.5,0.5,0.5), (-0.5,0.5,0.5), (0,0,0)]
                    uniform token subdivisionScheme = "none"
                    double3 xformOp:scale = (1.8, 0.68, 0.85)
                    double3 xformOp:translate = ({% if is_right %}{{ 0.945 }}{% else %}{{ -0.945 }}{% endif %}, 0.0, 2.0)
                    double3 xformOp:rotateXYZ = (0, 0, {% if is_right %}4.5{% else %}-4.5{% endif %})
                    uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:rotateXYZ", "xformOp:scale"]
                    rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
                }
                def Mesh "{{ row.primLocalName }}_blade_edge_{{ i }}"
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
                }
                {% endfor %}"""

new_content = re.sub(pattern, replacement, content, flags=re.DOTALL)
if new_content != content:
    with open(gen_file, "w") as f:
        f.write(new_content)
    print("Patched successfully.")
else:
    print("Pattern not found!")
