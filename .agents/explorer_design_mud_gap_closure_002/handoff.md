# Handoff Report: Explorer Design MUD Gap Checker

## 1. Observation

*   **Existing Gap Checker:** We observed the Python script at `scripts/mud_gap_check.py` which validates 21 requirements:
    *   Lines 44-53: Checks existence of 8 generated Rust files.
    *   Lines 58-67: Checks existence of 8 generated UE4 CSV files.
    *   Lines 72-76: Checks existence of 3 generated UE4 headers.
    *   Lines 80-91: Checks if `FactoryStations.csv` is canonical.
    *   Lines 94-104: Checks if `WalkthroughRoute.csv` is connected.
    *   Lines 107-138: Runs `cargo test -p mech_factory_mud` and parses its output using regex to verify passed, failed, and ignored tests.
    *   Lines 144-150: Runs replay and verify commands.
*   **Ontology Configuration:** We observed the ggen ontology pack configuration in `ontology/ggen-packs/mech_factory_mud/ggen.toml`.
    *   Line 12: `source = "schema/mech_factory_mud.ttl"`
    *   Lines 17-205: Lists generation rules that map SPARQL queries and Tera templates to generated output files.
*   **Ontology Definitions:** We observed `ontology/mech_factory_mud.ttl` defining classes such as `mud:Station`, `mud:RouteNode`, and properties like `mud:hasNextNode`.
*   **Test Results:** We executed `cargo test -p mech_factory_mud` using the run command and observed:
    *   `running 24 tests ... ok` (under `unittests src/lib.rs`)
    *   `running 25 tests ... ok` (under `tests/expanded.rs`)
    *   `running 5 tests ... ok` (under `tests/receipt_chain.rs`)
    *   `running 1 test ... ok` (under `tests/refusals.rs`)
    *   `running 1 test ... ok` (under `tests/ue4_export.rs`)
    *   Total tests running: 56 tests, all passed.

## 2. Logic Chain

1.  *From existing gap checker observations:* The list of expected files, command strings, and canonical check values are hardcoded in a python script (`scripts/mud_gap_check.py`).
2.  *From Combinatorial Maximalist Doctrine ($A = \mu(O^*)$) constraints:* Hardcoded logic contradicts the law of compile-time projection from the ontology. Thus, the list of expected files and check rules must be declared inside the ontology schema (`schema/mech_factory_mud.ttl`).
3.  *From ggen structure constraints:* ggen processes queries and templates. Defining new classes (`ExpectedFile`, `GapCheckRule`) in the turtle schema enables querying them via SPARQL.
4.  *From SPARQL Bounded Selection principle:* A dedicated SPARQL query `queries/gap_check.rq` with `UNION` avoids cluttering the main query and isolates rule extraction.
5.  *From Rust project bin layout:* Writing the generated Tera template output to `crates/mech_factory_mud/src/bin/mud_gap_check.rs` makes it a first-class project binary executable with `cargo run --bin mud_gap_check`.

## 3. Caveats

*   **Implementation Not Verified in Source:** Per the instructions ("Do not modify any files except files in your working directory"), the schema changes, queries, and templates have not been saved to the main project directories.
*   **Cargo Run Execution Assumes Serde/Regex:** The generated code assumes dependencies like `serde` and `regex` are declared in `crates/mech_factory_mud/Cargo.toml`. Since `serde` is already imported in `main.rs`, this is a valid assumption; `regex` might need to be added to Cargo dependencies if not already present.

## 4. Conclusion

The design proposal for a Rust-based, ontology-driven gap checker is functionally complete and ready for handoff to the implementer agent. By parameterizing expected files and check commands as turtle triples, the new checker adheres strictly to the $A = \mu(O^*)$ projection law.

## 5. Verification Method

To verify this design, the receiving implementer agent should:
1.  Add the turtle triples in `ontology/ggen-packs/mech_factory_mud/schema/mech_factory_mud.ttl`.
2.  Save the query as `ontology/ggen-packs/mech_factory_mud/queries/gap_check.rq`.
3.  Save the template as `ontology/ggen-packs/mech_factory_mud/templates/rust/mud_gap_check.rs.tera`.
4.  Add the generator rule to `ontology/ggen-packs/mech_factory_mud/ggen.toml`.
5.  Execute `ggen sync` to generate the file.
6.  Build and run the check binary:
    ```bash
    cargo run --bin mud_gap_check
    ```
7.  Verify that `generated/mech_factory_mud/gap_closure_report.json` and `.md` are generated correctly with a `PASSED` status.

---

## 6. Status Reporting

**Status:** PARTIAL
**Object under test:** mud_gap_check architecture and metadata design
**Observed evidence:** `scripts/mud_gap_check.py` file content, `Cargo.toml`, `ggen.toml` configuration, `ontology/mech_factory_mud.ttl`.
**Failure:** None.
**Repair:** Designed the Rust-based gap checker template and schema additions to replace Python check script.
**Receipt required:** `design.md` written in `/Users/sac/rocket-craft/.agents/explorer_design_mud_gap_closure_002/design.md`.
**Residuals:** Verification of actual template execution and binary compilation is deferred to the implementer agent.
