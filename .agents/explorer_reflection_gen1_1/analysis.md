# UE4 C++ Reflection Ontology Analysis

## Executive Summary
This report analyzes `/Users/sac/.ggen/packs/ue4_ontology/reflection.ttl` to evaluate its coverage, semantic correctness, and alignment with Epic Games' Unreal Engine 4 (UE4) C++ reflection model. While the ontology successfully captures the high-level class hierarchy and function signature structure required for basic graph validation, it diverges in inheritance structure for certain properties, relies on string-based flags instead of structural semantic models, and contains major functional gaps—most notably the complete absence of inner/key/value types for container properties (`TArray`, `TMap`, `TSet`), delegate signature associations, and metadata specifier support (`UMetaData`).

---

## 1. Core Reflection Classes & Hierarchy Coverage

The ontology declares the following core reflection classes:
- `ue4:UField` (subclass of `ue4:UObject`)
- `ue4:UStruct` (subclass of `ue4:UField`)
- `ue4:UClass` (subclass of `ue4:UStruct`)
- `ue4:UScriptStruct` (subclass of `ue4:UStruct`)
- `ue4:UFunction` (subclass of `ue4:UStruct`)
- `ue4:UEnum` (subclass of `ue4:UField`)
- `ue4:UProperty` (subclass of `ue4:UField`)

### Alignment with Epic's C++ Model:
- **Structural Integrity:** The core inheritance relationships align perfectly with Epic's `UObject` reflection hierarchy. In UE4 C++, `UStruct` is the parent of classes that contain fields (functions, structs, and classes). Therefore, having `UClass`, `UScriptStruct`, and `UFunction` inherit from `ue4:UStruct` is semantically correct.
- **Root Declaration Gap:** The ontology refers to `ue4:UObject` as the superclass of `ue4:UField` (lines 17-18):
  ```turtle
  ue4:UField a owl:Class ;
      rdfs:subClassOf ue4:UObject ;
  ```
  However, `ue4:UObject` is never declared anywhere else in the file. While RDF allows references to external/implicit resources, declaring `ue4:UObject` explicitly is standard practice for ontological completeness.

---

## 2. Property Hierarchy Coverage

The ontology defines a comprehensive hierarchy of subclasses under `ue4:UProperty`. However, several mismatches and missing elements exist when compared to Epic's C++ reflection model:

### Hierarchy Mismatch:
- **USoftClassProperty:** The ontology declares:
  ```turtle
  ue4:USoftClassProperty a owl:Class ;
      rdfs:subClassOf ue4:UObjectProperty ;
  ```
  In Epic's C++ reflection model, `USoftClassProperty` inherits from `USoftObjectProperty` (which in turn inherits from `UObjectPropertyBase` / `UObjectProperty`). By making it a direct subclass of `UObjectProperty` instead of `USoftObjectProperty`, the ontology fails to capture that soft class properties share all behaviors of soft object properties (such as lazy-loading paths).

### Missing Subclasses:
Epic's reflection system defines several numeric property classes under `UNumericProperty` which are omitted in the ontology:
- **Missing signed integer properties:** `UInt8Property` (holds `int8`), `UShortProperty` (holds `int16`).
- **Missing unsigned integer properties:** `UUInt16Property` (holds `uint16`), `UUInt32Property` (holds `uint32`), `UUInt64Property` (holds `uint64`).
- *Note:* The ontology only defines `UByteProperty` (uint8), `UIntProperty` (int32), `UInt64Property` (int64), `UFloatProperty` (float), and `UDoubleProperty` (double). While these cover the default Blueprint types, they do not cover the full range of C++ numeric types supported by Epic's reflection engine.
- **Missing Delegate Subclasses:** It misses `UMulticastInlineDelegateProperty` and `UMulticastSparseDelegateProperty`, which are separate classes in UE4 C++ and behave differently under reflection.

---

## 3. Structural Relationships & Schema Properties

The ontology specifies properties to relate structures to their children and definitions:
- `ue4:hasField` / `ue4:hasProperty` / `ue4:hasFunction`
- `ue4:superStruct`
- `ue4:propertyType`
- `ue4:returnProperty`
- `ue4:classFlags` / `ue4:functionFlags` / `ue4:propertyFlags`

### Alignment with Epic's C++ Model:
- **Scope Navigation:** In C++, `UStruct` maintains a single linked list of `UField*` objects (via the `Children` pointer). The ontology models this using explicit `hasProperty` and `hasFunction` properties. This is a beneficial semantic abstraction, as it allows direct SPARQL queries without traversing a generic linked list.
- **Inheritance Mapping:** `ue4:superStruct` corresponds directly to `UStruct::SuperStruct` in C++, correctly representing inheritance.
- **Flag representation (Loss of Semantics):** Flags are represented as `xsd:string` properties (e.g. `ue4:propertyFlags`). In Epic's reflection model, these are bitmasks (`EPropertyFlags`, `EFunctionFlags`, `EClassFlags`). Storing them as raw strings in the ontology forces code generators and SHACL validators to use string parsing / regex to detect flags (e.g. searching for `"CPF_BlueprintVisible"`), rather than using structured RDF individuals or boolean properties, which degrades querying efficiency and safety.

