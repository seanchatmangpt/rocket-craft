# Analysis: UE4 Networking Subsystem Domain Modeling (Replication & RPCs)

**Status:** PARTIAL
**Object under test:** UE4 Networking Subsystem Schema & Validations (`subsystems.ttl` and `validation.shacl.ttl`)
**Observed evidence:** File `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl` and `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`
**Failure:** Identified multiple modeling gaps and validation omission areas in the replication and RPC systems.
**Repair:** Drafted proposed schema extensions and 8 new SPARQL-based SHACL/validation rules.
**Receipt required:** Validation of updated ontology and SHACL shapes with `ggen sync --validate-only true` returning exit code 0.
**Residuals:** Verification of physical headers generation and Playwright browser execution deltas under the new constraints.

---

## 1. Direct Observations

Through forensic analysis of `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl`, `/Users/sac/.ggen/packs/ue4_ontology/core.ttl`, and `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`, the following modeling details were verified:

1. **Replication Lifetimes**: 
   - `ue4:ELifetimeCondition` defines 13 standard conditions (e.g., `COND_None`, `COND_InitialOnly`, `COND_OwnerOnly`, etc.) matching the native UE4 engine enum.
   - `ue4:FReplicationLifetime` represents a lifetime registration with properties `replicatedProperty`, `lifetimeCondition`, and `repNotifyFunction`.
   - **Gap**: There is no property linking `FReplicationLifetime` to a `UClass` or `UActorComponent` class.
2. **Actor and Component Replication**:
   - `ue4:bReplicates` is defined in `core.ttl` as a boolean property.
   - `ue4:ComponentReplicationOwnerShape` verifies that if a component replicates, its owner actor must also replicate.
   - **Gap**: If a component has no owner actor assigned in the graph (i.e. is an orphan), the shape query silently bypasses it instead of flagging it as an error.
3. **RPC Definitions**:
   - RPC types `UServerRPC`, `UClientRPC`, and `UNetMulticastRPC` inherit from `ue4:URPC`.
   - Properties `bReliable`, `bWithValidation`, and `validationFunction` are defined.
   - **Gaps**: 
     - No constraint checks if `URPC` returns `void` (i.e., must not have `ue4:returnProperty`).
     - No constraint prevents `bWithValidation` and `validationFunction` on client/multicast RPCs.
     - No constraint enforces that `UServerRPC` must have validation (for security/anti-cheat compliance).
     - No concept of actor-to-actor ownership (`AActor::GetOwner()`) or net roles (`ENetRole`) exists in the graph.

---

## 2. Modeling Gaps & Inconsistencies

### Gap 1: Absence of Class-to-ReplicationLifetime Mapping
- **Description**: In Unreal C++, property replication is registered per-class inside `GetLifetimeReplicatedProps`. Currently, `FReplicationLifetime` instances float in the graph with no relationship link back to the registering `UClass`.
- **Consequence**: If two different subclasses inherit the same property but register it with different replication conditions (e.g., `COND_None` vs. `COND_OwnerOnly`), the graph cannot resolve which class owns which configuration.
- **Proposed Solution**: Introduce `ue4:hasReplicationLifetime` to relate `ue4:UClass` to `ue4:FReplicationLifetime`.

### Gap 2: Absence of Actor-to-Actor Ownership Relation
- **Description**: Network ownership (`AActor::GetOwner()`) determines connection routing for RPCs and replication relevancy. The ontology currently defines `ue4:owner` as component-to-actor only (domain `UActorComponent`, range `AActor`).
- **Consequence**: Cannot model or validate client-to-server RPC permissions, possession boundaries, or replication relevancy groups.
- **Proposed Solution**: Rename component owner to `ue4:componentOwner` or define a dedicated `ue4:actorOwner` object property with domain `ue4:AActor` and range `ue4:AActor`.

### Gap 3: Missing ENetRole and Actor Role Properties
- **Description**: Unreal Engine relies on network roles (`ROLE_Authority`, `ROLE_AutonomousProxy`, `ROLE_SimulatedProxy`) to branch execution. These do not exist in the ontology.
- **Consequence**: Code generation and static analysis templates cannot verify local/remote execution context correctness.
- **Proposed Solution**: Define class `ue4:ENetRole` subclassing `ue4:UEnum`, define individuals (`ROLE_None`, `ROLE_SimulatedProxy`, `ROLE_AutonomousProxy`, `ROLE_Authority`), and add properties `ue4:role` and `ue4:remoteRole` on `ue4:AActor`.

