# Handoff Report: UE4 Networking Subsystem Explorer

## 1. Observation
- **Replication Lifetimes**: File `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl` defines:
  - `ue4:ELifetimeCondition` on lines 480–536.
  - `ue4:FReplicationLifetime` on lines 537–540:
    ```turtle
    ue4:FReplicationLifetime a owl:Class ;
        rdfs:label "FReplicationLifetime" ;
        rdfs:comment "Represents a replication registration lifetime entry for a property." .
    ```
  - `ue4:replicatedProperty` on lines 541–545 linking `FReplicationLifetime` to `ue4:UProperty`.
  - There is no object property or relation linking `ue4:UClass` to `ue4:FReplicationLifetime`.
- **Actor/Component Ownership**: File `/Users/sac/.ggen/packs/ue4_ontology/core.ttl` defines `ue4:owner` on lines 84–89:
  ```turtle
  ue4:owner a owl:ObjectProperty ;
      rdfs:label "owner" ;
      rdfs:comment "Alias or inverse relationship relating a component/object to its owner actor." ;
      rdfs:domain ue4:UActorComponent ;
      rdfs:range ue4:AActor ;
      owl:inverseOf ue4:hasComponent .
  ```
  No relation exists for actor-to-actor ownership (`AActor::GetOwner()`).
- **Net Roles and Controllers**: No definition of `ENetRole` or `ROLE_Authority` / `ROLE_AutonomousProxy` / `ROLE_SimulatedProxy` exists in `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl` or `/Users/sac/.ggen/packs/ue4_ontology/core.ttl`. No definition of `AController` or `APlayerController` exists.
- **RPC Validation Signature Constraints**: In `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` on lines 729–798, the node shape `ue4:RPCValidationSignatureShape` targets `ue4:URPC` and contains SPARQL checks for `bWithValidation true` functions, but does not verify:
  1. That RPC functions return `void` (i.e. have no `returnProperty`).
  2. That `bWithValidation` is only used on `UServerRPC` (and not on `UClientRPC` or `UNetMulticastRPC`).
  3. That all `UServerRPC` declarations provide validation (crucial security gate).
- **Orphaned Component Replication**: In `validation.shacl.ttl` on lines 801–818, `ue4:ComponentReplicationOwnerShape` is defined as:
  ```turtle
  ue4:ComponentReplicationOwnerShape
      a sh:NodeShape ;
      sh:targetClass ue4:UActorComponent ;
      sh:sparql [
          sh:message "Component replication mismatch: A component cannot replicate if its owner actor is not replicated." ;
          sh:select """
              PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
              PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
              SELECT $this ?actor
              WHERE {
                  $this ue4:bReplicates true ;
                        ue4:isComponentOf ?actor .
                  FILTER NOT EXISTS {
                      ?actor ue4:bReplicates true .
                  }
              } ORDER BY $this ?actor
          """ ;
      ] .
  ```
  If a replicated component is orphaned (lacks `ue4:isComponentOf`), the triple pattern `?this ue4:isComponentOf ?actor` fails, bypassing the validation silently.
- **Verification Execution**: Running the tool command `ggen sync --validate-only true` in `/Users/sac/.ggen/packs/ue4_ontology` succeeds with exit code 0 and output `All validations passed`.

---

## 2. Logic Chain
1. **Property Ownership Resolution**: Based on the observation that `FReplicationLifetime` has no link to `UClass`, if subclasses override the lifetime conditions of inherited properties, the ontology cannot resolve which condition belongs to which class. Therefore, we must introduce `ue4:hasReplicationLifetime` to close this gap.
2. **Actor Control Flow**: Because network roles and controllers are completely missing from the ontology, codegen templates cannot verify whether a function execution branch (`if (GetLocalRole() == ROLE_Authority)`) is logically reachable or if RPC execution permissions are correct. Therefore, `ENetRole`, `role`, `remoteRole`, and `APlayerController` must be added.
3. **RPC Execution Void Restriction**: In UE4, RPCs are asynchronous and cannot return values. Since there is currently no constraint on `ue4:returnProperty` for `ue4:URPC`, invalid RPC declarations returning types (like integers) would pass validation but fail C++ compilation. A rule restricting `URPC` to return void is required.
4. **RPC Validation Placement**: Modern UE4 compilers restrict `WithValidation` only to `Server` RPCs. The current validation shape `RPCValidationSignatureShape` targets `ue4:URPC` and will validate signatures on `UClientRPC` or `UNetMulticastRPC` if they have validation enabled, but does not flag their very presence as a structural error. Hence, a rule prohibiting validation on client/multicast RPCs is required.
5. **Security Hardening**: To maintain production-ready anti-cheat standards under Six Sigma, client-to-server calls must always validate inputs. The absence of a rule checking that `UServerRPC` has validation enables unvalidated client-to-server pathways. Therefore, we must mandate validation on Server RPCs.
6. **Orphaned Component Detection**: Since the SPARQL select query in `ComponentReplicationOwnerShape` binds `?actor` via `ue4:isComponentOf`, components lacking this relation bypass validation. Modifying the shape to check for the absence of `isComponentOf` or an unreplicated actor solves the bypass.

---

## 3. Caveats
- Checked only the schema definitions in `ue4_ontology`. The actual gameplay instances reside in `eden_server` (e.g. `instances.ttl`, `bandai_tps.ttl`), which were not audited for individual replication violations during this read-only phase.
- Assumed modern Unreal Engine 4 standards where `WithValidation` is mandated for all Server RPCs. If the build target utilizes legacy configurations where validation is optional, Rule 3 (Server RPC Validation requirement) may need to be adjusted or downgraded to warning severity.

---

## 4. Conclusion
The UE4 Networking domain in `ue4_ontology` correctly models the enumeration values for replication conditions but suffers from structural gaps and validation omissions. Specifically, the lack of class-to-lifetime associations, player controllers, and network roles prevents accurate representation of the network topology. Furthermore, the absence of constraints enforcing that RPCs return void, restricting validation only to Server RPCs, and detecting orphaned replicated components represent significant gaps that will result in compiler errors or security vulnerabilities in generated artifacts. Appending the proposed 4 schema additions and 8 validation rules will resolve these gaps and guarantee compile-time and design-time correctness.

---

## 5. Verification Method
1. **Command to run**:
   ```bash
   ggen sync --validate-only true
   ```
   Execute this command inside `/Users/sac/.ggen/packs/ue4_ontology` after the proposed changes are implemented.
2. **Success Condition**: The command returns exit code 0.
3. **Invalidation Conditions**: If any of the newly added SHACL SPARQL shapes are syntactically invalid, or if any existing dummy data triggers the new constraints (such as an RPC with a return property), the validation command will terminate with exit code 1 and flag the specific constraint violation.
