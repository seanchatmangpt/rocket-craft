# Handoff Report: Adversarial Verification of Eden Server Ontology & Queries

## 1. Observation
Adversarial tests were performed using a custom Python harness (`adversarial_test.py`) against the files in `/Users/sac/.ggen/packs/eden_server`:
- `ontology/pack.ttl`
- `ontology/deltas.ttl`
- `queries/substrate.rq`
- `queries/extract_authority_deltas.rq`
- `queries/extract_assembly_deltas.rq`
- `queries/extract_receipt_deltas.rq`

The verification was run via the command:
`python3 /Users/sac/rocket-craft/.agents/challenger_ontology_2/adversarial_test.py`

### Direct Output & Error Observations:
- **DataType Violation Parsing Warnings/Errors:**
  RDFLib successfully parsed syntactically incorrect/out-of-range literals without failing, but threw errors or warnings when resolving values or converting via `.toPython()`:
  ```
  Failed to convert Literal lexical form to value. Datatype=http://www.w3.org/2001/XMLSchema#unsignedByte, Converter=<class 'int'>
  ValueError: invalid literal for int() with base 10: 'not-a-byte'
  
  Failed to convert Literal lexical form to value. Datatype=http://www.w3.org/2001/XMLSchema#float, Converter=<class 'float'>
  ValueError: could not convert string to float: 'not-a-float'
  
  UserWarning: Parsing weird boolean, 'maybe' does not map to True or False
  ```
- **SPARQL Datatype Comparisons:**
  For `damageClass "-1"^^xsd:unsignedByte`:
  - `FILTER (xsd:integer(?damage) < 0)` correctly returned the part (`part_negative`).
  - `FILTER (?damage < 0)` returned an empty list, showing that standard type comparisons in SPARQL can silently fail/ignore out-of-range values.
- **Assembly Tree Cycles:**
  Running `substrate.rq` on a cycle (`root` -> `socket1` -> `subAssy1` -> `socket2` -> `subAssy2` -> `socket3` -> `subAssy1`) returned 3 rows and terminated in `0.0432 seconds` without hanging or causing recursion limits.
- **Namespace Trailing Slash Discrepancies:**
  Using `@prefix eden_noslash: <https://ggen.io/ontology/eden-server>` resulted in subjects resolving to `https://ggen.io/ontology/eden-serverpart1`. Searching for `https://ggen.io/ontology/eden-server/Part` returned 0 matches, demonstrating prefix sensitivity.
- **Scale Stress Testing Execution Times:**
  - **Tree Width scale:**
    - Width 100: 0.2883s
    - Width 1000: 1.2321s
    - Width 5000: 6.6317s
  - **Delta Volume scale (5,000 of each delta type, total 60,235 triples):**
    - `extract_authority_deltas.rq`: 5.7261s
    - `extract_assembly_deltas.rq`: 5.5454s
    - `extract_receipt_deltas.rq`: 10.2670s
- **Query Compilation Averages:**
  - `substrate`: 61.440 ms
  - `extract_authority_deltas`: 34.335 ms
  - `extract_assembly_deltas`: 31.460 ms
  - `extract_receipt_deltas`: 53.356 ms

---

## 2. Logic Chain
1. **Weak Datatype Enforcement:** Since RDFLib parses out-of-range unsigned bytes (`-1`, `256`, `9999`) and invalid strings (`"not-a-byte"`) without validation errors, invalid/malformed values can reside in the graph unnoticed.
2. **SPARQL Silent Type Errors:** In SPARQL, comparing derived or invalid literals without casting (`FILTER (?damage < 0)`) fails silently because the engine marks the expression invalid for that row and drops it. Consequently, validation checks designed to catch invalid data in queries will fail to report them unless explicit casts (e.g. `xsd:integer(?damage)`) are used.
3. **Performance Bottlenecks:**
   - Wide trees (e.g., 5000 sockets) require over 6 seconds for hierarchy resolution.
   - Large graph sizes (~60k triples) cause queries to run for 5 to 10 seconds.
   - This occurs because RDFLib uses an in-memory triple store without structural indexes for complex property paths and multi-optional queries.
4. **Compilation Overhead:** Parsing queries dynamically on each invocation introduces a 30-60ms delay.

---

## 3. Caveats
- Tests were executed on RDFLib 6.x. Behavior may slightly differ under different RDFLib versions or alternative triplestores (like GraphDB or Jena).
- Concurrency was not tested. RDFLib `Graph` instances are not thread-safe, so executing these queries concurrently could lead to data corruption or race conditions.

---

## 4. Conclusion
While the SPARQL queries are syntactically valid and robust against infinite looping in cycles, they exhibit high performance overhead at scale (5,000+ entities) and lack automatic datatype/schema-level enforcement. This makes the system susceptible to silent validation failures and execution timeouts.

---

## 5. Verification Method
1. Navigate to `/Users/sac/rocket-craft/.agents/challenger_ontology_2/`.
2. Run the test harness:
   `python3 adversarial_test.py`
3. Inspect output to verify datatype warnings, query execution times, and returned result row counts.

---

# Challenge Report

## Challenge Summary

**Overall risk assessment**: MEDIUM

## Challenges

### [Medium] Challenge 1: Silent DataType Validation Bypass
- **Assumption challenged**: That the ontology's `rdfs:range xsd:unsignedByte` constraint prevents or flags invalid or out-of-range byte values (e.g., negative numbers, values > 255, non-numeric strings).
- **Attack scenario**: An external service publishes telemetry with a damage class of `9999` or `-1`. RDFLib loads it into the graph without throwing any errors.
- **Blast radius**: Downstream components expecting a valid unsigned byte (0-255) will crash during runtime conversion, or query filters will produce incorrect results.
- **Mitigation**: Use a SHACL validator or explicit SPARQL queries with `xsd:integer(?val)` typecasts to perform schema-level validation before processing.

### [Medium] Challenge 2: Performance Degradation under Medium Loads
- **Assumption challenged**: That the SPARQL queries are lightweight and performant enough for real-time state extraction.
- **Attack scenario**: The active assembly tree grows to 5,000 components, or the history accumulation reaches 5,000 delta records.
- **Blast radius**: Substrate hierarchy resolution takes ~6.6s, and delta queries take 5-10s, blocking the web server/API.
- **Mitigation**: 
  - Cache prepared SPARQL query objects (`prepareQuery`) to save 30-60ms compilation overhead.
  - Implement pagination, filter by time, or utilize a persistence triplestore (e.g., SQLite-backed or GraphDB) instead of pure in-memory rdflib.

---

## Stress Test Results

- **DataType Violations** → RDFLib logs warnings, but continues execution. Python native conversion raises ValueErrors for strings, but parses out-of-range integers without issue. → **FAIL** (No strict validation)
- **Assembly Cycle Traversal** → Query terminates in 0.043s and returns exact cycle loop details. → **PASS** (Cyclic robustness)
- **Missing Namespace Slash** → Query misses matching triples due to strict prefix requirements. → **PASS** (Correct prefix alignment required)
- **Scale (5k width tree)** → Query executes, but requires 6.63s to resolve. → **FAIL** (Performance bottleneck)
- **Delta Scale (60k triples)** → Queries execute, but take up to 10.2s. → **FAIL** (Scale bottleneck)
