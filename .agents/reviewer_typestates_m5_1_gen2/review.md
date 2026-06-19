# Quality & Adversarial Review Report

## Quality Review Summary

**Verdict**: REQUEST_CHANGES

The implemented typestates schema has successfully passed the syntax, manifest, and template validations in the ontology package, and its validation tests correctly flag negative assertions. However, several critical logical bugs, stray debug constraints, and conceptual typestate mismatches have been identified that will block real-world instantiation and compromise correctness.

---

## Quality Findings

### [Critical] Finding 1: Stray SHACL Shape Blocks World Instantiation
- **What**: A stray test shape `ue4:TestWorldShape` unconditionally flags any instance of `ue4:UWorld` as a validation violation.
- **Where**: `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` (lines 1384–1396)
- **Why**: The SPARQL query selects any `$this` where `$this a ue4:UWorld`. Since `UWorld` must be instantiated to represent a game world mapping (as seen in `gundam_character.ttl`), this stray validation rule will cause any valid instance graph representing a game world to fail validation. It only passed the current repository validation check because the core ontology doesn't contain any world instances (only class definitions).
- **Suggestion**: Remove `ue4:TestWorldShape` entirely from `validation.shacl.ttl`.

### [Major] Finding 2: Logic Bug in HTML5 Audio Format Validation
- **What**: The logic checking that HTML5 audio formats use `OggVorbis` (or `PCM` if under 500KB) is logically broken and permits unsupported formats (e.g., `Bink` or `ADPCM`) to bypass validation.
- **Where**: 
  - `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` Shape 3 (lines 974–996)
  - `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml` `RuleHTML5AudioFormat` (lines 671–691)
- **Why**: The filter condition is:
  `FILTER (?format != ue4:AudioFormat_OggVorbis && (?format = ue4:AudioFormat_PCM && ?size > 512000))`
  If a developer specifies `AudioFormat_Bink`, the expression evaluates to `(Bink != OggVorbis) AND (Bink == PCM AND size > 512KB)`. Since `Bink == PCM` is false, the entire AND expression evaluates to false, which means NO violation is flagged. Consequently, formats like Bink and ADPCM pass validation unconditionally, which violates target packaging constraints.
- **Suggestion**: Change the filter to:
  `FILTER (?format != ue4:AudioFormat_OggVorbis && (?format != ue4:AudioFormat_PCM || ?size > 512000))`

### [Major] Finding 3: Conceptual Typestate Mismatch on Characters
- **What**: `RuleF` (Character Cooking State Constraint) requires that every character instance (`ACharacter` subclasses) has a cooking state (`CookingTypestate`).
- **Where**: 
  - `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` (Rule F, lines 205–233)
  - `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml` (Rule F, lines 214–241)
- **Why**: Characters (`ACharacter` / `AActor`) are runtime gameplay objects, not static compiled asset files. Dynamic character instances do not have a cooked binary file, cooked path, cooked hash, or cooking state in a real Unreal project. It is the underlying assets they reference (skeletal meshes, textures, sound waves) that are cooked. Forcing character instances to declare a `CookingTypestate` is a conceptual error that forces test scenarios to include fake `hasCookingState` triples on characters while omitting them on the actual assets.
- **Suggestion**: Shift the cooking typestate constraint from characters to asset classes (`UTexture`, `USkeletalMesh`, `UStaticMesh`, `USoundWave`).

### [Minor] Finding 4: Inconsistent Characterization of Shipping Configurations
- **What**: Inconsistent checks for shipping build configurations between `LinkingConfiguration` and `BuildConfiguration`.
- **Where**: `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` and `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml`
- **Why**: `LinkingConfiguration` identifies shipping builds using the type-safe datatype property `ue4:buildMode "Shipping"`, whereas `BuildConfiguration` relies on the fragile match of the string label `rdfs:label "Shipping"`. If a developer creates a custom build configuration representing a shipping build (e.g. named "Release" or "Shipping_Client"), the `BuildConfigurationConsistencyShape` will not catch optimizations/symbols mismatches.
- **Suggestion**: Unify both classes to use the same property-based classification (e.g., `ue4:buildMode`).

