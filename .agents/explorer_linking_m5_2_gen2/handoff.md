# Handoff Report — Compilation Linking and WASM Memory Layout Modeling

This handoff report summarizes findings, proposed ontology modifications, and validation rules to model compiler optimizations and WebAssembly memory layouts.

## 1. Observation
- **Current typestates.ttl Ontology (`/Users/sac/.ggen/packs/ue4_ontology/typestates.ttl`)**:
  - Contains basic typestate classes on lines 21-29:
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
  - It does not contain any properties or classes representing compiler optimization levels, stack size, heap size (initial/maximum memory), allow-memory-growth, or export lists.
- **Current validation.shacl.ttl Shapes (`/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`)**:
  - The only WASM-related constraint is **Rule M: WASM WebGL Fallback Compliance** (`ue4:WasmSubsystemWebGLFallbackShape`) on lines 556-575.
  - No memory boundaries, compiler flag alignments, or linker export lists are validated.
- **Unreal Build pipeline source files (`tools/rocket-sdk/src/html5.rs` and `tools/world-factory/cook_html5.sh`)**:
  - Run the Unreal Automation Tool (UAT) `BuildCookRun` script for the HTML5 target platform using statically defined configuration settings.

---

## 2. Logic Chain
1. Since the compilation linking states and WebAssembly memory settings are completely absent from the ontology (Observation 1), the system has no semantic awareness of compile-time configuration parameters.
2. Because the build pipeline uses shell variables and hardcoded arguments (Observation 3), a mismatch between compile-time memory layouts (e.g., small stack sizes causing runtime stack overflows in deep engine calls) is only detected late in the E2E Playwright validation step.
3. Introducing classes `ue4:WasmMemoryLayout`, `ue4:CompilerOptimizationLevel`, and `ue4:LinkingConfiguration` provides a structured, reified representation of Emscripten linker options directly in the RDF schema (Section 3 of `analysis.md`).
4. Writing SHACL validation shapes (`ue4:WasmMemoryLayoutShape`, `ue4:LinkingConfigurationShape`, `ue4:LinkingTypestateConfigurationShape`) allows verifying alignment parameters (WASM 64KB page size multiples), stack boundaries (stack size must be strictly smaller than initial heap memory), fixed-heap requirements, and optimization flag compatibility (e.g., denying unoptimized `-O0` flags in `Shipping` build modes) prior to compiling.
5. This compile-time guard aligns with the project's **Branchless Typestates** doctrine, preventing configuration conflicts and minimizing pipeline waste.

---

## 3. Caveats
- The proposed memory layout boundaries are designed for WASM32 (max 2GB/4GB heap). Target architectures using WASM64 (which allows larger heaps) would require updating the maximum bounds check in `ue4:WasmMemoryLayoutShape`.
- Integration of these RDF attributes into the actual generation code templates (e.g. rewriting rust-sdk or world-factory UAT commands to query target configurations using SPARQL) has not been implemented, as this is a read-only investigation.

---

## 4. Conclusion
We recommend updating `/Users/sac/.ggen/packs/ue4_ontology/typestates.ttl` and `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` with the schemas and SHACL validation shapes detailed in `analysis.md`. This extension establishes formal semantic contracts for compiling and linking configurations, moving defect detection to the early static validation gate.

---

## 5. Verification Method
- **Ontology Inspection**: Open and inspect `/Users/sac/rocket-craft/.agents/explorer_linking_m5_2_gen2/analysis.md` to verify the syntax correctness and structure of the proposed OWL classes, properties, and SHACL validation shapes.
- **Validation Test cases**:
  - Once implemented, verification can be performed by running a SHACL engine (or `ggen` validation CLI command) against a test graph:
    1. Verify that a configuration with initial memory = `100000` bytes (non-multiple of `65536`) fails validation.
    2. Verify that a configuration with stack size = `10485760` bytes (10MB) and initial memory = `8388608` bytes (8MB) fails validation.
    3. Verify that a `Shipping` build mode using `-O0` optimization level fails validation.
    4. Verify that a correct configuration (e.g. O3 optimization, 8MB stack, 512MB initial memory, 512MB max memory, memory growth false, and export `_main`) successfully passes validation.
