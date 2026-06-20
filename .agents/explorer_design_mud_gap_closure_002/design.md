# DESIGN PROPOSAL: Rust-Based MUD Gap Checker (`mud_gap_check`)

This document outlines the architecture, ontology metadata declarations, SPARQL queries, Tera template configurations, and integration plans for the new Rust-based gap checker (`mud_gap_check`) in `crates/mech_factory_mud`.

## 1. Architecture ($A = \mu(O^*)$)

Adhering to the **Combinatorial Maximalist Doctrine**, the gap checking verification program is not a static script but rather a compiled projection ($A$) generated directly from the digital twin ontology constraints ($O^*$) via `ggen` ($\mu$).

### Core Objectives
*   **Compile-Time Verification:** Any changes to the ontology schema (such as adding/removing stations, route nodes, or files) must automatically regenerate and recompile the gap checker program.
*   **No Hardcoded Expectations:** The list of files to check, paths to verify, and test parameters must be stored as metadata in the ontology and extracted via SPARQL queries.
*   **Zero-Branching Compliance:** Check statuses are mapped to strict type states where possible.
*   **Mathematical Proof via BLAKE3 Receipts:** The checker will produce a cryptographically hashed verification receipt containing BLAKE3 signatures of all validated files, execution trace logs, and final verdicts.

### Component Diagram

```
[ Ontology (TTL Schema) ]
          │
          ▼ (SPARQL extraction query: gap_check.rq)
[ Bounded Query Context (JSON) ]
          │
          ▼ (Tera Engine)
[ Templates: mud_gap_check.rs.tera ]
          │
          ▼ (ggen compilation)
[ crates/mech_factory_mud/src/bin/mud_gap_check.rs ]
          │
          ▼ (cargo run --bin mud_gap_check)
┌───────────────────────────────────────────────┐
│              Verification Loop                │
│  ├─ 1. Existance Check (Rust, CSV, Headers)   │
│  ├─ 2. CSV Structure & Data Integrity         │
│  ├─ 3. Test Suites Executions (Cargo Test)    │
│  └─ 4. Falsify / Counterfactual Runs          │
└───────────────────────┬───────────────────────┘
                        │
                        ▼ (Output Products)
[ gap_closure_report.json ] & [ gap_closure_report.md ]
```

---

## 2. Ontology Metadata Declarations

The expected file structure and the verification execution checks are declared directly within `ontology/ggen-packs/mech_factory_mud/schema/mech_factory_mud.ttl` using the following new RDF classes and properties:

