# Subsystem Topologies Remediation Changes Report

This report documents the changes implemented to address the five validation and schema defects identified by Reviewer 2 in Milestone 4.

## Modifications Made

### 1. Defect 1: Class Scope Validation for RPC Validation Functions (Major)
- **Target Pack File**: `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`
- **Validation Tests File**: `/Users/sac/rocket-craft/ggen-validation-tests/shacl/validation.shacl.ttl`
  - Added a new `sh:sparql` select constraint block to the `ue4:RPCValidationSignatureShape` node shape:
    ```turtle
    sh:sparql [
        sh:message "RuleRPCValidationClassScope: RPC validation function class scope violation: The validation function must be a member of the same class (or a base class) as the RPC." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
            SELECT $this ?valFunc ?rpcClass
            WHERE {
                $this ue4:bWithValidation true ;
                      ue4:validationFunction ?valFunc .
                ?rpcClass ue4:hasFunction $this .
                FILTER NOT EXISTS {
                    ?rpcClass rdfs:subClassOf* ?valClass .
                    ?valClass ue4:hasFunction ?valFunc .
                }
            } ORDER BY $this ?valFunc ?rpcClass
        """ ;
    ] .
    ```
- **Target Pack Rule File**: `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml`
- **Validation Tests Rule File**: `/Users/sac/rocket-craft/ggen-validation-tests/ggen.toml`
  - Added custom rule `RuleRPCValidationClassScope`:
    ```toml
    [[validation.rules]]
    name = "RuleRPCValidationClassScope"
    description = "RPC validation function class scope violation: The validation function must be a member of the same class (or a base class) as the RPC."
    ask = """
    PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
    PREFIX owl: <http://www.w3.org/2002/07/owl#>
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    ASK {
      ?ontology a owl:Ontology .
      FILTER NOT EXISTS {
        ?rpc ue4:bWithValidation true ;
             ue4:validationFunction ?valFunc .
        ?rpcClass ue4:hasFunction ?rpc .
        FILTER NOT EXISTS {
          ?rpcClass rdfs:subClassOf* ?valClass .
          ?valClass ue4:hasFunction ?valFunc .
        }
      }
    }
    """
    ```

### 2. Defect 2: Subclass-aware Parameter Type Check under Inheritance (Major)
- **Target Pack File**: `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`
- **Validation Tests File**: `/Users/sac/rocket-craft/ggen-validation-tests/shacl/validation.shacl.ttl`
  - In `ue4:MaterialInstanceParameterValueTypeShape`, replaced class direct equality checks:
    ```turtle
    # Before
    ?paramType = ue4:UScalarParameter
    
    # After
    EXISTS { ?paramDef a/rdfs:subClassOf* ue4:UScalarParameter }
    ```
    This ensures custom subclasses of material parameters (e.g. customized scalars, vectors, and textures) are fully subclass-aware and do not raise false positives.
- **Target Pack Rule File**: `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml`
- **Validation Tests Rule File**: `/Users/sac/rocket-craft/ggen-validation-tests/ggen.toml`
  - In `RuleMaterialInstanceParameterValueType`, updated target query filters with equivalent `EXISTS { ?paramDef a/rdfs:subClassOf* ... }` queries to align with SHACL rules.

### 3. Defect 3: Kinematic Parameter Disconnect Check (Medium)
- **Target Pack File**: `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`
- **Validation Tests File**: `/Users/sac/rocket-craft/ggen-validation-tests/shacl/validation.shacl.ttl`
  - Added new NodeShape `ue4:KinematicSimulationDisconnectShape`:
    ```turtle
    # Rule 27: Kinematic Simulation Disconnect (RuleKinematicSimulationDisconnect)
    ue4:KinematicSimulationDisconnectShape
        a sh:NodeShape ;
        sh:targetClass ue4:UPrimitiveComponent ;
        sh:sparql [
            sh:message "RuleKinematicSimulationDisconnect: Kinematic parameter disconnect: if bSimulatePhysics is true, the associated rigid body's physicsType must be PhysType_Simulated or PhysType_Default." ;
            sh:select """
                PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
                SELECT $this ?body
                WHERE {
                    $this ue4:bSimulatePhysics true ;
                          ue4:hasRigidBody ?body .
                    FILTER NOT EXISTS {
                        ?body ue4:physicsType ?physType .
                        FILTER (?physType = ue4:PhysType_Simulated || ?physType = ue4:PhysType_Default)
                    }
                } ORDER BY $this ?body
            """ ;
        ] .
    ```
- **Target Pack Rule File**: `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml`
- **Validation Tests Rule File**: `/Users/sac/rocket-craft/ggen-validation-tests/ggen.toml`
  - Added custom rule `RuleKinematicSimulationDisconnect`:
    ```toml
    [[validation.rules]]
    name = "RuleKinematicSimulationDisconnect"
    description = "Kinematic parameter disconnect: if bSimulatePhysics is true, the associated rigid body's physicsType must be PhysType_Simulated or PhysType_Default."
    ask = """
    PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
    PREFIX owl: <http://www.w3.org/2002/07/owl#>
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    ASK {
      ?ontology a owl:Ontology .
      FILTER NOT EXISTS {
        ?comp ue4:bSimulatePhysics true ;
              ue4:hasRigidBody ?body .
        FILTER NOT EXISTS {
          ?body ue4:physicsType ?physType .
          FILTER (?physType = ue4:PhysType_Simulated || ?physType = ue4:PhysType_Default)
        }
      }
    }
    """
    ```

### 4. Defect 4: Domain Limitation for Collision Channel Overrides (Medium)
- **Target Pack File**: `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl`
- **Validation Tests File**: `/Users/sac/rocket-craft/ggen-validation-tests/subsystems.ttl`
  - Expanded the domain of `ue4:hasChannelResponse` from strictly `ue4:UCollisionProfile` to the union class of `ue4:UPrimitiveComponent` and `ue4:UCollisionProfile`:
    ```turtle
    ue4:hasChannelResponse a owl:ObjectProperty ;
        rdfs:label "hasChannelResponse" ;
        rdfs:comment "Relates a collision profile or primitive component to individual channel response configurations." ;
        rdfs:domain [
            a owl:Class ;
            owl:unionOf ( ue4:UPrimitiveComponent ue4:UCollisionProfile )
        ] ;
        rdfs:range ue4:CollisionChannelResponse .
    ```

### 5. Defect 5: Enum Subclassing Inconsistency (Minor)
- **Target Pack File**: `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl`
- **Validation Tests File**: `/Users/sac/rocket-craft/ggen-validation-tests/subsystems.ttl`
  - Subclassed all specified C++ enum classes explicitly under `ue4:UEnum` using `rdfs:subClassOf ue4:UEnum ;`:
    - `EShaderFrequency`
    - `ERenderAPI`
    - `ECollisionResponse`
    - `ECollisionEnabled`
    - `ECollisionChannel`
    - `EPhysicsType`
    - `EDOFMode`

---

## Testing & Verification

1. **Rule verification execution**:
   - Incremented `TOTAL_TESTS=27` in `/Users/sac/rocket-craft/ggen-validation-tests/verify_all_rules.sh`.
   - Appended test case **26** checking RPC validation function class scope check.
   - Appended test case **27** checking Kinematic Simulation Disconnect check.
   - Executed `./verify_all_rules.sh` and `./verify_extra_rules.sh`. Both passed 100% of cases.
   - Executed `/Users/sac/rocket-craft/validate_ontology.sh`. It compiles and validates target pack cleanly (Custom validation rules increased from 61 to 63 rules).
