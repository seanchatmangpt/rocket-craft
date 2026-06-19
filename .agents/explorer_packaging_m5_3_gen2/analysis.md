# Packaging and RHI Target Config Modeling Analysis

This document explores and proposes extensions to the Unreal Engine 4 RDF typestates ontology (`typestates.ttl`) and SHACL validation rules (`validation.shacl.ttl`) to model packaging targets, RHI profiles, and Projection Law constraints.

---

## 1. Analysis of Current Ontology & SHACL Shapes

### Current Typestates Ontology (`typestates.ttl`)
The current ontology defines basic typestates:
- `ue4:Typestate`: Base class.
- `ue4:CookingTypestate`, `ue4:LinkingTypestate`, `ue4:WasmPackagingTypestate`: Derived classes.
- `ue4:hasCookingState`, `ue4:hasLinkingState`, `ue4:hasPackagingState`: Relational properties mapping projects or worlds to these states.

**Gaps Identified:**
- No concept of **Build Configurations** (Debug, Development, Shipping) or compiler optimizations.
- No concepts for **Target Platforms** (e.g. HTML5/WASM).
- No concepts for **RHI Profiles** (specifically targeting WebGL 2.0 / OpenGL ES3 parameters).
- Lack of structures to represent **The Projection Law constraints** (static baking flag, GGen authority output directories, preventing raw graphics/WebGL data from leaking into the ontology).

### Current SHACL Validation (`validation.shacl.ttl`)
The validation shapes enforce node structures, pin connections, execution flows, and some physics/networking rules.
- **Rule M (WASM WebGL Fallback Compliance):** Asserts that if a rendering subsystem is associated with a world in `WasmPackagingTypestate`, it must support `ue4:RHI_WebGL2`, `ue4:RHI_OpenGL_ES3`, or `ue4:RHI_WebGL`.

**Gaps Identified:**
- The shape verifies the subsystem capabilities but does not check the **packaging configuration** itself.
- There are no validations checking if the world is statically baked or if output paths (headers, DataTables, BOM, walkthrough, byte-classes, receipts) are present.
- There is no SHACL shape preventing the illegal injection of raw WebGL assets (meshes, textures) directly into the ontology.
- No verification that dynamic REST tools (like `VaRest`) are disabled when a world is configured for static baking.

---

## 2. Proposed RDF Schema Extensions

To bridge these gaps, we propose adding the following classes, properties, and relationships to `typestates.ttl`.

### A. Packaging Configuration and Build Configuration
```turtle
# Class Definitions
ue4:BuildConfiguration a owl:Class ;
    rdfs:label "BuildConfiguration" ;
    rdfs:comment "Specifies the compile-time optimization, symbol, and logging policies." .

ue4:PackagingTarget a owl:Class ;
    rdfs:label "PackagingTarget" ;
    rdfs:comment "Holds build, cooking, and packaging settings for a target world deployment." .

# Concrete Build Configuration Instances
ue4:Config_Debug a ue4:BuildConfiguration ;
    rdfs:label "Debug" ;
    rdfs:comment "Debug configuration. Optimizations disabled, full debug symbols, logging enabled." ;
    ue4:bOptimize false ;
    ue4:bEnableSymbols true ;
    ue4:bDisableLogging false ;
    ue4:bEnableDebugConsole true .

ue4:Config_Development a ue4:BuildConfiguration ;
    rdfs:label "Development" ;
    rdfs:comment "Development configuration. Optimizations enabled, symbols enabled, logging enabled." ;
    ue4:bOptimize true ;
    ue4:bEnableSymbols true ;
    ue4:bDisableLogging false ;
    ue4:bEnableDebugConsole true .

ue4:Config_Shipping a ue4:BuildConfiguration ;
    rdfs:label "Shipping" ;
    rdfs:comment "Shipping configuration. Optimizations enabled, symbols stripped, logging stripped, console disabled." ;
    ue4:bOptimize true ;
    ue4:bEnableSymbols false ;
    ue4:bDisableLogging true ;
    ue4:bEnableDebugConsole false .

# Relational Properties
ue4:targetWorld a owl:ObjectProperty ;
    rdfs:label "targetWorld" ;
    rdfs:comment "Relates a packaging target to the target UWorld being packaged." ;
    rdfs:domain ue4:PackagingTarget ;
    rdfs:range ue4:UWorld .

ue4:buildConfiguration a owl:ObjectProperty ;
    rdfs:label "buildConfiguration" ;
    rdfs:comment "Relates a packaging target to a build configuration (Debug, Development, Shipping)." ;
    rdfs:domain ue4:PackagingTarget ;
    rdfs:range ue4:BuildConfiguration .

ue4:targetPlatform a owl:DatatypeProperty ;
    rdfs:label "targetPlatform" ;
    rdfs:comment "Specifies target deployment environment (e.g. 'HTML5', 'WASM')." ;
    rdfs:domain ue4:PackagingTarget ;
    rdfs:range xsd:string .
```

