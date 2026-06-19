# Project: UE4 Universal RDF Mapping

## Architecture
This project represents the complete architecture and class hierarchy of Unreal Engine 4 (UE4) as a mathematically unified semantic graph in RDF (Turtle format). The output is a cohesive pack of ontologies and verification configurations stored in `/Users/sac/.ggen/packs/ue4_ontology`.

The ontology architecture unifies static C++ inheritance metadata with dynamic reflection networks, Blueprint graphs, subsystem boundaries (rendering, physics, networking), and compilation/packaging typestates.

```
       ┌─────────────────────────────────────────────────────────┐
       │                 UE4 Universal Ontology                  │
       └────────────────────────────┬────────────────────────────┘
                                    │
         ┌──────────────────────────┼──────────────────────────┐
         ▼                          ▼                          ▼
┌──────────────────┐       ┌──────────────────┐       ┌──────────────────┐
│     core.ttl     │       │  reflection.ttl  │       │  blueprints.ttl  │
│ (C++ Backbone)   │       │ (Reflection Meta)│       │(Blueprint Graphs)│
└──────────────────┘       └──────────────────┘       └──────────────────┘
         │                          │                          │
         └──────────────────────────┼──────────────────────────┘
                                    ▼
                       ┌──────────────────────────┐
                       │       subsystems.ttl     │
                       │ (Physics, Rendering, Net)│
                       └────────────┬─────────────┘
                                    │
                                    ▼
                       ┌──────────────────────────┐
                       │      typestates.ttl      │
                       │ (Cooking, Linking, WASM) │
                       └──────────────────────────┘
```

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | E2E Test Suite & Infra | Design E2E validation suite, setup ggen.toml, publish TEST_READY.md | None | IN_PROGRESS (Conv: 533f9425-b3c5-4c1c-9afc-ec03bf4fb344) |
| 2 | Core C++ Backbone | Author core.ttl modeling UObject, AActor, UActorComponent class hierarchy | M1 | PLANNED |
| 3 | Reflection & Blueprints | Author reflection.ttl and blueprints.ttl mapping UClass metadata and Blueprint execution graphs | M2 | PLANNED |
| 4 | Subsystem Topologies | Author subsystems.ttl mapping Rendering, Physics, and Networking domains | M3 | PLANNED |
| 5 | Cooking & Typestates | Author typestates.ttl representing compilation and WASM packaging pipelines | M4 | PLANNED |
| 6 | E2E Validation Pass | Verify 100% coverage and validation of all ontologies (Tiers 1-4) | M5 | PLANNED |
| 7 | Adversarial Hardening | Implement Tier 5 adversarial testing, verify zero ontological gaps | M6 | PLANNED |

## Interface Contracts
- **Ontology Directory**: All Turtle files must reside in `/Users/sac/.ggen/packs/ue4_ontology/`.
- **Validation Authority**: An independent execution of `ggen sync --validate-only` in the output pack folder must terminate with exit code 0.
- **Header Generation Readiness**: The graph must contain sufficient class, property, and relation definitions to generate standard C++ class headers.
- **Completeness**: Explicit mappings of class hierarchies (`UObject` -> `AActor` -> `APawn` -> `ACharacter`) and dynamic reflection definitions.
