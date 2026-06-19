# Gap Analysis Report: RDF Ontologies, SHACL Shapes, and SPARQL Queries Audit

- **Date of Audit:** 2026-06-19T04:32:31Z
- **Auditor:** `explorer_m1` (Teamwork Explorer)
- **Target Packs:** `/Users/sac/.ggen/packs/eden_server/` and `/Users/sac/.ggen/packs/ue4_ontology/`

---

## Executive Summary
This audit maps the complete class, property, shape, and validation rule space of the `eden_server` and `ue4_ontology` RDF packs. While both ontologies possess a strong foundation in C++ reflection, basic mechanics (racing/manufacturing), and serialization validation, they display substantial structural gaps regarding R1-R12 requirements—specifically missing 9 of the 12 gameplay cells, 4 of the 8 states of resolution, all 5 semantic importance LOD classes, all dynamic rendering parameters, all walkthrough closure details, and 5 of the 12 authority state dimensions. All SPARQL queries in `.rq` files and inline in `ggen.toml` manifests strictly include an `ORDER BY` clause, satisfying determinism.

---

## 1. Map of Currently Defined Classes, Properties, and Shapes

### 1.1 Eden Server Ontology Pack (`/Users/sac/.ggen/packs/eden_server/`)

| File Path | RDF Classes Defined | RDF Properties Defined | SHACL Shapes & Validation Rules |
|:---|:---|:---|:---|
| `ontology/pack.ttl` | `eden:AssemblyComponent`, `eden:MechRoot`, `eden:SubAssembly`, `eden:Part`, `eden:Socket`, `eden:ReliabilitySensor` | `eden:hasSocket`, `eden:plugsInto`, `eden:damageClass`, `eden:stressClass`, `eden:heatClass`, `eden:fatigueClass` | None. Imports FIBO, SOSA, QUDT, PROV-O properties. |
| `ontology/bandai_tps.ttl` | `tps:ManufacturingMaterial`, `tps:PS`, `tps:ABS`, `tps:KPS`, `tps:PE`, `tps:FoundryProcess`, `tps:MultiColorInjection`, `tps:KanbanSignal`, `tps:JidokaGate`, `tps:ScaleGrade`, `tps:GradeHG`, `tps:GradeRG`, `tps:GradeMG`, `tps:GradePG` | `tps:hasMaterial`, `tps:hasGrade` | OWL Restrictions: `eden:MechRoot` has exactly 1 `tps:ScaleGrade` value on `tps:hasGrade`. Material subclasses are explicitly disjoint. |
| `ontology/deltas.ttl` | `eden:Delta`, `eden:AuthorityDelta`, `eden:AssemblyDelta`, `eden:ProjectionDelta`, `eden:InterestDelta`, `eden:ReceiptDelta` | `eden:deltaId`, `eden:sequenceNumber`, `eden:timestamp`, `eden:targetGraph`, `eden:deltaPayload`, `eden:authorizedBy`, `eden:appliesTo`, `eden:receiptFor`, `eden:targetComponent`, `eden:targetSocket`, `eden:installedComponent`, `eden:removedComponent`, `eden:subscriber`, `eden:interestedComponent`, `eden:prompt`, `eden:contractHash`, `eden:buildLog`, `eden:packagePath`, `eden:baselineScreenshot`, `eden:afterScreenshot`, `eden:consoleLogs`, `eden:inputTrace`, `eden:visualDelta`, `eden:verdict` | None. Base classes mapped to PROV-O equivalents. |
| `ontology/egp_racing.ttl` | `egp:VehicleRoot`, `egp:Tire`, `egp:Engine`, `egp:Chassis`, `egp:PitStrategy`, `egp:SectorTime` | `egp:gripClass`, `egp:heatClass` | None. Subclasses of `eden:Part` are disjoint. |
| `ontology/mars_market.ttl` | `mars:DimensionalAsset`, `mars:OwnershipRecord` | `mars:riskClass`, `mars:proofClass` | OWL Restriction: `mars:DimensionalAsset` must have some `mars:proofClass` value. |
| `ontology/validation_shapes.ttl` | None. | None. | **Node Shapes for Byte-Classes:**<br>- `DamageClassShape` (0-255)<br>- `StressClassShape` (0-255)<br>- `HeatClassShape` (0-255)<br>- `FatigueClassShape` (0-255)<br>- `GripClassShape` (0-255)<br>- `HeatClassShape` (0-255)<br>- `RiskClassShape` (0-255)<br>- `ProofClassShape` (0-255)<br>**Topology Shapes:**<br>- `egp:VehicleTiresShape` (Exactly 4 tires)<br>**Provenance Shapes:**<br>- `mars:DimensionalAssetProofShape` (Min 1 proofClass). |
| `ggen.toml` | None. | None. | **Manifest Rules (ASK):**<br>- `RuleClassHierarchy` (Hierarchy check)<br>- `RuleDisjointness` (No overlapping types for disjoint classes). |

