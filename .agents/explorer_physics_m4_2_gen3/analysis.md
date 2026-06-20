# Physics Subsystem Ontology Analysis

## Executive Summary
This report presents a forensic gaps and consistency analysis of the UE4 Physics layers (Collision Volumes, Kinematics) as modeled in the `ue4_ontology` pack (`subsystems.ttl` and `shacl/validation.shacl.ttl`). While the core concepts are mapped, several critical structural issues, class hierarchy gaps, property domain mismatches, and validation gaps have been identified. Addressing these will prevent compile-time inconsistencies and runtime validation failures in the generated C++ headers and local browser client.

---

## Detailed Findings & Gaps

### 1. Component Class Hierarchy Gaps (Dangling Target Classes)
**Observation:**
The SHACL validator file `validation.shacl.ttl` targets specific component classes for verification:
- Line 386: `sh:targetClass ue4:USkeletalMeshComponent`
- Line 411: `sh:targetClass ue4:UBoxComponent`

However, a grep search across the Turtle files shows that neither `USkeletalMeshComponent` nor `UBoxComponent` is defined anywhere in `core.ttl`, `subsystems.ttl`, or any other ontology file. In addition, the base class representing collision/physics-enabled components, `UPrimitiveComponent` (which inherits from `USceneComponent`), is also missing. 

**Impact:**
- Any instance of `USkeletalMeshComponent` or `UBoxComponent` in a generated world graph will fail to align with the core backbone class tree because they are dangling URIs.
- Class-level property restrictions cannot be cleanly applied.

---

### 2. Property Domain Inconsistencies (Class Pun/Inference Defects)
**Observation:**
In `subsystems.ttl` (lines 367-378), the properties `collisionEnabled` and `collisionObjectType` are declared with domain `ue4:USceneComponent`:
```turtle
ue4:collisionEnabled a owl:ObjectProperty ;
    rdfs:domain ue4:USceneComponent ;
    rdfs:range ue4:ECollisionEnabled .

ue4:collisionObjectType a owl:ObjectProperty ;
    rdfs:domain ue4:USceneComponent ;
    rdfs:range ue4:ECollisionChannel .
```
However, the SHACL rule `ue4:SimulatedGravityCollisionShape` in `validation.shacl.ttl` checks:
```turtle
?comp ue4:hasCollisionProfile ?profile .
?profile ue4:collisionEnabled ue4:NoCollision .
```
Here, `collisionEnabled` is applied to `?profile` (which is a `UCollisionProfile`).

**Impact:**
- RDFS/OWL reasoners will infer that any `UCollisionProfile` instance is also a `USceneComponent` due to the domain constraint on `collisionEnabled`. This is a classic ontological consistency violation (class punning/incorrect classification).
- Domain constraints must be expanded using unions so they apply to both `USceneComponent` (or `UPrimitiveComponent`) and `UCollisionProfile`.

---

### 3. Kinematics Modeling Gaps
**Observation:**
In UE4, physics simulation toggle is governed by `bSimulatePhysics` on `UPrimitiveComponent`. Additionally, explicit mass configuration requires `bOverrideMass = true` on the component's body instance.
- **Missing `bSimulatePhysics`:** The property is not defined anywhere in the ontology. Currently, physics simulation is only inferred from `URigidBody/physicsType`, which misses the component-level toggle.
- **Missing `bOverrideMass`:** The ontology defines `ue4:massKg` on `URigidBody`, but lacks `ue4:bOverrideMass`. In Unreal C++, setting `MassInKg` without setting `bOverrideMass = true` results in the value being ignored and calculated from volume/density instead.
- **No Joint/Constraint Modeling:** There is no representation of `UPhysicsConstraintComponent` or its constraint limits, meaning complex multi-body kinematics (e.g. robotic arms, mechs) cannot be semantically represented.

---

### 4. Absence of Standard Profile Individuals
**Observation:**
Common UE4 collision profiles (like `Pawn`, `PhysicsActor`, `BlockAll`, `NoCollision`) are not pre-defined in `subsystems.ttl`. Any generated world must reinvent these profiles from scratch, increasing the risk of inconsistent naming or default values.

---

## Proposed Refinements & Recommendations

