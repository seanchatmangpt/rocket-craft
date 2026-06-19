## 2026-06-19T00:35:36Z
You are the E2E Testing Infrastructure Implementer.
Your goal is to set up the validation configuration and test infrastructure for the UE4 Universal RDF Mapping project.

## MANDATORY INTEGRITY WARNING
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

## Tasks
1. Check if the directory `/Users/sac/.ggen/packs/ue4_ontology` exists. If not, create it.
2. Locate the `ggen` executable (e.g., check `which ggen` or look in `/Users/sac/.cargo/bin/ggen`).
3. Write `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml` with:
   - `[project]` metadata (name = "ue4-ontology", version = "0.1.0", etc.)
   - `[ontology]` pointing to `core.ttl` as source, with imports = ["reflection.ttl", "blueprints.ttl", "subsystems.ttl", "typestates.ttl"], and standard_only = false.
   - `[generation]` with rules = [] (since E2E Testing Orchestration does not perform codegen).
   - `[validation]` with shacl = ["shacl/validation.shacl.ttl"], strict_mode = true, and a list of custom rules (SPARQL ASK queries) validating:
     - Rule R1: Verify class hierarchy (`UObject`, `AActor`, `APawn`, `ACharacter`, `UActorComponent`, `UWorld`, `ULevel` existence and subClassOf connections).
     - Rule R2: Verify subsystem domains (presence of Rendering, Physics, and Networking subsystem classes/relationships).
     - Rule R3: Verify Reflection & Blueprint graphs (presence of reflection metadata classes like `UClass`, `UProperty`, `UFunction`, and Blueprint graph structures/node classes).
     - Rule R4: Verify Cooking & WASM Typestates (presence of typestate classes/properties representing cooking, linking, WASM/HTML5 packaging states).
4. Create the directory `/Users/sac/.ggen/packs/ue4_ontology/shacl`.
5. Write `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` containing SHACL shapes that enforce:
   - Public classes must have an `rdfs:label` (at least 1).
   - Public classes should have an `rdfs:comment` description.
   - Namespace sanity (subjects must use resolvable public IRIs, not private/opaque ones like `urn:private:`).
6. Try running the validation command `ggen sync --validate-only` (or `/Users/sac/.cargo/bin/ggen sync --validate-only`) from `/Users/sac/.ggen/packs/ue4_ontology`.
7. Verify that the command executes and outputs a diagnostic message (it is expected to fail with a file-not-found error since the ontology files like `core.ttl` do not exist yet). Report the command run output and exit code.

Please write all file contents cleanly and accurately. Do not use placeholders or TODOs.
Report back with details on the created files, their paths, and the output of the validation command execution.

## 2026-06-19T00:38:30Z
You are the E2E Testing Harness and Documentation Implementer.
Your goal is to write the validation harness script and the E2E testing documentation for the UE4 Universal RDF Mapping project.

## MANDATORY INTEGRITY WARNING
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

## Tasks
1. Write the validation script `/Users/sac/rocket-craft/validate_ontology.sh`. It should change directory to `/Users/sac/.ggen/packs/ue4_ontology` and execute `/Users/sac/.local/bin/ggen sync --validate-only true`. It must print messages clearly and exit with the exit code of the `ggen` command. Make sure to handle errors robustly.
2. Make `/Users/sac/rocket-craft/validate_ontology.sh` executable (using chmod +x).
3. Write a comprehensive `/Users/sac/rocket-craft/TEST_INFRA.md` following the 4-tier acceptance methodology.
4. Write `/Users/sac/rocket-craft/TEST_READY.md` containing the E2E test suite ready status, test runner command, coverage metrics, and the feature checklist.
5. Execute the script `/Users/sac/rocket-craft/validate_ontology.sh` and capture the output and exit code. Report the exact output in your handoff.

Please write the files cleanly, completely, and without TODOs. Report back with the paths of all files created and the command execution logs.
