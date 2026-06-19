# UE4 Universal RDF Mapping: E2E Testing Infrastructure (TEST_INFRA)

## 1. Introduction & Objectives

This document establishes the end-to-end (E2E) testing and validation framework for the Unreal Engine 4 (UE4) Universal RDF Mapping project. The primary goal of this framework is to verify that the generated ontology files (`core.ttl`, `reflection.ttl`, `blueprints.ttl`, `subsystems.ttl`, and `typestates.ttl`) conform to strict syntactic, structural, and semantic requirements.

The final authority of this system is governed by the **TPS/DfLSS Playwright Manufacturing Strategy** outlined in `GEMINI.md`. A successful compilation or passing unit test is only a check on assembly; real victory is defined as the verified motion of a packaged WebGL/Unreal world artifact inside a local browser under Playwright automation. This RDF ontology mapping layer sits at **GATE 0 (Source Admission)** and **GATE 1 (Unreal Artifact Admission)**, ensuring that the semantic blueprint of the world is mathematically coherent before code generation and cooking begin.

---

## 2. The 4-Tier Acceptance Testing Methodology

To guarantee ontological integrity without relying on brittle implementation-specific mocks, the testing suite is structured into four distinct coverage tiers:

```
┌─────────────────────────────────────────────────────────────────┐
│              Tier 4: Real-World Application Scenarios           │
│              (Complete pathways, E2E character packaging)       │
└────────────────────────────────┬────────────────────────────────┘
                                 │
┌────────────────────────────────▼────────────────────────────────┐
│           Tier 3: Cross-Feature Combinations (Pairwise)         │
│           (Blueprint nodes referencing C++ Core classes, etc.)  │
└────────────────────────────────┬────────────────────────────────┘
                                 │
┌────────────────────────────────▼────────────────────────────────┐
│           Tier 2: Boundary & Corner Cases                       │
│           (Namespace validation, circular dependencies, etc.)   │
└────────────────────────────────┬────────────────────────────────┘
                                 │
┌────────────────────────────────▼────────────────────────────────┐
│           Tier 1: Feature Coverage                              │
│           (Key class existence and properties, >= 5 per feature)│
└─────────────────────────────────────────────────────────────────┘
```

### Tier 1: Feature Coverage
Validates the presence, declaration, and basic relationships of key classes and properties across all four core features. Every feature must have at least five distinct, documented test cases.

#### Feature 1: Core C++ Backbone
1. **Case 1.1: Root Object Declaration (`ue4:UObject`)**
   - Verification: Verify that `ue4:UObject` is defined as a class (`rdfs:Class` or `owl:Class`).
2. **Case 1.2: Actor Inheritance (`ue4:AActor`)**
   - Verification: Verify that `ue4:AActor` is defined and has an explicit `rdfs:subClassOf` relationship to `ue4:UObject`.
3. **Case 1.3: Pawn Inheritance (`ue4:APawn`)**
   - Verification: Verify that `ue4:APawn` is defined and has an explicit `rdfs:subClassOf` relationship to `ue4:AActor`.
4. **Case 1.4: Character Inheritance (`ue4:ACharacter`)**
   - Verification: Verify that `ue4:ACharacter` is defined and has an explicit `rdfs:subClassOf` relationship to `ue4:APawn`.
5. **Case 1.5: Component Backbone (`ue4:UActorComponent`)**
   - Verification: Verify that `ue4:UActorComponent` is defined and has an explicit `rdfs:subClassOf` relationship to `ue4:UObject`.
6. **Case 1.6: Spatial Hierarchy (`ue4:UWorld` and `ue4:ULevel`)**
   - Verification: Verify that `ue4:UWorld` and `ue4:ULevel` are defined as subclasses of `ue4:UObject`.

#### Feature 2: Subsystems
1. **Case 2.1: Subsystem Base Class (`ue4:USubsystem`)**
   - Verification: Verify that `ue4:USubsystem` is defined as a subclass of `ue4:UObject`.
2. **Case 2.2: Rendering Subsystem (`ue4:URenderingSubsystem`)**
   - Verification: Verify that `ue4:URenderingSubsystem` is defined and has an explicit `rdfs:subClassOf` relationship to `ue4:USubsystem`.
3. **Case 2.3: Physics Subsystem (`ue4:UPhysicsSubsystem`)**
   - Verification: Verify that `ue4:UPhysicsSubsystem` is defined and has an explicit `rdfs:subClassOf` relationship to `ue4:USubsystem`.
