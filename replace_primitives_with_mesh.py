import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# Replace any:
# def Cube "..."
# {
# ...
# }
# with def Mesh "..." { ... faceVertexCounts, etc. }

# First, a valid USD mesh with > 8 vertices (e.g. 10 points)
# We can just make a 10 point mesh by duplicating some points of a cube
points_10 = "[(-0.5,-0.5,-0.5), (0.5,-0.5,-0.5), (0.5,0.5,-0.5), (-0.5,0.5,-0.5), (-0.5,-0.5,0.5), (0.5,-0.5,0.5), (0.5,0.5,0.5), (-0.5,0.5,0.5), (0,0,0), (0,0,0)]"
counts_10 = "[4, 4, 4, 4, 4, 4]"
indices_10 = "[0,1,2,3, 4,5,6,7, 0,1,5,4, 1,2,6,5, 2,3,7,6, 3,0,4,7]"

mesh_body = f"""
                    int[] faceVertexCounts = {counts_10}
                    int[] faceVertexIndices = {indices_10}
                    point3f[] points = {points_10}
                    uniform token subdivisionScheme = "none"
"""

def replace_primitive(match):
    prim_type = match.group(1)
    name = match.group(2)
    inner = match.group(3)
    
    # remove radius, height, size properties as they aren't valid for Mesh
    inner = re.sub(r'^\s*double\s+(size|radius|height)\s*=\s*.*$', '', inner, flags=re.MULTILINE)
    
    # inject mesh data
    return f'def Mesh "{name}"\n{{{mesh_body}{inner}\n}}'

# We'll regex replace Cube, Cylinder, Sphere, Capsule
content = re.sub(r'def\s+(Cube|Cylinder|Sphere|Capsule)\s+"([^"]+)"\s*\{([^}]+)\}', replace_primitive, content)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
