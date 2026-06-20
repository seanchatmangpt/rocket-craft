import os

all_merged_path = "/Users/sac/rocket-craft/ontology/all_merged.ttl"
backup_path = "/Users/sac/rocket-craft/ontology/all_merged.ttl.bak"

graph_dir = "/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/graph"
files_to_append = [
    "asset_fabric.ttl",
    "visual_targets.ttl",
    "generator_parameters.ttl"
]

# Read original
with open(all_merged_path, "r") as f:
    original_content = f.read()

# Make backup if not already present
if not os.path.exists(backup_path):
    with open(backup_path, "w") as f:
        f.write(original_content)

new_content = original_content
for filename in files_to_append:
    file_path = os.path.join(graph_dir, filename)
    with open(file_path, "r") as f:
        file_content = f.read()
    new_content += "\n\n# =========================================================\n"
    new_content += f"# Appended from {filename}\n"
    new_content += "# =========================================================\n"
    new_content += file_content

with open(all_merged_path, "w") as f:
    f.write(new_content)

print("Successfully merged and appended ontology files to all_merged.ttl")