### 1.2 UE4 Ontology Pack (`/Users/sac/.ggen/packs/ue4_ontology/`)

| File Path | RDF Classes Defined | RDF Properties Defined | SHACL Shapes & Validation Rules |
|:---|:---|:---|:---|
| `core.ttl` | `ue4:UObject`, `ue4:AActor`, `ue4:APawn`, `ue4:ACharacter`, `ue4:UActorComponent`, `ue4:USceneComponent`, `ue4:UWorld`, `ue4:ULevel` | `ue4:hasComponent`, `ue4:hasRootComponent`, `ue4:isComponentOf`, `ue4:owner`, `ue4:hasLevel`, `ue4:isLevelOf`, `ue4:persistentLevel`, `ue4:hasActor`, `ue4:bReplicates`, `ue4:bIsActive`, `ue4:bHidden` | None. Core C++ actor model framework. |
| `blueprints.ttl` | `ue4:UBlueprint`, `ue4:UEdGraph`, `ue4:UEdGraphNode`, `ue4:UEdGraphPin`, `ue4:UK2Node`, `ue4:UK2Node_Event`, `ue4:UK2Node_CustomEvent`, `ue4:UK2Node_CallFunction`, `ue4:UK2Node_CommutativeAssociativeBinaryOperator`, `ue4:UK2Node_Variable`, `ue4:UK2Node_VariableGet`, `ue4:UK2Node_VariableSet`, `ue4:UK2Node_ExecutionSequence`, `ue4:UK2Node_IfThenElse`, `ue4:UK2Node_DynamicCast`, `ue4:UK2Node_Literal`, `ue4:UK2Node_InputKeyEvent`, `ue4:UK2Node_InputAction`, `ue4:UK2Node_InputAxisEvent` | `ue4:hasGraph`, `ue4:hasNode`, `ue4:nodeOf`, `ue4:hasPin`, `ue4:pinOf`, `ue4:linkedTo`, `ue4:connectedTo`, `ue4:pinTypeObject`, `ue4:pinDirection`, `ue4:pinCategory`, `ue4:pinSubCategory`, `ue4:pinSubCategoryObject`, `ue4:defaultValue`, `ue4:bIsReference`, `ue4:bIsConst`, `ue4:nodePosX`, `ue4:nodePosY`, `ue4:calledFunction`, `ue4:callsFunction`, `ue4:mapsToParameter`, `ue4:referencedProperty`, `ue4:targetType` | None. Blueprint logic node structures. |
| `reflection.ttl` | `ue4:UField`, `ue4:UStruct`, `ue4:UClass`, `ue4:UScriptStruct`, `ue4:UFunction`, `ue4:UEnum`, `ue4:UProperty`, `ue4:UBoolProperty`, `ue4:UNumericProperty`, `ue4:UByteProperty`, `ue4:UIntProperty`, `ue4:UInt64Property`, `ue4:UFloatProperty`, `ue4:UDoubleProperty`, `ue4:UObjectProperty`, `ue4:UClassProperty`, `ue4:USoftObjectProperty`, `ue4:USoftClassProperty`, `ue4:UWeakObjectProperty`, `ue4:ULazyObjectProperty`, `ue4:UInterfaceProperty`, `ue4:UStructProperty`, `ue4:UArrayProperty`, `ue4:UMapProperty`, `ue4:USetProperty`, `ue4:UEnumProperty`, `ue4:UStrProperty`, `ue4:UNameProperty`, `ue4:UTextProperty`, `ue4:UDelegateProperty`, `ue4:UMulticastDelegateProperty`, `ue4:UFunctionParameter`, `ue4:PinDirection` | `ue4:hasField`, `ue4:hasProperty`, `ue4:hasFunction`, `ue4:superStruct`, `ue4:propertyType`, `ue4:returnProperty`, `ue4:classFlags`, `ue4:functionFlags`, `ue4:propertyFlags`, `ue4:hasParameter`, `ue4:parameterOf`, `ue4:parameterDirection`, `ue4:parameterIndex` | None. Includes enumerated direction individuals (`Input`, `Output`, `InOut`, `Return`). |
| `subsystems.ttl` | `ue4:USubsystem`, `ue4:URenderingSubsystem`, `ue4:UPhysicsSubsystem`, `ue4:UNetworkingSubsystem` | `ue4:hasSubsystemLifecycle` | None. Subsystem hierarchy. |
| `typestates.ttl` | `ue4:Typestate`, `ue4:CookingTypestate`, `ue4:LinkingTypestate`, `ue4:WasmPackagingTypestate` | `ue4:hasCookingState`, `ue4:hasLinkingState`, `ue4:hasPackagingState` | None. State representation variables. |
| `shacl/validation.shacl.ttl` | None. | None. | **Metadata Shapes:**<br>- `ClassLabelShape` (Label validation)<br>- `ClassCommentShape` (Comment warning)<br>- `NamespaceSanityShape` (Checks HTTP/HTTPS scheme)<br>**Graph & Parameter Verification:**<br>- `UFunctionParameterShape` (Cardinality & direction)<br>- `UEdGraphPinShape` (Cardinality & category)<br>- `PinConnectionDirectionShape` (SPARQL: direction mismatch)<br>- `PinConnectionGraphShape` (SPARQL: graph isolation)<br>- `FunctionCallPinMappingShape` (SPARQL: mapping integrity)<br>- `PinParameterDirectionMatchShape` (SPARQL: parameter direction match)<br>- `ExecPinConnectionShape` (SPARQL: exec pin separation). |
| `ggen.toml` | None. | None. | **Manifest Rules (ASK):**<br>- `R1` (Backbone class hierarchy check)<br>- `R2` (Subsystems subclass check)<br>- `R3` (Reflection subclass check)<br>- `R4` (Typestates check)<br>- `RuleA` (Connection direction check)<br>- `RuleB` (Graph isolation check)<br>- `RuleC` (Parameter mapping target check)<br>- `RuleD` (Pin parameter direction match)<br>- `RuleE` (Exec pin separation)<br>- `RuleF` (Character must have 1 CookingTypestate)<br>- `RuleG` (World must have 1 WasmPackagingTypestate)<br>- `RuleH` (Input execution pins on CallFunction must connect)<br>- `RuleLabel` (Label check)<br>- `RuleNamespace` (URN block check). |

