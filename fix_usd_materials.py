import re

with open("patch_geometry_generator.py", "r") as f:
    content = f.read()

materials_scope = """
def Scope "ASSET_ReferenceFabric_001"
{
    def Scope "Materials"
    {
        def Material "M_WhiteArmor" (
            prepend references = @../materialx/M_WhiteArmor.mtlx@
        ) {}
        def Material "M_CyanBlade" (
            prepend references = @../materialx/M_CyanBlade.mtlx@
        ) {}
        def Material "M_DarkFrame" (
            prepend references = @../materialx/M_DarkFrame.mtlx@
        ) {}
        def Material "M_GoldVisor" (
            prepend references = @../materialx/M_GoldVisor.mtlx@
        ) {}
    }
}
"""

# Insert right after `metersPerUnit = 0.01\n)` in part_mesh_tera
content = content.replace("metersPerUnit = 0.01\n)", "metersPerUnit = 0.01\n)\n" + materials_scope)

with open("patch_geometry_generator.py", "w") as f:
    f.write(content)
