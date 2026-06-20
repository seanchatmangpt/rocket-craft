# Rendering Subsystem Analysis & Gap Report — 2026-06-19T06:05:15Z

## Summary
The current modeling of the Unreal Engine 4 Rendering Pipeline in `subsystems.ttl` contains major gaps in material parameter type safety, completely lacks SHACL validation shapes for shaders/shader parameters, and does not enforce consistency between WASM packaging target profiles and the active rendering subsystem's supported RHI APIs. We propose six new validation shapes to reconcile these issues and guarantee pipeline compilation integrity.

---

## TAI Status Reporting

**Status**: PARTIAL
**Object under test**: subsystems.ttl and shacl/validation.shacl.ttl rendering pipeline modeling
**Observed evidence**: `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl` (lines 71-289), `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` (lines 286-309, 383-432, 434-576), `ggen sync --validate-only true` (exit code: 0)
**Failure**: No compile or structural crash, but significant logical validation gaps exist. Material parameters lack type enforcement, shaders lack SHACL shapes, and packaging profiles are disconnected from subsystem support.
**Repair**: Defined six new SHACL validation rules and SPARQL constraint queries for inclusion in `validation.shacl.ttl`.
**Receipt required**: Updated `validation.shacl.ttl` containing the proposed shapes and a successful run of `ggen sync --validate-only true`.
**Residuals**: Runtime integration and WebGL compile validation remains unproven.

---

## Detailed Gap Analysis

### 1. Materials & Material Parameters
The following table highlights the findings and gaps identified in the material modeling of `subsystems.ttl` and `validation.shacl.ttl`:

| # | Gap / Inconsistency | File & Context | Description / Impact |
|---|---|---|---|
| **1.1** | **No Parameter Override Type Safety** | `validation.shacl.ttl` (Line 478: `MaterialInstanceParameterOverrideShape`) | Checks parameter name match but does not enforce that the overriding parameter value matches the type defined in the base material (e.g. `UScalarParameter` overridden by `vectorValue`). |
| **1.2** | **Domain/Range Mismatch of `parameterName`** | `subsystems.ttl` (Line 142) vs. `validation.shacl.ttl` (Line 501) | `parameterName` domain is restricted to `UMaterialParameterValue`, but SHACL queries it on `UMaterialParameter` (`?paramDef ue4:parameterName ?paramName`), leading to semantic mismatch. |
| **1.3** | **No Mutual Exclusion on Value Types** | `subsystems.ttl` (Line 131) | `UMaterialParameterValue` is a single class with optional `scalarValue`, `vectorValue`, and `textureValue`. No rules prevent an instance from defining multiple values simultaneously. |
| **1.4** | **Unvalidated Material Assignment** | `subsystems.ttl` (Line 171: `ue4:assignedMaterial`) | Relates scene components to material interfaces, but no validation shape exists to verify that components point to valid, reachable materials. |

### 2. Shaders & Shader Parameters
Shaders are completely unvalidated. The following table describes the shader gaps:

| # | Gap / Inconsistency | File & Context | Description / Impact |
|---|---|---|---|
| **2.1** | **Zero SHACL Validation for Shaders** | `validation.shacl.ttl` | There are absolutely no validation shapes for `UShaderClass`, `UShaderParameter`, or `EShaderFrequency` in the entire project. |
| **2.2** | **Missing Material-to-Shader Parameter Mapping** | `subsystems.ttl` (Lines 181-231) | High-level material parameters (e.g., `UScalarParameter`) are compiled into low-level shader parameters (uniforms), but there is no property mapping the relationship, leaving compilation bindings unvalidated. |
| **2.3** | **No Malformed Shader Detection** | `subsystems.ttl` | No rules verify that shader parameters have names, types, or valid execution stages. |

### 3. WebGL / RHI Fallbacks
The following table documents findings and gaps regarding RHI APIs and fallbacks:

