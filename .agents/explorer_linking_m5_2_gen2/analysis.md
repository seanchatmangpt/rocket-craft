# RDF Modeling of Compilation Linking and WASM Memory Layout for Unreal Engine 4

## 1. Introduction and Scope
This analysis outlines the semantic modeling required to represent compilation linking states, WebAssembly (WASM) memory layouts, and compiler optimization levels within the Unreal Engine 4 Universal RDF Mapping project. By encoding these compiler-level configuration options and requirements into the ontology, the project ensures that world packaging pipelines are mathematically validated prior to compilation, preventing compiler or runtime memory-exhaustion failures before executing the build toolchain.

The target system is the SpeculativeCoder UE4.27 HTML5 ES3 fork, which builds browser-native WASM outputs.

---

## 2. Current State Analysis
### 2.1. Existing Typestates Ontology
The current configuration in `/Users/sac/.ggen/packs/ue4_ontology/typestates.ttl` declares only high-level classes and properties for typestates:
- `ue4:LinkingTypestate` (Class representing compilation linking, lines 21-24)
- `ue4:WasmPackagingTypestate` (Class representing HTML5/WASM packaging, lines 26-29)
- `ue4:hasLinkingState` (ObjectProperty relating a component/module to its linking state, lines 35-37)
- `ue4:hasPackagingState` (ObjectProperty relating a world/level to its WASM packaging state, lines 39-41)

```ttl
ue4:LinkingTypestate a owl:Class ;
    rdfs:subClassOf ue4:Typestate ;
    rdfs:label "LinkingTypestate" ;
    rdfs:comment "State representing compilation linking." .

ue4:WasmPackagingTypestate a owl:Class ;
    rdfs:subClassOf ue4:Typestate ;
    rdfs:label "WasmPackagingTypestate" ;
    rdfs:comment "State representing HTML5/WASM packaging." .
```

### 2.2. Existing SHACL / SPARQL Rules
In `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`, the only rule referencing WASM packaging is **Rule M: WASM WebGL Fallback Compliance** (`ue4:WasmSubsystemWebGLFallbackShape`, lines 556-575). This rule verifies that rendering subsystems operating in a world with a packaging state support WebGL 1.0, WebGL 2.0 (OpenGL ES3), or OpenGL ES3.

There are **no current rules** validating:
1. WebAssembly memory boundaries (stack size vs heap boundaries).
2. Optimization flag correctness or build configuration consistency.
3. Linker symbols, export lists, or stack/heap page alignment.

---

## 3. Proposed RDF Modeling
To support compile-time verification of WASM memory and compiler optimizations, we propose introducing three new classes and several datatype/object properties.

### 3.1. Class Definitions
1. **`ue4:WasmMemoryLayout`**: Represents the memory footprint parameters passed to the Emscripten linker.
2. **`ue4:CompilerOptimizationLevel`**: Represents the specific optimization profiles (`-O0`, `-O1`, `-O2`, `-O3`, `-Os`, `-Oz`).
3. **`ue4:LinkingConfiguration`**: Integrates the memory layout, optimization settings, and target environment requirements into a single profile.

### 3.2. Turtle Ontology Extension Prosposal
The following Turtle content is proposed to extend `/Users/sac/.ggen/packs/ue4_ontology/typestates.ttl`:

