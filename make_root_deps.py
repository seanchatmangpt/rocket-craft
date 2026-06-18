import re
import os

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

deps = {}
patches = {}

for toml_path in workspace_tomls:
    if not os.path.exists(toml_path):
        continue
    with open(toml_path, 'r') as f:
        content = f.read()
    
    # Extract [workspace.dependencies] block
    deps_match = re.search(r'\[workspace\.dependencies\](.*?)(\n\[|$)', content, re.DOTALL)
    if deps_match:
        lines = deps_match.group(1).strip().split('\n')
        for line in lines:
            line = line.strip()
            if not line or line.startswith('#'): continue
            parts = line.split('=', 1)
            if len(parts) == 2:
                key = parts[0].strip()
                val = parts[1].strip()
                deps[key] = val
                
    # Extract [patch.crates-io] block
    patch_match = re.search(r'\[patch\.crates-io\](.*?)(\n\[|$)', content, re.DOTALL)
    if patch_match:
        lines = patch_match.group(1).strip().split('\n')
        for line in lines:
            line = line.strip()
            if not line or line.startswith('#'): continue
            parts = line.split('=', 1)
            if len(parts) == 2:
                key = parts[0].strip()
                val = parts[1].strip()
                # we need to adjust paths in patches
                # e.g. path = "../tools/wasm4pm-compat-stub" inside unify-rs becomes path = "tools/wasm4pm-compat-stub"
                path_match = re.search(r'path\s*=\s*"([^"]+)"', val)
                if path_match:
                    old_path = path_match.group(1)
                    new_path = os.path.normpath(os.path.join(os.path.dirname(toml_path), old_path))
                    val = re.sub(r'path\s*=\s*"([^"]+)"', f'path = "{new_path}"', val)
                patches[key] = val

root_content = """[workspace]
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
cargo-cicd = { git = "https://github.com/seanchatmangpt/cargo-cicd.git" }
"""

for k, v in deps.items():
    root_content += f"{k} = {v}\n"

if patches:
    root_content += "\n[patch.crates-io]\n"
    for k, v in patches.items():
        root_content += f"{k} = {v}\n"

with open('Cargo.toml', 'w') as f:
    f.write(root_content)

# Now delete the inner workspace Cargo.tomls that don't have [package]
for toml_path in workspace_tomls:
    if os.path.exists(toml_path):
        with open(toml_path, 'r') as f:
            if '[package]' not in f.read():
                os.remove(toml_path)
                print(f"Removed pure virtual manifest {toml_path}")

print("Root Cargo.toml created.")
