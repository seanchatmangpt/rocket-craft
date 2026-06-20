## 2026-06-18T18:34:34-07:00
Objective: Integrate the custom SPARQL validation rules into the production `ggen.toml` configuration to resolve the compiler's SHACL validation gaps.

## Instructions
1. Read `/Users/sac/rocket-craft/ggen-validation-tests/ggen.toml` from line 132 to the end of the file to extract the 10 custom validation rules (RuleA, RuleB, RuleC, RuleD, RuleE, RuleF, RuleG, RuleH, RuleLabel, RuleNamespace).
2. Append these 10 custom validation rules to the production `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml`.
3. Run `/Users/sac/rocket-craft/validate_ontology.sh` to verify that the baseline ontology compiles and passes all validations successfully.
4. Write your handoff report to `/Users/sac/rocket-craft/.agents/worker_integrate_validation_rules/handoff.md`.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.
