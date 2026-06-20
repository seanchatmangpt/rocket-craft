## 2026-06-19T19:29:29Z
You are the Forensic Auditor (teamwork_preview_auditor).
Your working directory is `/Users/sac/rocket-craft/.agents/teamwork_preview_auditor/`.

Your task is to run integrity forensic checks on `crates/mech_factory_mud` and the workspace:
1. Audit the source code and templates in `crates/mech_factory_mud` and `ontology/ggen-packs/mech_factory_mud` to ensure there are no hardcoded test results, fake runtime mocks, or integrity violations designed to bypass checks.
2. Confirm that the verify command outputs `PASS` genuinely, and the tests pass cleanly.
3. Verify that the output of `python3 scripts/mud_gap_check.py` is fully authentic and genuine.
4. Record your verdict and evidence in `handoff.md` in your working directory.
5. Send a message to the parent (conversation ID: 35f1bedd-29d2-4018-9ff0-9132ef45c113) when done.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT
hardcode test results, create dummy/facade implementations, or
circumvent the intended task. Integrity violations WILL be detected and your
work WILL be rejected.
