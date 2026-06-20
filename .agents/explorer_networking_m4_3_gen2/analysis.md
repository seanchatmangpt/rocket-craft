# Analysis: UE4 Networking Domain Semantic Modeling

## Executive Summary
This report analyzes and proposes a semantic representation for the Unreal Engine 4 (UE4) Networking subsystem, specifically focusing on **Replication** (lifetimes, conditions, properties) and **Remote Procedure Calls (RPCs)** (Server, Client, Multicast execution requirements and integrity checking). The recommendations are designed to be integrated into `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl` and validated using custom SHACL shapes and SPARQL rules in `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` and `ggen.toml`.

## Exploration Scope & Rationale
Under UE4's network architecture, server authority is absolute, but replication and remote execution represent complex dynamic behavior. If our RDF representation is to be a true "digital twin" of a working engine, it must enforce the exact physical boundaries of the network layers. Failure to do so would result in the generation of invalid header structures or broken C++ network execution code.

The domains analyzed:
1. **Replication Lifetimes & Conditions:** Modeling how properties are registered via C++ `GetLifetimeReplicatedProps` (using `DOREPLIFETIME_CONDITION`, etc.) and mapped to specific client connection states (`ELifetimeCondition`).
2. **RPC Structure & Integrity:** Modeling RPC qualifiers (`Server`, `Client`, `NetMulticast`), reliability state (`bReliable`), and security structures (`WithValidation`).
3. **Execution Topology Constraints:** Defining validation rules that ensure RPCs are only bound to replicated classes, that component replication requires actor replication, and that worlds containing replicated actors activate the networking subsystem.

---

## 1. Ontological Design Proposals

We propose extending the classes and properties in `subsystems.ttl` as follows:

### 1.1 Replication Modeling
*   **`ue4:ELifetimeCondition`**: Enum representing standard `ELifetimeCondition` states.
    *   *Individuals:* `ue4:COND_None`, `ue4:COND_InitialOnly`, `ue4:COND_OwnerOnly`, `ue4:COND_SkipOwner`, `ue4:COND_SimulatedOnly`, `ue4:COND_AutonomousOnly`, `ue4:COND_SimulatedOrOwner`, `ue4:COND_Custom`, `ue4:COND_ReplayOrOwner`, `ue4:COND_ReplayOnly`, `ue4:COND_SimulatedOnlyNoReplay`, `ue4:COND_SkipOwnerNoReplay`, `ue4:COND_Never`.
*   **`ue4:FReplicationLifetime`**: A structural registration object representing how a property is registered.
*   **`ue4:replicatedProperty`**: Relates `FReplicationLifetime` to a target `ue4:UProperty`.
*   **`ue4:lifetimeCondition`**: Relates replication specifications to `ue4:ELifetimeCondition`.
*   **`ue4:repNotifyFunction`**: Relates a property/specification to a callback `ue4:UFunction` invoked upon replication (`ReplicatedUsing`).
*   **`ue4:bReplicated`**: Boolean flag indicating direct replication status.

### 1.2 Remote Procedure Calls (RPCs)
*   **`ue4:URPC`**: Base class representing Remote Procedure Calls (subclass of `ue4:UFunction`).
    *   **`ue4:UServerRPC`**: RPC initiated by client, executed on server.
    *   **`ue4:UClientRPC`**: RPC initiated by server, executed on owning client.
    *   **`ue4:UNetMulticastRPC`**: RPC initiated by server, executed on server and all client connections.
*   **`ue4:bReliable`**: Boolean flag indicating connection reliability.
*   **`ue4:bWithValidation`**: Boolean flag indicating whether `_Validate()` check is implemented.
*   **`ue4:validationFunction`**: Points to the validation `ue4:UFunction` returning boolean and verifying arguments.

### 1.3 Subsystem Topology
*   **`ue4:UGameInstance`**: Top-level manager class.
*   **`ue4:UWorldSubsystem`, `ue4:UGameInstanceSubsystem`, `ue4:UEngineSubsystem`**: Classes grouping subsystems.
*   **`ue4:hasSubsystem`**: Object property relating World/GameInstance to subsystems.

---

## 2. Integrity and Validation Constraints

To enforce networking soundness, we define 5 core topological rules.

### Rule 1: RPC Location Constraints
*   **Constraint:** An RPC can only be declared on a class that inherits from `AActor` or `UActorComponent`.
*   **Integrity:** Attempting to invoke network replication or RPC routes on a raw `UObject` without actor ownership/replication channels is a fatal runtime failure.
*   **WASM/Playwright Impact:** Any WASM client executing an RPC on a raw object will crash or fail to actuate, violating GATE 5/GATE 6.

### Rule 2: Validation Signature Matching
*   **Constraint:** If an RPC is marked `bWithValidation true`, there must be a validation function (`ue4:validationFunction`) returning a boolean, and its parameter count and types must perfectly match the RPC's signature.
*   **Integrity:** Any parameter count/type mismatch between `Func` and `Func_Validate` will result in compilation failure when building the WASM client.

### Rule 3: Component Replication Ownership
*   **Constraint:** A component (`UActorComponent`) cannot replicate (`bReplicates true`) unless its owning actor (`AActor`) also replicates.
*   **Integrity:** Without actor replication, the actor's channel is never opened, rendering component replication completely inert.

### Rule 4: World Networking Subsystem Topology
*   **Constraint:** A world containing replicated actors must possess an active `UNetworkingSubsystem`.
*   **Integrity:** Guarantees that networking simulation and driver layers are instantiated, serving as a precondition for local server execution.

### Rule 5: RepNotify Callback Soundness
*   **Constraint:** A property's RepNotify callback function (`ue4:repNotifyFunction`) must exist in the class hierarchy of the property, and it must return void (i.e. have no returnProperty).
*   **Integrity:** Callback functions in UE4 triggered by network replication cannot return a value, and must be callable on the instance.

---

## 3. Proposal Files Reference

Detailed implementations of these proposals have been created in the working directory:
1. **`proposed_subsystems.ttl`**: The Turtle vocabulary extensions.
2. **`proposed_validation.shacl.ttl`**: The SHACL NodeShapes and SPARQL validation queries.
3. **`proposed_ggen_rules.toml`**: The SPARQL rules to append to `ggen.toml`.