```ttl
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ue4: <https://rocket-craft.io/ontology/ue4/> .

# --- New Classes ---

ue4:WasmMemoryLayout a owl:Class ;
    rdfs:label "WasmMemoryLayout" ;
    rdfs:comment "Memory allocation layout configuration for WASM/HTML5 compiles." .

ue4:CompilerOptimizationLevel a owl:Class ;
    rdfs:label "CompilerOptimizationLevel" ;
    rdfs:comment "Reified compiler optimization levels defining target behavior." .

ue4:LinkingConfiguration a owl:Class ;
    rdfs:label "LinkingConfiguration" ;
    rdfs:comment "Structural configuration that governs compilation linking." .

# --- New Properties ---

ue4:hasLinkingConfiguration a owl:ObjectProperty, rdf:Property ;
    rdfs:label "hasLinkingConfiguration" ;
    rdfs:comment "Relates a LinkingTypestate to its compiler configuration." ;
    rdfs:domain ue4:LinkingTypestate ;
    rdfs:range ue4:LinkingConfiguration .

ue4:hasMemoryLayout a owl:ObjectProperty, rdf:Property ;
    rdfs:label "hasMemoryLayout" ;
    rdfs:comment "Relates a linking configuration to its WASM memory layout." ;
    rdfs:domain ue4:LinkingConfiguration ;
    rdfs:range ue4:WasmMemoryLayout .

ue4:hasOptimizationLevel a owl:ObjectProperty, rdf:Property ;
    rdfs:label "hasOptimizationLevel" ;
    rdfs:comment "Relates a linking configuration to its optimization level." ;
    rdfs:domain ue4:LinkingConfiguration ;
    rdfs:range ue4:CompilerOptimizationLevel .

ue4:wasmStackSize a owl:DatatypeProperty, rdf:Property ;
    rdfs:label "wasmStackSize" ;
    rdfs:comment "Size of the WASM stack in bytes (Emscripten STACK_SIZE)." ;
    rdfs:domain ue4:WasmMemoryLayout ;
    rdfs:range xsd:integer .

ue4:wasmInitialMemory a owl:DatatypeProperty, rdf:Property ;
    rdfs:label "wasmInitialMemory" ;
    rdfs:comment "Initial WASM heap size in bytes (Emscripten INITIAL_MEMORY)." ;
    rdfs:domain ue4:WasmMemoryLayout ;
    rdfs:range xsd:integer .

ue4:wasmMaximumMemory a owl:DatatypeProperty, rdf:Property ;
    rdfs:label "wasmMaximumMemory" ;
    rdfs:comment "Maximum WASM heap limit in bytes (Emscripten MAXIMUM_MEMORY)." ;
    rdfs:domain ue4:WasmMemoryLayout ;
    rdfs:range xsd:integer .

ue4:wasmAllowMemoryGrowth a owl:DatatypeProperty, rdf:Property ;
    rdfs:label "wasmAllowMemoryGrowth" ;
    rdfs:comment "Boolean flag indicating if WASM heap is allowed to grow (Emscripten ALLOW_MEMORY_GROWTH)." ;
    rdfs:domain ue4:WasmMemoryLayout ;
    rdfs:range xsd:boolean .

ue4:wasmExportedSymbol a owl:DatatypeProperty, rdf:Property ;
    rdfs:label "wasmExportedSymbol" ;
    rdfs:comment "An exported function name exposed by the WASM module." ;
    rdfs:domain ue4:WasmMemoryLayout ;
    rdfs:range xsd:string .

ue4:buildMode a owl:DatatypeProperty, rdf:Property ;
    rdfs:label "buildMode" ;
    rdfs:comment "The target configuration mode for the build (e.g. Debug, Development, Shipping)." ;
    rdfs:domain ue4:LinkingConfiguration ;
    rdfs:range xsd:string .

ue4:optFlag a owl:DatatypeProperty, rdf:Property ;
    rdfs:label "optFlag" ;
    rdfs:comment "Command line optimization flag (e.g. -O3, -Oz)." ;
    rdfs:domain ue4:CompilerOptimizationLevel ;
    rdfs:range xsd:string .

ue4:favorsSize a owl:DatatypeProperty, rdf:Property ;
    rdfs:label "favorsSize" ;
    rdfs:comment "Indicates if the optimization level prioritizes reducing binary size." ;
    rdfs:domain ue4:CompilerOptimizationLevel ;
    rdfs:range xsd:boolean .

ue4:favorsPerformance a owl:DatatypeProperty, rdf:Property ;
    rdfs:label "favorsPerformance" ;
    rdfs:comment "Indicates if the optimization level prioritizes execution speed." ;
    rdfs:domain ue4:CompilerOptimizationLevel ;
    rdfs:range xsd:boolean .

# --- Optimization Individuals ---

ue4:Opt_O0 a ue4:CompilerOptimizationLevel ;
    rdfs:label "O0" ;
    ue4:optFlag "-O0" ;
    ue4:favorsSize false ;
    ue4:favorsPerformance false .

ue4:Opt_O1 a ue4:CompilerOptimizationLevel ;
    rdfs:label "O1" ;
    ue4:optFlag "-O1" ;
    ue4:favorsSize false ;
    ue4:favorsPerformance false .

ue4:Opt_O2 a ue4:CompilerOptimizationLevel ;
    rdfs:label "O2" ;
    ue4:optFlag "-O2" ;
    ue4:favorsSize false ;
    ue4:favorsPerformance true .

ue4:Opt_O3 a ue4:CompilerOptimizationLevel ;
    rdfs:label "O3" ;
    ue4:optFlag "-O3" ;
    ue4:favorsSize false ;
    ue4:favorsPerformance true .

ue4:Opt_Os a ue4:CompilerOptimizationLevel ;
    rdfs:label "Os" ;
    ue4:optFlag "-Os" ;
    ue4:favorsSize true ;
    ue4:favorsPerformance true .

ue4:Opt_Oz a ue4:CompilerOptimizationLevel ;
    rdfs:label "Oz" ;
    ue4:optFlag "-Oz" ;
    ue4:favorsSize true ;
    ue4:favorsPerformance false .
```

---

## 4. Proposed SHACL & SPARQL Validation Shapes
To validate WASM constraints, we propose adding three target shapes to `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`. These shapes execute compiler-level static analysis on the RDF graph before launching the actual build pipeline.

