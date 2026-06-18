import os
import re

all_members = []

for root, dirs, files in os.walk('.'):
    if '.git' in dirs: dirs.remove('.git')
    if 'target' in dirs: dirs.remove('target')
    
    for file in files:
        if file == 'Cargo.toml' and root != '.':
            filepath = os.path.join(root, file)
            with open(filepath, 'r') as f:
                content = f.read()
            if '[package]' in content:
                # normalize path to forward slashes and strip leading ./
                rel_path = filepath.replace('./', '', 1).replace('/Cargo.toml', '')
                all_members.append(rel_path)

members_str = "members = [\n"
for m in sorted(all_members):
    members_str += f'    "{m}",\n'
members_str += "]"

with open('Cargo.toml', 'r') as f:
    content = f.read()

content = re.sub(r'members\s*=\s*\[.*?\]', members_str, content, flags=re.DOTALL)

with open('Cargo.toml', 'w') as f:
    f.write(content)

