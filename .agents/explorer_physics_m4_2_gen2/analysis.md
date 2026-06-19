# Analysis: UE4 Physics Layer Modeling (Collision Volumes & Kinematics)

## Overview

This analysis proposes a concrete, mathematically rigorous RDF mapping for the Unreal Engine 4 (UE4) Physics layer in the Universal Ontology. Specifically, it covers **Collision Volumes** and **Kinematics** to be integrated into `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl`. 

Under the **TPS/DfLSS Playwright Manufacturing Strategy**, a compiled WASM world is only admitted if it passes interactive E2E visual verification without crashes or rendering anomalies. Improperly configured physics (such as objects falling through the floor, mass-induced $NaN$ simulation values, or duplicate subsystem instances) will cause silent run-time errors or visual test failures. This proposal establishes **Agent Jidoka (the Autonomation Law)** by introducing compile-time SHACL shapes and SPARQL rules that validate physics topologies *before* WASM generation, ensuring only valid configurations reach the packaging line.

---

## 1. Collision Volumes Ontological Modeling

To model UE4's collision profiles and channels, we define a set of enums, profiles, and reified mappings that allow custom profiles and channels to be declared extensibly in RDF.

### RDF Classes

- **`ue4:UCollisionProfile`**: Subclass of `ue4:UObject`. Represents an engine-level collision profile (e.g., `BlockAll`, `NoCollision`, `PhysicsActor`).
- **`ue4:ECollisionChannel`**: Enum class containing all collision object types and trace channels.
- **`ue4:ECollisionResponse`**: Enum class defining response actions when collision occurs (`ECR_Ignore`, `ECR_Overlap`, `ECR_Block`).
- **`ue4:ECollisionEnabled`**: Enum class defining what collision modes are active (`NoCollision`, `QueryOnly`, `PhysicsOnly`, `QueryAndPhysics`).
- **`ue4:CollisionChannelResponse`**: Reified mapping node associating a specific collision channel with a response value.

### RDF Properties

- **`ue4:profileName`**: Domain `ue4:UCollisionProfile`, Range `xsd:string` (Unique name identifying the profile).
- **`ue4:collisionObjectType`**: Domain `ue4:UCollisionProfile` or `ue4:USceneComponent`, Range `ue4:ECollisionChannel` (The default object channel/type).
- **`ue4:collisionEnabled`**: Domain `ue4:UCollisionProfile` or `ue4:USceneComponent`, Range `ue4:ECollisionEnabled` (Specifies query/physics collision configuration).
- **`ue4:hasChannelResponse`**: Domain `ue4:UCollisionProfile`, Range `ue4:CollisionChannelResponse` (Associates response mappings with the profile).
- **`ue4:responseChannel`**: Domain `ue4:CollisionChannelResponse`, Range `ue4:ECollisionChannel` (Target channel of the response mapping).
- **`ue4:responseValue`**: Domain `ue4:CollisionChannelResponse`, Range `ue4:ECollisionResponse` (The response behavior, e.g., block, overlap, ignore).
- **`ue4:hasCollisionProfile`**: Domain `ue4:USceneComponent`, Range `ue4:UCollisionProfile` (Associates a component with a collision profile).
- **`ue4:registersCollisionProfile`**: Domain `ue4:UPhysicsSubsystem`, Range `ue4:UCollisionProfile` (Associates the physics subsystem with the profiles it registers).

### Recommended RDF Vocabularies (subsystems.ttl Additions)

