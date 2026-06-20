import os

all_merged_path = "/Users/sac/rocket-craft/ontology/all_merged.ttl"
gen_params_path = "/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/graph/generator_parameters.ttl"

if not os.path.exists(all_merged_path):
    print(f"Error: {all_merged_path} not found.")
    exit(1)
if not os.path.exists(gen_params_path):
    print(f"Error: {gen_params_path} not found.")
    exit(1)

with open(all_merged_path, "r") as f:
    content = f.read()

# Let's find the split point. We want to split before the generator parameters ontology.
# We look for "mud:GeneratorParametersOntology" in the file.
split_token = "mud:GeneratorParametersOntology"
if split_token not in content:
    print(f"Error: Token '{split_token}' not found in all_merged.ttl")
    exit(1)

parts = content.split(split_token)
# The first part contains everything up to the prefixes before the token.
# Let's clean up the last few lines of parts[0] that contain the prefixes/comments of the generator ontology.
lines = parts[0].split("\n")
split_idx = -1
for idx in range(len(lines) - 1, -1, -1):
    if "# =========================================================" in lines[idx]:
        split_idx = idx
        break

if split_idx == -1:
    # fallback to just before prefixes
    for idx in range(len(lines) - 1, -1, -1):
        if "@prefix" not in lines[idx] and lines[idx].strip() != "":
            split_idx = idx + 1
            break

base_content = "\n".join(lines[:split_idx]) + "\n\n# =========================================================\n"

with open(gen_params_path, "r") as f:
    new_params = f.read()

final_content = base_content + new_params

with open(all_merged_path, "w") as f:
    f.write(final_content)

print("Ontology merge complete!")
