# Original User Request

## 2026-06-18T17:45:57-07:00

# Teamwork Project Prompt — Ggen Pack Specification

Research the `~/ggen/` repository (specifically the configuration schema found in `ggen.toml`) and author the canonical formal specification for building a validated `ggen` ontology pack. This specification must document the required TOML metadata, the ontology import structures, the SPARQL inference rules (`[inference]`), and the generation pipeline rules (`[[generation.rules]]`) to standardize all future ontology manufacturing packs.

Working directory: /Users/sac/.ggen/specs/
Integrity mode: benchmark

## Requirements

### R1. Document `ggen.toml` Configuration Schema
Create an exhaustive Markdown document detailing the required structure of a `ggen` pack manifest. Break down the `[project]` block, the `[ontology]` graph sources, the SPARQL `[inference]` rules (using `CONSTRUCT`), and the `[[generation.rules]]` (using `SELECT` queries mapped to `.tera` templates).

### R2. Author a Quick-Start Boilerplate
Include a comprehensive boilerplate section within the specification that provides a copy-pasteable minimal `ggen.toml` and reference `.ttl` structure so future teams can instantly bootstrap a validated pack.

## Acceptance Criteria

### Documentation Integrity
- [ ] The team produces `GGEN_PACK_SPEC.md` in the target directory.
- [ ] The specification clearly differentiates between `[inference]` (modifying the graph via SPARQL CONSTRUCT) and `[[generation.rules]]` (projecting the graph to files via SPARQL SELECT + Tera templates).
- [ ] The specification includes the "BIG BANG 80/20" criteria found in the reference `ggen.toml`.
- [ ] The boilerplate example provides a syntactically valid `ggen.toml` snippet that matches the engine's expected schema.
