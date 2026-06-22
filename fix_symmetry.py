import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# We need to replace xformOp:translate = (...) with mirrored logic if is_right
# But doing this everywhere is tedious. Let's just create a macro or variable.
# Actually, we can use Tera's inline if: {{ -0.3 if is_right else 0.3 }}
# Let's find all xformOp:translate = (X, Y, Z) and change the X part if it contains hardcoded numbers

# Let's look at specific cases:
# armor_piston_ ({{ 0.4 - (i * 0.02) }}) -> {{ -(0.4 - (i * 0.02)) if is_right else (0.4 - (i * 0.02)) }}
# feather_blade_ ({{ i * 0.1 }}) -> {{ -(i * 0.1) if is_right else (i * 0.1) }}
# blade_edge_ (0.3) -> {{ -0.3 if is_right else 0.3 }}

# Wait, the easiest way is to wrap the X coordinate in a macro or inline if.
# Let's write a python script to replace the lines manually.

replacements = [
    (r"double3 xformOp:translate = \(\{\{ 0\.4 - \(i \* 0\.02\) \}\}, \{\{ 0\.4 - \(i \* 0\.02\) \}\}, \{\{ i \* 0\.08 \}\}\)", 
     r"double3 xformOp:translate = ({% if is_right %}{{ -(0.4 - (i * 0.02)) }}{% else %}{{ 0.4 - (i * 0.02) }}{% endif %}, {{ 0.4 - (i * 0.02) }}, {{ i * 0.08 }})"),
    (r"double3 xformOp:translate = \(\{\{ -0\.4 \+ \(i \* 0\.02\) \}\}, \{\{ -0\.4 \+ \(i \* 0\.02\) \}\}, \{\{ i \* 0\.08 \}\}\)", 
     r"double3 xformOp:translate = ({% if is_right %}{{ -(-0.4 + (i * 0.02)) }}{% else %}{{ -0.4 + (i * 0.02) }}{% endif %}, {{ -0.4 + (i * 0.02) }}, {{ i * 0.08 }})"),
    (r"double3 xformOp:translate = \(\{\{ i \* 0\.1 \}\}, \{\{ i \* 0\.05 \}\}, 0\.0\)", 
     r"double3 xformOp:translate = ({% if is_right %}{{ -(i * 0.1) }}{% else %}{{ i * 0.1 }}{% endif %}, {{ i * 0.05 }}, 0.0)"),
    (r"double3 xformOp:translate = \(0\.3, \{\{ i \* 0\.05 \}\}, 0\.0\)", 
     r"double3 xformOp:translate = ({% if is_right %}-0.3{% else %}0.3{% endif %}, {{ i * 0.05 }}, 0.0)"),
    (r"double3 xformOp:translate = \(\{\{ i \* 0\.3 - 4\.5 \}\}, 0\.0, 0\.0\)", 
     r"double3 xformOp:translate = ({% if is_right %}{{ -(i * 0.3 - 4.5) }}{% else %}{{ i * 0.3 - 4.5 }}{% endif %}, 0.0, 0.0)"),
    (r"double3 xformOp:translate = \(\{\{ i \* 0\.6 - 2\.1 \}\}, \{\{ \(i % 2\) \* 0\.15 \}\}, 0\.5\)", 
     r"double3 xformOp:translate = ({% if is_right %}{{ -(i * 0.6 - 2.1) }}{% else %}{{ i * 0.6 - 2.1 }}{% endif %}, {{ (i % 2) * 0.15 }}, 0.5)"),
    (r"double3 xformOp:translate = \(2\.5, 0, 0\)", 
     r"double3 xformOp:translate = ({% if is_right %}-2.5{% else %}2.5{% endif %}, 0, 0)"),
    (r"double3 xformOp:translate = \(5\.2, 0, 0\)", 
     r"double3 xformOp:translate = ({% if is_right %}-5.2{% else %}5.2{% endif %}, 0, 0)")
]

for old, new in replacements:
    content = re.sub(old, new, content)

# Wait, what about xformOp:rotateXYZ? If we mirror X, we might need to negate Y and Z rotations.
# feather_blade_ rotate: (0, {{ i * 10.0 }}, 0) -> right side should be (0, -{{ i * 10.0 }}, 0)
content = re.sub(
    r"double3 xformOp:rotateXYZ = \(0, \{\{ i \* 10\.0 \}\}, 0\)",
    r"double3 xformOp:rotateXYZ = (0, {% if is_right %}{{ -(i * 10.0) }}{% else %}{{ i * 10.0 }}{% endif %}, 0)",
    content
)

# beveled_plate_ rotate: ({{ i * 5.0 }}, 0, 0) -> Wait, rotating around X axis doesn't change when mirroring X axis.
# Wait, X mirroring means X -> -X.
# Rotation around X axis: Y -> Y*cos - Z*sin, Z -> Y*sin + Z*cos.
# Rotation around Y axis: X -> X*cos + Z*sin. If X is mirrored, we must negate Y rotation to preserve visual symmetry?
# Let's just mirror Y and Z rotations if they exist.
# The feather_blade rotates around Y, so negating it makes sense.
# The beveled_plate rotates around X: ({{ i * 5.0 }}, 0, 0). Mirroring X doesn't change X rotation?
# Actually, let's just leave X rotation alone unless it fails.

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
