import re

# Resolve unify-mcp/src/tools.rs
with open('unify-rs/unify-mcp/src/tools.rs', 'r') as f:
    tools_content = f.read()

tools_resolved = re.sub(
    r'<<<<<<< HEAD.*?=======\n(.*?)\n>>>>>>> origin/claude/[^\n]+',
    r'\1',
    tools_content,
    flags=re.DOTALL
)

with open('unify-rs/unify-mcp/src/tools.rs', 'w') as f:
    f.write(tools_resolved)

# Resolve unify/src/commands.rs
with open('unify-rs/unify/src/commands.rs', 'r') as f:
    commands_content = f.read()

# 1. cmd_gate (Keep HEAD)
commands_content = re.sub(
    r'<<<<<<< HEAD\n(pub fn cmd_gate.*?)\n=======\n.*?>>>>>>> origin/claude/[^\n]+',
    r'\1',
    commands_content,
    flags=re.DOTALL
)

# 2. cmd_query (Keep HEAD)
commands_content = re.sub(
    r'<<<<<<< HEAD\n(pub fn cmd_query.*?)\n=======\n.*?>>>>>>> origin/claude/[^\n]+',
    r'\1',
    commands_content,
    flags=re.DOTALL
)

# 3. For cmd_witnesses, we kept HEAD in the previous step because the conflict 
# started *before* cmd_query and ended *after* cmd_witnesses.
# Wait, let me check the original git conflict layout!