### B. WebGL 2.0 / OpenGL ES3 Target RHI Profiles
```turtle
ue4:RHIProfile a owl:Class ;
    rdfs:label "RHIProfile" ;
    rdfs:comment "Specifies target rendering hardware interface limits and extension configuration." .

ue4:targetRHIProfile a owl:ObjectProperty ;
    rdfs:label "targetRHIProfile" ;
    rdfs:comment "Links a packaging target to its target RHI profile." ;
    rdfs:domain ue4:PackagingTarget ;
    rdfs:range ue4:RHIProfile .

ue4:hasRenderAPI a owl:ObjectProperty ;
    rdfs:label "hasRenderAPI" ;
    rdfs:comment "Links the RHI profile to an ERenderAPI instance." ;
    rdfs:domain ue4:RHIProfile ;
    rdfs:range ue4:ERenderAPI .

ue4:glslVersion a owl:DatatypeProperty ;
    rdfs:label "glslVersion" ;
    rdfs:comment "Target shading language version string (e.g. '300 es' for WebGL 2.0)." ;
    rdfs:domain ue4:RHIProfile ;
    rdfs:range xsd:string .

ue4:bUseInstancedArrays a owl:DatatypeProperty ;
    rdfs:label "bUseInstancedArrays" ;
    rdfs:comment "Enables WebGL hardware-accelerated geometry instancing." ;
    rdfs:domain ue4:RHIProfile ;
    rdfs:range xsd:boolean .

ue4:bEnableWebGLVsync a owl:DatatypeProperty ;
    rdfs:label "bEnableWebGLVsync" ;
    rdfs:comment "Instructs WASM wrapper to synchronize canvas flips with browser screen vsync." ;
    rdfs:domain ue4:RHIProfile ;
    rdfs:range xsd:boolean .

# Standard Profiles
ue4:WebGL2_RHI_Profile a ue4:RHIProfile ;
    rdfs:label "WebGL 2.0 Profile" ;
    ue4:hasRenderAPI ue4:RHI_WebGL2 ;
    ue4:glslVersion "300 es" ;
    ue4:bUseInstancedArrays true ;
    ue4:bEnableWebGLVsync true .

ue4:OpenGLES3_RHI_Profile a ue4:RHIProfile ;
    rdfs:label "OpenGL ES 3.0 Profile" ;
    ue4:hasRenderAPI ue4:RHI_OpenGL_ES3 ;
    ue4:glslVersion "300 es" ;
    ue4:bUseInstancedArrays true ;
    ue4:bEnableWebGLVsync true .
```

### C. The Projection Law Constraints
Under the Projection Law, the ontology must act as semantic authority, while Unreal owns the actual rendering assets (pixels). We must represent:
1. **Static Baking:** Ensuring that layout/LOD structure is baked, not fetched dynamically at runtime via plugins like VaRest.
2. **Output Paths:** Declaring output paths for the generated C++ headers, JSON/CSV DataTables, Render BOM metadata, walkthrough coordinates, byte-class matrices, and receipt paths.
3. **Preventing Direct WebGL/Mesh Representation in RDF:** Forbidding raw binary mesh data or textures from leaking into the ontology.