```turtle
# --- Enums ---
ue4:ECollisionResponse a owl:Class ;
    rdfs:label "ECollisionResponse" ;
    rdfs:comment "Enum representing the response to a collision channel." .

ue4:ECR_Ignore a ue4:ECollisionResponse ; rdfs:label "ECR_Ignore" .
ue4:ECR_Overlap a ue4:ECollisionResponse ; rdfs:label "ECR_Overlap" .
ue4:ECR_Block a ue4:ECollisionResponse ; rdfs:label "ECR_Block" .

ue4:ECollisionEnabled a owl:Class ;
    rdfs:label "ECollisionEnabled" ;
    rdfs:comment "Enum representing what types of collision are enabled." .

ue4:NoCollision a ue4:ECollisionEnabled ; rdfs:label "NoCollision" .
ue4:QueryOnly a ue4:ECollisionEnabled ; rdfs:label "QueryOnly" .
ue4:PhysicsOnly a ue4:ECollisionEnabled ; rdfs:label "PhysicsOnly" .
ue4:QueryAndPhysics a ue4:ECollisionEnabled ; rdfs:label "QueryAndPhysics" .

ue4:ECollisionChannel a owl:Class ;
    rdfs:label "ECollisionChannel" ;
    rdfs:comment "Enum representing the collision channel / type." .

# Standard Engine Channels
ue4:ECC_WorldStatic a ue4:ECollisionChannel ; rdfs:label "ECC_WorldStatic" .
ue4:ECC_WorldDynamic a ue4:ECollisionChannel ; rdfs:label "ECC_WorldDynamic" .
ue4:ECC_Pawn a ue4:ECollisionChannel ; rdfs:label "ECC_Pawn" .
ue4:ECC_Visibility a ue4:ECollisionChannel ; rdfs:label "ECC_Visibility" .
ue4:ECC_Camera a ue4:ECollisionChannel ; rdfs:label "ECC_Camera" .
ue4:ECC_PhysicsBody a ue4:ECollisionChannel ; rdfs:label "ECC_PhysicsBody" .
ue4:ECC_Vehicle a ue4:ECollisionChannel ; rdfs:label "ECC_Vehicle" .
ue4:ECC_Destructible a ue4:ECollisionChannel ; rdfs:label "ECC_Destructible" .

# --- Reified Channel Mappings ---
ue4:CollisionChannelResponse a owl:Class ;
    rdfs:label "CollisionChannelResponse" ;
    rdfs:comment "A mapping from a specific collision channel to a collision response." .

ue4:hasResponseChannel a owl:ObjectProperty ;
    rdfs:label "hasResponseChannel" ;
    rdfs:comment "Specifies the target collision channel." ;
    rdfs:domain ue4:CollisionChannelResponse ;
    rdfs:range ue4:ECollisionChannel .

ue4:hasResponseValue a owl:ObjectProperty ;
    rdfs:label "hasResponseValue" ;
    rdfs:comment "Specifies the response value (Block, Overlap, Ignore)." ;
    rdfs:domain ue4:CollisionChannelResponse ;
    rdfs:range ue4:ECollisionResponse .

# --- Collision Profile ---
ue4:UCollisionProfile a owl:Class ;
    rdfs:subClassOf ue4:UObject ;
    rdfs:label "UCollisionProfile" ;
    rdfs:comment "Configuration profile defining collision responses for various channels." .

ue4:profileName a owl:DatatypeProperty ;
    rdfs:label "profileName" ;
    rdfs:comment "The string name of the collision profile." ;
    rdfs:domain ue4:UCollisionProfile ;
    rdfs:range xsd:string .

ue4:hasChannelResponse a owl:ObjectProperty ;
    rdfs:label "hasChannelResponse" ;
    rdfs:comment "Relates a collision profile to individual channel response configurations." ;
    rdfs:domain ue4:UCollisionProfile ;
    rdfs:range ue4:CollisionChannelResponse .

# --- Component Association Properties ---
ue4:hasCollisionProfile a owl:ObjectProperty ;
    rdfs:label "hasCollisionProfile" ;
    rdfs:comment "Associates a scene component with a collision profile." ;
    rdfs:domain ue4:USceneComponent ;
    rdfs:range ue4:UCollisionProfile .

ue4:collisionEnabled a owl:ObjectProperty ;
    rdfs:label "collisionEnabled" ;
    rdfs:comment "Direct override configuration for whether collision is enabled." ;
    rdfs:domain ue4:USceneComponent ;
    rdfs:range ue4:ECollisionEnabled .

ue4:collisionObjectType a owl:ObjectProperty ;
    rdfs:label "collisionObjectType" ;
    rdfs:comment "Direct override of the collision channel classification for the scene component." ;
    rdfs:domain ue4:USceneComponent ;
    rdfs:range ue4:ECollisionChannel .

# --- Physics Subsystem Linkages ---
ue4:registersCollisionProfile a owl:ObjectProperty ;
    rdfs:label "registersCollisionProfile" ;
    rdfs:comment "Relates the physics subsystem to the profiles registered within it." ;
    rdfs:domain ue4:UPhysicsSubsystem ;
    rdfs:range ue4:UCollisionProfile .
```