4. **Case 2.4: Networking Subsystem (`ue4:UNetworkingSubsystem`)**
   - Verification: Verify that `ue4:UNetworkingSubsystem` is defined and has an explicit `rdfs:subClassOf` relationship to `ue4:USubsystem`.
5. **Case 2.5: Subsystem Lifecycle Property (`ue4:hasSubsystemLifecycle`)**
   - Verification: Verify that the property `ue4:hasSubsystemLifecycle` exists with `ue4:USubsystem` as its domain.

#### Feature 3: Reflection & Blueprints
1. **Case 3.1: Metaclass Declarations (`ue4:UClass`, `ue4:UStruct`, `ue4:UField`)**
   - Verification: Verify the presence of reflection metaclasses and their inheritance (`ue4:UClass rdfs:subClassOf ue4:UStruct`).
2. **Case 3.2: Property Declaration (`ue4:UProperty`)**
   - Verification: Verify that `ue4:UProperty` is defined as a subclass of `ue4:UField`.
3. **Case 3.3: Function Declaration (`ue4:UFunction`)**
   - Verification: Verify that `ue4:UFunction` is defined as a subclass of `ue4:UStruct`.
4. **Case 3.4: Blueprint Graph Representation (`ue4:UEdGraph`)**
   - Verification: Verify that `ue4:UEdGraph` is defined as a subclass of `ue4:UObject`.
5. **Case 3.5: Blueprint Node Base (`ue4:UEdGraphNode`)**
   - Verification: Verify that `ue4:UEdGraphNode` is defined as a subclass of `ue4:UObject`.
6. **Case 3.6: Kismet Compiler Nodes (`ue4:UK2Node`)**
   - Verification: Verify that `ue4:UK2Node` is defined as a subclass of `ue4:UEdGraphNode`.

#### Feature 4: Cooking & WASM Typestates
1. **Case 4.1: Typestate Base Definition (`ue4:Typestate`)**
   - Verification: Verify that `ue4:Typestate` is defined as an ontological class.
2. **Case 4.2: Cooking Typestate (`ue4:CookingTypestate`)**
   - Verification: Verify that `ue4:CookingTypestate` is defined and inherits from `ue4:Typestate`.
3. **Case 4.3: Linking Typestate (`ue4:LinkingTypestate`)**
   - Verification: Verify that `ue4:LinkingTypestate` is defined and inherits from `ue4:Typestate`.
4. **Case 4.4: Packaging Typestate (`ue4:WasmPackagingTypestate`)**
   - Verification: Verify that `ue4:WasmPackagingTypestate` is defined and inherits from `ue4:Typestate`.
5. **Case 4.5: Cooking/Linking/Packaging State Properties**
   - Verification: Verify properties `ue4:hasCookingState`, `ue4:hasLinkingState`, and `ue4:hasPackagingState` are declared as `rdf:Property`.

---

### Tier 2: Boundary & Corner Cases
Enforces structural rules, quality constraints, and naming conventions. This tier detects defects that compile successfully but violate ontological standards. Every feature area is subject to at least five boundary checks.

1. **Case 2.1: Mandatory Labels (SHACL Rule)**
   - Check: All classes must have at least one `rdfs:label`.
   - Violation: Any class without an `rdfs:label` fails validation.
2. **Case 2.2: Mandatory Descriptions (SHACL Rule)**
   - Check: All classes should have at least one `rdfs:comment`.
   - Violation: Class without `rdfs:comment` raises a warning.
3. **Case 2.3: Namespace Sanity (SHACL Rule)**
   - Check: All subjects must use public, resolvable HTTP/HTTPS IRIs (e.g., `<https://rocket-craft.io/ontology/ue4/>`).
   - Violation: Subjects using private, opaque URIs like `<urn:private:ue4:AActor>` fail validation.
4. **Case 2.4: Circular Inheritance Detection (SPARQL Rule)**
   - Check: Class inheritance graphs must be Directed Acyclic Graphs (DAGs).
   - Violation: If `A subClassOf B` and `B subClassOf A`, validation fails.
5. **Case 2.5: Root Orphan Prevention (SPARQL Rule)**
   - Check: All classes must terminate at `ue4:UObject` or a known primitive, preventing disconnected islands.
   - Violation: Disconnected class trees fail strict validation.

---