---

## 2. Requirement Gap Analysis (R1-R12 Compliance)

### 2.1 Coverage of the 12 Gameplay Cells
* **Current Status:** Only 3 of the 12 gameplay cells have any model representation in the ontologies.
  * **Manufacturing:** Covered via `bandai_tps.ttl` (foundry processes, material polymers, Jidoka gates).
  * **Race:** Covered via `egp_racing.ttl` (racing vehicles, engine heat/tire grip parameters, pit strategy, sector records).
  * **Trade:** Covered via `mars_market.ttl` (dimensional assets, ownership, risk parameters).
* **Missing Cells (9):**
  1. **Repair:** No classes for repair shops, repair activities, maintenance schedules, or tool actuation.
  2. **Insurance:** No classes/properties tracking policies, damage claims, premiums, coverage status, or payout activities.
  3. **Prediction:** No representation for prediction contracts, forecasting telemetry, oracle consensus, or betting events.
  4. **Resource Collection:** No concepts for mining drills, resource veins, harvesting throughput, or storage cargo grids.
  5. **Infrastructure:** No spatial/structural nodes for power grids, roads, physical factory foundations, or factory nodes.
  6. **Defense:** No armor plating thickness, shield integrity, weapon hardpoints, active defense nodes, or faction territories.
  7. **Exploration:** No mapping for navigation sensors, scanned sectors, planetary surface anomalies, or scan progress states.
  8. **Discovery:** No unique artifacts, archeological relics, celestial object coordinates, or registration registry.
  9. **Research:** No tech tree hierarchies, research labs, laboratory experiments, or unlocking requirements.

