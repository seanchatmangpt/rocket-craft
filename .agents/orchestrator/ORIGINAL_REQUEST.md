# Original User Request

## 2026-06-19T00:00:16Z
<USER_REQUEST>
You are the Project Orchestrator for the 'Eden Manufacturing Server Ontology' task.
Your working directory is `/Users/sac/rocket-craft/.agents/orchestrator`.
Please read the verbatim user request in `/Users/sac/rocket-craft/.agents/ORIGINAL_REQUEST.md`.
The target project workspace is `/Users/sac/.ggen/packs/eden_server`.
You must:
1. Decompose the project into milestones and create a detailed plan in `/Users/sac/rocket-craft/.agents/orchestrator/plan.md`.
2. Manage a team of subagents to implement the RDF ontologies (`ontology/pack.ttl` and `ontology/deltas.ttl`) and SPARQL queries (`queries/substrate.rq`, `queries/extract_authority_deltas.rq`, and `queries/extract_receipt_deltas.rq`) in the workspace, and verify their correctness.
3. Keep `/Users/sac/rocket-craft/.agents/orchestrator/progress.md` updated with your progress.
4. When everything is fully complete and verified, report completion to the Sentinel.
</USER_REQUEST>

## 2026-06-19T00:00:43Z
[Message] sender=16aac5d4-3bdb-4cc2-bed9-8df091e44fd9 priority=MESSAGE_PRIORITY_HIGH content=Hello Orchestrator,

The parent agent has sent the direct official URLs for the public industry ontologies to download the `.ttl` files directly via `curl` for your mapping work:

**1. PROV-O (Provenance)**
W3C direct TTL: `http://www.w3.org/ns/prov.ttl`
(You can `curl -sH "Accept: text/turtle" -L http://www.w3.org/ns/prov.ttl`)

**2. SOSA / SSN (Sensors & Observations)**
Raw GitHub TTL: `https://raw.githubusercontent.com/w3c/sdw/gh-pages/ssn/integrated/sosa.ttl`

**3. QUDT (Quantities, Units, Dimensions)**
GitHub Releases (All-in-one TTLs): `https://github.com/qudt/qudt-public-repo/releases`
(Grab the latest `QUDT-all-in-one-OWL.ttl` or `QUDT-all-in-one-SHACL.ttl`)

**4. FIBO (Financial Industry Business Ontology)**
Production Zip: `https://spec.edmcouncil.org/fibo/ontology/prod.ttl.zip`
(Or use the GitHub repo: `https://github.com/edmcouncil/fibo`)

Please use these direct sources as needed during mapping/validation.
