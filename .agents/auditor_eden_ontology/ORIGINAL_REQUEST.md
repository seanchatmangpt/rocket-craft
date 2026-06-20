## 2026-06-19T02:04:00Z
You are the Forensic Integrity Auditor (teamwork_preview_auditor). Your working directory is /Users/sac/rocket-craft/.agents/auditor_eden_ontology/.

Audit the refactored eden_server ontology registry and validation harness configured in /Users/sac/.ggen/packs/eden_server/.

Perform the following checks:
1. Verify OWL 2 DL validity and that imports (FIBO, SOSA, QUDT, PROV-O) parse with zero syntax errors using Raptor / rapper.
2. Verify that ggen.toml has strict_mode=true and successfully compiles and validates all shapes and rules using /Users/sac/.local/bin/ggen sync --validate-only true.
3. Perform mutation testing to check if the validation harness and SHACL validation rules are genuinely wired up and functioning. Mutate an ontology turtle file to inject a class hierarchy violation or out-of-bounds byte-class value (e.g. riskClass = 256), run ggen sync --validate-only true, verify that the compiler fails and prints the validation error, and then restore the ontology file.
4. Verify that there are no placeholders, stubs, TODOs, or bypasses.
5. Report detailed findings to /Users/sac/rocket-craft/.agents/auditor_eden_ontology/handoff.md and update progress.md. When complete, send a message to parent (Recipient: a3da08eb-0131-43a9-9c13-f9c39fdd291b) with your verdict.