---

## 4. Function Parameters, Directions, and Indices

To represent function signatures, the ontology introduces:
- `ue4:UFunctionParameter` (subclass of `ue4:UProperty`)
- `ue4:PinDirection` (with individuals `ue4:Input`, `ue4:Output`, `ue4:InOut`, `ue4:Return`)
- `ue4:hasParameter` / `ue4:parameterOf`
- `ue4:parameterDirection`
- `ue4:parameterIndex`

### Alignment with Epic's C++ Model:
- **Conceptual Abstraction:** In Epic's C++ reflection model, there is no distinct `UFunctionParameter` class. Instead, parameters are simply standard `UProperty` instances stored in the `UFunction`'s child field list, flagged with `CPF_Parm`. They are distinguished as input, output, or return values using bitmask flags:
  - Input: `CPF_Parm` (without output/return flags)
  - Output / Ref: `CPF_Parm | CPF_OutParm`
  - Return Value: `CPF_Parm | CPF_ReturnParm`
- **Ontological Utility:** The ontology's abstraction of creating a dedicated `ue4:UFunctionParameter` class and a `PinDirection` enumeration is highly effective. It simplifies graph queries and SHACL verification rules (as seen in `validation.shacl.ttl` where pin directions are validated).
- **Ordering Representation:** In C++, parameter ordering is determined by their position in the linked list. Because RDF is naturally unordered, the introduction of `ue4:parameterIndex` (an `xsd:integer`) is an essential and correct mechanism to maintain the exact C++ function signature sequence.

---

## 5. Identified Gaps & Areas of Improvement

### Gap A: Container Inner/Key/Value Types (Critical)
In Epic's C++ reflection model, collection types require pointers to the properties they contain:
- `UArrayProperty` contains a pointer to `Inner` (`UProperty*` representing the element type).
- `UMapProperty` contains pointers to `KeyProp` and `ValueProp` (`UProperty*`).
- `USetProperty` contains a pointer to `ElementProp` (`UProperty*`).

The ontology has **no relationships** to map these inner types. Consequently, there is no way to represent a nested type structure like `TArray<FVector>` or `TMap<FString, int32>` in the ontology. A query would only see that a property is a `UArrayProperty`, but not what type of array it is.

### Gap B: Delegate Signatures (High)
Delegate and Multicast Delegate properties (`UDelegateProperty`, `UMulticastDelegateProperty`) refer to a signature function that defines their parameters. In C++, this is accessed via `SignatureFunction` (`UFunction*`). The ontology lacks a property to relate a delegate property to its signature, preventing type-safety checks on event bindings.

### Gap C: Metadata Specifiers (`UMetaData`) (High)
Unreal Engine uses the `meta=(...)` syntax in macros to store arbitrary key-value metadata on classes, structures, functions, and properties (e.g. `ToolTip`, `Category`, `DisplayName`). In C++, these are managed by `UMetaData`. The ontology contains no elements representing metadata, meaning essential generation cues like categories or tooltips cannot be extracted.

### Gap D: Replication and Networking (Medium)
For game servers and network synchronization, property replication metadata is critical. Epic's reflection tracks if properties are replicated (`CPF_Net`), their replication condition (`ELifetimeCondition`), and their replication notification callback function (RepNotify). The ontology lacks properties to model this, limiting its capability to represent authoritative network state.

---

## 6. Actionable Recommendations & Proposed Schema Patches

To align the ontology fully with Epic's C++ reflection model and support advanced code generation, the following modifications are recommended:

### 1. Fix the USoftClassProperty Hierarchy
Change the `rdfs:subClassOf` from `ue4:UObjectProperty` to `ue4:USoftObjectProperty`:
```turtle
# Fix USoftClassProperty hierarchy to inherit from USoftObjectProperty
ue4:USoftClassProperty a owl:Class ;
    rdfs:subClassOf ue4:USoftObjectProperty ;
    rdfs:label "USoftClassProperty" ;
    rdfs:comment "Property representing a soft reference to a class type metadata." .
```

