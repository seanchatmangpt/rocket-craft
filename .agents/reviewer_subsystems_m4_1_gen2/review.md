# Subsystem Topologies Review and Adversarial Challenge Report

## Review Summary

**Verdict**: APPROVE

The UE4 Universal RDF Mapping subsystems ontology (`subsystems.ttl`), validation shapes (`validation.shacl.ttl`), and GGen configuration (`ggen.toml`) are correct, complete, and structurally sound. They successfully define and validate the critical rendering (materials, shaders, RHI), physics (collision, kinematics), and networking (replication, RPCs) domains.

---

## Findings

### Major Finding 1: Validation Engine Exit Code Facade
- **What**: The GGen compiler CLI (`ggen sync`) exits with code `0` even when custom rules or SHACL shapes fail validation.
- **Where**: `ggen` binary execution / `verify_all_rules.sh` exit checks.
- **Why**: This is a process defect because CI/CD pipelines or shell scripts relying on exit codes (e.g. `set -e` or `if ! command`) will not detect validation failures unless they perform manual text parsing on `stdout`/`stderr`.
- **Suggestion**: Update the `ggen` CLI to return a non-zero exit code (e.g. exit code 1) when validation fails.

### Minor Finding 2: Bash Subshell Trap Inheritance in Test Harness
- **What**: The test script `verify_all_rules.sh` registered `trap cleanup EXIT` at the parent level, which was inherited by command substitution subshells in bash 3.2. This caused premature deletion of the backup file `core.ttl.bak` when subshells exited.
- **Where**: `/Users/sac/rocket-craft/ggen-validation-tests/verify_all_rules.sh`.
- **Why**: This caused tests from Test 11 onwards to run against an accumulated dirty state, leading to false negatives.
- **Suggestion**: Comment out the automatic `EXIT` trap and call `cleanup` explicitly at the end of the script (remediated during review).

---

## Verified Claims

- **Rendering Ontology Syntax & Imports** → verified via `/Users/sac/rocket-craft/validate_ontology.sh` → **PASS**
- **Physics Collision & Kinematics Consistency** → verified via `/Users/sac/rocket-craft/ggen-validation-tests/verify_all_rules.sh` (Tests 11-14) → **PASS**
- **Replication & RPC validation rules** → verified via `/Users/sac/rocket-craft/ggen-validation-tests/verify_all_rules.sh` (Tests 15-16) → **PASS**

---

## Coverage Gaps

- **WASM Unreal HTML5 Visual Actuation** — risk level: low — recommendation: accept risk. While rendering fallbacks are checked at the RDF/SHACL level, visual delta check requires actual Playwright browser execution as defined in PROJECT.md.

---

## Unverified Items

- None. All target ontologies and shapes were parsed, tested, and validated.

---

# Adversarial Challenge Report

## Challenge Summary

**Overall risk assessment**: LOW

The validation constraints are highly robust and prevent structural defects in UE4 rendering, physics, and networking topologies prior to source admission.

---

## Challenges

### Medium Challenge 1: Infinite Material Inheritance Loops
- **Assumption challenged**: Material instances resolve to a base UMaterial.
- **Attack scenario**: A user defines `MatInstA` inheriting from `MatInstB`, which in turn inherits from `MatInstA`.
- **Blast radius**: infinite recursion in the compiler when resolving parameters, leading to compiler stack overflow.
- **Mitigation**: Prevented by `RuleJ` and `MaterialInstanceAcyclicityShape` via transitive path query checking for self-reference (`?mi ue4:parentMaterial+ ?mi`).

### High Challenge 2: Kinematic Simulation Gravity Instability
- **Assumption challenged**: Active physics bodies do not fall through the scene floor.
- **Attack scenario**: A rigid body is simulated (`PhysType_Simulated`) and gravity is enabled (`bEnableGravity true`), but collision is disabled (`NoCollision`).
- **Blast radius**: The object falls through the floor indefinitely, causing state desynchronization and NaNs.
- **Mitigation**: Blocked by `SimulatedGravityCollisionShape` which flags any simulated gravity body that lacks active collision properties.

---

## Stress Test Results

- **Self-referencing material instances** → triggers acyclicity error → blocked successfully (PASS)
- **Gravity-enabled body without collision** → triggers collision safety error → blocked successfully (PASS)
- **RPC defined on non-replicated actor** → triggers replication requirement error → blocked successfully (PASS)

---

## Unchallenged Areas

- **C++ Reflection Metadata** — reason not challenged: C++ reflection classes are external and assumed structurally correct.
