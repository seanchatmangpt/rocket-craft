## 2026-06-18T23:19:00Z
You are the Subsystem Topologies Remediation Worker (Worker) for the UE4 Universal RDF Mapping project.
Your working directory is `/Users/sac/rocket-craft/.agents/worker_remediation_m4_gen3`.
Your task is to fix the five schema and validation defects identified by Reviewer 2 in the Subsystem Topologies implementation.

Specifically, you must:
1. Read the Reviewer 2 Review Report at `/Users/sac/rocket-craft/.agents/reviewer_subsystems_m4_2_gen3/review.md` and their Handoff Report at `/Users/sac/rocket-craft/.agents/reviewer_subsystems_m4_2_gen3/handoff.md`.

2. Apply the following corrections to BOTH the target pack (`/Users/sac/.ggen/packs/ue4_ontology/`) and the validation tests folder (`/Users/sac/rocket-craft/ggen-validation-tests/`):

   - **Defect 1: Class Scope Validation for RPC Validation Functions (Major)**
     - Add a SHACL constraint in `validation.shacl.ttl` and a corresponding validation rule in `ggen.toml` (e.g., `RuleRPCValidationClassScope`) to ensure that if an RPC specifies `bWithValidation true` and a `validationFunction`, the validation function is a member of the same class (or a base class) as the RPC.
     - SPARQL check logic:
       ```sparql
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
       ```
     - Make sure to add this rule to `ggen.toml` and implement a corresponding SHACL shape in `validation.shacl.ttl`.
     
   - **Defect 2: Subclass-aware Parameter Type Check under Inheritance (Major)**
     - In `MaterialInstanceParameterValueTypeShape` (`validation.shacl.ttl`) and `RuleMaterialInstanceParameterValueType` (`ggen.toml`), update the checks so they don't use direct class equality (`?paramType = ue4:UScalarParameter`). Instead, traverse subclasses using `a/rdfs:subClassOf* ue4:UScalarParameter`, `a/rdfs:subClassOf* ue4:UVectorParameter`, and `a/rdfs:subClassOf* ue4:UTextureParameter`.
     
   - **Defect 3: Kinematic Parameter Disconnect check (Medium)**
     - Add a SHACL constraint shape and a `ggen.toml` rule (e.g., `RuleKinematicSimulationDisconnect`) ensuring that if a primitive component has `ue4:bSimulatePhysics true`, its associated rigid body (`ue4:hasRigidBody ?body`) must have `ue4:physicsType` set to `ue4:PhysType_Simulated` or `ue4:PhysType_Default`.
     
   - **Defect 4: Domain Limitation for Collision Channel Overrides (Medium)**
     - In `subsystems.ttl`, expand the domain of `ue4:hasChannelResponse` from strictly `ue4:UCollisionProfile` to the union of `ue4:UPrimitiveComponent` and `ue4:UCollisionProfile` to support component-level overrides without RDFS classification punning.
     - Code example:
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

   - **Defect 5: Enum Subclassing Inconsistency (Minor)**
     - Ensure that all classes representing C++ enums in `subsystems.ttl` explicitly subclass `ue4:UEnum` (including `EShaderFrequency`, `ERenderAPI`, `ECollisionResponse`, `ECollisionEnabled`, `ECollisionChannel`, `EPhysicsType`, `EDOFMode`).

3. Add new test cases to `verify_all_rules.sh` to verify the new constraints (such as an RPC validation function mismatch, or a kinematic simulation contradiction), incrementing `TOTAL_TESTS` accordingly. Make sure the new tests pass.

4. Run the validation command `/Users/sac/rocket-craft/validate_ontology.sh` to ensure it compiles/validates cleanly.

5. Run `verify_all_rules.sh` and `verify_extra_rules.sh` to ensure all tests pass successfully.

6. Write a detailed report to `changes.md` in your working directory.
7. Write a handoff report at `handoff.md` and send a message back to the parent orchestrator (conversation ID: 26a55229-63bc-48fa-bc48-7ec491f0dfa3) with the path to your handoff.md.
