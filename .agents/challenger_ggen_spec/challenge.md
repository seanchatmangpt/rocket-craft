## Challenge Summary

**Overall risk assessment**: MEDIUM

While the core ontology loading, inference processing, SPARQL validation, Tera templating, and file rendering function perfectly and produce correct code, there are significant quality issues with developer feedback (error code collision), onboarding ergonomics (boilerplate block on consecutive runs), and a major defect in the user-facing cryptographic audit trail feature (`audit.json`).

## Challenges

### [High] Challenge 1: Broken Audit Trail Lineage (`audit.json`)

- **Assumption challenged**: Enabling `require_audit_trail = true` in the `[generation]` block generates a correct user-facing cryptographic lineage receipt (`audit.json`) that maps all manifest, ontology, query, and template inputs to generated output files.
- **Attack scenario**: A security or compliance process relies on `audit.json` to verify that generated source files match the declared ontology schemas. The file is generated, but the JSON contains empty/blank hashes for all inputs, empty pipeline arrays, and reports `0` size and an empty content hash (`e3b0c442...` which is the SHA-256 of empty bytes) for all generated files (even though they contain actual data).
- **Blast radius**: Complete failure of the code lineage audit gate. Downstream verification tools will fail because the computed hash of the generated file (e.g., `90171e83...` for a 224-byte file) does not match the dummy hash (`e3b0c442...`) recorded in `audit.json`.
- **Mitigation**: Update the audit export module in the `ggen` CLI to serialize the actual calculated hashes and metadata instead of writing unpopulated placeholder maps/structs. The internal receipt `.ggen/receipts/latest.json` does contain the correct hashes, so the underlying hashing implementation is correct, but the serialization to `audit.json` is broken.

### [Medium] Challenge 2: Error Code Collision/Overloading (`E0011`)

- **Assumption challenged**: Each compiler error code uniquely maps to a single, distinct validation rule or error condition.
- **Attack scenario**: A developer executes `ggen sync` and gets `error[E0011]: Output file already exists in 'Create' mode`. Following the formal specification, they look up `E0011` and find "E0011 (Inference Query Determinism)", which states that all `CONSTRUCT` queries must end with `ORDER BY`. They spend time debugging their query syntax instead of simply deleting the existing file or setting `mode = "Overwrite"`.
- **Blast radius**: Developer confusion, incorrect diagnosis, and wasted debugging time.
- **Mitigation**: Reassign the "Output file already exists in 'Create' mode" error to a new distinct code (e.g., `E0015` or `E0016`) so that `E0011` remains dedicated exclusively to CONSTRUCT query determinism.

### [Low] Challenge 3: Boilerplate Default Mode Blocker

- **Assumption challenged**: The quick-start boilerplate can be run multiple times out-of-the-box as a developer makes incremental changes to their ontology.
- **Attack scenario**: A developer sets up the boilerplate from the specification and runs `ggen sync`. On the second run (or whenever they save a change), they get a build failure because the default generation mode is `Create`, which forbids the output file from existing.
- **Blast radius**: Friction during onboarding.
- **Mitigation**: Update the specification's `ggen.toml` boilerplate to specify `mode = "Overwrite"` or add a prominent note explaining that `Create` mode requires manual cleanup of files between runs.

## Stress Test Results

- **Run boilerplate first time** → Compiles and validates cleanly; writes 224 bytes to `output_structs.txt` → **PASS**
- **Run boilerplate second time** → Fails with `error[E0011]: Output file already exists in 'Create' mode` → **FAIL** (Boilerplate blocker)
- **Remove ORDER BY from generation SELECT query with strict_mode=true** → Correctly aborts validation with `error[E0013]: Generation rule 'generate-structs' SELECT query lacks ORDER BY` → **PASS**
- **Remove ORDER BY from inference CONSTRUCT query with strict_mode=true** → Correctly aborts validation with `error[E0011]: Inference rule 'standard-normalization' CONSTRUCT query lacks ORDER BY` → **PASS**
- **Enable require_audit_trail=true** → Generates `audit.json`, but all input metadata maps are empty and output hashes/sizes are recorded as empty/0 → **FAIL** (Defect in audit trail)

## Unchallenged Areas

- **Dependency resolution via [[packs]]** — Out of scope for quick-start validation testing.
- **External query file imports (VALUES inline checks)** — Inline query string was used; external query file imports were not stress-tested.
