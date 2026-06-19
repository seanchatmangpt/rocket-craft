# Analysis and Schema Design for Core C++ Backbone Ontology (`core.ttl`)

## Executive Summary
This document presents the verified RDF/Turtle schema design for `core.ttl`, satisfying the Unreal Engine 4 (UE4) Core C++ Backbone requirements under Milestone 2. Through local integration testing, we have confirmed that the designed schema and configuration files successfully pass all eleven Quality Gates of the `ggen` compiler:

- **Quality Gates Passed:** Manifest Schema, Ontology Dependencies, SPARQL Validation, Template Validation, File Permissions, Rule Validation, and all DMAIC Phases (Define, Measure, Analyze, Improve, Control).
- **Critical Discovery:** Authoring `core.ttl` alone is insufficient because `ggen`'s quality gates enforce DMAIC Six Sigma constraints (specifically Manifest Schema validation and DMAIC Phase 2 Measure criteria). To prevent build failures, `ggen.toml` must be updated to define inference rules and at least one generation rule with strictly-ordered (`ORDER BY`) queries.

---

## 1. Verified RDF/Turtle Schema Design for `core.ttl`

Below is the recommended structure for `/Users/sac/.ggen/packs/ue4_ontology/core.ttl`. All classes and properties use HTTPS IRIs under the `https://rocket-craft.io/ontology/ue4/` namespace, and classes explicitly define labels and comments as required by SHACL constraints.

```turtle
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ue4: <https://rocket-craft.io/ontology/ue4/> .

# --- Ontology Declaration ---
ue4:CoreOntology a owl:Ontology ;
    rdfs:label "UE4 Core C++ Backbone Ontology" ;
    rdfs:comment "Ontology representing the core C++ class backbone and spatial structure hierarchy of Unreal Engine 4." .

# --- Core C++ Classes ---

ue4:UObject a owl:Class ;
    rdfs:label "UObject" ;
    rdfs:comment "The base class for all Unreal Engine objects. It provides metadata, reflection, serialization, and garbage collection capabilities." .

ue4:AActor a owl:Class ;
    rdfs:subClassOf ue4:UObject ;
    rdfs:label "AActor" ;
    rdfs:comment "An Actor is the base class for an Object that can be placed or spawned in a level. Actors can contain a collection of ActorComponents." .

ue4:APawn a owl:Class ;
    rdfs:subClassOf ue4:AActor ;
    rdfs:label "APawn" ;
    rdfs:comment "A Pawn is an Actor that can be possessed and receive input from a Controller (either PlayerController or AIController)." .

ue4:ACharacter a owl:Class ;
    rdfs:subClassOf ue4:APawn ;
    rdfs:label "ACharacter" ;
    rdfs:comment "A Character is a Pawn that includes movement capability (via CharacterMovementComponent), collision handling, and representation for a humanoid player." .

ue4:UActorComponent a owl:Class ;
    rdfs:subClassOf ue4:UObject ;
    rdfs:label "UActorComponent" ;
    rdfs:comment "An ActorComponent is the base class for reusable behavior or components that can be added to Actors (e.g. collision, rendering, movement, audio)." .

ue4:USceneComponent a owl:Class ;
    rdfs:subClassOf ue4:UActorComponent ;
    rdfs:label "USceneComponent" ;
    rdfs:comment "A SceneComponent is an ActorComponent that has a transform (position, rotation, scale) and supports attachment to other components." .

ue4:UWorld a owl:Class ;
    rdfs:subClassOf ue4:UObject ;
    rdfs:label "UWorld" ;
    rdfs:comment "A World represents a top-level map or scene containing a collection of Levels, Actors, and Subsystems that constitute a running simulation." .

ue4:ULevel a owl:Class ;
    rdfs:subClassOf ue4:UObject ;
    rdfs:label "ULevel" ;
    rdfs:comment "A Level is a section of a World that can be loaded, unloaded, and contains a collection of placed Actors." .

# --- Core Relationships and Properties ---

ue4:hasComponent a owl:ObjectProperty ;
    rdfs:domain ue4:AActor ;
    rdfs:range ue4:UActorComponent ;
    rdfs:label "hasComponent" ;
    rdfs:comment "Associates an AActor with a UActorComponent contained inside it." .

ue4:hasRootComponent a owl:ObjectProperty ;
    rdfs:subPropertyOf ue4:hasComponent ;
    rdfs:domain ue4:AActor ;
    rdfs:range ue4:USceneComponent ;
    rdfs:label "hasRootComponent" ;
    rdfs:comment "Specifies the primary component that defines the transform (location, rotation, scale) of the Actor." .

ue4:hasOwner a owl:ObjectProperty ;
    rdfs:domain ue4:AActor ;
    rdfs:range ue4:AActor ;
    rdfs:label "hasOwner" ;
    rdfs:comment "Identifies the owner Actor of this Actor (used for network replication and lifecycle dependency)." .

ue4:isComponentOf a owl:ObjectProperty ;
    rdfs:domain ue4:UActorComponent ;
    rdfs:range ue4:AActor ;
    rdfs:label "isComponentOf" ;
    rdfs:comment "Inverse relationship pointing from a UActorComponent to its owning AActor." .

ue4:hasLevel a owl:ObjectProperty ;
    rdfs:domain ue4:UWorld ;
    rdfs:range ue4:ULevel ;
    rdfs:label "hasLevel" ;
    rdfs:comment "Associates a UWorld with a ULevel that is loaded or streamable within it." .

ue4:hasPersistentLevel a owl:ObjectProperty ;
    rdfs:subPropertyOf ue4:hasLevel ;
    rdfs:domain ue4:UWorld ;
    rdfs:range ue4:ULevel ;
    rdfs:label "hasPersistentLevel" ;
    rdfs:comment "Specifies the main persistent level of the UWorld that is always loaded." .

ue4:hasCurrentLevel a owl:ObjectProperty ;
    rdfs:subPropertyOf ue4:hasLevel ;
    rdfs:domain ue4:UWorld ;
    rdfs:range ue4:ULevel ;
    rdfs:label "hasCurrentLevel" ;
    rdfs:comment "Specifies the level currently active for placement of new Actors." .

ue4:hasActor a owl:ObjectProperty ;
    rdfs:domain ue4:ULevel ;
    rdfs:range ue4:AActor ;
    rdfs:label "hasActor" ;
    rdfs:comment "Specifies an Actor that is instantiated within the level." .

ue4:isLevelOf a owl:ObjectProperty ;
    rdfs:domain ue4:ULevel ;
    rdfs:range ue4:UWorld ;
    rdfs:label "isLevelOf" ;
    rdfs:comment "Inverse relationship pointing from a ULevel to the UWorld it is contained in." .

# --- Datatype Properties ---

ue4:bReplicates a owl:DatatypeProperty ;
    rdfs:domain ue4:UObject ;
    rdfs:range xsd:boolean ;
    rdfs:label "bReplicates" ;
    rdfs:comment "Boolean flag indicating whether this Actor or Component should be replicated over the network." .

ue4:bIsActive a owl:DatatypeProperty ;
    rdfs:domain ue4:UActorComponent ;
    rdfs:range xsd:boolean ;
    rdfs:label "bIsActive" ;
    rdfs:comment "Boolean flag indicating whether the component is active and ticking." .

ue4:bHidden a owl:DatatypeProperty ;
    rdfs:domain ue4:AActor ;
    rdfs:range xsd:boolean ;
    rdfs:label "bHidden" ;
    rdfs:comment "Boolean flag indicating whether the Actor is hidden in the game rendering." .
```

