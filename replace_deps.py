import os
import re

mappings = [
    (r'chicago-tdd-tools\s*=\s*\{\s*path\s*=\s*"[^"]+"\s*\}', 'chicago-tdd-tools = { git = "https://github.com/seanchatmangpt/chicago-tdd-tools.git" }'),
    (r'unrdf\s*=\s*\{\s*path\s*=\s*"[^"]+"\s*\}', 'unrdf = { git = "https://github.com/seanchatmangpt/unrdf.git" }'),
    (r'unify-rdf\s*=\s*\{\s*path\s*=\s*"[^"]+"\s*\}', 'unify-rdf = { git = "https://github.com/seanchatmangpt/unrdf.git", package = "unrdf" }'),
    (r'anti-llm-cheat-lsp\s*=\s*\{\s*path\s*=\s*"[^"]+"\s*\}', 'anti-llm-cheat-lsp = { git = "https://github.com/seanchatmangpt/lsp-max", package = "lsp-max" }'),
    (r'lsp-max\s*=\s*"[^"]+"', 'lsp-max = { git = "https://github.com/seanchatmangpt/lsp-max" }'),
    (r'lsp-max-protocol\s*=\s*"[^"]+"', 'lsp-max-protocol = { git = "https://github.com/seanchatmangpt/lsp-max" }'),
    (r'lsp-max-runtime\s*=\s*"[^"]+"', 'lsp-max-runtime = { git = "https://github.com/seanchatmangpt/lsp-max" }')
]

for root, dirs, files in os.walk('.'):
    if '.git' in dirs:
        dirs.remove('.git')
    for file in files:
        if file == 'Cargo.toml':
            filepath = os.path.join(root, file)
            with open(filepath, 'r') as f:
                content = f.read()
            
            modified = False
            for pattern, replacement in mappings:
                if re.search(pattern, content):
                    content = re.sub(pattern, replacement, content)
                    modified = True
            
            if modified:
                with open(filepath, 'w') as f:
                    f.write(content)
                print(f"Updated {filepath}")
