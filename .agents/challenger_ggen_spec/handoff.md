# Handoff Report - Challenger Ggen Spec

## 1. Observation

- **Boilerplate File Setup**:
  The boilerplate setup files were extracted from `/Users/sac/.ggen/specs/GGEN_PACK_SPEC.md` and written to `/Users/sac/rocket-craft/ggen-challenger-verify/` as `ggen.toml`, `schema/domain.ttl`, and `templates/struct.tera`.
- **First Execution**:
  Command `/Users/sac/.local/bin/ggen sync` ran successfully on the clean boilerplate.
  Output:
  ```text
  All Gates: ✅ PASSED → Proceeding to generation phase
  ℹ Generating 1 files...
  ✓ Generated 1 files in 2ms
    1 inference rules, 1 generation rules
    224 total bytes written
  ```
- **Consecutive Execution**:
  Running `/Users/sac/.local/bin/ggen sync` a second time produced the following error:
  ```text
  ERROR: CLI execution failed: Command execution failed: error[E0003]: Pipeline execution failed
    |
    = error: error[E0011]: Output file already exists in 'Create' mode
    --> rule: 'generate-structs', output: './output_structs.txt'
  ```
- **Determinism Query Validation**:
  Removing `ORDER BY` from `[inference.rules]` under `strict_mode = true` triggered:
  ```text
  = error: error[E0011]: Inference rule 'standard-normalization' CONSTRUCT query lacks ORDER BY
  ```
  Removing `ORDER BY` from `[[generation.rules]]` under `strict_mode = true` triggered:
  ```text
  = error: error[E0013]: Generation rule 'generate-structs' SELECT query lacks ORDER BY
  ```
- **Cryptographic Audit Trail**:
  Running with `require_audit_trail = true` generated `audit.json` with the following content:
  ```json
  {
    "generated_at": "2026-06-19T00:56:01.909828+00:00",
    "ggen_version": "26.6.9",
    "inputs": {
      "manifest_hash": "",
      "ontology_hashes": {},
      "template_hashes": {},
      "query_hashes": {}
    },
    "pipeline": [],
    "outputs": [
      {
        "path": "./output_structs.txt",
        "content_hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "size_bytes": 0,
        "source_rule": "rule-./output_structs.txt"
      }
    ],
    "validation_passed": true,
    "total_duration_ms": 0
  }
  ```
  However, the internal receipt file `.ggen/receipts/latest.json` contained:
  ```json
  "output_hashes": [
    "./output_structs.txt:90171e83f58dc79948d83718cd091836a0e5400b3c6c977c6e695ebeb444a155"
  ]
  ```

## 2. Logic Chain

1. **E0011 Error Code Overloading**:
   - Manifest validation for missing `ORDER BY` in inference CONSTRUCT query produces an error with code `E0011` (Observation: "Inference rule ... lacks ORDER BY").
   - Pipeline execution when the output file exists in `Create` mode produces an error also coded `E0011` (Observation: "Output file already exists in 'Create' mode").
   - Therefore, the error code `E0011` is overloaded between two distinct error stages/meanings.
2. **Boilerplate Ergonomics Blocker**:
   - The quick-start guide doesn't configure `mode = "Overwrite"`.
   - Running the boilerplate consecutively triggers the file exists error coded `E0011` (Observation: "Output file already exists in 'Create' mode").
   - Therefore, developers using the default quick-start boilerplate will face an immediate compilation/generation failure upon running it a second time.
3. **Broken Audit Trail Export**:
   - The CLI successfully computes cryptographic hashes of generated files as evidenced by `.ggen/receipts/latest.json` (Observation: `"output_hashes": ["./output_structs.txt:90171e83..."]`).
   - The generated `audit.json` contains blank/empty inputs and records output size 0 and content hash as `e3b0c442...` (SHA-256 of empty bytes) (Observation: `"content_hash": "e3b0c442...", "size_bytes": 0`).
   - Therefore, the export serialization for the user-facing `audit.json` file is broken and does not write the computed hashes.

## 3. Caveats

- We assumed the `ggen` executable located at `/Users/sac/.local/bin/ggen` is the authoritative binary for the platform, which matches behavior described in `GGEN_PACK_SPEC.md`.
- We did not test importing remote packs via Git URLs or package version constraints.

## 4. Conclusion

The boilerplate example in `/Users/sac/.ggen/specs/GGEN_PACK_SPEC.md` is syntactically correct and successfully generates the output files, but presents developer friction and a critical audit lineage bug:
1. Running it consecutively triggers a file exists error due to the default `Create` mode.
2. Error code `E0011` is overloaded, which will mislead developers looking at the specification for query determinism issues.
3. Enabling `require_audit_trail` outputs a broken, unpopulated `audit.json` file with dummy size 0 and empty-string hashes.

## 5. Verification Method

To independently verify these findings, perform the following steps:
1. Run `cd /Users/sac/rocket-craft/ggen-challenger-verify/`
2. Run `/Users/sac/.local/bin/ggen sync` to generate the file once.
3. Run `/Users/sac/.local/bin/ggen sync` again. Verify it fails with `error[E0011]`.
4. Inspect `audit.json` and verify `content_hash` is `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` and inputs/outputs maps are unpopulated, while `.ggen/receipts/latest.json` contains correct hashes.
