## 2026-06-18T17:46:18-07:00
You are auditor_core. Your working directory is `/Users/sac/rocket-craft/.agents/auditor_core`.
Please initialize your progress.md and context.md.

## Mission
Perform forensic integrity auditing on the C++ Backbone ontology (core.ttl) and ggen.toml.

## Scope
- Verify that there are no hardcoded test results, bypasses, or facade/fake implementations in `core.ttl` or `ggen.toml`.
- Check if all class hierarchies, namespace definitions, and properties are genuinely and fully defined in `core.ttl`.
- Confirm that the validation harness `/Users/sac/rocket-craft/validate_ontology.sh` executes the actual `ggen` compiler and checks actual RDF structure.
- Write your audit report to `audit.md` in your directory.

Report completion and audit verdict to parent ID: 4f79cb22-2adb-466d-9e20-d8baef6e934d.
