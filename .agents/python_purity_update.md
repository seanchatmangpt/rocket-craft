# AGENTS.md Addendum — Python Control Surface Purity

## NO EPHEMERAL PYTHON CONTROL SURFACES

Do not create ad hoc Python scripts merely to append, patch, or manually mutate files.
Use direct patch/edit tools for file edits.
Use Python only when the script is a declared durable tool with tests, receipts, and a named role.

## PYTHON_CONTROL_SURFACE_PURITY Gate

Rules:
* Python may verify.
* Python may compile.
* Python may extract.
* Python may transform graph-selected rows.
* Python may run deterministic reports.

Forbidden in Python:
* Python may not become hidden source law.
* Python may not hand-write morphology.
* Python may not silently mutate policy.
* Python may not patch generated artifacts as source.
* Python may not exist as a one-off helper left behind.