### 2.2 States of Resolution
* **Current Status:** The ontologies model the lower end of the physical/logical assembly hierarchy:
  * `eden:SubAssembly` $\rightarrow$ Represents **Subassembly**.
  * `eden:Part` $\rightarrow$ Represents **Part**.
  * `eden:Socket` $\rightarrow$ Represents **Socket**.
  * `eden:MechRoot` or `egp:VehicleRoot` $\rightarrow$ Partially covers **Assembly**.
* **Missing States (4):**
  1. **Global:** No class modeling the overarching server/world system state or planetary registry level.
  2. **Regional:** No representation of geographic or network regions (e.g. Server Clusters, Regional Zones).
  3. **Zone:** No spatial zone classes (e.g. sector grids, race track coordinates, manufacturing plant sectors).
  4. **Facility:** No class structure representing individual facilities (e.g. a specific factory building, foundry site, or hangar).
* **Validation Gaps:** No SHACL constraints enforce hierarchical nested ownership between these layers (e.g. verifying that a `Socket` belongs to a `Part` which belongs to a `SubAssembly` which belongs to a `Facility` in a `Zone`).

### 2.3 Semantic Importance Classification LOD Classes
* **Current Status:** There is zero modeling of visual LOD (Level of Detail) or semantic priority in the RDF packs.
* **Missing Elements:**
  * No classes or individuals representing **CROWN**, **PRIMARY**, **SECONDARY**, **TERTIARY**, or **BACKGROUND** importance levels.
  * No properties to assign these importance classes to components.
  * No SHACL validation to assert that critical gameplay-active parts must have `CROWN` or `PRIMARY` status, while static clutter holds `BACKGROUND`.

### 2.4 Dynamic Rendering Parameters
* **Current Status:** While `bandai_tps.ttl` defines physical materials (`tps:PS`, `tps:ABS`), it lacks dynamic rendering descriptors.
* **Missing Parameters:**
  * **LOD Class Property:** Missing property mapping to LOD classes.
  * **Material Property:** No properties referencing engine materials/shaders (e.g., UE4 Material Instances).
  * **Instancing Property:** No properties indicating whether a part is instanced (`bIsInstanced`, instance group index).
  * **Semantic Importance Property:** Missing linkage to semantic importance classification.
  * **Silhouette Property:** No rendering parameters for highlight overlays or outline rendering.
  * **Interaction Distance Property:** Missing float/double property specifying threshold distances for interactable activation.

### 2.5 Walkthrough Closure Information
* **Current Status:** Entirely missing. There is no concept of game navigation graph mapping in the ontologies.
* **Missing Elements:**
  * **Locations:** No representation of navigation waypoints.
  * **Exits:** No classes representing portals, gates, doors, or level transition trigger points.
  * **Routes:** No concept of pathing vectors, corridors, or racing track lanes.
  * **Zones:** No zone-to-zone spatial connectivity graphs.
  * **Interactables:** No generic classes for consoles, buttons, levers, or sockets requiring user proximity.
  * **Facilities:** No representation of facility layouts or room networks.

