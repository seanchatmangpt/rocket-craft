# Context

## Mission
Verify the correctness, consistency, and hierarchy extraction capabilities of the C++ Backbone ontology (core.ttl).

## Scope
- Run validate_ontology.sh
- Verify core.ttl structure and subclasses of UObject
- Implement SPARQL queries to extract the hierarchy
- Report empirical findings in challenger_report.md

## Current Findings
- Verified `validate_ontology.sh` succeeds with code `0`.
- Extracted class hierarchy successfully: 19 transitive subclasses under `ue4:UObject`.
- Stress test findings: `ggen sync` ignores validation rules during `--validate-only true` and executes naive string matching on generation queries.
- Report written to `challenger_report.md`.
