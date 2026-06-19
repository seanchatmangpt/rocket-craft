# UE4 Cooking Pipeline & Asset Compression Modeling Analysis

This document presents the semantic analysis and design recommendations for extending the Unreal Engine 4 Universal RDF Mapping ontology (`typestates.ttl`) and its SHACL validation constraints (`validation.shacl.ttl` / `ggen.toml`) to support asset cooking states, intermediate cooked asset structures, and compression profiles.

---

## 1. Architectural Strategy: The Projection Law & Platform Representations

According to the **Projection Law** (`GEMINI.md`/`AGENTS.md`), semantic authority resides in the RDF/gGen ontology while visual rendering and packaging pixels belong to Unreal Engine 4. When modeling the cooking pipeline:
1. **Source Assets** (uncompressed WAV, raw FBX, authoring PNGs) reside as primary `UObject` resources in the graph.
2. **Platform representations** must be explicitly modeled. Unreal cooks assets for specific target platforms (e.g. `HTML5`, `WindowsNoEditor`).
3. We introduce the class `ue4:AssetPlatformRepresentation` to bridge a source asset with its platform-specific cooked binary metadata, compression profiles, and validation states.

---

## 2. Proposed RDF Ontology Definitions (`typestates.ttl` extensions)

The following Turtle snippet defines the concrete classes, properties, and relationship mappings. These should be merged into `typestates.ttl` by the implementation Worker.

