import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# Replace Cube with Mesh and add geometry for feather_blade and beveled_plate
# Wait, let's just make it a Mesh with the points of a Cube.
cube_geom = """
                    int[] faceVertexCounts = [4, 4, 4, 4, 4, 4]
                    int[] faceVertexIndices = [0, 1, 2, 3, 4, 5, 6, 7, 0, 4, 5, 1, 1, 5, 6, 2, 2, 6, 7, 3, 3, 7, 4, 0]
                    point3f[] points = [(-0.5, -0.5, -0.5), (0.5, -0.5, -0.5), (0.5, 0.5, -0.5), (-0.5, 0.5, -0.5), (-0.5, -0.5, 0.5), (0.5, -0.5, 0.5), (0.5, 0.5, 0.5), (-0.5, 0.5, 0.5)]
"""

content = content.replace('def Cube "feather_blade_{{ i }}"\n                {', 'def Mesh "feather_blade_{{ i }}"\n                {' + cube_geom)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