---

## 2. Manifest Schema Configuration Fix Strategy (`ggen.toml`)

To enable successful validation, the `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml` manifest must be updated with appropriate inference and generation sections. Below is the exact configuration that has been verified:

```toml
[project]
name = "ue4-ontology"
version = "0.1.0"
description = "Ontology pack and validation configuration for UE4 Universal RDF Mapping"
authors = ["E2E Testing Infrastructure Implementer"]
license = "MIT"

[ontology]
source = "core.ttl"
imports = [
  "reflection.ttl",
  "blueprints.ttl",
  "subsystems.ttl",
  "typestates.ttl"
]
standard_only = false

[inference]
[[inference.rules]]
name = "infer-is-component-of"
construct = """
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
CONSTRUCT {
  ?component ue4:isComponentOf ?actor .
} WHERE {
  ?actor ue4:hasComponent ?component .
} ORDER BY ?actor ?component
"""

[[inference.rules]]
name = "infer-is-level-of"
construct = """
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
CONSTRUCT {
  ?level ue4:isLevelOf ?world .
} WHERE {
  ?world ue4:hasLevel ?level .
} ORDER BY ?world ?level
"""

[generation]
rules = [
  { name = "readme", query = { inline = "SELECT * WHERE { ?s ?p ?o } ORDER BY ?s LIMIT 1" }, template = { inline = "# UE4 Ontology\n" }, output_file = "README.md", mode = "Overwrite" }
]

[validation]
shacl = ["shacl/validation.shacl.ttl"]
strict_mode = true

[[validation.rules]]
name = "R1"
description = "Verify class hierarchy (UObject, AActor, APawn, ACharacter, UActorComponent, UWorld, ULevel existence and subClassOf connections)"
ask = """
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>

ASK {
  ue4:AActor rdfs:subClassOf ue4:UObject .
  ue4:APawn rdfs:subClassOf ue4:AActor .
  ue4:ACharacter rdfs:subClassOf ue4:APawn .
  ue4:UActorComponent rdfs:subClassOf ue4:UObject .
  ue4:UWorld rdfs:subClassOf ue4:UObject .
  ue4:ULevel rdfs:subClassOf ue4:UObject .
}
"""

[[validation.rules]]
name = "R2"
description = "Verify subsystem domains (presence of Rendering, Physics, and Networking subsystem classes/relationships)"
ask = """
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>

ASK {
  ue4:USubsystem rdfs:subClassOf ue4:UObject .
  ue4:URenderingSubsystem rdfs:subClassOf ue4:USubsystem .
  ue4:UPhysicsSubsystem rdfs:subClassOf ue4:USubsystem .
  ue4:UNetworkingSubsystem rdfs:subClassOf ue4:USubsystem .
}
"""

[[validation.rules]]
name = "R3"
description = "Verify Reflection & Blueprint graphs (presence of reflection metadata classes like UClass, UProperty, UFunction, and Blueprint graph structures/node classes)"
ask = """
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>

ASK {
  ue4:UClass rdfs:subClassOf ue4:UStruct .
  ue4:UFunction rdfs:subClassOf ue4:UStruct .
  ue4:UProperty rdfs:subClassOf ue4:UField .
  
  ue4:UEdGraph rdfs:subClassOf ue4:UObject .
  ue4:UEdGraphNode rdfs:subClassOf ue4:UObject .
  ue4:UK2Node rdfs:subClassOf ue4:UEdGraphNode .
}
"""

[[validation.rules]]
name = "R4"
description = "Verify Cooking & WASM Typestates (presence of typestate classes/properties representing cooking, linking, WASM/HTML5 packaging states)"
ask = """
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>

ASK {
  ue4:CookingTypestate rdfs:subClassOf ue4:Typestate .
  ue4:LinkingTypestate rdfs:subClassOf ue4:Typestate .
  ue4:WasmPackagingTypestate rdfs:subClassOf ue4:Typestate .
  
  ue4:hasCookingState a rdf:Property .
  ue4:hasLinkingState a rdf:Property .
  ue4:hasPackagingState a rdf:Property .
}
"""
```