### 4.1. `ue4:WasmMemoryLayoutShape`
Enforces page alignment, stack-heap separation bounds, and browser-native maximum limits.

```ttl
ue4:WasmMemoryLayoutShape
    a sh:NodeShape ;
    sh:targetClass ue4:WasmMemoryLayout ;
    
    # 1. Stack Size validation
    sh:property [
        sh:path ue4:wasmStackSize ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:datatype xsd:integer ;
        sh:minInclusive 65536 ; # Minimal execution stack (64KB)
        sh:message "WASM stack size must be a single positive integer (minimum 65536 bytes)." ;
    ] ;
    
    # 2. Initial Memory presence
    sh:property [
        sh:path ue4:wasmInitialMemory ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:datatype xsd:integer ;
        sh:message "WASM initial memory must be a single positive integer." ;
    ] ;
    
    # 3. Maximum Memory presence
    sh:property [
        sh:path ue4:wasmMaximumMemory ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:datatype xsd:integer ;
        sh:message "WASM maximum memory must be a single positive integer." ;
    ] ;
    
    # 4. Memory Growth flag presence
    sh:property [
        sh:path ue4:wasmAllowMemoryGrowth ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:datatype xsd:boolean ;
        sh:message "WASM allow memory growth must be a single boolean." ;
    ] ;

    # 5. SPARQL Constraint: Initial Memory must be aligned to 64KB (65536 bytes) page size
    sh:sparql [
        sh:message "WASM Initial Memory page alignment violation: Initial Memory must be a multiple of 65536 bytes (WASM page size)." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?initialMemory
            WHERE {
                $this ue4:wasmInitialMemory ?initialMemory .
                # Verify that modulo 65536 is not zero
                FILTER (?initialMemory <= 0 || (?initialMemory - (65536 * FLOOR(?initialMemory / 65536))) != 0)
            }
        """ ;
    ] ;

    # 6. SPARQL Constraint: Maximum Memory must be aligned to 64KB page size
    sh:sparql [
        sh:message "WASM Maximum Memory page alignment violation: Maximum Memory must be a multiple of 65536 bytes (WASM page size)." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?maxMemory
            WHERE {
                $this ue4:wasmMaximumMemory ?maxMemory .
                # Verify that modulo 65536 is not zero
                FILTER (?maxMemory <= 0 || (?maxMemory - (65536 * FLOOR(?maxMemory / 65536))) != 0)
            }
        """ ;
    ] ;

    # 7. SPARQL Constraint: Stack size < Initial Memory size
    sh:sparql [
        sh:message "WASM Memory boundary mismatch: Stack size must be strictly smaller than the Initial Memory size." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?stack ?initialMemory
            WHERE {
                $this ue4:wasmStackSize ?stack .
                $this ue4:wasmInitialMemory ?initialMemory .
                FILTER (?stack >= ?initialMemory)
            }
        """ ;
    ] ;

    # 8. SPARQL Constraint: Initial Memory <= Maximum Memory
    sh:sparql [
        sh:message "WASM Heap limit violation: Initial Memory must be less than or equal to Maximum Memory." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?initialMemory ?maxMemory
            WHERE {
                $this ue4:wasmInitialMemory ?initialMemory .
                $this ue4:wasmMaximumMemory ?maxMemory .
                FILTER (?initialMemory > ?maxMemory)
            }
        """ ;
    ] ;

    # 9. SPARQL Constraint: Fixed heap requires matching bounds (AllowMemoryGrowth = false)
    sh:sparql [
        sh:message "WASM Fixed Heap constraint violation: When memory growth is disabled, Initial Memory must equal Maximum Memory." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?initialMemory ?maxMemory
            WHERE {
                $this ue4:wasmAllowMemoryGrowth false .
                $this ue4:wasmInitialMemory ?initialMemory .
                $this ue4:wasmMaximumMemory ?maxMemory .
                FILTER (?initialMemory != ?maxMemory)
            }
        """ ;
    ] ;

    # 10. SPARQL Constraint: WASM32 address space boundaries (maximum 2GB / 2,147,483,648 bytes)
    sh:sparql [
        sh:message "WASM32 Maximum Memory limit exceeded: Maximum Memory cannot exceed 2147483648 bytes (2GB address limit in 32-bit WASM)." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?maxMemory
            WHERE {
                $this ue4:wasmMaximumMemory ?maxMemory .
                FILTER (?maxMemory > 2147483648)
            }
        """ ;
    ] ;

    # 11. SPARQL Constraint: Essential Entry Point Symbol Exports
    sh:sparql [
        sh:message "WASM exported symbol list incomplete: The memory layout must export the '_main' symbol to be runnable by the browser harness." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this
            WHERE {
                $this a ue4:WasmMemoryLayout .
                FILTER NOT EXISTS {
                    $this ue4:wasmExportedSymbol "_main" .
                }
            }
        """ ;
    ] .
```