```turtle
# =========================================================================
# Asset Cooking Pipeline Extension Types & Classes
# =========================================================================

ue4:TargetPlatform a owl:Class ;
    rdfs:label "TargetPlatform" ;
    rdfs:comment "A target deployment platform for compilation, cooking, and packaging." .

ue4:AssetPlatformRepresentation a owl:Class ;
    rdfs:subClassOf ue4:UObject ;
    rdfs:label "AssetPlatformRepresentation" ;
    rdfs:comment "The platform-specific cooked representation of a source asset." .

# Target Platform Individuals
ue4:Platform_HTML5 a ue4:TargetPlatform ;
    rdfs:label "Platform_HTML5" ;
    rdfs:comment "HTML5 / WebGL / WASM target platform." .

ue4:Platform_WindowsNoEditor a ue4:TargetPlatform ;
    rdfs:label "Platform_WindowsNoEditor" ;
    rdfs:comment "Windows Desktop standalone client target platform." .

ue4:Platform_Android a ue4:TargetPlatform ;
    rdfs:label "Platform_Android" ;
    rdfs:comment "Android mobile target platform." .

# Source Asset Classes (under core UObject)
ue4:UStaticMesh a owl:Class ;
    rdfs:subClassOf ue4:UObject ;
    rdfs:label "UStaticMesh" ;
    rdfs:comment "Represents static polygonal geometry assets." .

ue4:USoundWave a owl:Class ;
    rdfs:subClassOf ue4:UObject ;
    rdfs:label "USoundWave" ;
    rdfs:comment "Represents raw imported audio wave assets." .

# =========================================================================
# Cooking Status Individuals (Instances of CookingTypestate)
# =========================================================================

ue4:CookState_Uncooked a ue4:CookingTypestate ;
    rdfs:label "CookState_Uncooked" ;
    rdfs:comment "The asset exists only in its source/uncooked format." .

ue4:CookState_Cooking a ue4:CookingTypestate ;
    rdfs:label "CookState_Cooking" ;
    rdfs:comment "The asset is currently undergoing cooker translation." .

ue4:CookState_Cooked a ue4:CookingTypestate ;
    rdfs:label "CookState_Cooked" ;
    rdfs:comment "The asset has successfully compiled into target platform binary format." .

ue4:CookState_Failed a ue4:CookingTypestate ;
    rdfs:label "CookState_Failed" ;
    rdfs:comment "The cooker failed to process the asset (e.g. out of memory, corrupt source file)." .

ue4:CookState_Stale a ue4:CookingTypestate ;
    rdfs:label "CookState_Stale" ;
    rdfs:comment "Source asset has been modified, and the cooked cache needs to be regenerated." .

# =========================================================================
# Compression Profiles & Enums
# =========================================================================

ue4:CompressionProfile a owl:Class ;
    rdfs:label "CompressionProfile" ;
    rdfs:comment "Configuration specifications applied during the cook translation phase." .

ue4:TextureCompressionProfile a owl:Class ;
    rdfs:subClassOf ue4:CompressionProfile ;
    rdfs:label "TextureCompressionProfile" ;
    rdfs:comment "Compression rules governing raw texture arrays." .

ue4:MeshCompressionProfile a owl:Class ;
    rdfs:subClassOf ue4:CompressionProfile ;
    rdfs:label "MeshCompressionProfile" ;
    rdfs:comment "Simplification and index buffer layout profiles for geometry." .

ue4:AudioCompressionProfile a owl:Class ;
    rdfs:subClassOf ue4:CompressionProfile ;
    rdfs:label "AudioCompressionProfile" ;
    rdfs:comment "Codec options and quality profiles for sound waves." .

# Format Enums
ue4:ETextureFormat a owl:Class ;
    rdfs:label "ETextureFormat" ;
    rdfs:comment "Texture compression format standards." .

ue4:TexFormat_DXT1 a ue4:ETextureFormat ; rdfs:label "TexFormat_DXT1" ; rdfs:comment "BC1 compression for opaque colors." .
ue4:TexFormat_DXT5 a ue4:ETextureFormat ; rdfs:label "TexFormat_DXT5" ; rdfs:comment "BC3 compression with alpha channels." .
ue4:TexFormat_ASTC_4x4 a ue4:ETextureFormat ; rdfs:label "TexFormat_ASTC_4x4" ; rdfs:comment "ASTC compression at 4x4 block size (high quality mobile/WASM)." .
ue4:TexFormat_ASTC_8x8 a ue4:ETextureFormat ; rdfs:label "TexFormat_ASTC_8x8" ; rdfs:comment "ASTC compression at 8x8 block size (lower quality, smaller footprint)." .
ue4:TexFormat_ETC2 a ue4:ETextureFormat ; rdfs:label "TexFormat_ETC2" ; rdfs:comment "Standard OpenGL ES 3.0 texture compression." .
ue4:TexFormat_BC7 a ue4:ETextureFormat ; rdfs:label "TexFormat_BC7" ; rdfs:comment "High-quality desktop-only BC7 compression." .
ue4:TexFormat_RGBA8 a ue4:ETextureFormat ; rdfs:label "TexFormat_RGBA8" ; rdfs:comment "Uncompressed raw color array." .

ue4:EMeshFormat a owl:Class ;
    rdfs:label "EMeshFormat" ;
    rdfs:comment "Cooked mesh serialization layout." .

ue4:MeshFormat_Standard a ue4:EMeshFormat ; rdfs:label "MeshFormat_Standard" ; rdfs:comment "Standard vertex, index, and LOD channels." .
ue4:MeshFormat_Nanite a ue4:EMeshFormat ; rdfs:label "MeshFormat_Nanite" ; rdfs:comment "Unreal Engine 5 virtualized geometry clusters (unsupported in standard UE4/WebGL)." .
ue4:MeshFormat_CompressedIndices a ue4:EMeshFormat ; rdfs:label "MeshFormat_CompressedIndices" ; rdfs:comment "Optimized triangle index ordering and compression." .

ue4:EAudioFormat a owl:Class ;
    rdfs:label "EAudioFormat" ;
    rdfs:comment "Target compression formats for raw sound streams." .

ue4:AudioFormat_OggVorbis a ue4:EAudioFormat ; rdfs:label "AudioFormat_OggVorbis" ; rdfs:comment "Ogg Vorbis compression (highly recommended for WASM)." .
ue4:AudioFormat_ADPCM a ue4:EAudioFormat ; rdfs:label "AudioFormat_ADPCM" ; rdfs:comment "Low-CPU compression codec." .
ue4:AudioFormat_PCM a ue4:EAudioFormat ; rdfs:label "AudioFormat_PCM" ; rdfs:comment "Uncompressed WAV audio data." .
ue4:AudioFormat_Bink a ue4:EAudioFormat ; rdfs:label "AudioFormat_Bink" ; rdfs:comment "Bink Audio compression codec." .

ue4:ECollisionComplexity a owl:Class ;
    rdfs:label "ECollisionComplexity" ;
    rdfs:comment "Collision representation generated during cook." .

ue4:Complexity_Default a ue4:ECollisionComplexity ; rdfs:label "Complexity_Default" .
ue4:Complexity_UseSimpleAsComplex a ue4:ECollisionComplexity ; rdfs:label "Complexity_UseSimpleAsComplex" .
ue4:Complexity_UseComplexAsSimple a ue4:ECollisionComplexity ; rdfs:label "Complexity_UseComplexAsSimple" .

# =========================================================================
# Properties & Relationships
# =========================================================================

# Object Properties
ue4:hasRepresentation a owl:ObjectProperty ;
    rdfs:label "hasRepresentation" ;
    rdfs:comment "Relates a raw asset to its platform-specific representation." ;
    rdfs:domain ue4:UObject ;
    rdfs:range ue4:AssetPlatformRepresentation .

ue4:hasAsset a owl:ObjectProperty ;
    rdfs:label "hasAsset" ;
    rdfs:comment "Relates a platform representation back to its raw UObject." ;
    rdfs:domain ue4:AssetPlatformRepresentation ;
    rdfs:range ue4:UObject ;
    owl:inverseOf ue4:hasRepresentation .

ue4:targetPlatform a owl:ObjectProperty ;
    rdfs:label "targetPlatform" ;
    rdfs:comment "Relates an asset representation to its target platform." ;
    rdfs:domain ue4:AssetPlatformRepresentation ;
    rdfs:range ue4:TargetPlatform .

ue4:hasCompressionProfile a owl:ObjectProperty ;
    rdfs:label "hasCompressionProfile" ;
    rdfs:comment "Relates a platform representation to its compression profile settings." ;
    rdfs:domain ue4:AssetPlatformRepresentation ;
    rdfs:range ue4:CompressionProfile .

ue4:textureFormat a owl:ObjectProperty ;
    rdfs:label "textureFormat" ;
    rdfs:comment "The format encoding used for texture compression." ;
    rdfs:domain ue4:TextureCompressionProfile ;
    rdfs:range ue4:ETextureFormat .

ue4:meshFormat a owl:ObjectProperty ;
    rdfs:label "meshFormat" ;
    rdfs:comment "The target mesh geometry format." ;
    rdfs:domain ue4:MeshCompressionProfile ;
    rdfs:range ue4:EMeshFormat .

ue4:meshCollisionComplexity a owl:ObjectProperty ;
    rdfs:label "meshCollisionComplexity" ;
    rdfs:comment "Target collision complexity setting." ;
    rdfs:domain ue4:MeshCompressionProfile ;
    rdfs:range ue4:ECollisionComplexity .

ue4:audioFormat a owl:ObjectProperty ;
    rdfs:label "audioFormat" ;
    rdfs:comment "The format encoding used for audio compression." ;
    rdfs:domain ue4:AudioCompressionProfile ;
    rdfs:range ue4:EAudioFormat .

# Datatype Properties for Cook Metadata & Layouts
ue4:cookedPath a owl:DatatypeProperty ;
    rdfs:label "cookedPath" ;
    rdfs:comment "Physical cooked file path on the host build system." ;
    rdfs:domain ue4:AssetPlatformRepresentation ;
    rdfs:range xsd:string .

ue4:cookedHash a owl:DatatypeProperty ;
    rdfs:label "cookedHash" ;
    rdfs:comment "BLAKE3 cryptographic signature of the cooked binary." ;
    rdfs:domain ue4:AssetPlatformRepresentation ;
    rdfs:range xsd:string .

ue4:lastCookedTime a owl:DatatypeProperty ;
    rdfs:label "lastCookedTime" ;
    rdfs:comment "Timestamp of the last successful cook run." ;
    rdfs:domain ue4:AssetPlatformRepresentation ;
    rdfs:range xsd:dateTime .

ue4:headerSize a owl:DatatypeProperty ;
    rdfs:label "headerSize" ;
    rdfs:comment "Size of the .uasset header file in bytes." ;
    rdfs:domain ue4:AssetPlatformRepresentation ;
    rdfs:range xsd:long .

ue4:exportsSize a owl:DatatypeProperty ;
    rdfs:label "exportsSize" ;
    rdfs:comment "Size of the .uexp export registry in bytes." ;
    rdfs:domain ue4:AssetPlatformRepresentation ;
    rdfs:range xsd:long .

ue4:bulkDataSize a owl:DatatypeProperty ;
    rdfs:label "bulkDataSize" ;
    rdfs:comment "Size of external .ubulk stream arrays in bytes." ;
    rdfs:domain ue4:AssetPlatformRepresentation ;
    rdfs:range xsd:long .

# Texture properties
ue4:textureWidth a owl:DatatypeProperty ;
    rdfs:label "textureWidth" ;
    rdfs:comment "Width of texture representation in pixels." ;
    rdfs:domain ue4:AssetPlatformRepresentation ;
    rdfs:range xsd:integer .

ue4:textureHeight a owl:DatatypeProperty ;
    rdfs:label "textureHeight" ;
    rdfs:comment "Height of texture representation in pixels." ;
    rdfs:domain ue4:AssetPlatformRepresentation ;
    rdfs:range xsd:integer .

ue4:mipmapCount a owl:DatatypeProperty ;
    rdfs:label "mipmapCount" ;
    rdfs:comment "Number of levels generated in the mipmap chain." ;
    rdfs:domain ue4:AssetPlatformRepresentation ;
    rdfs:range xsd:integer .

ue4:maxTextureSize a owl:DatatypeProperty ;
    rdfs:label "maxTextureSize" ;
    rdfs:comment "Max allowable dimension clamp configuration." ;
    rdfs:domain ue4:TextureCompressionProfile ;
    rdfs:range xsd:integer .

ue4:sRGB a owl:DatatypeProperty ;
    rdfs:label "sRGB" ;
    rdfs:comment "sRGB color profile setting." ;
    rdfs:domain ue4:TextureCompressionProfile ;
    rdfs:range xsd:boolean .

ue4:generateMipmaps a owl:DatatypeProperty ;
    rdfs:label "generateMipmaps" ;
    rdfs:comment "Mipmap compilation flag." ;
    rdfs:domain ue4:TextureCompressionProfile ;
    rdfs:range xsd:boolean .

# Mesh properties
ue4:vertexCount a owl:DatatypeProperty ;
    rdfs:label "vertexCount" ;
    rdfs:comment "Number of active vertices in the representation." ;
    rdfs:domain ue4:AssetPlatformRepresentation ;
    rdfs:range xsd:integer .

ue4:triangleCount a owl:DatatypeProperty ;
    rdfs:label "triangleCount" ;
    rdfs:comment "Number of indices-based triangles in the representation." ;
    rdfs:domain ue4:AssetPlatformRepresentation ;
    rdfs:range xsd:integer .

ue4:lodCount a owl:DatatypeProperty ;
    rdfs:label "lodCount" ;
    rdfs:comment "Number of Level-of-Detail geometry structures." ;
    rdfs:domain ue4:AssetPlatformRepresentation ;
    rdfs:range xsd:integer .

ue4:useFullPrecisionUVs a owl:DatatypeProperty ;
    rdfs:label "useFullPrecisionUVs" ;
    rdfs:comment "UV resolution encoding setting (32-bit floats vs 16-bit halfs)." ;
    rdfs:domain ue4:MeshCompressionProfile ;
    rdfs:range xsd:boolean .

ue4:enableBuildAdjacencyBuffer a owl:DatatypeProperty ;
    rdfs:label "enableBuildAdjacencyBuffer" ;
    rdfs:comment "Tessellation support buffer generation." ;
    rdfs:domain ue4:MeshCompressionProfile ;
    rdfs:range xsd:boolean .

# Audio properties
ue4:audioQuality a owl:DatatypeProperty ;
    rdfs:label "audioQuality" ;
    rdfs:comment "Encoding quality parameter (0-100)." ;
    rdfs:domain ue4:AudioCompressionProfile ;
    rdfs:range xsd:integer .

ue4:sampleRate a owl:DatatypeProperty ;
    rdfs:label "sampleRate" ;
    rdfs:comment "Frequency in Hertz." ;
    rdfs:domain ue4:AudioCompressionProfile ;
    rdfs:range xsd:integer .

ue4:channelCount a owl:DatatypeProperty ;
    rdfs:label "channelCount" ;
    rdfs:comment "Audio channels count." ;
    rdfs:domain ue4:AudioCompressionProfile ;
    rdfs:range xsd:integer .

ue4:streamedAudioChunks a owl:DatatypeProperty ;
    rdfs:label "streamedAudioChunks" ;
    rdfs:comment "Number of chunks streamed externally." ;
    rdfs:domain ue4:AssetPlatformRepresentation ;
    rdfs:range xsd:integer .
```