| # | Gap / Inconsistency | File & Context | Description / Impact |
|---|---|---|---|
| **3.1** | **Fallback Precedence Unchecked** | `subsystems.ttl` (Line 277) | Subsystems define `primaryRHI` and `fallbackRHI`, but SHACL does not verify if the fallback is valid according to the `fallbackTo` precedence chain. |
| **3.2** | **Fallback Loops Unchecked** | `subsystems.ttl` (Line 284: `fallbackTo`) | There is no loop detection in `fallbackTo` definitions, creating risk of infinite recursion loops. |
| **3.3** | **Packaging and Subsystem Disconnection** | `typestates.ttl` vs. `subsystems.ttl` | `PackagingTarget` configures an RHI Profile API, but SHACL does not check if it is supported by the world's rendering subsystem (`supportsRHI`), leading to compiler/profile mismatch. |

---

## Proposed Ontology Refinements & Validation Rules

To resolve the identified gaps, we recommend adding the following SHACL validation shapes to `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`:

### Refinement 1: Parameter Override Type Safety
Enforces that material instances override parameters with values of the correct datatype.

```turtle
# Proposed Rule I.1: Material Instance Parameter Value Type Safety
ue4:MaterialInstanceParameterValueTypeShape
    a sh:NodeShape ;
    sh:targetClass ue4:UMaterialInstance , ue4:UMaterialInstanceConstant , ue4:UMaterialInstanceDynamic ;
    sh:sparql [
        sh:message "Material parameter type mismatch: Overridden parameter value type does not match the parameter definition type in the base material." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
            SELECT $this ?paramVal ?paramName ?paramType
            WHERE {
                $this ue4:hasParameterValue ?paramVal .
                ?paramVal ue4:parameterName ?paramName .
                
                # Trace parent chain to root UMaterial
                $this ue4:parentMaterial* ?ancestor .
                ?ancestor a/rdfs:subClassOf* ue4:UMaterial .
                ?ancestor ue4:definesParameter ?paramDef .
                
                # Match parameter name
                { ?paramDef rdfs:label ?paramName }
                UNION
                { ?paramDef ue4:parameterName ?paramName }
                
                ?paramDef a ?paramType .
                
                # Filter for type mismatches
                FILTER (
                    ( ?paramType = ue4:UScalarParameter && 
                        ( NOT EXISTS { ?paramVal ue4:scalarValue ?v } || EXISTS { ?paramVal ue4:vectorValue ?v2 } || EXISTS { ?paramVal ue4:textureValue ?v3 } ) ) ||
                    ( ?paramType = ue4:UVectorParameter && 
                        ( NOT EXISTS { ?paramVal ue4:vectorValue ?v } || EXISTS { ?paramVal ue4:scalarValue ?v2 } || EXISTS { ?paramVal ue4:textureValue ?v3 } ) ) ||
                    ( ?paramType = ue4:UTextureParameter && 
                        ( NOT EXISTS { ?paramVal ue4:textureValue ?v } || EXISTS { ?paramVal ue4:scalarValue ?v2 } || EXISTS { ?paramVal ue4:vectorValue ?v3 } ) )
                )
            } ORDER BY $this ?paramVal
        """ ;
    ] .
```

### Refinement 2: Parameter Value Exclusivity
Ensures that a single `UMaterialParameterValue` specifies exactly one type of value.

```turtle
# Proposed Rule I.2: Material Parameter Value Exclusivity Shape
ue4:UMaterialParameterValueExclusionShape
    a sh:NodeShape ;
    sh:targetClass ue4:UMaterialParameterValue ;
    sh:sparql [
        sh:message "Invalid parameter value definition: A UMaterialParameterValue must contain exactly one type of value (scalarValue, vectorValue, or textureValue)." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this
            WHERE {
                BIND(IF(EXISTS { $this ue4:scalarValue ?s }, 1, 0) +
                     IF(EXISTS { $this ue4:vectorValue ?v }, 1, 0) +
                     IF(EXISTS { $this ue4:textureValue ?t }, 1, 0) AS ?valCount)
                FILTER (?valCount != 1)
            }
        """ ;
    ] .
```

### Refinement 3: Shader Parameter Integrity
Ensures that shader classes are well-formed and parameter names and types are specified.