### Gap 4: Missing AController, APlayerController, and Connection Constructs
- **Description**: Player controllers represent the connection interface. The controller classes are missing.
- **Consequence**: Cannot model user input actuation pathways or trace connection ownership.
- **Proposed Solution**: Add `ue4:AController` subclassing `ue4:AActor`, and `ue4:APlayerController` subclassing `ue4:AController`.

---

## 3. Proposed Refinements (Turtle Ontology Additions)

Below are the exact proposed additions to `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl` to resolve the modeling gaps:

```turtle
# --- Class-to-ReplicationLifetime Relationship ---
ue4:hasReplicationLifetime a owl:ObjectProperty ;
    rdfs:label "hasReplicationLifetime" ;
    rdfs:comment "Relates a UClass to its registered property replication lifetimes." ;
    rdfs:domain ue4:UClass ;
    rdfs:range ue4:FReplicationLifetime .

# --- Actor-to-Actor Ownership ---
ue4:actorOwner a owl:ObjectProperty ;
    rdfs:label "actorOwner" ;
    rdfs:comment "Relates an actor to the owner actor that controls its network authority/routing." ;
    rdfs:domain ue4:AActor ;
    rdfs:range ue4:AActor .

# --- Network Roles ---
ue4:ENetRole a owl:Class ;
    rdfs:subClassOf ue4:UEnum ;
    rdfs:label "ENetRole" ;
    rdfs:comment "Enum representing network replication authority and control roles." .

ue4:ROLE_None a ue4:ENetRole ; rdfs:label "ROLE_None" ; rdfs:comment "No role. Not replicated." .
ue4:ROLE_SimulatedProxy a ue4:ENetRole ; rdfs:label "ROLE_SimulatedProxy" ; rdfs:comment "Simulated network proxy clone." .
ue4:ROLE_AutonomousProxy a ue4:ENetRole ; rdfs:label "ROLE_AutonomousProxy" ; rdfs:comment "Locally controlled player pawn proxy." .
ue4:ROLE_Authority a ue4:ENetRole ; rdfs:label "ROLE_Authority" ; rdfs:comment "Authoritative copy (usually on the server)." .

ue4:role a owl:ObjectProperty ;
    rdfs:label "role" ;
    rdfs:comment "The local network role of the Actor." ;
    rdfs:domain ue4:AActor ;
    rdfs:range ue4:ENetRole .

ue4:remoteRole a owl:ObjectProperty ;
    rdfs:label "remoteRole" ;
    rdfs:comment "The remote network role of the Actor." ;
    rdfs:domain ue4:AActor ;
    rdfs:range ue4:ENetRole .

# --- Controllers ---
ue4:AController a owl:Class ;
    rdfs:subClassOf ue4:AActor ;
    rdfs:label "AController" ;
    rdfs:comment "Controllers are non-physical actors that can possess a Pawn to control its actions." .

ue4:APlayerController a owl:Class ;
    rdfs:subClassOf ue4:AController ;
    rdfs:label "APlayerController" ;
    rdfs:comment "Player controllers represent human players and manage networking connections." .

ue4:possesses a owl:ObjectProperty ;
    rdfs:label "possesses" ;
    rdfs:comment "Relates a controller to the pawn it possesses." ;
    rdfs:domain ue4:AController ;
    rdfs:range ue4:APawn .

ue4:possessedBy a owl:ObjectProperty ;
    rdfs:label "possessedBy" ;
    rdfs:comment "Relates a pawn to the controller possessing it." ;
    rdfs:domain ue4:APawn ;
    rdfs:range ue4:AController ;
    owl:inverseOf ue4:possesses .
```

---

## 4. Proposed Validation Rules (SHACL/SPARQL)

To harden the ontology, the following custom verification rules should be appended to `validation.shacl.ttl` (as SHACL shapes) and `ggen.toml` (as custom validation rules):

### Rule 1: RPC Return Type Must Be Void
- **Logic**: Remote Procedure Calls are executed asynchronously and cannot return a value.
- **SPARQL**:
```sparql
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
SELECT $this ?retProp
WHERE {
    $this a/rdfs:subClassOf* ue4:URPC .
    $this ue4:returnProperty ?retProp .
}
```

