# Context Log — explorer_core_1

## Environment Details
- **User OS**: mac
- **Active Workspace**: `/Users/sac/rocket-craft`
- **Output Pack Directory**: `/Users/sac/.ggen/packs/ue4_ontology`
- **Read-Only Mode**: Yes (Explorer archetype)

## Key Inputs
- **Project Blueprint**: `/Users/sac/rocket-craft/PROJECT.md`
- **Test Infrastructure**: `/Users/sac/rocket-craft/TEST_INFRA.md`
- **Ggen Pack Config**: `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml`
- **SHACL Shapes**: `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`

## Targets of Analysis
- **Core Backbone Ontology File**: `core.ttl` (to be written to `/Users/sac/.ggen/packs/ue4_ontology/core.ttl` by implementer, analyzed by us)
- **Classes to Model**:
  - `ue4:UObject` (Root)
  - `ue4:AActor`
  - `ue4:APawn`
  - `ue4:ACharacter`
  - `ue4:UActorComponent`
  - `ue4:UWorld`
  - `ue4:ULevel`
- **Inheritance Hierarchy**:
  - `ue4:AActor` subclass of `ue4:UObject`
  - `ue4:APawn` subclass of `ue4:AActor`
  - `ue4:ACharacter` subclass of `ue4:APawn`
  - `ue4:UActorComponent` subclass of `ue4:UObject`
  - `ue4:UWorld` subclass of `ue4:UObject`
  - `ue4:ULevel` subclass of `ue4:UObject`