---

## 2. Kinematics Ontological Modeling

UE4 kinematics covers the mass, damping, gravity, and axis-locking constraints (degrees of freedom) of dynamic actors, modeled via a dedicated `URigidBody` (BodyInstance) abstraction.

### RDF Classes

- **`ue4:URigidBody`**: Subclass of `ue4:UObject`. Represents the physical body instance properties of a scene component (similar to `FBodyInstance` or `UBodySetup` in C++).
- **`ue4:EPhysicsType`**: Enum class containing kinematic simulation states (`PhysType_Default`, `PhysType_Kinematic`, `PhysType_Simulated`).
- **`ue4:EDOFMode`**: Enum class defining locked degrees of freedom plane profiles (`DOFMode_None`, `DOFMode_XYPlane`, `DOFMode_XZPlane`, `DOFMode_YZPlane`, `DOFMode_SixDOF`).

### RDF Properties

- **`ue4:hasRigidBody`**: Domain `ue4:USceneComponent`, Range `ue4:URigidBody` (Associates a component with its physical rigid body properties).
- **`ue4:physicsType`**: Domain `ue4:URigidBody`, Range `ue4:EPhysicsType` (The kinematic simulation state).
- **`ue4:massKg`**: Domain `ue4:URigidBody`, Range `xsd:float` (The physical mass of the rigid body in kilograms).
- **`ue4:bEnableGravity`**: Domain `ue4:URigidBody`, Range `xsd:boolean` (Specifies whether gravity influences the simulation of this body).
- **`ue4:linearDamping`**: Domain `ue4:URigidBody`, Range `xsd:float` (Drag force applied to linear velocity).
- **`ue4:angularDamping`**: Domain `ue4:URigidBody`, Range `xsd:float` (Drag force applied to rotational velocity).
- **`ue4:hasDOFMode`**: Domain `ue4:URigidBody`, Range `ue4:EDOFMode` (Specifies spatial degrees of freedom plane locks).
- **`ue4:maxLinearVelocity`**: Domain `ue4:URigidBody`, Range `xsd:float` (Linear velocity speed limit).
- **`ue4:maxAngularVelocity`**: Domain `ue4:URigidBody`, Range `xsd:float` (Angular velocity speed limit).
- **`ue4:tracksRigidBody`**: Domain `ue4:UPhysicsSubsystem`, Range `ue4:URigidBody` (Tracks active bodies in the subsystem).

### Recommended RDF Vocabularies (subsystems.ttl Additions)