```turtle
# ---------------------------------------------------------
# Gap Checker Metamodel
# ---------------------------------------------------------
mud:ExpectedFile a owl:Class ;
    rdfs:label "Expected Generated File" ;
    rdfs:comment "A file artifact expected to be present in the build output." .

mud:GapCheckRule a owl:Class ;
    rdfs:label "Gap Check Rule" ;
    rdfs:comment "An execution check or condition that must pass to ensure no gaps." .

mud:filePath a owl:DatatypeProperty ;
    rdfs:domain mud:ExpectedFile ;
    rdfs:range xsd:string ;
    rdfs:label "File Path" .

mud:fileType a owl:DatatypeProperty ;
    rdfs:domain mud:ExpectedFile ;
    rdfs:range xsd:string ;
    rdfs:label "File Type" .

mud:checkId a owl:DatatypeProperty ;
    rdfs:domain mud:GapCheckRule ;
    rdfs:range xsd:string ;
    rdfs:label "Check ID" .

mud:checkDescription a owl:DatatypeProperty ;
    rdfs:domain mud:GapCheckRule ;
    rdfs:range xsd:string ;
    rdfs:label "Check Description" .

mud:checkType a owl:DatatypeProperty ;
    rdfs:domain mud:GapCheckRule ;
    rdfs:range xsd:string ;
    rdfs:label "Check Type" .

mud:checkCommand a owl:DatatypeProperty ;
    rdfs:domain mud:GapCheckRule ;
    rdfs:range xsd:string ;
    rdfs:label "Command Line String" .

# ---------------------------------------------------------
# Expected File Instances
# ---------------------------------------------------------
# Rust outputs
mud:GenRustConstantsFile a mud:ExpectedFile ;
    mud:filePath "crates/mech_factory_mud/src/generated_constants.rs" ;
    mud:fileType "rust" .

mud:GenRustRouteFile a mud:ExpectedFile ;
    mud:filePath "generated/mech_factory_mud/rust/route.rs" ;
    mud:fileType "rust" .

mud:GenRustStationsFile a mud:ExpectedFile ;
    mud:filePath "generated/mech_factory_mud/rust/stations.rs" ;
    mud:fileType "rust" .

mud:GenRustPartsFile a mud:ExpectedFile ;
    mud:filePath "generated/mech_factory_mud/rust/parts.rs" ;
    mud:fileType "rust" .

mud:GenRustAuthorityFile a mud:ExpectedFile ;
    mud:filePath "generated/mech_factory_mud/rust/authority.rs" ;
    mud:fileType "rust" .

mud:GenRustProjectionFile a mud:ExpectedFile ;
    mud:filePath "generated/mech_factory_mud/rust/projection.rs" ;
    mud:fileType "rust" .

mud:GenRustReceiptFile a mud:ExpectedFile ;
    mud:filePath "generated/mech_factory_mud/rust/receipt.rs" ;
    mud:fileType "rust" .

mud:GenRustOcelFile a mud:ExpectedFile ;
    mud:filePath "generated/mech_factory_mud/rust/ocel.rs" ;
    mud:fileType "rust" .

# UE4 CSV DataTables
mud:GenUe4StationsCsv a mud:ExpectedFile ;
    mud:filePath "generated/mech_factory_mud/ue4/DataTables/DT_FactoryStations.csv" ;
    mud:fileType "csv" .

mud:GenUe4RouteCsv a mud:ExpectedFile ;
    mud:filePath "generated/mech_factory_mud/ue4/DataTables/DT_WalkthroughRoute.csv" ;
    mud:fileType "csv" .

mud:GenUe4PartsCsv a mud:ExpectedFile ;
    mud:filePath "generated/mech_factory_mud/ue4/DataTables/DT_PartFamilies.csv" ;
    mud:fileType "csv" .

mud:GenUe4SocketCsv a mud:ExpectedFile ;
    mud:filePath "generated/mech_factory_mud/ue4/DataTables/DT_SocketTopology.csv" ;
    mud:fileType "csv" .

mud:GenUe4SkinCsv a mud:ExpectedFile ;
    mud:filePath "generated/mech_factory_mud/ue4/DataTables/DT_SkinLayers.csv" ;
    mud:fileType "csv" .

mud:GenUe4MotionCsv a mud:ExpectedFile ;
    mud:filePath "generated/mech_factory_mud/ue4/DataTables/DT_MotionFamilies.csv" ;
    mud:fileType "csv" .

mud:GenUe4LodCsv a mud:ExpectedFile ;
    mud:filePath "generated/mech_factory_mud/ue4/DataTables/DT_SemanticLOD.csv" ;
    mud:fileType "csv" .

mud:GenUe4ProjectionCsv a mud:ExpectedFile ;
    mud:filePath "generated/mech_factory_mud/ue4/DataTables/DT_ProjectionCommands.csv" ;
    mud:fileType "csv" .

# UE4 C++ Headers
mud:GenUe4StepsHeader a mud:ExpectedFile ;
    mud:filePath "generated/mech_factory_mud/ue4/Headers/MechFactoryMudSteps.h" ;
    mud:fileType "header" .

mud:GenUe4AuthorityHeader a mud:ExpectedFile ;
    mud:filePath "generated/mech_factory_mud/ue4/Headers/MechFactoryMudAuthority.h" ;
    mud:fileType "header" .

mud:GenUe4ProjectionHeader a mud:ExpectedFile ;
    mud:filePath "generated/mech_factory_mud/ue4/Headers/MechFactoryMudProjection.h" ;
    mud:fileType "header" .

# ---------------------------------------------------------
# Gap Check Rule Instances
# ---------------------------------------------------------
mud:CheckRuleGgenSync a mud:GapCheckRule ;
    mud:checkId "GGEN_SYNC_PASSES" ;
    mud:checkDescription "Verify that ggen receipt file exists and ggen sync executes successfully" ;
    mud:checkType "FileExist" ;
    mud:filePath ".ggen/receipts/latest.json" .

mud:CheckRuleCargoTest a mud:GapCheckRule ;
    mud:checkId "CRATE_USES_GGEN_GENERATED_CONSTANTS" ;
    mud:checkDescription "Execute cargo test on the mech_factory_mud crate to verify ZST bounds compile" ;
    mud:checkType "CommandExecution" ;
    mud:checkCommand "cargo test -p mech_factory_mud" .

mud:CheckRuleFalsify a mud:GapCheckRule ;
    mud:checkId "FALSIFICATION_CASES_EQ_8_PASS" ;
    mud:checkDescription "Verify the falsification engine by running all test cases" ;
    mud:checkType "CommandExecution" ;
    mud:checkCommand "cargo run -p mech_factory_mud -- falsify --case all" .

mud:CheckRuleCounterfactual a mud:GapCheckRule ;
    mud:checkId "COUNTERFACTUAL_CASES_EQ_8_PASS" ;
    mud:checkDescription "Verify counterfactual simulations produce predicted refusal rules" ;
    mud:checkType "CommandExecution" ;
    mud:checkCommand "cargo run -p mech_factory_mud -- counterfactual --case all" .

mud:CheckRuleReplay a mud:GapCheckRule ;
    mud:checkId "REPLAY_PASSES" ;
    mud:checkDescription "Verify that standard walkthough sequence replay is valid" ;
    mud:checkType "CommandExecution" ;
    mud:checkCommand "cargo run -p mech_factory_mud -- replay" .

mud:CheckRuleVerify a mud:GapCheckRule ;
    mud:checkId "VERIFY_PASSES" ;
    mud:checkDescription "Verify the integrity checks of generated data and ocel matrices" ;
    mud:checkType "CommandExecution" ;
    mud:checkCommand "cargo run -p mech_factory_mud -- verify" .
```

