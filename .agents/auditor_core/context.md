# Operational Context

## Objective
To perform a rigorous, independent forensic audit on the C++ Backbone ontology (`core.ttl`), `ggen.toml`, and the validation harness `validate_ontology.sh`.

## Environment
- Workspace: `/Users/sac/rocket-craft`
- Target Files:
  - Ontology: `core.ttl` (exact location TBD, likely under `ontology/` or similar)
  - Config: `ggen.toml` (exact location TBD)
  - Harness: `/Users/sac/rocket-craft/validate_ontology.sh`
- Agent Working Directory: `/Users/sac/rocket-craft/.agents/auditor_core`

## Key Stakeholders
- Parent Agent: `4f79cb22-2adb-466d-9e20-d8baef6e934d` (parent)
- Auditor: `auditor_core` (this agent)

## Audit Integrity Level
Based on `ORIGINAL_REQUEST.md` (and generic project instructions), we must check for:
- Hardcoded test results / expected outputs
- Facade or dummy implementations
- Bypasses in compiler execution/RDF checks
- Incomplete class hierarchies or namespace issues

We will verify everything empirically and provide raw tool logs.
