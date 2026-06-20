# Context — GC-GUNDAM-FACTORY-001

## Workspace Paths
- **rocket-craft**: `/Users/sac/rocket-craft`
- **powlv2lsp**: `/Users/sac/powlv2lsp`
- **ggen**: `/Users/sac/ggen`
- **wasm4pm**: `/Users/sac/wasm4pm`
- **wasm4pm-compat**: `/Users/sac/wasm4pm-compat`

## Repositories Boundary Law
- `powlv2lsp`: Owns POWL authoring, grammar, traversal, diagnostics, and trace emission.
- `wasm4pm-compat`: Owns canonical structural Rust representations only.
- `wasm4pm`: Owns replay, conformance, OCEL/process verification, and process evidence.
- `ggen`: Owns deterministic manufacturing from admitted semantic/process rows into artifacts.
- `rocket-craft`: Owns Rocket-Craft fixtures, game-law verifier, generated artifacts, UE4 projection harness, Playwright tests, and final verifier reports.
