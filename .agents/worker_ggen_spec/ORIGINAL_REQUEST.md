## 2026-06-18T17:51:11-07:00
You are a Worker subagent (Worker Ggen Spec Author).
Your working directory is: /Users/sac/rocket-craft/.agents/worker_ggen_spec/
Your identity is: worker_ggen_spec

Your task:
1. Read the synthesized findings from `/Users/sac/rocket-craft/.agents/orchestrator_ggen_spec/synthesis.md`.
2. Write the canonical formal specification for building a validated `ggen` ontology pack to `/Users/sac/.ggen/specs/GGEN_PACK_SPEC.md`. Ensure the target directory `/Users/sac/.ggen/specs/` exists.
3. The specification document `GGEN_PACK_SPEC.md` must include:
   - Detailed documentation of the required structure of the `ggen.toml` manifest, including descriptions, fields, and types for `[project]`, `[ontology]`, `[inference]`, `[[generation.rules]]`, `[validation]`, and `[[packs]]` blocks.
   - A clear explanation of the difference between `[inference]` rules (using `CONSTRUCT` queries to enrich the RDF graph) and `[[generation.rules]]` (using `SELECT` queries mapped to `.tera` templates to project the graph to output files).
   - Documentation of the validation error guards: E0010 (VALUES Inline Guard), E0011 (Inference Query Determinism), E0013 (Generation Query Determinism), and E0014 (Pack Dependency Guard).
   - Detailed explanation of the "BIG BANG 80/20" criteria (Specification Closure First) checklist with the 5 reference conditions.
   - A comprehensive Quick-Start Boilerplate section containing:
     * A copy-pasteable minimal `ggen.toml` manifest conforming to the engine's schema.
     * A reference `.ttl` Turtle structure.
     * A sample SPARQL CONSTRUCT query for inference.
     * A sample SPARQL SELECT query for generation.
     * A sample Tera template.
4. Verify that the file `/Users/sac/.ggen/specs/GGEN_PACK_SPEC.md` is successfully written. If there are any validation scripts or test commands in `~/ggen/` that check manifest syntax, run them to confirm the boilerplate example is valid.
5. Write your handoff report to `/Users/sac/rocket-craft/.agents/worker_ggen_spec/handoff.md` summarizing the created file, its exact contents/sections, and verification steps.
6. When done, call send_message to report back to your parent orchestrator at b6f958b7-c50a-4ec3-8e16-40ef0a23f032 with a summary and the path to your handoff report.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.