---

## 3. Recommended Validation Rules (`ggen.toml` & `validation.shacl.ttl` additions)

We propose the following SHACL validation shapes and SPARQL query validation constraints to enforce correct target platform compilation states and prevent build pollution for WebGL/WASM/HTML5 targets.

### Shape 1: Mandatory Cooked Representations for Assets (`ue4:AssetHTML5CookingReadyShape`)
*Rule:* For any asset active in the scene structure (textures, meshes, and audio), a compiled `Platform_HTML5` representation must exist and its status must be exactly `CookState_Cooked`.

#### SHACL Shape:
```turtle
ue4:AssetHTML5CookingReadyShape
    a sh:NodeShape ;
    sh:targetSubjectsOf rdf:type ;
    sh:sparql [
        sh:message "Active asset does not have a successfully cooked representation for target Platform_HTML5." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
            SELECT $this
            WHERE {
                # Identify if $this is a texture, static mesh, or sound wave asset
                { $this a/rdfs:subClassOf* ue4:UTexture }
                UNION
                { $this a/rdfs:subClassOf* ue4:UStaticMesh }
                UNION
                { $this a/rdfs:subClassOf* ue4:USoundWave }

                # Verify absence of a successful HTML5 cook
                FILTER NOT EXISTS {
                    ?rep ue4:hasAsset $this ;
                         ue4:targetPlatform ue4:Platform_HTML5 ;
                         ue4:hasCookingState ue4:CookState_Cooked .
                }
            }
        """ ;
    ] .
```

