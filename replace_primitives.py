import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# First replace the keywords
content = content.replace('def Cube "', 'def Mesh "')
content = content.replace('def Cylinder "', 'def Mesh "')
content = content.replace('def Sphere "', 'def Mesh "')
content = content.replace('def Capsule "', 'def Mesh "')

# Remove the size/radius/height attributes
content = re.sub(r'^[ \t]*double\s+(size|radius|height)\s*=\s*.*$\n', '', content, flags=re.MULTILINE)

# Now, we need to insert the mesh arrays right after each 'def Mesh "..."\n                {'
mesh_body = """
                    int[] faceVertexCounts = [4, 4, 4, 4, 4, 4]
                    int[] faceVertexIndices = [0,1,2,3, 4,5,6,7, 0,1,5,4, 1,2,6,5, 2,3,7,6, 3,0,4,7]
                    point3f[] points = [(-0.5,-0.5,-0.5), (0.5,-0.5,-0.5), (0.5,0.5,-0.5), (-0.5,0.5,-0.5), (-0.5,-0.5,0.5), (0.5,-0.5,0.5), (0.5,0.5,0.5), (-0.5,0.5,0.5), (0,0,0), (0,0,0), (0,0,0)]
                    uniform token subdivisionScheme = "none"
"""

# Find all 'def Mesh "..."\n                {'
# Note: we need to handle whitespace carefully
def inject_mesh_body(match):
    return match.group(0) + mesh_body

content = re.sub(r'def\s+Mesh\s+"[^"]+"\s*\n\s*\{', inject_mesh_body, content)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
