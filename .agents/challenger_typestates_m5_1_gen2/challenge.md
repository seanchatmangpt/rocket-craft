## Challenge Summary

**Overall risk assessment**: HIGH

This assessment is HIGH due to a critical topological scoping issue in the `StaticBakingNoVaRestShape` SHACL rule that flags VaRest usage globally rather than scoping it to the target world, as well as literal string matching vulnerabilities in the build configuration validation that can easily be bypassed.

---

## Challenges

### [High] Challenge 1: Lack of Topological Scoping in VaRest Prohibition SHACL Shape

- **Assumption challenged**: The validator assumes that the `StaticBakingNoVaRestShape` SPARQL constraint correctly checks if the statically baked world target utilizes VaRest.
- **Attack scenario**: If the RDF dataset contains multiple worlds (e.g., a static world target `StaticWorld` and a dynamic world target `DynamicWorld` that uses VaRest), the SPARQL check:
  ```sparql
  SELECT $this ?node
  WHERE {
      $this ue4:hasStaticBaking ?bakeConfig .
      ?bakeConfig ue4:isStaticallyBaked true .
      $this ue4:targetWorld ?world .
      
      ?node ue4:callsFunction ?func .
      FILTER (
          CONTAINS(STR(?func), "VaRest") || 
          CONTAINS(LCASE(STR(?func)), "varest")
      )
  }
  ```
  will match ANY `?node` calling VaRest in the entire dataset, completely ignoring the connection to `?world`. As a result, the static target `StaticWorld` will be rejected simply because `DynamicWorld` contains a VaRest call, even though `StaticWorld` itself has no VaRest dependencies.
- **Blast radius**: Breaks multi-world configuration within a shared repository and prevents independent static and dynamic target builds.
- **Mitigation**: Update the SPARQL query to topologically bind `?node` to `?world` (e.g., via `?node ue4:nodeOf/ue4:hasGraph* ?world` or an equivalent component-to-world relationship).

### [Medium] Challenge 2: Label-Based Validation Bypass for Shipping Builds

- **Assumption challenged**: The validation rule `BuildConfigurationConsistencyShape` assumes that a Shipping build configuration will always be labeled with the exact string `"Shipping"`.
- **Attack scenario**: A user or misconfigured generator can define a build configuration instance of type `ue4:BuildConfiguration` but label it `"shipping"` (lowercase), `"ShippingBuild"`, or `"Shipping-HTML5"`. Because the SPARQL check does:
  ```sparql
  $this rdfs:label "Shipping" .
  ```
  any case variations or suffixes will bypass the consistency checks for optimization (`bOptimize`), symbols (`bEnableSymbols`), and logging (`bDisableLogging`).
- **Blast radius**: Unoptimized or debug-enabled builds could be quietly packaged for Shipping without throwing errors.
- **Mitigation**: Enforce the consistency rules based on an explicit enum class or type rather than the literal `rdfs:label`.

### [Medium] Challenge 3: Mismatch and Typo in `verify_extra_rules.sh` Test Case 5

- **Assumption challenged**: The test script assumes that the validation error pattern for VaRest prohibition matches the exact string `"Static baking target worlds must not use dynamic VaRest calls"`.
- **Attack scenario**: The actual validation error message defined in `validation.shacl.ttl` (Shape 13) and `ggen.toml` uses the phrase `"Statically baked target worlds must not use dynamic VaRest calls"`. Due to this typo ("Static baking" vs "Statically baked"), the test runner fails to match the output and reports a false test failure.
- **Blast radius**: The extra test suite consistently fails.
- **Mitigation**: Update the expected error string in `verify_extra_rules.sh` on line 136 to `"Statically baked target worlds must not use dynamic VaRest calls"`.

### [Low] Challenge 4: Shared Temp File Collision in Backup Mechanism

- **Assumption challenged**: The testing scripts assume that backup and restore operations via `/tmp/core.ttl.bak` are isolated and safe.
- **Attack scenario**: Since both scripts use `/tmp` with fixed names, concurrent test runs on the same machine will overwrite each other's backup files, leading to corrupted baselines and test pollution.
- **Blast radius**: Flaky and untrustworthy CI/CD test results.
- **Mitigation**: Use unique temporary file names (e.g. `mktemp`) or copy from a read-only template `core_temp.ttl` located within the workspace rather than backing up the target `core.ttl`.

---

## Stress Test Results

- **Run `./verify_all_rules.sh` from clean baseline** → All 22 tests pass → **PASS**
- **Run `./verify_extra_rules.sh` from clean baseline** → Test 5 fails due to string mismatch → **FAIL** (expected error: `"Static baking target worlds..."`, actual: `"Statically baked target worlds..."`)
- **Topological Scoping of VaRest check** → A dynamic node completely outside the static world causes static target failure → **FAIL** (Cartesian product/over-matching confirmed)

---

## Unchallenged Areas

- **Inference performance** — The performance impact of executing recursive SPARQL validation rules on large graphs containing thousands of nodes was not challenged as no large-scale graphs were provided.