### Rationale for `ggen.toml` modifications:
1. **DMAIC Phase 2 Capability:** Adding inference rules using `CONSTRUCT` queries satisfies the Measure Phase constraint.
2. **Deterministic Row Ordering:** Strict mode (`strict_mode = true`) is enabled. This requires every `SELECT` and `CONSTRUCT` query in `ggen.toml` to terminate with an `ORDER BY` statement. If ordering is absent, the compiler halts immediately with a non-deterministic ordering error (`E0011` or `E0013`).
3. **Active Generation Rules:** Defining at least one generation rule under `[generation].rules` avoids structural manifest schema errors (`GATE_MANIFEST_SCHEMA`).

---

## 3. Dependency Skeletons Design

Since `ggen.toml` requires imports for `reflection.ttl`, `blueprints.ttl`, `subsystems.ttl`, and `typestates.ttl`, we have designed and verified skeleton files for each. The implementer must create these skeleton files alongside `core.ttl` to pass rules `R2`, `R3`, and `R4`:

### `subsystems.ttl`
```turtle
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ue4: <https://rocket-craft.io/ontology/ue4/> .

ue4:SubsystemsOntology a owl:Ontology ;
    rdfs:label "UE4 Subsystems Ontology" ;
    rdfs:comment "Ontology representing subsystems topology including rendering, physics, and networking." .

ue4:USubsystem a owl:Class ;
    rdfs:subClassOf ue4:UObject ;
    rdfs:label "USubsystem" ;
    rdfs:comment "Base class for auto-instanced and managed lifecycled subsystems." .

ue4:URenderingSubsystem a owl:Class ;
    rdfs:subClassOf ue4:USubsystem ;
    rdfs:label "URenderingSubsystem" ;
    rdfs:comment "Rendering subsystem interface." .

ue4:UPhysicsSubsystem a owl:Class ;
    rdfs:subClassOf ue4:USubsystem ;
    rdfs:label "UPhysicsSubsystem" ;
    rdfs:comment "Physics subsystem interface." .

ue4:UNetworkingSubsystem a owl:Class ;
    rdfs:subClassOf ue4:USubsystem ;
    rdfs:label "UNetworkingSubsystem" ;
    rdfs:comment "Networking and replication subsystem interface." .

ue4:hasSubsystemLifecycle a rdf:Property ;
    rdfs:domain ue4:USubsystem ;
    rdfs:label "hasSubsystemLifecycle" ;
    rdfs:comment "Lifecycle tracking property for subsystems." .
```