### 2.6 Authority State Dimensions
* **Current Status:** 7 dimensions are modeled (either directly in `eden_server` or via extensions):
  * **Damage:** `eden:damageClass` (xsd:unsignedByte, [0-255])
  * **Heat:** `eden:heatClass` / `egp:heatClass` (xsd:unsignedByte, [0-255])
  * **Stress:** `eden:stressClass` (xsd:unsignedByte, [0-255])
  * **Fatigue:** `eden:fatigueClass` (xsd:unsignedByte, [0-255])
  * **Grip:** `egp:gripClass` (xsd:unsignedByte, [0-255])
  * **Risk:** `mars:riskClass` (xsd:unsignedByte, [0-255])
  * **Provenance:** `mars:proofClass` (xsd:unsignedByte, [0-255])
* **Missing Dimensions (5):**
  1. **Energy:** Missing byte-class representation for remaining fuel, battery, or power grid charge.
  2. **Resource:** Missing byte-class representation for raw material stock levels or cargo mass.
  3. **Market Condition:** Missing byte-class representation for market volatility, inflation indices, or transaction tax rates.
  4. **Conformance:** Missing byte-class representation for structural compliance to blueprints (spec compliance).
  5. **Standing:** Missing byte-class representation for player faction reputation levels or licensing clearances.
* **Byte-Class Verification:** The existing byte-classes have SHACL shapes verifying `xsd:unsignedByte` within `[0, 255]`. The missing 5 dimensions do not have properties, meaning they also lack SHACL validator coverage.

---

## 3. SPARQL Query Audit

All SPARQL queries in `.rq` files and inline in `ggen.toml` manifests were audited.

### 3.1 Verification Matrix

| File Path | Query Location / Name | Query Type | Explicit `ORDER BY` Clause | Deterministic Verdict |
|:---|:---|:---|:---|:---|
| `eden_server/ggen.toml` | `infer-socket-hosts-component` | CONSTRUCT | Yes (`ORDER BY ?socket ?component`) | **PASS** |
| `eden_server/ggen.toml` | `infer-component-is-hosted-by` | CONSTRUCT | Yes (`ORDER BY ?component ?socket`) | **PASS** |
| `eden_server/ggen.toml` | `dummy` (generation rules) | SELECT | Yes (`ORDER BY ?s`) | **PASS** |
| `eden_server/queries/extract_assembly_deltas.rq` | File root query | SELECT | Yes (`ORDER BY DESC(?timestamp) ?delta`) | **PASS** |
| `eden_server/queries/extract_authority_deltas.rq` | File root query | SELECT | Yes (`ORDER BY DESC(?timestamp) ?delta`) | **PASS** |
| `eden_server/queries/extract_receipt_deltas.rq` | File root query | SELECT | Yes (`ORDER BY DESC(?timestamp) ?delta`) | **PASS** |
| `eden_server/queries/substrate.rq` | File root query | SELECT | Yes (`ORDER BY ?root ?parent ?socket ?child`) | **PASS** |
| `ue4_ontology/ggen.toml` | `infer-is-component-of` | CONSTRUCT | Yes (`ORDER BY ?actor ?component`) | **PASS** |
| `ue4_ontology/ggen.toml` | `infer-is-level-of` | CONSTRUCT | Yes (`ORDER BY ?world ?level`) | **PASS** |
| `ue4_ontology/ggen.toml` | `readme` (generation rules) | SELECT | Yes (`ORDER BY ?s`) | **PASS** |

