import re

with open('Cargo.toml', 'r') as f:
    content = f.read()

# Replace the members array with the explicit paths we know have Cargo.tomls
explicit_members = [
    "asset-pipeline/pipeline-cli",
    "asset-pipeline/pipeline-core",
    "blueprint-rs/blueprint-cli",
    "blueprint-rs/blueprint-core",
    "blueprint-rs/blueprint-macros",
    "blueprint-rs/blueprint-testing",
    "chicago-tdd-tools",
    "genie3-rs",
    "infinity-blade-4/mud",
    "nexus-engine/crates/nexus-gfx",
    "nexus-engine/crates/nexus-tests",
    "rocket-simulator/simulator-core",
    "tools/knhk",
    "tools/rocket-cmd",
    "tools/rocket-sdk",
    "tools/un-test-utils",
    "tools/unrdf",
    "tools/wasm4pm-compat-stub",
    "unify-rs/anti-llm-cheat-lsp",
    "unify-rs/genie-core",
    "unify-rs/unify",
    "unify-rs/unify-admission",
    "unify-rs/unify-automl",
    "unify-rs/unify-bp",
    "unify-rs/unify-cli",
    "unify-rs/unify-codegen",
    "unify-rs/unify-config",
    "unify-rs/unify-core",
    "unify-rs/unify-ffi",
    "unify-rs/unify-integration-tests",
    "unify-rs/unify-lsp",
    "unify-rs/unify-macros",
    "unify-rs/unify-mcp",
    "unify-rs/unify-ocel",
    "unify-rs/unify-otel",
    "unify-rs/unify-pm",
    "unify-rs/unify-rdf",
    "unify-rs/unify-receipts",
    "unify-rs/unify-rocket",
    "unify-rs/unify-sem",
    "unify-rs/unify-test",
    "unify-rs/unify-wasm",
    "wasm-threads/wasm-core",
    "wasm-threads/wasm-game-logic",
    "wasm-threads/wasm-patterns",
    "wasm-threads/wasm-tests",
    "wasm-threads/wasm-ui"
]

members_str = "members = [\n"
for m in explicit_members:
    members_str += f'    "{m}",\n'
members_str += "]"

content = re.sub(r'members\s*=\s*\[.*?\]', members_str, content, flags=re.DOTALL)

with open('Cargo.toml', 'w') as f:
    f.write(content)