### A. Refinements to Class Hierarchy (`subsystems.ttl` or `core.ttl`)
Introduce the missing C++ component classes to establish a mathematically sound hierarchy:
```turtle
ue4:UPrimitiveComponent a owl:Class ;
    rdfs:subClassOf ue4:USceneComponent ;
    rdfs:label "UPrimitiveComponent" ;
    rdfs:comment "Base class for all components that contain some form of spatial rendering or collision geometry." .

ue4:UMeshComponent a owl:Class ;
    rdfs:subClassOf ue4:UPrimitiveComponent ;
    rdfs:label "UMeshComponent" ;
    rdfs:comment "Base class for components that render skeletal or static meshes." .

ue4:UStaticMeshComponent a owl:Class ;
    rdfs:subClassOf ue4:UMeshComponent ;
    rdfs:label "UStaticMeshComponent" ;
    rdfs:comment "Renders a static mesh geometry." .

ue4:USkeletalMeshComponent a owl:Class ;
    rdfs:subClassOf ue4:UMeshComponent ;
    rdfs:label "USkeletalMeshComponent" ;
    rdfs:comment "Renders a skeletal mesh geometry for animated actors." .

ue4:UShapeComponent a owl:Class ;
    rdfs:subClassOf ue4:UPrimitiveComponent ;
    rdfs:label "UShapeComponent" ;
    rdfs:comment "Base class for simple geometric collision shapes." .

ue4:UBoxComponent a owl:Class ;
    rdfs:subClassOf ue4:UShapeComponent ;
    rdfs:label "UBoxComponent" ;
    rdfs:comment "A box-shaped collision volume." .

ue4:USphereComponent a owl:Class ;
    rdfs:subClassOf ue4:UShapeComponent ;
    rdfs:label "USphereComponent" ;
    rdfs:comment "A sphere-shaped collision volume." .

ue4:UCapsuleComponent a owl:Class ;
    rdfs:subClassOf ue4:UShapeComponent ;
    rdfs:label "UCapsuleComponent" ;
    rdfs:comment "A capsule-shaped collision volume." .
```

Restrict collision and rigid body properties to `UPrimitiveComponent` instead of the more general `USceneComponent`:
- Update domain of `hasCollisionProfile`, `collisionEnabled`, `collisionObjectType`, and `hasRigidBody` to `ue4:UPrimitiveComponent`.

---

### B. Resolution of Domain Mismatches
Redefine `collisionEnabled` and `collisionObjectType` with domains that are union-typed to allow usage on profiles:
```turtle
ue4:collisionEnabled a owl:ObjectProperty ;
    rdfs:label "collisionEnabled" ;
    rdfs:comment "Direct override or profile definition for whether collision is enabled." ;
    rdfs:domain [
        a owl:Class ;
        owl:unionOf ( ue4:UPrimitiveComponent ue4:UCollisionProfile )
    ] ;
    rdfs:range ue4:ECollisionEnabled .

ue4:collisionObjectType a owl:ObjectProperty ;
    rdfs:label "collisionObjectType" ;
    rdfs:comment "Direct override or profile definition of the collision channel classification." ;
    rdfs:domain [
        a owl:Class ;
        owl:unionOf ( ue4:UPrimitiveComponent ue4:UCollisionProfile )
    ] ;
    rdfs:range ue4:ECollisionChannel .
```

---

### C. Proposed SHACL Verification Rules (`shacl/validation.shacl.ttl`)

#### 1. Collision Profile Completeness and Uniqueness
Ensures profiles define unique and non-conflicting channel responses:
```turtle
# Rule: Collision profiles must have a name, enabled state, and object type
ue4:UCollisionProfileShape
    a sh:NodeShape ;
    sh:targetClass ue4:UCollisionProfile ;
    sh:property [
        sh:path ue4:profileName ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:datatype xsd:string ;
        sh:message "A collision profile must have exactly one string profileName." ;
    ] ;
    sh:property [
        sh:path ue4:collisionEnabled ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:class ue4:ECollisionEnabled ;
        sh:message "A collision profile must have exactly one collisionEnabled configuration." ;
    ] ;
    sh:property [
        sh:path ue4:collisionObjectType ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:class ue4:ECollisionChannel ;
        sh:message "A collision profile must have exactly one collisionObjectType." ;
    ] .

# Rule: Collision profiles must not map duplicate or conflicting responses
ue4:CollisionProfileUniqueChannelsShape
    a sh:NodeShape ;
    sh:targetClass ue4:UCollisionProfile ;
    sh:sparql [
        sh:message "Collision profile channel conflict: A collision profile cannot define multiple responses for the same collision channel." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?channel
            WHERE {
                $this ue4:hasChannelResponse ?resp1 .
                $this ue4:hasChannelResponse ?resp2 .
                ?resp1 ue4:hasResponseChannel ?channel .
                ?resp2 ue4:hasResponseChannel ?channel .
                FILTER (?resp1 != ?resp2)
            } ORDER BY $this ?channel
        """ ;
    ] .
```