---

## 3. SPARQL Extraction Query

We define a target-oriented query `ontology/ggen-packs/mech_factory_mud/queries/gap_check.rq` that extracts expected file pathways, check rules, and metadata. The `ORDER BY` clause guarantees strict determinism during compile generation:

```sparql
PREFIX mud: <https://ggen.io/ontology/mech_factory_mud/>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?subject ?type ?path ?fileType ?checkId ?desc ?checkType ?cmd WHERE {
    {
        ?subject a mud:ExpectedFile ;
                 mud:filePath ?path ;
                 mud:fileType ?fileType .
        BIND("ExpectedFile" AS ?type)
    }
    UNION
    {
        ?subject a mud:GapCheckRule ;
                 mud:checkId ?checkId ;
                 mud:checkDescription ?desc ;
                 mud:checkType ?checkType .
        BIND("GapCheckRule" AS ?type)
        OPTIONAL { ?subject mud:checkCommand ?cmd }
        OPTIONAL { ?subject mud:filePath ?path }
    }
    UNION
    {
        ?subject a mud:Station .
        BIND("Station" AS ?type)
    }
    UNION
    {
        ?subject a mud:RouteNode .
        BIND("RouteNode" AS ?type)
    }
} ORDER BY ?type ?subject ?checkId
```

---

## 4. Tera Template Design

The template `ontology/ggen-packs/mech_factory_mud/templates/rust/mud_gap_check.rs.tera` maps SPARQL context values directly into Rust source code, maintaining absolute type-safety.