### Tier 3: Cross-Feature Combinations (Pairwise Interaction)
Validates interactions between different components of the UE4 architecture to verify that static and dynamic models are fully unified.

1. **Case 3.1: Blueprint Executable Node to C++ Backbone**
   - Check: Verify that `ue4:UK2Node` (Blueprint Node) references a valid `ue4:UFunction` (C++ Reflection) and executes within a `ue4:ACharacter` (C++ Core Backbone) context.
2. **Case 3.2: Subsystem Lifecycle Registered via Reflection**
   - Check: Verify that `ue4:URenderingSubsystem` lifecycle events map to valid `ue4:UFunction` hooks defined in the reflection graph.
3. **Case 3.3: Typestate Transitions Associated with C++ Classes**
   - Check: Verify that the `ue4:WasmPackagingTypestate` maps directly to a specific target `ue4:UWorld` and `ue4:ULevel` configuration.

---

### Tier 4: Real-World Application Scenarios
Simulates realistic, complex usage patterns that compile all sub-systems together into a single cohesive game loop definition.

1. **Case 4.1: The Gundam Player Character Scenario**
   - Scenario: Define a `ue4:ACharacter` subclass representing a Gundam. It contains:
     - A rendering component (`ue4:USkeletalMeshComponent`).
     - A physics component (`ue4:UBoxComponent`).
     - A blueprint graph (`ue4:UEdGraph`) with input events mapping keys to movement.
     - A subsystem handler (`ue4:UNetworkingSubsystem`) for server replication.
     - A typestate tracking its cooking status (`ue4:CookingTypestate` status: `ue4:Cooked`) and packaging status (`ue4:WasmPackagingTypestate` status: `ue4:WasmReady`).
   - Verification: SPARQL queries must verify that all parts of the Gundam character are structurally and logically connected without dangling links.

---

## 3. Playwright Manufacturing Strategy Alignment

The 4-tier acceptance methodology directly feeds into the **Playwright Manufacturing Strategy** described in `GEMINI.md`:

1. **Gate 0 (Source Admission):** Validated by `validate_ontology.sh`. The ontology structure must pass all Tier 1, 2, and 3 validation shapes/rules before source code generation is allowed to begin.
2. **Gate 1 (Unreal Artifact Admission):** The RDF graph must provide complete, valid C++ declarations so that the `ggen` compiler can output compilation-ready `.h` and `.cpp` files.
3. **Gate 2 (HTML5/WASM Package Admission):** Typestates defined in `typestates.ttl` track the WASM/HTML5 packaging states, verifying that build systems can produce browser-deployable output.
4. **Gates 3-7 (Runtime and Verification):** Playwright uses the generated bindings to interact with the browser-native Unreal 4 WebGL environment, executing inputs and verifying visual movement deltas.

---

## 4. Repair Routing Taxonomy

If the E2E testing pipeline detects a failure, the pipeline does not execute a generic repair loop. Instead, the failure is categorized and routed to specific engineering cells:

| Failure Mode | Defect Origin | Assigned Routing Cell |
|---|---|---|
| SPARQL Rule R1-R4 failure | Incomplete class model or hierarchy | **Rocket-Craft Contract Cell** |
| SHACL syntax / layout failure | Naming conventions, missing metadata | **SHACL Validation / RDF Cell** |
| Missing source files (`core.ttl`, etc.) | Packaging/sync config error | **Local Serving / Build Cell** |
| WASM compilation / packaging errors | WebGL/Emscripten settings | **HTML5 Packaging Cell** |
| Playwright assertion or motion delta fail | Input mapping or WebGL rendering failure | **WebGL / Input-Binding Cell** |

---

## 5. Validation Infrastructure & Commands

The validation suite is driven by:

1. **`ggen.toml`**: Located at `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml`. Contains the pack metadata, schema imports, and validation rules.
2. **`validation.shacl.ttl`**: SHACL shapes file at `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`. Enforces naming conventions and metadata requirements.
3. **`validate_ontology.sh`**: Harness script at `/Users/sac/rocket-craft/validate_ontology.sh`. Changes directory to the pack and runs the compiler's validation sync command.

### Verification Command

To run the validation test suite manually or from CI/CD, execute:

```bash
/Users/sac/rocket-craft/validate_ontology.sh
```

A non-zero exit code indicates a validation defect. Detailed outputs are printed directly to `stdout` and `stderr`.