### 3.2 Audit Findings on Validation Queries
While all `.rq` and `ggen.toml` query blocks pass the determinism audit, there is a discrepancy in the **SHACL validation file** (`/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`):
* SPARQL-based validator shapes (`PinConnectionDirectionShape`, `PinConnectionGraphShape`, `FunctionCallPinMappingShape`, `PinParameterDirectionMatchShape`, `ExecPinConnectionShape`) utilize a `sh:select` query to find invalid patterns.
* None of these validation queries have an `ORDER BY` clause.
* **Rationale:** Although SHACL validation queries are meant to return violation bindings (where order is generally handled by the SHACL engine itself), omitting `ORDER BY` may cause non-deterministic reporting sequences in specific testing tools that dump validation violations directly to raw logs.
* **Recommendation:** Add an explicit `ORDER BY $this ?other` to all `sh:select` blocks in `validation.shacl.ttl` to maintain absolute determinism across all engines.

---

## 4. Comprehensive File Gap Catalog

### 4.1 `eden_server` Pack
1. **`ontology/pack.ttl`**
   * *Gap:* Missing base vocabulary for States of Resolution (Global, Regional, Zone, Facility), LOD categories, and pathing/walkthrough topology.
   * *Solution:* Define classes: `eden:GlobalState`, `eden:RegionalState`, `eden:ZoneState`, `eden:FacilityState`. Define `eden:LodClass` (with individuals `eden:Crown`, `eden:Primary`, etc.). Define walkthrough/pathing classes: `eden:Location`, `eden:Exit`, `eden:Route`, `eden:Interactable`.
2. **`ontology/egp_racing.ttl`**
   * *Gap:* Limited in scope only to Race. Missing other game cells like Prediction, Repair, and Defense.
   * *Solution:* Create corresponding domain files or extend EGP with class nodes like `egp:RacePrediction`, `egp:WeaponHardpoint`.
3. **`ontology/validation_shapes.ttl`**
   * *Gap:* Missing validation shapes for new authority dimensions (Energy, Resource, Market Condition, Conformance, Standing) and new resolution hierarchies.
   * *Solution:* Implement `eden:EnergyClassShape`, `eden:ResourceClassShape`, etc., limiting values to `xsd:unsignedByte` [0-255]. Write shape rules verifying that a `Facility` must target nested `Zones`.
4. **`ggen.toml`**
   * *Gap:* Manifest validation rules do not test for states of resolution and lod-classification constraints.
   * *Solution:* Append ASK rules checking resolution class taxonomy structure.

### 4.2 `ue4_ontology` Pack
1. **`core.ttl`**
   * *Gap:* Lacks linkage between the core C++ backbone (`AActor`, `UActorComponent`) and dynamic rendering properties (silhouettes, interaction distance, instancing).
   * *Solution:* Declare datatype properties on `ue4:USceneComponent` like `ue4:interactionDistance` (xsd:float), `ue4:bIsInstanced` (xsd:boolean), and `ue4:silhouetteColor` (xsd:string).
2. **`shacl/validation.shacl.ttl`**
   * *Gap 1:* Lack of explicit `ORDER BY` clauses in SPARQL validation shapes.
   * *Gap 2:* Lack of shape representation for `RuleF` (Character must have 1 CookingState), `RuleG` (World must have 1 PackagingState), and `RuleH` (Unconnected input execution pins).
   * *Solution:* Update all `sh:select` query fields with `ORDER BY $this ...`. Transpile `RuleF`, `RuleG`, and `RuleH` into native SHACL NodeShapes to match the manifest validation rules.
3. **`ggen.toml`**
   * *Gap:* Validation rules are redundant with SHACL shapes but lack a strict 1-to-1 sync.
   * *Solution:* Synchronize ASK validation queries with the SHACL file to ensure both validation layers compile and execute identically.

---

## 5. Proposed Remediation Artifacts

Below are the suggested RDF declarations and SHACL validation structures to bridge the gaps.