### 4.2. `ue4:LinkingConfigurationShape`
Enforces that compile/link configurations are consistent with their build configuration mode.

```ttl
ue4:LinkingConfigurationShape
    a sh:NodeShape ;
    sh:targetClass ue4:LinkingConfiguration ;
    
    sh:property [
        sh:path ue4:hasMemoryLayout ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:class ue4:WasmMemoryLayout ;
        sh:message "A linking configuration must reference exactly one WASM memory layout." ;
    ] ;
    
    sh:property [
        sh:path ue4:hasOptimizationLevel ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:class ue4:CompilerOptimizationLevel ;
        sh:message "A linking configuration must reference exactly one compiler optimization level." ;
    ] ;
    
    sh:property [
        sh:path ue4:buildMode ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:datatype xsd:string ;
        sh:in ( "Debug" "Development" "Shipping" ) ;
        sh:message "A linking configuration must specify a buildMode ('Debug', 'Development', or 'Shipping')." ;
    ] ;

    # SPARQL: Shipping build must use optimized compiler flags
    sh:sparql [
        sh:message "Shipping build optimization violation: A 'Shipping' build mode must not use -O0 (unoptimized) flag configuration." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?opt
            WHERE {
                $this ue4:buildMode "Shipping" .
                $this ue4:hasOptimizationLevel ?opt .
                ?opt ue4:optFlag "-O0" .
            }
        """ ;
    ] ;

    # SPARQL: Debug build should avoid high performance optimization (Warning)
    sh:sparql [
        sh:message "Debug build optimization warning: A 'Debug' build mode should avoid high optimization levels (prefer -O0 or -O1)." ;
        sh:severity sh:Warning ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?opt ?flag
            WHERE {
                $this ue4:buildMode "Debug" .
                $this ue4:hasOptimizationLevel ?opt .
                ?opt ue4:optFlag ?flag .
                FILTER (?flag != "-O0" && ?flag != "-O1")
            }
        """ ;
    ] .
```

### 4.3. `ue4:LinkingTypestateConfigurationShape`
Ensures that any linking process state instance is backed by a valid linking configuration profile.

```ttl
ue4:LinkingTypestateConfigurationShape
    a sh:NodeShape ;
    sh:targetClass ue4:LinkingTypestate ;
    
    sh:property [
        sh:path ue4:hasLinkingConfiguration ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:class ue4:LinkingConfiguration ;
        sh:message "A linking typestate must refer to exactly one linking configuration." ;
    ] .
```

---

## 5. Architectural Alignment with Project Doctrine
1. **Branchless Typestates ($A = \mu(O^*)$)**: This proposal aligns with the Project Doctrine by pushing compilation constraints into compile-time logic checks. If any configuration parameters (e.g. stack size vs heap size boundaries) violate layout requirements, the SHACL validator fails *before* wasting compilation CPU cycles.
2. **WebGL Fallback and HTML5 Integration**: Memory profiles can be associated directly with `ue4:WasmPackagingTypestate`, validating that the chosen memory boundaries do not exceed target browser constraints (e.g., 2GB boundaries for 32-bit browsers running WebGL/ES3).
3. **Playwright Acceptance Guard**: Misconfigured memory layouts (e.g., stack overflows due to insufficient `wasmStackSize` for Unreal's engine initialization stack) are caught deterministically. By validating configuration states, we can route target configuration defects to the "HTML5 packaging cell" or "input-binding cell" as dictated by the Repair Routing Law.

---

## 6. Recommendations & Implementation Pathway
To deploy these changes, the following incremental steps are recommended for the implementer:
1. **Merge Turtle Definitions**: Append the class and property declarations from Section 3 into `/Users/sac/.ggen/packs/ue4_ontology/typestates.ttl`.
2. **Merge SHACL validation shapes**: Append the three validation shapes in Section 4 to `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`.
3. **Template Generator Update**: Update the compiler build generator templates (such as the Tera templates generating Emscripten compilation configs or shell arguments in `tools/rocket-sdk`) to query these properties from the RDF graph and automatically output the corresponding `-s STACK_SIZE`, `-s INITIAL_MEMORY`, `-s MAXIMUM_MEMORY`, and `-O<level>` compiler options.
4. **Validation Test Integration**: Create test graphs representing:
   - *Valid Shipping configuration*: O3 optimization, 8MB stack, 512MB initial memory, 1024MB max memory, allow growth true, exported `_main`.
   - *Invalid configurations*: misaligned heap (e.g., 50MB, not a multiple of 64KB), stack size exceeding initial memory, shipping build using `-O0`.
   Verify that `ggen` validation rejects the invalid ones and passes the valid one.