### 2. Declare `UObject` and Missing Numeric Properties
Explicitly define `UObject` and add the missing C++ numeric types:
```turtle
# Declare UObject
ue4:UObject a owl:Class ;
    rdfs:label "UObject" ;
    rdfs:comment "The root class for all reflected Unreal Engine objects." .

# Add missing integer properties
ue4:UInt8Property a owl:Class ;
    rdfs:subClassOf ue4:UNumericProperty ;
    rdfs:label "UInt8Property" ;
    rdfs:comment "8-bit signed integer variable property metadata." .

ue4:UShortProperty a owl:Class ;
    rdfs:subClassOf ue4:UNumericProperty ;
    rdfs:label "UShortProperty" ;
    rdfs:comment "16-bit signed integer variable property metadata." .

ue4:UUInt16Property a owl:Class ;
    rdfs:subClassOf ue4:UNumericProperty ;
    rdfs:label "UUInt16Property" ;
    rdfs:comment "16-bit unsigned integer variable property metadata." .

ue4:UUInt32Property a owl:Class ;
    rdfs:subClassOf ue4:UNumericProperty ;
    rdfs:label "UUInt32Property" ;
    rdfs:comment "32-bit unsigned integer variable property metadata." .

ue4:UUInt64Property a owl:Class ;
    rdfs:subClassOf ue4:UNumericProperty ;
    rdfs:label "UUInt64Property" ;
    rdfs:comment "64-bit unsigned integer variable property metadata." .
```

### 3. Add Container Inner and Delegate Signature Properties
Introduce object properties to model the inner schemas of arrays, maps, sets, and delegate signatures:
```turtle
# Relationships for Collections
ue4:innerProperty a owl:ObjectProperty ;
    rdfs:label "innerProperty" ;
    rdfs:comment "Relates a UArrayProperty to the property defining its elements." ;
    rdfs:domain ue4:UArrayProperty ;
    rdfs:range ue4:UProperty .

ue4:keyProperty a owl:ObjectProperty ;
    rdfs:label "keyProperty" ;
    rdfs:comment "Relates a UMapProperty to the property representing its keys." ;
    rdfs:domain ue4:UMapProperty ;
    rdfs:range ue4:UProperty .

ue4:valueProperty a owl:ObjectProperty ;
    rdfs:label "valueProperty" ;
    rdfs:comment "Relates a UMapProperty to the property representing its values." ;
    rdfs:domain ue4:UMapProperty ;
    rdfs:range ue4:UProperty .

ue4:elementProperty a owl:ObjectProperty ;
    rdfs:label "elementProperty" ;
    rdfs:comment "Relates a USetProperty to the property representing its unique elements." ;
    rdfs:domain ue4:USetProperty ;
    rdfs:range ue4:UProperty .

# Relationship for Delegate Signatures
ue4:delegateSignature a owl:ObjectProperty ;
    rdfs:label "delegateSignature" ;
    rdfs:comment "Relates a delegate or multicast delegate property to its signature function." ;
    rdfs:domain [ owl:unionOf (ue4:UDelegateProperty ue4:UMulticastDelegateProperty) ] ;
    rdfs:range ue4:UFunction .
```

### 4. Model Metadata Key-Value pairs
Instead of ignoring metadata, define a clean key-value mapping structure:
```turtle
# Metadata Support
ue4:UMetaData a owl:Class ;
    rdfs:label "UMetaData" ;
    rdfs:comment "Represents metadata specifier key-value maps associated with reflected entities." .

ue4:hasMetaData a owl:ObjectProperty ;
    rdfs:label "hasMetaData" ;
    rdfs:comment "Relates a UField to its metadata." ;
    rdfs:domain ue4:UField ;
    rdfs:range ue4:UMetaData .

ue4:metaKey a owl:DatatypeProperty ;
    rdfs:label "metaKey" ;
    rdfs:comment "The key name of a metadata specifier (e.g., 'Category', 'ToolTip')." ;
    rdfs:domain ue4:UMetaData ;
    rdfs:range xsd:string .

ue4:metaValue a owl:DatatypeProperty ;
    rdfs:label "metaValue" ;
    rdfs:comment "The value of a metadata specifier." ;
    rdfs:domain ue4:UMetaData ;
    rdfs:range xsd:string .
```

### 5. Structured Flag Enums/Taxonomies
Instead of representing flags as plain strings, define them as a taxonomy of individual properties or resources. For example, for properties:
```turtle
ue4:isBlueprintWritable a owl:DatatypeProperty ;
    rdfs:label "isBlueprintWritable" ;
    rdfs:comment "Indicates if the property can be written to from Blueprint graphs." ;
    rdfs:domain ue4:UProperty ;
    rdfs:range xsd:boolean .

ue4:isBlueprintReadOnly a owl:DatatypeProperty ;
    rdfs:label "isBlueprintReadOnly" ;
    rdfs:comment "Indicates if the property is read-only in Blueprint graphs." ;
    rdfs:domain ue4:UProperty ;
    rdfs:range xsd:boolean .
```
This structured approach enables clean, compile-time SHACL checks on whether generated elements respect visibility and capability constraints.