---

### Shape 2: WebGL Compression Standard Enforcement (`ue4:HTML5TextureFormatShape`)
*Rule:* For `Platform_HTML5` targets, texture compression profiles must strictly utilize WebGL-admitted formats (ASTC or DXT). Desktop-only formats like BC7 or opaque uncompressed arrays (`RGBA8` for sizes > 1024x1024) are forbidden to ensure low VRAM pressure and cross-browser loading compatibility.

#### SHACL Shape:
```turtle
ue4:HTML5TextureFormatShape
    a sh:NodeShape ;
    sh:targetClass ue4:AssetPlatformRepresentation ;
    sh:sparql [
        sh:message "WebGL texture format constraint violation: HTML5 cooked textures must use ASTC or DXT formats." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
            SELECT $this ?format
            WHERE {
                $this ue4:targetPlatform ue4:Platform_HTML5 ;
                      ue4:hasAsset ?asset ;
                      ue4:hasCompressionProfile ?profile .
                ?asset a/rdfs:subClassOf* ue4:UTexture .
                ?profile ue4:textureFormat ?format .

                FILTER (?format != ue4:TexFormat_ASTC_4x4 && 
                        ?format != ue4:TexFormat_ASTC_8x8 && 
                        ?format != ue4:TexFormat_DXT1 && 
                        ?format != ue4:TexFormat_DXT5)
            }
        """ ;
    ] .
```