### [Minor] Finding 5: Test Script Mismatch in `verify_extra_rules.sh`
- **What**: Test 5 in `verify_extra_rules.sh` fails due to a mismatched expected error string.
- **Where**: `/Users/sac/rocket-craft/ggen-validation-tests/verify_extra_rules.sh` (line 136)
- **Why**: The test expects the string `"Static baking target worlds must not use dynamic VaRest calls"`. However, the message in `ggen.toml` is `"Projection Law violation: Statically baked target worlds must not use dynamic VaRest calls."`. The mismatch between "Static baking" and "Statically baked" causes the test script to report a failure even though GGen correctly aborted the build.
- **Suggestion**: Update `verify_extra_rules.sh` to match the exact description string.

---

## Verified Claims

- **Ontology Syntax and Compilation** → verified via `./validate_ontology.sh` → **PASS**
- **Negative Constraint Validation** → verified via `ggen-validation-tests/verify_all_rules.sh` → **PASS** (all 22 negative tests correctly abort build with expected error codes)
- **WASM Memory Constraints** (Initial/Max memory page alignment, stack bounds, max limit) → verified via `verify_all_rules.sh` and `verify_extra_rules.sh` -> **PASS**
- **Projection Law Constraints** (Prohibition of raw assets and VaRest REST calls in static configurations) → verified via rules in `ggen.toml` -> **PASS**

---

## Coverage Gaps

- **Asset Cooking vs Actor Reference** — The ontology currently lacks explicit relationships linking actors and scene components to their cooked assets (e.g., relating `USkeletalMeshComponent` to a cooked `USkeletalMesh` representation). Risk level: **MEDIUM**. Recommendation: Extend reflection shapes to validate asset attachments.

---

## Unverified Items

- None. All target files were read, verified against tests, and tested via the validation scripts.

---
---

## Adversarial Challenge Report

## Challenge Summary

**Overall risk assessment**: HIGH

While the validation constraints successfully block simple violations, the core design contains gaps that allow invalid compilations to bypass verification. Specifically, the audio validator fails to catch arbitrary unsupported formats, the shipping configuration checks are fragile, and there are redundant execution paths.

---

## Challenges

### [High] Challenge 1: Audio Format Checker Bypass
- **Assumption challenged**: The validator ensures that all HTML5 cooked audio formats are either OggVorbis or small PCMs.
- **Attack scenario**: A developer sets the audio format to `AudioFormat_Bink` or `AudioFormat_ADPCM` for an HTML5 representation.
- **Blast radius**: The validation passes without errors. When deployed to a browser-native WASM build, the audio playback fails because the WASM runtime does not contain licensed Bink/ADPCM decoders.
- **Mitigation**: Update the filter in SHACL and ggen.toml to explicitly negate the allowed set.

### [Medium] Challenge 2: Fragile Label-Based Build Configuration Matching
- **Assumption challenged**: The validator ensures that all Shipping builds are optimized and have debugging console/symbols disabled.
- **Attack scenario**: A user creates an instance of `ue4:BuildConfiguration` representing a shipping client but names it `rdfs:label "Shipping_Client"`.
- **Blast radius**: `BuildConfigurationConsistencyShape` only matches if the label is exactly `"Shipping"`. The rule is bypassed, allowing unoptimized or console-enabled configurations to compile as shipping targets.
- **Mitigation**: Introduce a dedicated enum property or typestate property (e.g., `ue4:buildMode`) on `BuildConfiguration` rather than matching on labels.

### [Low] Challenge 3: Redundant Case-Insensitive Filtering
- **Assumption challenged**: The validator needs multiple checks to ensure VaRest is not called.
- **Attack scenario**: Matching `CONTAINS(STR(?func), "VaRest") || CONTAINS(LCASE(STR(?func)), "varest")`.
- **Blast radius**: Redundancy in SPARQL queries. The second part already covers all variations (since LCASE of "VaRest" is "varest").
- **Mitigation**: Simplify to `CONTAINS(LCASE(STR(?func)), "varest")`.

---

## Stress Test Results

- **Injecting non-Ogg/non-PCM audio (e.g., Bink)** → Expected: FAIL → Actual: **PASS** (Bypass successful)
- **Injecting UWorld instance** → Expected: PASS (valid world graph) → Actual: **FAIL** (Blocked by stray debug shape `TestWorldShape`)
- **Naming shipping configuration "Release"** → Expected: FAIL (if unoptimized) → Actual: **PASS** (Bypass successful due to label match limit)
