gen_file = "generated/mech_assets/reference_fabric_001/templates/usd/part_mesh.usda.tera"
with open(gen_file, "r") as f:
    lines = f.readlines()

for i in range(len(lines)):
    if "M_CyanBlade" in lines[i]:
        lines[i] = lines[i].replace("M_CyanBlade", "M_GoldVisor")

for i in range(len(lines)):
    if "def Mesh" in lines[i] and "blade_edge" in lines[i]:
        # found blade edge, look for material binding below
        for j in range(i, i+20):
            if "rel material:binding" in lines[j] and "M_GoldVisor" in lines[j]:
                lines[j] = lines[j].replace("M_GoldVisor", "M_CyanBlade")
                break

with open(gen_file, "w") as f:
    f.write("".join(lines))

print("Fixed template.")
