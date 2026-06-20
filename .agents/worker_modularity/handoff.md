# Handoff Report: Modular Identity Checks (USD300 Series)

**Status:** VERIFIED
**Object under test:** LSP diagnostics module (`crates/ggen-asset-lsp/src/diagnostics.rs`)
**Observed evidence:** 5 unit tests passing successfully in `cargo test -p ggen-asset-lsp`.
**Failure:** None.
**Repair:** Implemented USD301 to USD307 checks inside `run_diagnostics` and created `test_usd300_series_modularity_diagnostics`.
**Receipt required:** Successful execution of `cargo test -p ggen-asset-lsp`.
**Residuals:** No gameplay/walkthrough proof checked (out of scope for LSP diagnostics check).

---

## 1. Observation
- Modified file path: `/Users/sac/rocket-craft/crates/ggen-asset-lsp/src/diagnostics.rs`
- Diagnostic rules read from `/Users/sac/rocket-craft/.agents/SPR_MODULAR_IDENTITY.md`:
  - USD301 ERROR: duplicate USD geometry fingerprint
  - USD302 ERROR: part file renders full assembly
  - USD303 ERROR: part-local file contains foreign component prims
  - USD304 ERROR: expected part root missing
  - USD305 ERROR: mirrored part lacks mirror transform proof
  - USD306 ERROR: generated USD files share identical source template expansion
  - USD307 ERROR: part bounding box overlaps full-asset bounds
- Ran unit tests successfully via `cargo test -p ggen-asset-lsp` and got:
  ```
  running 5 tests
  test diagnostics::tests::test_vis200_morphology_diagnostics ... ok
  test diagnostics::tests::test_vis200_morphology_diagnostics_passing ... ok
  test diagnostics::tests::test_diagnostics_pipeline ... ok
  test code_actions::tests::test_code_actions ... ok
  test diagnostics::tests::test_usd300_series_modularity_diagnostics ... ok

  test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
  ```

## 2. Logic Chain
1. By examining the workspace, we found that `run_diagnostics` accepts `doc_path: &Path` and `content: &str`.
2. To detect modularity errors (USD301-USD307) for a part file, we first check if the file is a part file (having a `.usda` extension and not starting with `"ASSET_"`).
3. Under the asset's `usd/` directory, we fetch all other `.usda` files to perform the necessary context-dependent checks:
   - For USD301 & USD306: We read the file content of other part files. If any has the exact same content, we emit the diagnostics on the first line.
   - For USD302: We search for full assembly root `"/World"` or if the file contains all key component types (Head, Torso, Blade, Wing).
   - For USD303: We match prim declaration lines against local component rules. If `SM_Head.usda` contains declarations for `SM_Torso`, `SM_Blade_Left`, etc., they are flagged as foreign on that specific line.
   - For USD304: We check if expected root prim names (e.g. `SM_Head` or `Head` for head files) are present in the parsed prim definitions. If missing, we emit USD304 on the first line.
   - For USD305: If the file is a mirrored part (like Left/Right Blade or Wing), we read its counterpart's content. If they are identical or have identical translation/scale coordinates without sign inversion on the X axis, we emit USD305.
   - For USD307: We locate the master asset file `ASSET_*.usda` and extract its bounding box extents. If the part file specifies bounding box extents matching the master's extents, we emit USD307.
4. Unit tests were added in `test_usd300_series_modularity_diagnostics` covering all these cases, checking the correct diagnostics and exact lines.
5. All tests compiled and passed cleanly.

## 3. Caveats
- Checked coordinates/translations using simple parsing of parenthesized values (e.g. `(x, y, z)`). Assumptions were made that vectors are specified as numeric tuples on lines containing `translate` or `scale`. This is consistent with USDA specs.
- The bounds matching check parses floats/doubles in `extents = ` and uses a tolerance of `1e-5` to accommodate rounding variations.

## 4. Conclusion
The Modular Identity checks (USD300 series) have been successfully integrated into `ggen-asset-lsp/src/diagnostics.rs` in accordance with the specification. The new test suite confirms the diagnostics trigger on the correct files and lines.

## 5. Verification Method
To verify the implementation, run the project tests in `crates/ggen-asset-lsp`:
```bash
cargo test -p ggen-asset-lsp
```
Inspect `/Users/sac/rocket-craft/crates/ggen-asset-lsp/src/diagnostics.rs` to verify the USD301-USD307 rules are applied correctly.
