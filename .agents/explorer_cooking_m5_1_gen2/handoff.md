# Handoff Report: UE4 Cooking Pipeline & Asset Compression Modeling (Milestone 5.1)

## 1. Observation

Direct file contents and structures observed during exploration:

- **File Path**: `/Users/sac/.ggen/packs/ue4_ontology/typestates.ttl` (Lines 16-19):
  ```turtle
  ue4:CookingTypestate a owl:Class ;
      rdfs:subClassOf ue4:Typestate ;
      rdfs:label "CookingTypestate" ;
      rdfs:comment "State representing the asset cooking process." .
  ```
- **File Path**: `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` (Lines 204-210):
  ```turtle
  # Rule F: Character must have exactly 1 hasCookingState of type CookingTypestate
  ue4:CharacterCookingStateShape
      a sh:NodeShape ;
      sh:targetSubjectsOf rdf:type ;
      sh:sparql [
          sh:message "A character must have exactly one cooking state of type CookingTypestate." ;
  ```
- **File Path**: `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl` (Lines 166-169):
  ```turtle
  ue4:UTexture a owl:Class ;
      rdfs:subClassOf ue4:UObject ;
      rdfs:label "UTexture" ;
      rdfs:comment "Base class representing texture resources used in materials and rendering." .
  ```
  *(Observation: While `UTexture` is present in `subsystems.ttl`, no classes exist in the current ontology files for `UStaticMesh` or `USoundWave` (sound wave/audio assets).)*

- **File Path**: `/Users/sac/rocket-craft/validate_ontology.sh` (Lines 29-36):
  ```bash
  # Execute ggen sync with validation
  echo "Running: $GGEN_BIN sync --validate-only true"
  ...
  "$GGEN_BIN" sync --validate-only true
  ```
  *(Observation: Validation in this workspace is powered by `ggen sync --validate-only true` which evaluates both SHACL shapes and SPARQL validation rules defined in `validation.shacl.ttl` and `ggen.toml`.)*

---

## 2. Logic Chain

1. **Modeling target-platform specificity:** Because Unreal Engine cooks assets target-by-target (e.g. differently for HTML5/WASM vs Windows Desktop), simple direct relationships like `asset ue4:hasCookingState Cooked` fail to capture platform-specific compilation state or target compression format profiles. We must introduce an intermediate class `ue4:AssetPlatformRepresentation` referencing `ue4:TargetPlatform`.
2. **Missing core asset types:** In order to model cooking states for meshes and audio, subclasses of `ue4:UObject` representing these structures (namely `ue4:UStaticMesh` and `ue4:USoundWave`) must be declared in the ontology (`typestates.ttl`).
3. **WASM-specific constraints:** WebGL and WASM runtimes have rigid hardware compression support (ASTC/DXT for textures, Ogg Vorbis for audio) and strict memory envelopes (typically 2GB heap limits, requiring asset budgets below 50MB and LOD reduction). The SHACL shapes must enforce these constraints statically to reject unviable builds before compilation.
4. **Validation deployment:** The validation rules proposed in `analysis.md` should be implemented in both `validation.shacl.ttl` (as SHACL NodeShapes with `sh:sparql` constraints) and `ggen.toml` (as SPARQL rules) to be verified by `validate_ontology.sh` during the Worker stage.

---

## 3. Caveats

- **WebGL Fallbacks**: We assumed desktop-class WebGL (which natively supports DXT/BC1-5 formats via extension) and mobile WebGL (supporting ASTC) are the target runtimes. Platforms utilizing older PVRTC/ETC1 compression profiles were omitted as they are deprecated in UE4.27 HTML5.
- **Nanite Support**: We noted that Nanite (`MeshFormat_Nanite`) is unsupported in standard UE4 and WebGL pipelines. The validation shapes specifically restrict cooked mesh representations from targeting Nanite for HTML5 platforms.
- **Physical Integration**: As an Explorer (read-only), we did not modify `typestates.ttl` or `validation.shacl.ttl`. The actual insertion and execution tests will be done by the Worker.

---

## 4. Conclusion

We recommend extending the ontology in `/Users/sac/.ggen/packs/ue4_ontology/typestates.ttl` with:
- Target platforms (`ue4:Platform_HTML5`, `ue4:Platform_WindowsNoEditor`).
- Explicit cooking status individuals (`CookState_Uncooked`, `CookState_Cooking`, `CookState_Cooked`, `CookState_Failed`, `CookState_Stale`).
- Intermediate platform representation model (`ue4:AssetPlatformRepresentation`) mapping source assets to target platform configurations.
- Compression profiles (`ue4:TextureCompressionProfile`, `ue4:MeshCompressionProfile`, `ue4:AudioCompressionProfile`) and format enums (`ETextureFormat`, `EMeshFormat`, `EAudioFormat`).
- Custom SHACL shapes enforcing WebGL texture standards (ASTC/DXT), audio optimizations (Ogg Vorbis), polygon LOD counts, and size budgets (<50MB per asset).

---

## 5. Verification Method

To verify these recommendations:
1. **Implementation Step**: The Worker should copy the RDF schema extensions from Section 2 of `analysis.md` and append them to `/Users/sac/.ggen/packs/ue4_ontology/typestates.ttl`.
2. **Validation Rules Step**: The Worker should copy the custom SHACL shapes from Section 3 of `analysis.md` and append them to `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`.
3. **Execution Command**: Run the validation script:
   ```bash
   /Users/sac/rocket-craft/validate_ontology.sh
   ```
4. **Pass/Fail Criteria**: The validator must return `SUCCESS: Ontology validation passed.` with exit code `0`. If any syntax error or logic violation is introduced, the validator will return a non-zero exit code.