---

### Shape 3: Audio Compression Optimization (`ue4:HTML5AudioFormatShape`)
*Rule:* Audio cooked for `Platform_HTML5` must be compressed via Ogg Vorbis codec to ensure low memory footprints in browser-native execution. Uncompressed WAV/PCM is flagged if the cooked file size exceeds a liveness budget (e.g. 500KB).

#### SHACL Shape:
```turtle
ue4:HTML5AudioFormatShape
    a sh:NodeShape ;
    sh:targetClass ue4:AssetPlatformRepresentation ;
    sh:sparql [
        sh:message "Audio format constraint violation: HTML5 cooked audio must use OggVorbis format, or PCM if size is below 500KB." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
            SELECT $this ?format ?size
            WHERE {
                $this ue4:targetPlatform ue4:Platform_HTML5 ;
                      ue4:hasAsset ?asset ;
                      ue4:hasCompressionProfile ?profile ;
                      ue4:bulkDataSize ?size .
                ?asset a/rdfs:subClassOf* ue4:USoundWave .
                ?profile ue4:audioFormat ?format .

                FILTER (?format != ue4:AudioFormat_OggVorbis && 
                        (?format = ue4:AudioFormat_PCM && ?size > 512000))
            }
        """ ;
    ] .
```

---

### Shape 4: Level-of-Detail (LOD) Budgets for HTML5 Meshes (`ue4:HTML5MeshLODConstraintShape`)
*Rule:* To prevent WebGL out-of-memory crashes, static meshes cooked for `Platform_HTML5` must have at least 2 LODs if the base LOD (LOD0) contains more than 20,000 triangles.