```turtle
# Proposed Rule J.1: UShaderClass Well-Formedness
ue4:UShaderClassShape
    a sh:NodeShape ;
    sh:targetClass ue4:UShaderClass ;
    sh:property [
        sh:path ue4:shaderFrequency ;
        sh:minCount 1 ;
        sh:class ue4:EShaderFrequency ;
        sh:message "A UShaderClass must have at least one shaderFrequency stage." ;
    ] .

# Proposed Rule J.2: UShaderParameter Well-Formedness
ue4:UShaderParameterShape
    a sh:NodeShape ;
    sh:targetClass ue4:UShaderParameter ;
    sh:property [
        sh:path ue4:shaderParameterName ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:datatype xsd:string ;
        sh:message "A UShaderParameter must have exactly one name string." ;
    ] ;
    sh:property [
        sh:path ue4:shaderParameterType ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:datatype xsd:string ;
        sh:message "A UShaderParameter must have exactly one type declaration string." ;
    ] .
```

### Refinement 4: Base Material Shaders Verification
Enforces that a base material always compiles to vertex and pixel shaders.

```turtle
# Proposed Rule J.3: Material Shaders Compilation Minimums
ue4:UMaterialCompiledShadersShape
    a sh:NodeShape ;
    sh:targetClass ue4:UMaterial ;
    sh:sparql [
        sh:message "Material shader compilation incomplete: A UMaterial must compile to at least one Vertex Shader (SF_Vertex) and one Pixel Shader (SF_Pixel)." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this
            WHERE {
                {
                    FILTER NOT EXISTS {
                        $this ue4:compilesToShader ?shader .
                        ?shader ue4:shaderFrequency ue4:SF_Vertex .
                    }
                }
                UNION
                {
                    FILTER NOT EXISTS {
                        $this ue4:compilesToShader ?shader .
                        ?shader ue4:shaderFrequency ue4:SF_Pixel .
                    }
                }
            }
        """ ;
    ] .
```

### Refinement 5: RHI Fallback Path Validation
Ensures acyclicity and valid transition chains in rendering subsystems.

```turtle
# Proposed Rule K.1: ERenderAPI Fallback Acyclicity Shape
ue4:ERenderAPIFallbackAcyclicityShape
    a sh:NodeShape ;
    sh:targetClass ue4:ERenderAPI ;
    sh:sparql [
        sh:message "Render API fallback loop detected: A rendering API cannot fallback to itself through transitively chained transitions." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?next
            WHERE {
                $this ue4:fallbackTo+ $this .
            }
        """ ;
    ] .

# Proposed Rule K.2: Rendering Subsystem Fallback Reachability Shape
ue4:URenderingSubsystemFallbackReachabilityShape
    a sh:NodeShape ;
    sh:targetClass ue4:URenderingSubsystem ;
    sh:sparql [
        sh:message "Rendering Subsystem fallback path error: The configured fallbackRHI is not reachable from the primaryRHI via the fallbackTo chain." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?primary ?fallback
            WHERE {
                $this ue4:primaryRHI ?primary .
                $this ue4:fallbackRHI ?fallback .
                FILTER NOT EXISTS { ?primary ue4:fallbackTo+ ?fallback }
            }
        """ ;
    ] .
```

### Refinement 6: Packaging Target RHI Alignment
Validates that the compilation/packaging RHI configuration aligns with the rendering subsystem.

```turtle
# Proposed Rule L.1: Packaging Target RHI Subsystem Compatibility
ue4:PackagingTargetRhiSupportShape
    a sh:NodeShape ;
    sh:targetClass ue4:PackagingTarget ;
    sh:sparql [
        sh:message "RHI profile mismatch: The target RHI Profile API configured in the PackagingTarget must be supported by the target world's RenderingSubsystem." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?world ?subsystem ?renderAPI
            WHERE {
                $this ue4:targetWorld ?world .
                $this ue4:targetRHIProfile ?profile .
                ?profile ue4:hasRenderAPI ?renderAPI .
                ?world ue4:hasSubsystem ?subsystem .
                ?subsystem a/rdfs:subClassOf* ue4:URenderingSubsystem .
                FILTER NOT EXISTS { ?subsystem ue4:supportsRHI ?renderAPI }
            }
        """ ;
    ] .
```