#### 2. Unique Profile Names within a Subsystem
Ensures two profiles in the same subsystem do not share a name:
```turtle
ue4:PhysicsSubsystemUniqueProfileNamesShape
    a sh:NodeShape ;
    sh:targetClass ue4:UPhysicsSubsystem ;
    sh:sparql [
        sh:message "Duplicate collision profile names registered: Each registered collision profile must have a unique profileName." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?name ?profile1 ?profile2
            WHERE {
                $this ue4:registersCollisionProfile ?profile1 .
                $this ue4:registersCollisionProfile ?profile2 .
                ?profile1 ue4:profileName ?name .
                ?profile2 ue4:profileName ?name .
                FILTER (?profile1 != ?profile2)
            } ORDER BY $this ?name
        """ ;
    ] .
```

#### 3. Unregistered Profile Usage Shape
Ensures that if a component uses a profile, it must be registered in the active physics subsystem:
```turtle
ue4:ComponentCollisionProfileRegistrationShape
    a sh:NodeShape ;
    sh:targetClass ue4:UPrimitiveComponent ;
    sh:sparql [
        sh:message "Unregistered collision profile: The collision profile used by this component is not registered in the world's physics subsystem." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?profile ?world
            WHERE {
                ?world ue4:hasLevel/ue4:hasActor/ue4:hasComponent $this .
                $this ue4:hasCollisionProfile ?profile .
                ?world ue4:hasSubsystem ?physSub .
                ?physSub a/rdfs:subClassOf* ue4:UPhysicsSubsystem .
                FILTER NOT EXISTS {
                    ?physSub ue4:registersCollisionProfile ?profile .
                }
            } ORDER BY $this ?profile
        """ ;
    ] .
```

#### 4. Physical Parameter Limit Validation
Prevents negative physical damping or velocity limits:
```turtle
ue4:RigidBodyPhysicalLimitsShape
    a sh:NodeShape ;
    sh:targetClass ue4:URigidBody ;
    sh:property [
        sh:path ue4:linearDamping ;
        sh:minInclusive 0.0 ;
        sh:message "Linear damping must be non-negative." ;
    ] ;
    sh:property [
        sh:path ue4:angularDamping ;
        sh:minInclusive 0.0 ;
        sh:message "Angular damping must be non-negative." ;
    ] ;
    sh:property [
        sh:path ue4:maxLinearVelocity ;
        sh:minInclusive 0.0 ;
        sh:message "Maximum linear velocity must be non-negative." ;
    ] ;
    sh:property [
        sh:path ue4:maxAngularVelocity ;
        sh:minInclusive 0.0 ;
        sh:message "Maximum angular velocity must be non-negative." ;
    ] .
```

#### 5. Symmetric Collision Response Warning (Warning Severity)
Flags asymmetrical collision response settings:
```turtle
ue4:SymmetricCollisionResponseWarningShape
    a sh:NodeShape ;
    sh:targetClass ue4:UCollisionProfile ;
    sh:sparql [
        sh:severity sh:Warning ;
        sh:message "Asymmetric collision response warning: Responses between profile object types do not match (Profile1 responds to Type2 differently than Profile2 responds to Type1)." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?profile2 ?type1 ?type2 ?respVal1 ?respVal2
            WHERE {
                $this ue4:collisionObjectType ?type1 ;
                      ue4:hasChannelResponse ?resp1 .
                ?resp1 ue4:hasResponseChannel ?type2 ;
                       ue4:hasResponseValue ?respVal1 .
                       
                ?profile2 a ue4:UCollisionProfile ;
                          ue4:collisionObjectType ?type2 ;
                          ue4:hasChannelResponse ?resp2 .
                ?resp2 ue4:hasResponseChannel ?type1 ;
                       ue4:hasResponseValue ?respVal2 .
                       
                FILTER ($this != ?profile2)
                FILTER (?respVal1 != ?respVal2)
            } ORDER BY $this ?profile2
        """ ;
    ] .
```