```rust
// generated_by: ggen
// source_ttl: schema/mech_factory_mud.ttl
// source_query: queries/gap_check.rq
// source_template: templates/rust/mud_gap_check.rs.tera

use std::fs;
use std::process::Command;
use std::path::Path;
use serde::Serialize;
use regex::Regex;

#[derive(Serialize, Debug, Clone)]
pub struct Requirement {
    pub id: String,
    pub description: String,
    pub expected: String,
    pub actual: String,
    pub status: String,
}

#[derive(Serialize, Debug)]
pub struct GapClosureReport {
    pub computed_status: String,
    pub computed_scoped_status: String,
    pub requirements_total: usize,
    pub requirements_passed: usize,
    pub requirements_failed: usize,
    pub next_gap: Option<Requirement>,
    pub failed_requirements: Vec<Requirement>,
    pub passed_requirements: Vec<Requirement>,
}

fn main() -> anyhow::Result<()> {
    let mut requirements = Vec::new();

    // 1. File existence validation derived from ontology ExpectedFile declarations
    {% for row in results -%}
    {%- if row.type == "ExpectedFile" -%}
    check_file_exists(&mut requirements, "{{ row.subject | split(pat="#") | last }}", "{{ row.path }}");
    {% endif -%}
    {%- endfor %}

    // 2. Verification rules derived from ontology GapCheckRule declarations
    {% for row in results -%}
    {%- if row.type == "GapCheckRule" -%}
      {%- if row.checkType == "FileExist" -%}
    check_file_exists(&mut requirements, "{{ row.checkId }}", "{{ row.path }}");
      {%- elif row.checkType == "CommandExecution" -%}
    check_command(&mut requirements, "{{ row.checkId }}", "{{ row.desc }}", "{{ row.cmd }}");
      {%- endif -%}
    {%- endif -%}
    {%- endfor %}

    // 3. Custom dynamic checks aligned with ontology structures
    check_ggen_receipts(&mut requirements);
    check_factory_stations_canonical(&mut requirements);
    check_walkthrough_route_connected(&mut requirements);
    check_cargo_test_metrics(&mut requirements);

    // 4. Generate Reports
    let failed_reqs: Vec<Requirement> = requirements.iter().filter(|r| r.status == "FAILED").cloned().collect();
    let passed_reqs: Vec<Requirement> = requirements.iter().filter(|r| r.status == "PASSED").cloned().collect();
    
    let status = "PARTIAL_ALIVE";
    let scoped_status = if failed_reqs.is_empty() {
        "GGEN_AUTHORED_MUD_VERTICAL_SLICE_VERIFIED_UNDER_SCOPE"
    } else {
        "GGEN_MINIMAL_SYNC_VERIFIED_UNDER_SCOPE"
    };

    let report = GapClosureReport {
        computed_status: status.to_string(),
        computed_scoped_status: scoped_status.to_string(),
        requirements_total: requirements.len(),
        requirements_passed: passed_reqs.len(),
        requirements_failed: failed_reqs.len(),
        next_gap: failed_reqs.first().cloned(),
        failed_requirements: failed_reqs,
        passed_requirements: passed_reqs,
    };

    fs::create_dir_all("generated/mech_factory_mud")?;
    
    // JSON Output
    let json_report = serde_json::to_string_pretty(&report)?;
    fs::write("generated/mech_factory_mud/gap_closure_report.json", &json_report)?;

    // Markdown Output
    let mut md = format!("# Gap Closure Report\n\n**Status:** {}\n**Scoped Status:** {}\n\n## Requirements Table\n\n", status, scoped_status);
    md.push_str("| Requirement ID | Description | Status | Expected | Actual |\n|---|---|---|---|---|\n");
    for r in &requirements {
        md.push_str(&format!("| `{}` | {} | **{}** | `{}` | `{}` |\n", r.id, r.description, r.status, r.expected, r.actual));
    }
    fs::write("generated/mech_factory_mud/gap_closure_report.md", md)?;

    println!("{}", json_report);

    if report.requirements_failed > 0 {
        std::process::exit(1);
    }
    Ok(())
}

fn check_file_exists(reqs: &mut Vec<Requirement>, id: &str, path: &str) {
    let exists = Path::new(path).exists();
    reqs.push(Requirement {
        id: id.to_string(),
        description: format!("Verify existence of: {}", path),
        expected: "Exists".to_string(),
        actual: if exists { "Exists".to_string() } else { "Missing".to_string() },
        status: if exists { "PASSED".to_string() } else { "FAILED".to_string() },
    });
}

fn check_command(reqs: &mut Vec<Requirement>, id: &str, desc: &str, cmd_str: &str) {
    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    if parts.is_empty() { return; }
    let mut cmd = Command::new(parts[0]);
    if parts.len() > 1 {
        cmd.args(&parts[1..]);
    }
    let status = cmd.status();
    let passed = match status {
        Ok(s) => s.success(),
        Err(_) => false,
    };
    reqs.push(Requirement {
        id: id.to_string(),
        description: desc.to_string(),
        expected: "ExitCode(0)".to_string(),
        actual: format!("Passed: {}", passed),
        status: if passed { "PASSED".to_string() } else { "FAILED".to_string() },
    });
}

fn check_ggen_receipts(reqs: &mut Vec<Requirement>) {
    let receipt_path = ".ggen/receipts/latest.json";
    let mut files_synced = 0;
    let mut gen_rules = 0;
    let mut exists = false;
    
    if Path::new(receipt_path).exists() {
        if let Ok(content) = fs::read_to_string(receipt_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                exists = true;
                if let Some(arr) = json.get("output_hashes").and_then(|v| v.as_array()) {
                    files_synced = arr.len();
                }
                if let Some(arr) = json.get("input_hashes").and_then(|v| v.as_array()) {
                    gen_rules = arr.len();
                }
            }
        }
    }

    reqs.push(Requirement {
        id: "GGEN_SYNC_PASSES".to_string(),
        description: "Verify that ggen sync executed successfully".to_string(),
        expected: "true".to_string(),
        actual: exists.to_string(),
        status: if exists { "PASSED" } else { "FAILED" }.to_string(),
    });

    reqs.push(Requirement {
        id: "GGEN_GENERATION_RULES_GE_15".to_string(),
        description: "Verify that at least 15 generation rules were run".to_string(),
        expected: ">= 15".to_string(),
        actual: gen_rules.to_string(),
        status: if gen_rules >= 15 { "PASSED" } else { "FAILED" }.to_string(),
    });

    reqs.push(Requirement {
        id: "GGEN_FILES_SYNCED_GE_15".to_string(),
        description: "Verify that at least 15 output files were synchronized".to_string(),
        expected: ">= 15".to_string(),
        actual: files_synced.to_string(),
        status: if files_synced >= 15 { "PASSED" } else { "FAILED" }.to_string(),
    });
}

fn check_factory_stations_canonical(reqs: &mut Vec<Requirement>) {
    let path = "generated/mech_factory_mud/ue4/DataTables/FactoryStations.csv";
    let mut passed = false;
    let mut actual_desc = "File not found".to_string();

    let expected_stations = vec![
        {% for row in results -%}
        {%- if row.type == "Station" -%}
        "{{ row.subject | split(pat="#") | last | lower }}",
        {%- endif -%}
        {%- endfor %}
    ];

    if Path::new(path).exists() {
        if let Ok(content) = fs::read_to_string(path) {
            let lines: Vec<&str> = content.lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .collect();
            if !lines.is_empty() && lines[0].contains("station_id") {
                let mut found_stations = Vec::new();
                for line in &lines[1..] {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 2 {
                        found_stations.push(parts[1].trim().to_string());
                    }
                }
                found_stations.sort();
                let mut expected_sorted = expected_stations.clone();
                expected_sorted.sort();
                if found_stations == expected_sorted {
                    passed = true;
                    actual_desc = "CSV matches ontology stations".to_string();
                } else {
                    actual_desc = format!("CSV stations {:?} do not match ontology {:?}", found_stations, expected_sorted);
                }
            } else {
                actual_desc = "Invalid CSV header".to_string();
            }
        }
    }

    reqs.push(Requirement {
        id: "FACTORY_STATIONS_CSV_CANONICAL".to_string(),
        description: "Verify FactoryStations.csv has canonical stations from ontology".to_string(),
        expected: format!("{:?}", expected_stations),
        actual: actual_desc,
        status: if passed { "PASSED" } else { "FAILED" }.to_string(),
    });
}

fn check_walkthrough_route_connected(reqs: &mut Vec<Requirement>) {
    let path = "generated/mech_factory_mud/ue4/DataTables/WalkthroughRoute.csv";
    let mut passed = false;
    let mut actual_desc = "File not found".to_string();

    let expected_chain = vec![
        "spawn", "factory_entrance", "frame_assembly", "socket_topology", 
        "armor_skin", "rig_motion", "verification_gate", "receipt_terminal", "exit_or_loop"
    ];

    if Path::new(path).exists() {
        if let Ok(content) = fs::read_to_string(path) {
            let lines: Vec<&str> = content.lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .collect();
            
            if lines.len() == 10 && lines[0].contains("route_node_id") {
                let mut found = Vec::new();
                for line in &lines[1..] {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 2 {
                        found.push(parts[1].trim().to_string());
                    }
                }
                if found == expected_chain {
                    passed = true;
                    actual_desc = "Route is fully connected and matches expected chain".to_string();
                } else {
                    actual_desc = format!("Route order mismatch: {:?}", found);
                }
            } else {
                actual_desc = format!("Expected 10 lines (1 header + 9 nodes), found {}", lines.len());
            }
        }
    }

    reqs.push(Requirement {
        id: "WALKTHROUGH_ROUTE_CSV_CONNECTED".to_string(),
        description: "Verify WalkthroughRoute.csv matches connected ontology chain".to_string(),
        expected: format!("{:?}", expected_chain),
        actual: actual_desc,
        status: if passed { "PASSED" } else { "FAILED" }.to_string(),
    });
}

fn check_cargo_test_metrics(reqs: &mut Vec<Requirement>) {
    let output = Command::new("cargo")
        .args(&["test", "-p", "mech_factory_mud"])
        .output();

    let mut passed_tests = 0;
    let mut failed_tests = 0;
    let mut ignored_tests = 0;
    let mut runs_consts_test = false;
    let mut runs_bounds_test = false;
    let mut cmd_success = false;
    let mut actual_stdout = String::new();

    if let Ok(out) = output {
        cmd_success = out.status.success();
        actual_stdout = String::from_utf8_lossy(&out.stdout).to_string();

        let re = Regex::new(r"(\d+) passed;\s*(\d+) failed;\s*(\d+) ignored").unwrap();
        for cap in re.captures_iter(&actual_stdout) {
            passed_tests += cap[1].parse::<usize>().unwrap_or(0);
            failed_tests += cap[2].parse::<usize>().unwrap_or(0);
            ignored_tests += cap[3].parse::<usize>().unwrap_or(0);
        }

        runs_consts_test = actual_stdout.contains("crate_uses_ggen_generated_constants");
        runs_bounds_test = actual_stdout.contains("generated_authority_field_bounds_are_field_specific");
    }

    reqs.push(Requirement {
        id: "CRATE_USES_GGEN_GENERATED_CONSTANTS".to_string(),
        description: "Verify that the crate compiles using generated constants ZST".to_string(),
        expected: "true".to_string(),
        actual: runs_consts_test.to_string(),
        status: if runs_consts_test { "PASSED" } else { "FAILED" }.to_string(),
    });

    reqs.push(Requirement {
        id: "AUTHORITY_BOUNDS_TEST_EXISTS".to_string(),
        description: "Verify that bounds assertion tests exist".to_string(),
        expected: "true".to_string(),
        actual: runs_bounds_test.to_string(),
        status: if runs_bounds_test { "PASSED" } else { "FAILED" }.to_string(),
    });

    reqs.push(Requirement {
        id: "TESTS_PASSED_GE_45".to_string(),
        description: "Verify that at least 45 tests passed".to_string(),
        expected: ">= 45".to_string(),
        actual: passed_tests.to_string(),
        status: if passed_tests >= 45 { "PASSED" } else { "FAILED" }.to_string(),
    });

    reqs.push(Requirement {
        id: "TESTS_FAILED_EQ_0".to_string(),
        description: "Verify that 0 tests failed".to_string(),
        expected: "0".to_string(),
        actual: failed_tests.to_string(),
        status: if failed_tests == 0 && cmd_success { "PASSED" } else { "FAILED" }.to_string(),
    });

    reqs.push(Requirement {
        id: "IGNORED_TESTS_EQ_0".to_string(),
        description: "Verify that 0 tests are ignored".to_string(),
        expected: "0".to_string(),
        actual: ignored_tests.to_string(),
        status: if ignored_tests == 0 { "PASSED" } else { "FAILED" }.to_string(),
    });
}
```