```turtle
# --- Kinematics Enums ---
ue4:EPhysicsType a owl:Class ;
    rdfs:label "EPhysicsType" ;
    rdfs:comment "Enum representing the kinematic simulation type." .

ue4:PhysType_Default a ue4:EPhysicsType ; rdfs:label "PhysType_Default" .
ue4:PhysType_Kinematic a ue4:EPhysicsType ; rdfs:label "PhysType_Kinematic" .
ue4:PhysType_Simulated a ue4:EPhysicsType ; rdfs:label "PhysType_Simulated" .

ue4:EDOFMode a owl:Class ;
    rdfs:label "EDOFMode" ;
    rdfs:comment "Enum representing degrees of freedom constraint modes." .

ue4:DOFMode_None a ue4:EDOFMode ; rdfs:label "DOFMode_None" .
ue4:DOFMode_XYPlane a ue4:EDOFMode ; rdfs:label "DOFMode_XYPlane" .
ue4:DOFMode_XZPlane a ue4:EDOFMode ; rdfs:label "DOFMode_XZPlane" .
ue4:DOFMode_YZPlane a ue4:EDOFMode ; rdfs:label "DOFMode_YZPlane" .
ue4:DOFMode_SixDOF a ue4:EDOFMode ; rdfs:label "DOFMode_SixDOF" .

# --- Rigid Body Instance ---
ue4:URigidBody a owl:Class ;
    rdfs:subClassOf ue4:UObject ;
    rdfs:label "URigidBody" ;
    rdfs:comment "Represents the dynamic physical state, mass, and velocity constraints of a component." .

ue4:hasRigidBody a owl:ObjectProperty ;
    rdfs:label "hasRigidBody" ;
    rdfs:comment "Relates a scene component to its rigid body instance configuration." ;
    rdfs:domain ue4:USceneComponent ;
    rdfs:range ue4:URigidBody .

ue4:physicsType a owl:ObjectProperty ;
    rdfs:label "physicsType" ;
    rdfs:comment "Determines if the rigid body is simulated, kinematic, or uses parent default." ;
    rdfs:domain ue4:URigidBody ;
    rdfs:range ue4:EPhysicsType .

ue4:massKg a owl:DatatypeProperty ;
    rdfs:label "massKg" ;
    rdfs:comment "The mass of the object in kilograms." ;
    rdfs:domain ue4:URigidBody ;
    rdfs:range xsd:float .

ue4:bEnableGravity a owl:DatatypeProperty ;
    rdfs:label "bEnableGravity" ;
    rdfs:comment "Indicates whether this body is affected by scene gravity." ;
    rdfs:domain ue4:URigidBody ;
    rdfs:range xsd:boolean .

ue4:linearDamping a owl:DatatypeProperty ;
    rdfs:label "linearDamping" ;
    rdfs:comment "Linear velocity resistance factor." ;
    rdfs:domain ue4:URigidBody ;
    rdfs:range xsd:float .

ue4:angularDamping a owl:DatatypeProperty ;
    rdfs:label "angularDamping" ;
    rdfs:comment "Angular velocity resistance factor." ;
    rdfs:domain ue4:URigidBody ;
    rdfs:range xsd:float .

ue4:hasDOFMode a owl:ObjectProperty ;
    rdfs:label "hasDOFMode" ;
    rdfs:comment "Restricts linear or angular movement to specific plane sets." ;
    rdfs:domain ue4:URigidBody ;
    rdfs:range ue4:EDOFMode .

ue4:maxLinearVelocity a owl:DatatypeProperty ;
    rdfs:label "maxLinearVelocity" ;
    rdfs:comment "The maximum linear speed threshold." ;
    rdfs:domain ue4:URigidBody ;
    rdfs:range xsd:float .

ue4:maxAngularVelocity a owl:DatatypeProperty ;
    rdfs:label "maxAngularVelocity" ;
    rdfs:comment "The maximum angular speed threshold." ;
    rdfs:domain ue4:URigidBody ;
    rdfs:range xsd:float .

# --- Subsystem Linkage ---
ue4:tracksRigidBody a owl:ObjectProperty ;
    rdfs:label "tracksRigidBody" ;
    rdfs:comment "Relates the physics subsystem to an active rigid body tracking state." ;
    rdfs:domain ue4:UPhysicsSubsystem ;
    rdfs:range ue4:URigidBody .

ue4:hasSubsystem a owl:ObjectProperty ;
    rdfs:label "hasSubsystem" ;
    rdfs:comment "Relates a world to its registered sub-systems." ;
    rdfs:domain ue4:UWorld ;
    rdfs:range ue4:USubsystem .
```

---

## 3. Custom SHACL Validation Shapes & SPARQL Rules

The following shapes and SPARQL queries validate the structural integrity of the physics graph, catching runtime errors during semantic parsing.

### 1. Simulated Body Mass Shape (`ue4:SimulatedBodyMassShape`)
**Constraint:** If a rigid body is simulated (`ue4:physicsType` is `ue4:PhysType_Simulated`), it must define a positive mass (`ue4:massKg` > 0.0). Missing or negative mass values will result in division-by-zero errors ($NaN$ output) in the physics solver.

```turtle
ue4:SimulatedBodyMassShape
    a sh:NodeShape ;
    sh:targetClass ue4:URigidBody ;
    sh:sparql [
        sh:message "Simulated rigid bodies (PhysType_Simulated) must have a declared mass greater than 0.0 kg to prevent calculation instability (NaNs)." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this
            WHERE {
                $this ue4:physicsType ue4:PhysType_Simulated .
                FILTER NOT EXISTS {
                    $this ue4:massKg ?mass .
                    FILTER (?mass > 0.0)
                }
            } ORDER BY $this
        """ ;
    ] .
```

### 2. Single Physics Subsystem per World (`ue4:WorldPhysicsSubsystemCountShape`)
**Constraint:** A `UWorld` must contain exactly one registered `UPhysicsSubsystem`. Zero subsystems results in no physics simulations occurring, and multiple subsystems would lead to ticked step duplication conflicts.