#### SHACL Shape:
```turtle
ue4:HTML5MeshLODConstraintShape
    a sh:NodeShape ;
    sh:targetClass ue4:AssetPlatformRepresentation ;
    sh:sparql [
        sh:message "Mesh polygon budget violation: High-poly HTML5 cooked meshes must have at least 2 LOD channels." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
            SELECT $this ?lodCount ?triCount
            WHERE {
                $this ue4:targetPlatform ue4:Platform_HTML5 ;
                      ue4:hasAsset ?asset ;
                      ue4:lodCount ?lodCount ;
                      ue4:triangleCount ?triCount .
                ?asset a/rdfs:subClassOf* ue4:UStaticMesh .

                FILTER (?lodCount < 2 && ?triCount > 20000)
            }
        """ ;
    ] .
```

---

### Shape 5: Individual Asset Size Budget (`ue4:HTML5AssetSizeBudgetShape`)
*Rule:* In WASM, the browser client package represents a major download barrier. An individual asset representation cooked for HTML5 must not exceed a size of 50MB (52,428,800 bytes) in total cooked footprint (header + exports + bulk data).

#### SHACL Shape:
```turtle
ue4:HTML5AssetSizeBudgetShape
    a sh:NodeShape ;
    sh:targetClass ue4:AssetPlatformRepresentation ;
    sh:sparql [
        sh:message "Total cooked asset footprint exceeds maximum 50MB budget for HTML5 deployment." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?totalSize
            WHERE {
                $this ue4:targetPlatform ue4:Platform_HTML5 ;
                      ue4:headerSize ?hSize ;
                      ue4:exportsSize ?eSize ;
                      ue4:bulkDataSize ?bSize .
                BIND (?hSize + ?eSize + ?bSize AS ?totalSize)
                FILTER (?totalSize > 52428800)
            }
        """ ;
    ] .
```

---

## 4. XML Schema Datatype Compliance Checklist
All literal validation targets in the graphs must use strict types to enable compiler-level validation of bounds:
- **cookedPath**: `xsd:string` (valid filesystem relative/absolute paths)
- **cookedHash**: `xsd:string` (BLAKE3 hex encoded, exactly 64 characters)
- **lastCookedTime**: `xsd:dateTime` (standard ISO UTC timestamp)
- **headerSize, exportsSize, bulkDataSize**: `xsd:long` (non-negative)
- **textureWidth, textureHeight, mipmapCount, maxTextureSize**: `xsd:integer` (non-negative)
- **vertexCount, triangleCount, lodCount, audioQuality, sampleRate, channelCount, streamedAudioChunks**: `xsd:integer` (non-negative)
- **sRGB, generateMipmaps, useFullPrecisionUVs, enableBuildAdjacencyBuffer**: `xsd:boolean`
