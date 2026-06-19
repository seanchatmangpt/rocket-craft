# Rendering Subsystem Modeling Analysis and Recommendations

This report explores the semantic modeling of the Unreal Engine 4 (UE4) rendering pipeline, focusing on materials, shader compilation mapping, and WebGL/RHI runtime API fallbacks in the context of the HTML5/WASM target platform.

---

## 1. Materials and Parameter Override Hierarchy

In UE4, materials are structured hierarchically:
- **`UMaterialInterface`**: The common abstract base class for both materials and material instances.
- **`UMaterial`**: The base asset that contains the full graph node definition (expressions, compilers) and default parameter values.
- **`UMaterialInstance`**: Inherits parameters from a parent `UMaterialInterface` and overrides them. Can be constant (cooked/static) or dynamic (instantiated and modified at runtime via execution graphs).

### Concrete Class and Relation Hierarchy
We propose mapping this inheritance using `owl:Class` and standard rdfs sub-class relations:
```turtle
ue4:UMaterialInterface a owl:Class ;
    rdfs:subClassOf ue4:UObject .

ue4:UMaterial a owl:Class ;
    rdfs:subClassOf ue4:UMaterialInterface .

ue4:UMaterialInstance a owl:Class ;
    rdfs:subClassOf ue4:UMaterialInterface .

ue4:UMaterialInstanceConstant a owl:Class ;
    rdfs:subClassOf ue4:UMaterialInstance .

ue4:UMaterialInstanceDynamic a owl:Class ;
    rdfs:subClassOf ue4:UMaterialInstance .
```

To link material instances to their parent, we define:
```turtle
ue4:parentMaterial a owl:ObjectProperty ;
    rdfs:domain ue4:UMaterialInstance ;
    rdfs:range ue4:UMaterialInterface .
```

### Parameterization and Overrides
We must distinguish between parameter *definitions* (made in the root `UMaterial`) and parameter *values* (which may override defaults inside a `UMaterialInstance`).

1. **Parameter Definitions**:
   - `definesParameter` connects a `UMaterial` to a `UMaterialParameter` resource.
   - `UMaterialParameter` has subclasses: `UScalarParameter`, `UVectorParameter`, `UTextureParameter`.
2. **Parameter Values/Overrides**:
   - `hasParameterValue` connects a `UMaterialInterface` to a `UMaterialParameterValue` resource.
   - `UMaterialParameterValue` specifies the overriding value (using datatype properties `scalarValue`, `vectorValue`, or object property `textureValue`) and identifies the parameter by name via `parameterName` (xsd:string).

---

## 2. Shader Representation and Mapping

Shaders in the C++ engine (e.g. `FShader`, `FMeshMaterialShader`) are represented as reflection targets:
- **`UShaderClass`**: Reprsents compiled shader permutations.
- **`UShaderParameter`**: Input uniforms or sampler bindings.
- **`EShaderFrequency`**: Execution stages (Vertex, Pixel, Compute, etc.).

We map the linkage from Materials to Shaders:
- **`compilesToShader`**: Connects a `UMaterial` to its compiled `UShaderClass` targets, which represent the shader map generated during the cooking phase.

```turtle
ue4:UShaderClass a owl:Class ;
    rdfs:subClassOf ue4:UObject .

ue4:UShaderParameter a owl:Class ;
    rdfs:subClassOf ue4:UObject .

ue4:EShaderFrequency a owl:Class .
# Instances: ue4:SF_Vertex, ue4:SF_Pixel, ue4:SF_Compute, etc.

ue4:compilesToShader a owl:ObjectProperty ;
    rdfs:domain ue4:UMaterial ;
    rdfs:range ue4:UShaderClass .
```

---

## 3. WebGL and RHI API Fallback Architecture

For the WebGL/WASM target pipeline, runtime stability depends on graphics RHI API support. The browser client will try to load the packaged WASM module using the primary RHI (typically WebGL 2.0 / OpenGL ES3) and fall back if unsupported.

We model this via:
- **`ERenderAPI`**: Enumeration of graphics APIs (`RHI_DirectX11`, `RHI_DirectX12`, `RHI_Vulkan`, `RHI_OpenGL_ES3`, `RHI_WebGL2`, `RHI_WebGL`).
- **`supportsRHI`**: Relates `URenderingSubsystem` to supported APIs.
- **`primaryRHI` / `fallbackRHI`**: Ordered preferences for the active rendering context.
- **`fallbackTo`**: A directed fallback graph representing the global RHI fallback pipeline precedence.

---

## 4. Topographical Validation Rules

To prevent broken material graphs or RHI setups that would fail Playwright E2E visual tests (Gate 4-6 failures), we design the following topology checks:

### Rule 1: Material Instance Chain Integrity (SHACL / SPARQL)
Ensures parent references do not contain cycles, and every chain terminates in a base `UMaterial`.

### Rule 2: Orphan Parameter Value Validation (SPARQL)
Ensures any parameter overridden in a material instance is actually declared in its parent chain, protecting against typos in parameters.

### Rule 3: Rendering Subsystem API Loop & Validity Check (SHACL)
Checks that `primaryRHI != fallbackRHI` and both are supported by the subsystem.

### Rule 4: WASM target WebGL Compliance (SPARQL)
Ensures that if the world target is cooked for WASM (`WasmPackagingTypestate`), the active rendering subsystem has WebGL 2.0 (ES3) or WebGL 1.0 support.

---

## 5. Summary of Recommended Files

The concrete implementations for these designs are written to:
1. `proposed_subsystems.ttl` — The expanded rendering subsystems, materials, and shaders classes.
2. `proposed_validation.shacl.ttl` — The complete SHACL shape file including new shapes for material parent loops, parameter matches, RHI loops, and WASM WebGL compatibility.
3. `proposed_ggen_additions.toml` — The SPARQL ASK rules to be integrated under the `validation.rules` section of `ggen.toml`.