```turtle
ue4:StaticBakingConfiguration a owl:Class ;
    rdfs:label "StaticBakingConfiguration" ;
    rdfs:comment "Baking policy configuration mapping semantic graphs to compiler artifacts." .

ue4:hasStaticBaking a owl:ObjectProperty ;
    rdfs:label "hasStaticBaking" ;
    rdfs:comment "Links a packaging target to its static baking configuration." ;
    rdfs:domain ue4:PackagingTarget ;
    rdfs:range ue4:StaticBakingConfiguration .

ue4:isStaticallyBaked a owl:DatatypeProperty ;
    rdfs:label "isStaticallyBaked" ;
    rdfs:comment "Enforces compile-time static injection (eliminating runtime REST queries)." ;
    rdfs:domain ue4:StaticBakingConfiguration ;
    rdfs:range xsd:boolean .

# Projection Law Mandated Output Directories
ue4:headerOutputPath a owl:DatatypeProperty ;
    rdfs:label "headerOutputPath" ;
    rdfs:comment "Relative or absolute path for generated C++ headers." ;
    rdfs:domain ue4:StaticBakingConfiguration ;
    rdfs:range xsd:string .

ue4:dataTableOutputPath a owl:DatatypeProperty ;
    rdfs:label "dataTableOutputPath" ;
    rdfs:comment "Relative or absolute path for generated Unreal Engine DataTables." ;
    rdfs:domain ue4:StaticBakingConfiguration ;
    rdfs:range xsd:string .

ue4:bomOutputPath a owl:DatatypeProperty ;
    rdfs:label "bomOutputPath" ;
    rdfs:comment "Relative or absolute path for Render Bill of Materials (BOM)." ;
    rdfs:domain ue4:StaticBakingConfiguration ;
    rdfs:range xsd:string .

ue4:walkthroughOutputPath a owl:DatatypeProperty ;
    rdfs:label "walkthroughOutputPath" ;
    rdfs:comment "Relative or absolute path for Playwright walkthrough coordinate list." ;
    rdfs:domain ue4:StaticBakingConfiguration ;
    rdfs:range xsd:string .

ue4:byteClassMatrixOutputPath a owl:DatatypeProperty ;
    rdfs:label "byteClassMatrixOutputPath" ;
    rdfs:comment "Relative or absolute path for byte-class matrices." ;
    rdfs:domain ue4:StaticBakingConfiguration ;
    rdfs:range xsd:string .

ue4:receiptOutputPath a owl:DatatypeProperty ;
    rdfs:label "receiptOutputPath" ;
    rdfs:comment "Relative or absolute path for build/run cryptographic BLAKE3 receipts." ;
    rdfs:domain ue4:StaticBakingConfiguration ;
    rdfs:range xsd:string .

# Semantic Reference Class (No pixel/mesh data allowed here)
ue4:SemanticAssetReference a owl:Class ;
    rdfs:label "SemanticAssetReference" ;
    rdfs:comment "Declares a reference to a asset packages handled by Unreal. Holds no binary mesh or raw texture data in RDF." .

ue4:ueProjectAssetPath a owl:DatatypeProperty ;
    rdfs:label "ueProjectAssetPath" ;
    rdfs:comment "Path of the asset in the Unreal content directory (e.g. '/Game/Props/Spaceship.uasset')." ;
    rdfs:domain ue4:SemanticAssetReference ;
    rdfs:range xsd:string .
```

---

## 3. Proposed SHACL Validation Shapes

To enforce these configurations and prevent structural violations of the Projection Law, we propose the following shapes to be added to `validation.shacl.ttl`:

### A. Packaging Target Constraints Shape
Ensures that packaging configurations contain all required relationships and target valid platforms.
```turtle
ue4:PackagingTargetShape
    a sh:NodeShape ;
    sh:targetClass ue4:PackagingTarget ;
    sh:property [
        sh:path ue4:targetWorld ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:class ue4:UWorld ;
        sh:message "A packaging target must specify exactly one targetWorld." ;
    ] ;
    sh:property [
        sh:path ue4:buildConfiguration ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:class ue4:BuildConfiguration ;
        sh:message "A packaging target must specify exactly one buildConfiguration." ;
    ] ;
    sh:property [
        sh:path ue4:targetRHIProfile ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:class ue4:RHIProfile ;
        sh:message "A packaging target must specify exactly one RHIProfile." ;
    ] ;
    sh:property [
        sh:path ue4:targetPlatform ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:datatype xsd:string ;
        sh:in ( "HTML5" "WASM" ) ;
        sh:message "A packaging target for WASM packaging must target either 'HTML5' or 'WASM'." ;
    ] .
```

