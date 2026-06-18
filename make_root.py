import os
import re
import glob

workspace_tomls = [
    'tools/Cargo.toml',
    'blueprint-rs/Cargo.toml',
    'nexus-engine/Cargo.toml',
    'wasm-threads/Cargo.toml',
    'unify-rs/Cargo.toml',
    'infinity-blade-4/mud/Cargo.toml',
    'rocket-simulator/Cargo.toml',
    'asset-pipeline/Cargo.toml'
]

all_members = set()
all_deps = {}
all_patches = {}

for toml_path in workspace_tomls:
    if not os.path.exists(toml_path):
        continue
    with open(toml_path, 'r') as f:
        content = f.read()
    
    # Is it pure virtual?
    if '[package]' in content:
        # If it has a package, we can't just delete it.
        print(f"WARNING: {toml_path} has [package]")
        
    # Extract members
    members_match = re.search(r'members\s*=\s*\[(.*?)\]', content, re.DOTALL)
    if members_match:
        members_str = members_match.group(1)
        members = re.findall(r'"([^"]+)"', members_str)
        for member in members:
            # resolve relative to root
            # e.g. tools/knhk -> tools/knhk
            dir_name = os.path.dirname(toml_path)
            abs_member = os.path.normpath(os.path.join(dir_name, member))
            all_members.add(abs_member)
    
    # We will just use glob members in the root to be easier.

# Add the standalone crates we know about
all_members.add('chicago-tdd-tools')
all_members.add('genie3-rs')

root_toml = f"""[workspace]
resolver = "2"
members = [
    "asset-pipeline/*",
    "blueprint-rs/*",
    "chicago-tdd-tools",
    "genie3-rs",
    "infinity-blade-4/mud",
    "nexus-engine/crates/*",
    "rocket-simulator/*",
    "tools/*",
    "unify-rs/*",
    "wasm-threads/*"
]

[workspace.dependencies]
cargo-cicd = {{ git = "https://github.com/seanchatmangpt/cargo-cicd.git" }}

# We will let cargo automatically resolve inherited workspace.dependencies from the original files 
# wait, if we delete the original files, we lose the workspace.dependencies!
"""

with open('Cargo.toml', 'w') as f:
    f.write(root_toml)

print("Generated root Cargo.toml")
