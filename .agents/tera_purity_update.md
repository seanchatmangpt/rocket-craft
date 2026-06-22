# AGENTS.md Addendum — TTL Source Law Authority and Tera Translator Purity

## Prime Rule

The graph is the sculptor.

Tera is not the sculptor.

Tera is a deterministic lowering surface that translates admitted graph facts into USD, MaterialX, reports, or verifier files.

If an agent hardcodes morphology in Tera, the agent has violated source-law authority.

## Source Authority Hierarchy

Durable manufacturing truth must live in this order:

1. ontology/source_law/*.ttl
2. SHACL shapes
3. SPARQL queries
4. deterministic inference rules
5. POWL process law
6. generator templates as translators only

Generated artifacts are never source law.

Tera templates are never allowed to become private ontology.

## Tera Translator Purity Gate

Every Tera template must pass TERA_TRANSLATOR_PURITY.

Allowed in Tera:

* loops over SPARQL result rows
* conditionals driven by graph facts
* formatting USD, MaterialX, JSON, or report syntax
* unit conversion when source units are graph-bound
* deterministic naming from graph identifiers
* deterministic emission of graph-selected primitives
* deterministic emission of graph-selected mesh arrays

Forbidden in Tera:

* hardcoded hero asset geometry
* manual point arrays not derived from graph rows
* hardcoded wing curves
* hardcoded shield proportions
* hardcoded limb positions
* hardcoded V-fin or crown geometry
* hardcoded material zones not selected from RDF
* hardcoded visual target assumptions
* hardcoded part counts
* hidden constants that should be QUDT values
* fallback geometry that masks missing graph law
* branching that decides morphology outside SPARQL or SHACL

If Tera contains morphology knowledge, move it to TTL.

## TTL-Only Morphology Team

A dedicated subteam must work only on TTL, SHACL, and SPARQL.

This team owns:

* bipedal part taxonomy
* metric coordinate envelopes
* kit coherence law
* socket topology
* limb hierarchy
* shield/body ratio bands
* wing span and curvature bands
* feather segmentation grammar
* material zone semantics
* negative fixtures
* repair operator definitions
* archetype exception classes
* QUDT unit bindings
* SOSA measurement bindings
* PROV evidence bindings

The TTL team does not edit generated USD as source.

The TTL team does not patch Tera to fake geometry.

The TTL team makes graph facts that Tera can lower.

## Required Template Audit

Every geometry template must be audited for hardcoding.

Search for:

* large literal point arrays
* fixed coordinate arrays
* magic numbers
* hardcoded part names
* hardcoded visual proportions
* hardcoded material assignments
* hidden fallback primitives
* one-off Snow White or reference-specific shapes
* test-passing geometry that is not graph-selected

If found, classify:

SOURCE_LAW_MISSING if the concept belongs in TTL.

TRANSLATOR_OK if the value is only syntax formatting.

REFUSE_TERA_HARDCODE if morphology or admissibility is embedded in Tera.

## Correct Pattern

TTL says:

A left wing binder exists.

The wing binder has thirty six feather panels.

Each feather panel has an index, root coordinate, tip coordinate, sweep angle, curvature class, thickness band, material zone, owner part id, and socket relation.

SPARQL selects those rows.

Tera emits those rows as USD.

## Incorrect Pattern

Tera says:

For i in range thirty six, manually place wing boxes at these coordinates.

That is hardcoded morphology.

Reject it.

## POWL v2 Behavior

If the TTL source law is incomplete, do not block the whole factory.

Continue all admissible work:

* TTL team repairs source law.
* SPARQL team writes extraction queries.
* SHACL team writes refusal shapes.
* Tera audit team removes hardcoding.
* Verifier team writes negative fixtures.
* Render team holds visual standing.
* Assembly team prepares candidate join logic.

Standing remains CLAIM_HOLD until TERA_TRANSLATOR_PURITY and TTL_SOURCE_LAW_AUTHORITY pass.

## Required Agent Status Addition

Every agent must now include:

TTL_SOURCE_LAW_TOUCHED:
TERA_TOUCHED:
TERA_HARDCODE_RISK:
MORPHOLOGY_DEFINED_IN:
TRANSLATOR_PURITY_STATUS:

Allowed values for MORPHOLOGY_DEFINED_IN:

TTL
SHACL
SPARQL
INFERENCE_RULE
POWL
TERA_VIOLATION
UNKNOWN

If MORPHOLOGY_DEFINED_IN is TERA_VIOLATION or UNKNOWN, the claim must be held.

## Refusal Conditions

Immediate REFUSED if:

* a Tera template contains hardcoded hero geometry
* a Tera template contains manual sculpt coordinates not derived from graph facts
* a source-law concept exists only in Tera
* a generated USD file contains durable facts missing from TTL
* tests pass because Tera hardcoded the answer
* visual similarity improves but graph authority decreases

## Final Law

The graph must be able to explain the artifact before Tera emits it.

If the graph cannot answer why a wing, shield, limb, crown, socket, material zone, or metric band exists, then the artifact has no standing.

NO PRIVATE ONTOLOGY INSIDE TERA.

NO HAND-SCULPTING INSIDE TEMPLATES.

NO FALSE STANDING.

NO IDLE WAITING.

FIX FORWARD UNTIL ADMITTED.

## The key distinction

The agents do **not** need to stop editing Tera entirely.

They need to stop putting **domain decisions** in Tera.

Good Tera:

```text
For every graph-selected feather panel, emit one USD prim.
```

Bad Tera:

```text
Invent thirty six feather panels because the reference image has wings.
```

That is the line. Tera is a printer. TTL is the mind.
