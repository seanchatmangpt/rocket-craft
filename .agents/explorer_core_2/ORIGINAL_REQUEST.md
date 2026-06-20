## 2026-06-19T00:40:23Z
You are explorer_core_2. Your working directory is `/Users/sac/rocket-craft/.agents/explorer_core_2`.
Please initialize your progress.md and context.md.

## Mission
Analyze the requirements for the Core C++ Backbone ontology (`core.ttl`) for Unreal Engine 4 (UE4). Develop a detailed schema design and fix strategy.

## Scope
- Model UObject, AActor, APawn, ACharacter, UActorComponent, UWorld, ULevel, and their inheritance hierarchy.
- Design relevant properties, relationships, and metadata (like labels and descriptions) conforming to SHACL rules (labels and comments required, namespace: `https://rocket-craft.io/ontology/ue4/`).
- Do NOT write the final file directly to `/Users/sac/.ggen/packs/ue4_ontology/core.ttl` (as you are read-only). Only write your recommendation to `analysis.md` in your directory.

## Input Information
- PROJECT.md: `/Users/sac/rocket-craft/PROJECT.md`
- TEST_INFRA.md: `/Users/sac/rocket-craft/TEST_INFRA.md`
- ggen.toml: `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml`
- shacl rules: `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`

Report completion and handoff path to parent ID: 4f79cb22-2adb-466d-9e20-d8baef6e934d.
