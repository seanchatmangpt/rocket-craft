gen_file = "generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera"
with open(gen_file, "r") as f:
    lines = f.readlines()

for i in range(len(lines)):
    if "def Mesh" in lines[i] and "blade_edge" in lines[i]:
        # found blade edge, update translate and scale
        for j in range(i, i+15):
            if "double3 xformOp:scale" in lines[j]:
                lines[j] = "                    double3 xformOp:scale = (2.45, 0.1, 0.5)\n"
            if "double3 xformOp:translate" in lines[j]:
                lines[j] = "                    double3 xformOp:translate = ({% if is_right %}0.9{% else %}-0.9{% endif %}, 0.0, 2.0)\n"

with open(gen_file, "w") as f:
    f.write("".join(lines))

print("Fixed template.")