### B. Build Configuration Optimizations Validator
Validates that shipping builds conform to release optimizations and safety requirements.
```turtle
ue4:BuildConfigurationConsistencyShape
    a sh:NodeShape ;
    sh:targetClass ue4:BuildConfiguration ;
    sh:sparql [
        sh:message "Shipping configuration violation: Shipping builds must optimize code, disable logging, and disable debugging symbols." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this
            WHERE {
                $this rdfs:label "Shipping" .
                { $this ue4:bOptimize false }
                UNION
                { $this ue4:bEnableSymbols true }
                UNION
                { $this ue4:bDisableLogging false }
            }
        """ ;
    ] .
```

### C. Projection Law: Outputs Enforce Validator
Ensures that a statically baked project configuration defines every single mandated GGen output path.
```turtle
ue4:StaticBakingPathsShape
    a sh:NodeShape ;
    sh:targetClass ue4:StaticBakingConfiguration ;
    sh:property [
        sh:path ue4:isStaticallyBaked ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:datatype xsd:boolean ;
    ] ;
    sh:sparql [
        sh:message "Projection Law violation: Statically baked configurations must declare output paths for C++ headers, DataTables, BOM, walkthroughs, byte-class matrices, and receipts." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this
            WHERE {
                $this ue4:isStaticallyBaked true .
                FILTER (
                    NOT EXISTS { $this ue4:headerOutputPath ?h } ||
                    NOT EXISTS { $this ue4:dataTableOutputPath ?d } ||
                    NOT EXISTS { $this ue4:bomOutputPath ?b } ||
                    NOT EXISTS { $this ue4:walkthroughOutputPath ?w } ||
                    NOT EXISTS { $this ue4:byteClassMatrixOutputPath ?m } ||
                    NOT EXISTS { $this ue4:receiptOutputPath ?r }
                )
            }
        """ ;
    ] .
```

### D. Anti-Asset Injection (Asset Ownership Boundaries)
Directly checks if any RDF resource attempts to generate or define raw binary graphics assets within the ontology, enforcing the division of labor: **"We own the authority. Unreal owns the pixels."**
```turtle
ue4:PreventRawAssetGenerationShape
    a sh:NodeShape ;
    sh:targetSubjectsOf rdf:type ;
    sh:sparql [
        sh:message "Projection Law violation: RDF ontology cannot define raw pixel, mesh binary, or geometry data. It must only contain semantic references to Unreal-managed assets." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?property
            WHERE {
                $this ?property ?value .
                FILTER (?property IN (
                    ue4:binaryMeshData,
                    ue4:rawGltfString,
                    ue4:texturePixelBytes,
                    ue4:objFileContent,
                    ue4:fbxFileContent
                ))
            }
        """ ;
    ] .
```

### E. Static Baking: Prohibition of Dynamic REST (VaRest)
Strictly enforces that a statically baked target world does not contain blueprint logic executing dynamic REST calls at runtime (which violates GGen compiler-based layout injection).
```turtle
ue4:StaticBakingNoVaRestShape
    a sh:NodeShape ;
    sh:targetClass ue4:PackagingTarget ;
    sh:sparql [
        sh:message "Projection Law violation: Statically baked target worlds must not use dynamic VaRest calls for layout or Semantic LOD retrieval." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?node
            WHERE {
                $this ue4:hasStaticBaking ?bakeConfig .
                ?bakeConfig ue4:isStaticallyBaked true .
                $this ue4:targetWorld ?world .
                
                # Search for VaRest calls in the blueprint nodes of the target world
                ?world (ue4:hasLevel/ue4:hasActor/ue4:hasBlueprintNode|ue4:hasBlueprintNode) ?node .
                ?node ue4:callsFunction ?func .
                FILTER (
                    CONTAINS(STR(?func), "VaRest") || 
                    CONTAINS(LCASE(STR(?func)), "varest")
                )
            }
        """ ;
    ] .
```

---

## 4. Key Recommendations

1. **Adopt Typestate Configuration Separation**: Maintain separate RDF files for topology description vs build target configuration (e.g. keep `typestates.ttl` focused on taxonomy, while instantiating concrete configurations in a dedicated target file, such as `targets.ttl`).
2. **Strict Output Paths Binding**: GGen must ingest the output paths from the `StaticBakingConfiguration` directly to determine where C++ code and DataTables are generated. This makes the RDF ontology the direct orchestrator of GGen's generation step.
3. **Mandate Playwright Walkthrough coordinates in Ontology**: Do not store the verification trajectory separately. Walkthrough locations should be modeled as an ordered list of coordinates linked directly to the `targetWorld` in the ontology, guaranteeing that visual testing is bound to the semantic structure of the world.
