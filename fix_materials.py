import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

# Replace all absolute material bindings with relative ones (or local absolute ones)
# Currently it is: rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
# Wait, some are hardcoded like </ASSET_ReferenceFabric_001/Materials/M_CyanBlade>
# Let's replace </ASSET_ReferenceFabric_001/Materials/...> with <../Materials/...> which is relative!
content = content.replace("</ASSET_ReferenceFabric_001/Materials/", "<../Materials/")

# Wait, the root of the file is SM_WingArray_Left. If I put the Materials scope inside it,
# relative path <../Materials/M_CyanBlade> from inside a Mesh (e.g. /SM_WingArray_Left/prim_group/feather_blade)
# would go up to prim_group, then look for Materials. But Materials is at /SM_WingArray_Left/Materials!
# So <../Materials/...> doesn't work. We need the exact parent name.
# Wait! In Tera, we can't easily know the root name for the current file unless we pass it.
# Actually, the python script writes the name exactly!
# For example:
# def Xform "SM_WingArray_Left"
# { ... }
# I can just use a regex to replace </ASSET_ReferenceFabric_001/Materials/X> with <Materials/X> inside each Xform block?
# If I inject `def Scope "Materials" ... ` into `part_mesh_tera`.
