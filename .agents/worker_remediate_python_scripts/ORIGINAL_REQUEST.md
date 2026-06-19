## 2026-06-19T01:32:42Z
Objective: Immediately purge all Python scripts from `/Users/sac/rocket-craft/ggen-validation-tests` and rewrite the verification test runner as a shell script to satisfy Project Constraints.

## Instructions
1. Inspect `/Users/sac/rocket-craft/ggen-validation-tests` and delete the following Python files:
   - `verify_all_rules.py`
   - `test_pyshacl_direct.py`
   - `test_query.py`
   - `test_shacl.py`
   
2. Implement `/Users/sac/rocket-craft/ggen-validation-tests/verify_all_rules.sh` to perform the exact same validation logic as `verify_all_rules.py` did:
   - Make a backup of `/Users/sac/rocket-craft/ggen-validation-tests/core.ttl` (e.g. to `core.ttl.bak`).
   - Run the baseline `ggen sync` command:
     `/Users/sac/.local/bin/ggen sync --manifest /Users/sac/rocket-craft/ggen-validation-tests/ggen.toml --validate-only true`
     Ensure it exits with 0. If it fails, report failure.
   - For each of the 11 validation check cases:
     1. Restore `core.ttl` from `core.ttl.bak`.
     2. Apply the mutation to `core.ttl` (you can use helper shell constructs or string tools like `sed` or temporary appends, but wait! The rule rule_global says: "Do NOT use sed, awk, or similar stream editors to modify source files." Oh! Wait, rule_global says: "Do NOT use sed, awk, or similar stream editors to modify source files. All file modifications must be performed using the replace or write_file tools to ensure precise, atomic, and verifiable edits." Wait, does this apply to the Worker tool calls or to script execution?
     Ah! "Do NOT use sed, awk, or similar stream editors to modify source files. All file modifications must be performed using the replace or write_file tools to ensure precise, atomic, and verifiable edits." This applies to how the agent modifies source files, NOT how a shell script running tests temporarily modifies files during testing. However, to be 100% compliant with the spirit of the project, the shell script can use simple, standard echo/printf appends, or prepare pre-authored mutated turtle files in a temporary directory and copy them over, rather than using `sed` or stream editing on target source code. For example, the shell script can just append custom text to `core.ttl` or replace the whole file using a pre-constructed mutated file. Let's suggest that the shell script uses simple appends or copies prepared files.
     Wait, actually, simple appends (e.g. `echo "triples" >> core.ttl`) and standard python-free shell replacements (or preparated snippet files) is much cleaner and avoids any `sed` stream editing concerns!
     3. Run the validation command:
        `OUTPUT=$(/Users/sac/.local/bin/ggen sync --manifest /Users/sac/rocket-craft/ggen-validation-tests/ggen.toml --validate-only true 2>&1 || true)`
     4. Check if the expected error string (e.g. "RuleA", "RuleB", etc.) exists in `$OUTPUT`. If not, report failure for that case.
     5. Restore `core.ttl` from `core.ttl.bak`.
   - Cleanup: delete `core.ttl.bak` and any temporary files.
   - Make sure `verify_all_rules.sh` is executable (`chmod +x`).
   - Run it to verify all tests pass.

3. Verify that `validate_ontology.sh` (at the project root) still runs successfully and passes.

4. Write your handoff report to `/Users/sac/rocket-craft/.agents/worker_remediate_python_scripts/handoff.md`.