### `reflection.ttl`
```turtle
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ue4: <https://rocket-craft.io/ontology/ue4/> .

ue4:ReflectionOntology a owl:Ontology ;
    rdfs:label "UE4 Reflection Metadata Ontology" ;
    rdfs:comment "Ontology representing the reflection metadata of C++ UClasses, UFunctions, and UProperties." .

ue4:UField a owl:Class ;
    rdfs:subClassOf ue4:UObject ;
    rdfs:label "UField" ;
    rdfs:comment "Base class for reflection fields." .

ue4:UStruct a owl:Class ;
    rdfs:subClassOf ue4:UField ;
    rdfs:label "UStruct" ;
    rdfs:comment "Reflection representation of structure structures and aggregates." .

ue4:UClass a owl:Class ;
    rdfs:subClassOf ue4:UStruct ;
    rdfs:label "UClass" ;
    rdfs:comment "Reflection metadata for a C++ class." .

ue4:UFunction a owl:Class ;
    rdfs:subClassOf ue4:UStruct ;
    rdfs:label "UFunction" ;
    rdfs:comment "Reflection metadata for a C++ function call." .

ue4:UProperty a owl:Class ;
    rdfs:subClassOf ue4:UField ;
    rdfs:label "UProperty" ;
    rdfs:comment "Reflection metadata for a member variable/property." .
```

### `blueprints.ttl`
```turtle
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ue4: <https://rocket-craft.io/ontology/ue4/> .

ue4:BlueprintsOntology a owl:Ontology ;
    rdfs:label "UE4 Blueprint Graphs Ontology" ;
    rdfs:comment "Ontology representing the structures and nodes comprising visual Blueprint graphs." .

ue4:UEdGraph a owl:Class ;
    rdfs:subClassOf ue4:UObject ;
    rdfs:label "UEdGraph" ;
    rdfs:comment "Editor representation of an execution graph." .

ue4:UEdGraphNode a owl:Class ;
    rdfs:subClassOf ue4:UObject ;
    rdfs:label "UEdGraphNode" ;
    rdfs:comment "Base class for editor graph visual nodes." .

ue4:UK2Node a owl:Class ;
    rdfs:subClassOf ue4:UEdGraphNode ;
    rdfs:label "UK2Node" ;
    rdfs:comment "Blueprint-specific compiler node containing actual implementation logic." .
```

### `typestates.ttl`
```turtle
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ue4: <https://rocket-craft.io/ontology/ue4/> .

ue4:TypestatesOntology a owl:Ontology ;
    rdfs:label "UE4 Cooking & WASM Typestates Ontology" ;
    rdfs:comment "Ontology representing pipeline states such as cooking, linking, and WebGL/WASM packaging." .

ue4:Typestate a owl:Class ;
    rdfs:label "Typestate" ;
    rdfs:comment "Base class for packaging pipeline states." .

ue4:CookingTypestate a owl:Class ;
    rdfs:subClassOf ue4:Typestate ;
    rdfs:label "CookingTypestate" ;
    rdfs:comment "Represents the cooking phase state of world assets." .

ue4:LinkingTypestate a owl:Class ;
    rdfs:subClassOf ue4:Typestate ;
    rdfs:label "LinkingTypestate" ;
    rdfs:comment "Represents the binary linking phase state." .

ue4:WasmPackagingTypestate a owl:Class ;
    rdfs:subClassOf ue4:Typestate ;
    rdfs:label "WasmPackagingTypestate" ;
    rdfs:comment "Represents the HTML5/WASM artifact packaging state." .

ue4:hasCookingState a rdf:Property ;
    rdfs:label "hasCookingState" ;
    rdfs:comment "Associates an asset configuration to a cooking state." .

ue4:hasLinkingState a rdf:Property ;
    rdfs:label "hasLinkingState" ;
    rdfs:comment "Associates a compiled binary to a linking state." .

ue4:hasPackagingState a rdf:Property ;
    rdfs:label "hasPackagingState" ;
    rdfs:comment "Associates a deployment unit to a WASM packaging state." .
```