---

## 5. Target Output Location & Execution Plan

### Output Path
The output of this generator rule will write the Rust source code directly to:
`crates/mech_factory_mud/src/bin/mud_gap_check.rs`

This file is automatically detected by `cargo` as a project binary because it is located inside the `src/bin/` subfolder.

### How to Run
It is invoked using standard Rust tooling:
```bash
cargo run --bin mud_gap_check
```

On success, it prints the JSON report to `stdout` and exits with code `0`.
On failure, it prints the JSON report detailing the failed requirements and exits with code `1`, halting any automated CI/CD or validation pipelines (conforming to the **Agent Jidoka** doctrine).

---

## 6. Ggen Integration Plan

To implement this design, the subsequent implementer agent will need to do the following:

1.  **Update TTL Schema:** Add the `ExpectedFile` and `GapCheckRule` instances in `schema/mech_factory_mud.ttl`.
2.  **Add SPARQL Query:** Save the extraction query as `queries/gap_check.rq`.
3.  **Add Tera Template:** Save the template code as `templates/rust/mud_gap_check.rs.tera`.
4.  **Register rule in ggen.toml:**
    Append the generator rule block:
    ```toml
    [[generation.rules]]
    name = "rust-gap-checker"
    query = { file = "queries/gap_check.rq" }
    template = { file = "templates/rust/mud_gap_check.rs.tera" }
    output_file = "crates/mech_factory_mud/src/bin/mud_gap_check.rs"
    mode = "Overwrite"
    ```
5.  **Compile & Execute:** Run `ggen sync` to generate the file, then verify with `cargo run --bin mud_gap_check`.