```turtle
ue4:WorldPhysicsSubsystemCountShape
    a sh:NodeShape ;
    sh:targetSubjectsOf rdf:type ;
    sh:sparql [
        sh:message "Each UWorld must have exactly one registered instance of UPhysicsSubsystem." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
            SELECT $this
            WHERE {
                $this a ?worldClass .
                ?worldClass rdfs:subClassOf* ue4:UWorld .
                {
                    FILTER NOT EXISTS {
                        $this ue4:hasSubsystem ?sub .
                        ?sub a/rdfs:subClassOf* ue4:UPhysicsSubsystem .
                    }
                }
                UNION
                {
                    $this ue4:hasSubsystem ?sub1 .
                    ?sub1 a/rdfs:subClassOf* ue4:UPhysicsSubsystem .
                    $this ue4:hasSubsystem ?sub2 .
                    ?sub2 a/rdfs:subClassOf* ue4:UPhysicsSubsystem .
                    FILTER (?sub1 != ?sub2)
                }
            } ORDER BY $this
        """ ;
    ] .
```

### 3. Gravity without Collision Safety (`ue4:SimulatedGravityCollisionShape`)
**Constraint:** If a rigid body is simulated (`ue4:physicsType` is `ue4:PhysType_Simulated`) and gravity is enabled (`ue4:bEnableGravity` is `true`), its parent component must have an active collision mode (i.e. not `ue4:NoCollision`). If collision is disabled or missing, the object falls through the scene floor, causing immediate Playwright test failure.

```turtle
ue4:SimulatedGravityCollisionShape
    a sh:NodeShape ;
    sh:targetClass ue4:URigidBody ;
    sh:sparql [
        sh:message "Simulated rigid bodies with gravity enabled must have active collision (not NoCollision) to prevent objects falling through the floor." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?comp
            WHERE {
                ?comp ue4:hasRigidBody $this .
                $this ue4:physicsType ue4:PhysType_Simulated .
                $this ue4:bEnableGravity true .
                FILTER (
                    EXISTS { ?comp ue4:collisionEnabled ue4:NoCollision } ||
                    EXISTS {
                        ?comp ue4:hasCollisionProfile ?profile .
                        ?profile ue4:collisionEnabled ue4:NoCollision .
                    } ||
                    (
                        NOT EXISTS { ?comp ue4:collisionEnabled ?anyEnabled } &&
                        NOT EXISTS { ?comp ue4:hasCollisionProfile ?anyProfile }
                    )
                )
            } ORDER BY $this
        """ ;
    ] .
```

### 4. Reified Response Completeness (`ue4:CollisionChannelResponseShape`)
**Constraint:** Every `ue4:CollisionChannelResponse` mapping node must have exactly one channel and exactly one response defined.

```turtle
ue4:CollisionChannelResponseShape
    a sh:NodeShape ;
    sh:targetClass ue4:CollisionChannelResponse ;
    sh:property [
        sh:path ue4:hasResponseChannel ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:class ue4:ECollisionChannel ;
        sh:message "A CollisionChannelResponse must map to exactly one ECollisionChannel." ;
    ] ;
    sh:property [
        sh:path ue4:hasResponseValue ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:class ue4:ECollisionResponse ;
        sh:message "A CollisionChannelResponse must map to exactly one ECollisionResponse." ;
    ] .
```

---

## 4. Alignment with the Playwright Manufacturing Strategy

Under the **TPS/DfLSS Playwright Manufacturing Strategy**, the pipeline enforces validation from Source to WASM Package to Browser Actuation. This model supports the pipeline as follows:

1. **Gate 3 (Browser Load) & Gate 4 (Visual World):** The `WorldPhysicsSubsystemCountShape` and `SimulatedBodyMassShape` prevent physics thread crashes, which typically lock up the HTML5/WASM runtime or throw JS errors in the browser console.
2. **Gate 5 (Actuation) & Gate 6 (Motion):** The `SimulatedGravityCollisionShape` ensures simulated dynamic actors block movement keys properly rather than clip through walls or drop indefinitely out of sight (zero visual delta).
3. **Gate 7 (Receipt):** The deterministic RDF graph constraints (such as `ORDER BY` requirements) guarantee that generating the exact same ontology graph produces the exact same physics behavior and visual tests, securing reproducibility in the final BLAKE3 cryptographic receipt.
