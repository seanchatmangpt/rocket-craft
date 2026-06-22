import re

with open("patch_geometry_generator.py", "r") as f:
    lines = f.readlines()

new_lines = []
mesh_data = """
                    int[] faceVertexCounts = [4, 4, 4, 4, 4, 4]
                    int[] faceVertexIndices = [0,1,2,3, 4,5,6,7, 0,1,5,4, 1,2,6,5, 2,3,7,6, 3,0,4,7]
                    point3f[] points = [(-0.5,-0.5,-0.5), (0.5,-0.5,-0.5), (0.5,0.5,-0.5), (-0.5,0.5,-0.5), (-0.5,-0.5,0.5), (0.5,-0.5,0.5), (0.5,0.5,0.5), (-0.5,0.5,0.5), (0,0,0)]
                    uniform token subdivisionScheme = "none"
"""

insert_mesh_next = False
for line in lines:
    if re.search(r'def\s+(Cube|Cylinder|Sphere|Cone|Capsule)\s+"', line):
        line = re.sub(r'def\s+(Cube|Cylinder|Sphere|Cone|Capsule)\s+"', 'def Mesh "', line)
        insert_mesh_next = True
        new_lines.append(line)
        continue
    
    if insert_mesh_next and "{" in line:
        new_lines.append(line)
        new_lines.append(mesh_data)
        insert_mesh_next = False
        continue

    # remove double size = ... etc
    if re.search(r'^\s*double\s+(size|radius|height)\s*=', line):
        continue

    new_lines.append(line)

with open("patch_geometry_generator.py", "w") as f:
    f.writelines(new_lines)