### 5.1 Proposed Walkthrough, LOD, and Resolution Extensions (To add to `eden_server/ontology/pack.ttl`)
```turtle
# --- States of Resolution ---
eden:ResolutionState a owl:Class ;
    rdfs:label "Resolution State" ;
    rdfs:comment "Represents the hierarchical levels of spatial/logical resolution." .

eden:Global a eden:ResolutionState, owl:NamedIndividual ; rdfs:label "Global Resolution" .
eden:Regional a eden:ResolutionState, owl:NamedIndividual ; rdfs:label "Regional Resolution" .
eden:Zone a eden:ResolutionState, owl:NamedIndividual ; rdfs:label "Zone Resolution" .
eden:Facility a eden:ResolutionState, owl:NamedIndividual ; rdfs:label "Facility Resolution" .
eden:Assembly a eden:ResolutionState, owl:NamedIndividual ; rdfs:label "Assembly Resolution" .
eden:Subassembly a eden:ResolutionState, owl:NamedIndividual ; rdfs:label "Subassembly Resolution" .
eden:Part a eden:ResolutionState, owl:NamedIndividual ; rdfs:label "Part Resolution" .
eden:Socket a eden:ResolutionState, owl:NamedIndividual ; rdfs:label "Socket Resolution" .

# --- LOD Classes ---
eden:LodImportance a owl:Class ;
    rdfs:label "LOD Importance Class" .

eden:CROWN a eden:LodImportance, owl:NamedIndividual ; rdfs:label "Crown LOD Level" .
eden:PRIMARY a eden:LodImportance, owl:NamedIndividual ; rdfs:label "Primary LOD Level" .
eden:SECONDARY a eden:LodImportance, owl:NamedIndividual ; rdfs:label "Secondary LOD Level" .
eden:TERTIARY a eden:LodImportance, owl:NamedIndividual ; rdfs:label "Tertiary LOD Level" .
eden:BACKGROUND a eden:LodImportance, owl:NamedIndividual ; rdfs:label "Background LOD Level" .

# --- Walkthrough / Pathing ---
eden:WaypointNode a owl:Class ;
    rdfs:label "Waypoint Node" ;
    rdfs:comment "A navigation node within the walkthrough graph." .

eden:ExitPortal a owl:Class ;
    rdfs:label "Exit Portal" ;
    rdfs:comment "A connection path between two facilities or zones." .

eden:connectedToWaypoint a owl:ObjectProperty ;
    rdfs:label "connected to waypoint" ;
    rdfs:domain eden:WaypointNode ;
    rdfs:range eden:WaypointNode .
```

### 5.2 Proposed Byte-Class Authority Extensions (To add to `eden_server/ontology/pack.ttl` and `validation_shapes.ttl`)
```turtle
# --- New Datatype Properties ---
eden:energyClass a owl:DatatypeProperty , owl:FunctionalProperty ;
    rdfs:subPropertyOf qudt:value ;
    rdfs:domain eden:AssemblyComponent ;
    rdfs:range xsd:unsignedByte .

eden:resourceClass a owl:DatatypeProperty , owl:FunctionalProperty ;
    rdfs:subPropertyOf qudt:value ;
    rdfs:domain eden:AssemblyComponent ;
    rdfs:range xsd:unsignedByte .

eden:marketConditionClass a owl:DatatypeProperty , owl:FunctionalProperty ;
    rdfs:subPropertyOf qudt:value ;
    rdfs:domain eden:AssemblyComponent ;
    rdfs:range xsd:unsignedByte .

eden:conformanceClass a owl:DatatypeProperty , owl:FunctionalProperty ;
    rdfs:subPropertyOf qudt:value ;
    rdfs:domain eden:AssemblyComponent ;
    rdfs:range xsd:unsignedByte .

eden:standingClass a owl:DatatypeProperty , owl:FunctionalProperty ;
    rdfs:subPropertyOf qudt:value ;
    rdfs:domain eden:AssemblyComponent ;
    rdfs:range xsd:unsignedByte .

# --- Associated SHACL Validation Shapes ---
eden:EnergyClassShape a sh:NodeShape ;
    sh:targetSubjectsOf eden:energyClass ;
    sh:property [
        sh:path eden:energyClass ;
        sh:datatype xsd:unsignedByte ;
        sh:minInclusive 0 ;
        sh:maxInclusive 255 ;
        sh:message "energyClass must be an unsignedByte in range [0, 255]" ;
    ] .
```