### Rule 2: Validation Prohibited on Client/Multicast RPCs
- **Logic**: In UE4, `WithValidation` is only legal on Server RPCs. Specifying it on Client or Multicast RPCs is a compilation failure.
- **SPARQL**:
```sparql
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
SELECT $this
WHERE {
    $this a ?type .
    FILTER (?type = ue4:UClientRPC || ?type = ue4:UNetMulticastRPC)
    {
        $this ue4:bWithValidation true .
    }
    UNION
    {
        $this ue4:validationFunction ?func .
    }
}
```

### Rule 3: Server RPCs Must Have Validation (Anti-Cheat Compliance)
- **Logic**: Any Server RPC sent from a client is a vector for exploits. Standard production guidelines mandate validation functions.
- **SPARQL**:
```sparql
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
SELECT $this
WHERE {
    $this a/rdfs:subClassOf* ue4:UServerRPC .
    FILTER NOT EXISTS {
        $this ue4:bWithValidation true ;
              ue4:validationFunction ?valFunc .
    }
}
```

### Rule 4: Strengthened Component Replication Owner Shape
- **Logic**: Catch orphaned replicated components as well as replicated components whose owner actor does not replicate.
- **SPARQL**:
```sparql
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
SELECT $this ?actor
WHERE {
    $this a/rdfs:subClassOf* ue4:UActorComponent ;
          ue4:bReplicates true .
    {
        $this ue4:isComponentOf ?actor .
        FILTER NOT EXISTS { ?actor ue4:bReplicates true }
    }
    UNION
    {
        FILTER NOT EXISTS { $this ue4:isComponentOf ?anyActor }
    }
}
```

### Rule 5: Replication Condition Consistency Shape
- **Logic**: Properties must not have replication conditions or lifetimes assigned if they are not explicitly declared as replicating (`bReplicated true`).
- **SPARQL**:
```sparql
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
SELECT $this ?prop
WHERE {
    {
        # Direct condition check
        $this a/rdfs:subClassOf* ue4:UProperty ;
              ue4:lifetimeCondition ?cond .
        BIND($this AS ?prop)
    }
    UNION
    {
        # Lifetime registration check
        $this a/rdfs:subClassOf* ue4:FReplicationLifetime ;
              ue4:replicatedProperty ?prop .
    }
    FILTER NOT EXISTS { ?prop ue4:bReplicated true }
}
```

### Rule 6: RepNotify Requires Replication
- **Logic**: RepNotify callbacks are only invoked when a property is replicated. Declaring them on non-replicated properties is invalid.
- **SPARQL**:
```sparql
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
SELECT $this ?prop
WHERE {
    {
        # Direct property RepNotify
        $this a/rdfs:subClassOf* ue4:UProperty ;
              ue4:repNotifyFunction ?func .
        BIND($this AS ?prop)
    }
    UNION
    {
        # Lifetime registration RepNotify
        $this a/rdfs:subClassOf* ue4:FReplicationLifetime ;
              ue4:replicatedProperty ?prop ;
              ue4:repNotifyFunction ?func .
    }
    FILTER NOT EXISTS { ?prop ue4:bReplicated true }
}
```

### Rule 7: Complete Replicated Property Specification
- **Logic**: Replicated properties must either specify a direct lifetime condition or have a corresponding replication lifetime registration instance in the graph.
- **SPARQL**:
```sparql
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
SELECT $this
WHERE {
    $this a/rdfs:subClassOf* ue4:UProperty ;
          ue4:bReplicated true .
    FILTER NOT EXISTS {
        { $this ue4:lifetimeCondition ?anyCond }
        UNION
        { ?lifetime ue4:replicatedProperty $this }
    }
}
```

### Rule 8: RPC Parameter Object Type Safety
- **Logic**: Object parameters passed to RPCs must inherit from `AActor` or `UActorComponent` to ensure network indexability via NetGUIDs.
- **SPARQL**:
```sparql
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
SELECT $this ?param ?paramType
WHERE {
    $this a/rdfs:subClassOf* ue4:URPC ;
          ue4:hasParameter ?param .
    ?param a/rdfs:subClassOf* ue4:UObjectProperty ;
           ue4:propertyType ?paramType .
    FILTER NOT EXISTS {
        { ?paramType rdfs:subClassOf* ue4:AActor }
        UNION
        { ?paramType rdfs:subClassOf* ue4:UActorComponent }
    }
}
```
