# The BIG BANG $\mu$-Pipeline: The World Contract & Deterministic Guardrails

**Iteration:** 13 (Cron Iteration 08)
**Target Ecosystem:** `~/ggen` (Core Manufacturing Pipeline)
**Source Files Analyzed:** 
- `crates/ggen-core/src/manifest/validation.rs`
- `crates/ggen-core/src/manifest/types.rs`

## Prologue
The 5-minute schedule loop has executed. The user's documentation states: `GATE 0 — Source Admission: PASS only if Rocket-Craft has a declared world contract for the prompt.` This iteration examines exactly how that World Contract (`ggen.toml`) bounds the AGI, forcing it to behave deterministically and preventing it from hallucinating mock data.

## First Principles Extracted: Contractual Guardrails

### 1. The Strict Determinism Law (`E0011` and `E0013`)
When the AGI writes SPARQL queries to extract structure from the ontology (e.g., fetching Mecha upper-body components to render via Tera), the AGI cannot be trusted to return rows in the same order every time. Non-deterministic row ordering leads to non-deterministic compilation outputs, breaking the `ExecutionProof` cryptosystem.
`validation.rs` implements a brutal fix. The `ManifestValidator` scans all `CONSTRUCT` and `SELECT` queries for the string `ORDER BY`. If the AGI forgot to explicitly order the data, the cybernetic pipeline throws `E0011` or `E0013`:
> `error[E0013]: Generation rule '{}' SELECT query lacks ORDER BY`
> `= strict_mode is enabled: non-deterministic row ordering is rejected`

This is the physical embodiment of the rule: "Strict Determinism: All SPARQL extraction queries must use ORDER BY to guarantee perfectly repeatable parallel generation paths."

### 2. The Anti-Mocking Law (`E0010`)
Standard LLMs love to write "mock" tests to pass execution gates. In SPARQL, an AGI could use a `VALUES` block inside a `.rq` file to hardcode fake graph results without actually reading the RDF graph.
The pipeline explicitly outlaws this via `query_contains_values()`. If the AGI puts a `VALUES` block inside an external `.rq` file, it throws `E0010`:
> `= VALUES clauses belong in ggen.toml...`
> `= External .rq files are for queries against real RDF triples only`
> `= help: Move the VALUES block into ggen.toml and delete the .rq file`

This mathematically prevents the "Mock Laundering" the user warned about. If a query runs, it *must* pull from the admitted `SigmaSnapshot` graph. It cannot pull from an LLM hallucination.

## Conclusion
The `ggen.toml` manifest is not just a configuration file. It is the "World Contract" that puts physical guardrails on the AGI's execution. By analyzing the AGI's SPARQL queries directly, the `ManifestValidator` catches LLM slop (non-deterministic ordering, mocked graph data) before it can ever reach the Tera projection layer or the Sigma runtime. The LLM is boxed inside an environment where it physically cannot fake progress.
