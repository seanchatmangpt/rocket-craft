# Ontology Research Findings for Rocket-Craft Documentation & Code Introspection

**Date**: 2026-06-18  
**Scope**: Public ontologies for documentation automation, API specification, and type system introspection.  
**Focus**: Applicability to rocket-craft's 7 workspaces, 44 crates, typestate patterns, and MCP integration.

---

## Executive Summary

Rocket-craft is uniquely positioned to pioneer **semantic documentation automation** by leveraging public ontologies (DCAT, PROV, SHACL, Schema.org) in combination with existing `unify-rs` RDF infrastructure. The research identified five core ontologies and a practical multi-phase integration strategy spanning 6–12 weeks of development.

**Key Finding**: No existing production tool converts Rust AST → RDF automatically. This is a significant ecosystem gap that rocket-craft can fill by building a syn-based code generator integrated with unify-rdf.

---

## Part 1: Public Ontologies – Ranked by Applicability

### 1. **PROV (Provenance Ontology)** ⭐⭐⭐⭐⭐
- **Namespace**: `http://www.w3.org/ns/prov#`
- **W3C Status**: Recommendation (2013)
- **Primary Use Case**: Dependency graphs, artifact lineage, crate interdependencies
- **Formats**: RDF/XML, Turtle, JSON-LD
- **Rocket-Craft Application**:
  - Model the 7 workspaces (tools, nexus-engine, blueprint-rs, unify-rs, ib4-mud, chicago-tdd-tools, asset-pipeline) as `prov:Agent` entities.
  - Track crate dependencies using `prov:wasDerivedFrom` (nexus-combat depends on nexus-types).
  - Express build provenance: "ShooterGame-Win64 was derived from rocket-sdk v1.2.3 + unify-mcp v2.0.0".
  - SPARQL query: `SELECT ?workspace ?crate WHERE { ?workspace prov:hadMember ?crate }` → auto-generate dependency matrix.
- **Maturity**: Production-grade; W3C flagship for linked data supply chains.
- **Effort to Adopt**: Low (1–2 weeks to map workspace/crate metadata).
- **Tooling**: Apache Jena, RDF4J support PROV natively.

### 2. **DCAT (Data Catalog Vocabulary)** ⭐⭐⭐⭐⭐
- **Namespace**: `http://www.w3.org/ns/dcat#`
- **W3C Status**: Recommendation (2014)
- **Primary Use Case**: Code documentation cataloging, crate inventory, API resource discovery
- **Formats**: RDF/XML, Turtle, JSON-LD
- **Rocket-Craft Application**:
  - Catalog each crate as `dcat:Dataset` with properties:
    - `dcat:title`: "nexus-combat: Combat state machine engine"
    - `dcat:description`: docstring from `Cargo.toml [package] description`
    - `dcat:distribution` → `dcat:Distribution` for published rustdoc (URL to docs.rs)
    - `dcat:keyword`: ["typestate", "combat", "game-engine"]
    - `dcat:contact` → maintainer email from workspace metadata.
  - Model nexus-engine's documented load order (nexus-types → nexus-combat, nexus-net, etc.) as distribution dependencies.
  - Output: DCAT RDF → auto-generate structured crate index in CLAUDE.md.
  - SPARQL: `SELECT ?crate ?desc WHERE { ?crate a dcat:Dataset . ?crate dcat:description ?desc }` → auto-populate "All Crates" section.
- **Maturity**: Production-grade; widely adopted for data portal indexing.
- **Effort to Adopt**: Low (1–2 weeks to map Cargo.toml → DCAT).
- **Tooling**: CKAN (open-source DCAT portal engine), Apache Jena.

### 3. **SHACL (Shapes Constraint Language)** ⭐⭐⭐⭐
- **Namespace**: `http://www.w3.org/ns/shacl#`
- **W3C Status**: Recommendation (2017)
- **Primary Use Case**: Type system constraints, trait bounds, phantom-type validation, state machine integrity
- **Formats**: Turtle, RDF/XML, JSON-LD
- **Rocket-Craft Application**:
  - Express the `Machine<L: Law, P>` typestate pattern as SHACL shape definitions:
    ```turtle
    @prefix rc: <http://rocket-craft.local/ontology#> .
    @prefix shacl: <http://www.w3.org/ns/shacl#> .

    rc:ConnectionShape a shacl:NodeShape ;
      shacl:targetClass rc:Connection ;
      shacl:property [
        shacl:path rc:state ;
        shacl:in (rc:Disconnected rc:Handshaking rc:Connected rc:Authenticated rc:InLobby rc:InMatch) ;
        shacl:minCount 1 ;
        shacl:maxCount 1
      ] .

    # Constraint: transition from Disconnected to InMatch is invalid (not adjacent)
    rc:DisconnectedToHandshakingOnly a shacl:SPARQLConstraint ;
      shacl:message "Disconnected can only transition to Handshaking" ;
      shacl:sparql """
        SELECT $this WHERE {
          $this rc:state rc:Disconnected .
          $this rc:nextState ?s .
          FILTER (?s NOT IN (rc:Handshaking))
        }
      """ .
    ```
  - Validate phantom-type constraints: `PhantomData<S>` as SHACL property shape with cardinality.
  - Continuous validation: SHACL engine rejects RDF models that violate state machine topology.
  - Goal: Make illegal state transitions unrepresentable in RDF (analogous to compile-time Rust type system).
- **Maturity**: Production-grade; Jena SHACL engine widely adopted.
- **Effort to Adopt**: Medium (2–4 weeks to model all typestate machines).
- **Tooling**: Apache Jena SHACL, Topquadrant TopBraid Composer.

### 4. **Schema.org + CodeRepository** ⭐⭐⭐⭐
- **Namespace**: `https://schema.org/CodeRepository`, `https://schema.org/SoftwareSourceCode`
- **Status**: Actively maintained by Schema.org community
- **Primary Use Case**: High-level codebase structure, module documentation, search engine discoverability
- **Formats**: JSON-LD (native), RDF/XML, Turtle
- **Rocket-Craft Application**:
  - Document each Rust workspace as `CodeRepository`:
    ```json-ld
    {
      "@context": "https://schema.org/",
      "@type": "CodeRepository",
      "name": "rocket-craft/nexus-engine",
      "url": "https://github.com/owner/rocket-craft/tree/main/nexus-engine",
      "description": "Gundam Nexus game engine: 10-crate Rust workspace with combat simulation, networking, economy, and ECS subsystems.",
      "codeRepository": "https://github.com/owner/rocket-craft",
      "programmingLanguage": "Rust",
      "license": "MIT",
      "hasPart": [
        {
          "@type": "SoftwareSourceCode",
          "name": "nexus-types",
          "description": "Phantom-typed units (Hp, Gold, Damage) and typestate markers.",
          "url": "https://docs.rs/nexus-types",
          "dependencies": ["no internal deps"]
        },
        { "@type": "SoftwareSourceCode", "name": "nexus-combat", ... }
      ]
    }
    ```
  - Embed Schema.org JSON-LD in `.well-known/coderepository.jsonld` for search engines.
  - Generate human-readable README from schema.org by templating.
  - Google Dataset Search indexes the metadata → rocket-craft appears in "code search" results.
- **Maturity**: Production-grade; integrates with Google, DuckDuckGo, Bing search indexing.
- **Effort to Adopt**: Low (1 week to generate JSON-LD from workspace metadata).
- **Tooling**: json-ld.org validator, Structured Data Linter.

### 5. **W3C ShEx (Shape Expressions)** ⭐⭐⭐
- **Namespace**: Direct syntax (no URI namespace)
- **Status**: W3C Community Group (active since 2014)
- **Primary Use Case**: API contract specification, method signatures, parameter types, return types
- **Formats**: ShEx textual syntax, RDF/XML via RDFa
- **Rocket-Craft Application**:
  - Express pub fn signatures in traits as ShEx shape definitions. Example:
    ```
    <http://rocket-craft.local/api/nexus-combat#resolve_parry> {
      rdf:type IriExclusive <http://rocket-craft.local/api#Method> ;
      <http://rocket-craft.local/api#parameter> @<ParryInput> ;
      <http://rocket-craft.local/api#returns> @<ParryResult> ;
      <http://rocket-craft.local/api#precondition> IriExclusive <http://rocket-craft.local/state#ParryingState> ;
      <http://rocket-craft.local/api#postcondition> IriExclusive <http://rocket-craft.local/state#ConnectedState>
    }
    ```
  - Validate that all trait implementations satisfy shape constraints.
  - Pair with OpenAPI/AsyncAPI bridging for MCP protocol alignment (MCP tools ↔ ShEx shapes).
- **Maturity**: Stable language; tooling: ShExJava, py-shex.
- **Effort to Adopt**: High (3–5 weeks to model all public APIs).
- **Tooling**: ShExJava (reference implementation), ShExSchema validator.

### 6. **DOAP (Description of a Project)** (Honorable Mention) ⭐⭐⭐
- **Namespace**: `http://usefulinc.com/ns/doap#`
- **Primary Use Case**: Lightweight project metadata (language, maintainer, repository, license)
- **Rocket-Craft Application**:
  - Quick export of project-manifest.json → DOAP RDF for integration tests, CI metadata.
  - Example: `doap:programming-language "Rust"`, `doap:license "MIT"`, `doap:homepage "https://…"`.
- **Maturity**: Stable but lighter-weight than DCAT/PROV.
- **Effort to Adopt**: Very Low (< 1 week).

---

## Part 2: Rust-to-RDF Tooling Gap Analysis

### Current State in Rocket-Craft
- `unify-rs` implements a **custom in-memory RDF triple store** (unify-rdf/src/) with hand-rolled `Term` enum and pattern-matching queries.
- **No external RDF library dependencies** (oxrdf, rustredis not in use).
- **No syn-based AST → RDF converter** exists anywhere in the Rust ecosystem (this is a critical gap).

### Recommended Tools (Maturity, Effort, ROI)

| Tool | Maturity | Input | Output | Effort | ROI | Use Case |
|------|----------|-------|--------|--------|-----|----------|
| **rustdoc-json** + `rustdoc-types` | Experimental (nightly, RFC 2963) | `cargo doc --json` | Structured JSON (types/docs) | Low (1–2 days) | High | Extract type signatures, doc comments from compiled crates |
| **oxrdf** | Stable (Oxigraph) | RDF data | RDF 1.1 compliant triples | Medium (1 week) | High | Replace unify-rs' custom store with standards-compliant impl |
| **horned-owl** | Stable | OWL ontologies | Rust structures | High (2–3 weeks) | Medium | Validate RDF against OWL schemas (SHACL shapes) |
| **syn + custom codegen** | N/A (build it) | Rust AST (via syn 2.0) | RDF triples | **Very High (4–6 weeks)** | **Very High** | Bridge unify-rs to auto-generate RDF from source AST |
| **cargo metadata** | Stable | Cargo.toml + lockfile | JSON dependency graph | Low (< 1 day) | High | Export workspace/crate dependency topology |

### Actionable Rust-to-RDF Path for Rocket-Craft

#### **Phase 1 (Low Effort, High ROI): Dependency Graph Export** — 1–2 weeks
1. Parse `cargo metadata --format-version 1` to extract nexus-engine's 10-crate topology.
2. Feed crate metadata (name, version, description, authors) to unify-rdf's TripleStore.
3. Emit PROV + DCAT RDF for all 44 crates.
4. **Output**: Turtle file with all workspace dependencies; ingest into SPARQL store.
5. **Validation**: Query "nexus-types has no dependencies" should return true; "nexus-combat depends on nexus-types" should return true.

#### **Phase 2 (Medium Effort, High ROI): Standards-Compliant RDF Storage** — 1–2 weeks
1. Swap unify-rs's custom `store.rs` for **oxrdf** (Apache-compatible, production-grade).
2. Gain automatic Turtle/N-Triples/N-Quads serialization (no custom code).
3. Enable compatibility with external SPARQL engines (Jena, RDF4J).
4. **Output**: unify-rdf can now export valid RDF 1.1 artifacts for external consumption.

#### **Phase 3 (High Effort, Very High ROI): Rust AST → RDF Code Generator** — 4–6 weeks
1. Build `nexus-codegen` sub-crate in tools/ workspace.
2. Use `syn` 2.0 + `quote!` to parse Rust source files (nexus-types/src/lib.rs, etc.).
3. Extract:
   - `pub struct`/`enum`/`trait` definitions → OWL classes
   - `PhantomData<S>` markers → SHACL property shapes (cardinality, domain, range)
   - `impl` blocks → SHACL property paths (method signatures)
   - Doc comments → `rdfs:comment` + `dcat:description` triples.
4. Emit RDF/TTL output to be ingested into unify-rdf.
5. **Output**: `./rocket codegen --format rdf` generates nexus-types.ttl from source.
6. **Validation**: SHACL engine validates that typestate state machines have no invalid transitions.

#### **Phase 4 (High Effort, Medium ROI): Documentation Generation** — 3–4 weeks
1. Model rocket-craft as an RDF ontology at `http://rocket-craft.local/ontology#`.
2. Define classes: `Workspace`, `Crate`, `Dependency`, `Type`, `Method`, `State`.
3. Use **WIDOCO** to auto-generate architecture diagrams + HTML from RDF.
4. **Output**: `./rocket docs --format html` generates indexed, searchable docs.

---

## Part 3: Rustdoc JSON & Documentation Extraction

### Status & Availability
- **RFC 2963** formally stabilized rustdoc JSON on nightly.
- **Stable Rust**: Requires `RUSTC_BOOTSTRAP=1` (non-standard but functional).
- **ETA for stable**: 2026–2027 (pending rustdoc JSON refinement).

### Practical Tools for Rocket-Craft
| Tool | Input | Output | Effort | Use Case |
|------|-------|--------|--------|----------|
| **rustdoc-md** | rustdoc JSON | Markdown | Low (< 1 day) | Auto-generate `.md` from compiled crate docs |
| **cargo-json-docs** | cargo doc --json | Programmatic API | Low (< 1 day) | Extract type signatures for downstream processing |
| **syn** + custom parser | Rust source files | RDF/AST | High (4–6 weeks) | Full AST introspection without compilation |

### Recommended Pipeline
```bash
# Step 1: Export rustdoc JSON (nightly)
RUSTC_BOOTSTRAP=1 cargo +nightly doc --no-deps --document-private-items --format json \
  --output target/rustdoc-json

# Step 2: Extract nexus-engine crates into structured metadata
cargo-json-docs extract target/rustdoc-json > nexus-engine-metadata.json

# Step 3: Convert to RDF (via custom tools or rustdoc-md)
./rocket codegen --from rustdoc-json --to rdf nexus-engine-metadata.json > nexus-engine.ttl

# Step 4: Generate docs from RDF
./rocket docs --from nexus-engine.ttl --format markdown > NEXUS_ENGINE_REFERENCE.md
```

**Effort to Integrate**: Low (1–2 weeks to add `./rocket docs` subcommand; moderate CI burden).

---

## Part 4: MCP & Semantic Documentation Integration

### Critical Finding
- **MCP Specification** (v2025-06-18) uses **JSON Schema only** for tool discovery.
- No built-in RDF/semantic web support in the protocol.

### Current unify-mcp Design
- Tools registered via `McpServer::new()` + `register_server_tools()` (JSON-RPC over stdio).
- Tool schema is purely structural: name, description, `inputSchema` (JSON Schema).
- MCP clients (including Claude) do NOT consume RDF.

### Opportunity: Augmented MCP Tooling (Future-Proof)
```rust
// Concept (Phase 4 enhancement)
#[rdf_tool]
#[describe("http://rocket-craft.local/tools/manifest/list")]
#[rdf_shape(rc:ManifestTool)]
pub fn rocket_manifest_list() -> Vec<Project> { ... }
```

**Rationale**: While Claude API (via Tool Use) ignores RDF annotations, other MCP clients or internal semantic reasoning could leverage them. This is a forward-looking investment.

**Estimated Effort**: Medium (2–3 weeks to add RDF metadata attributes to unify-mcp/src/).

**ROI**: Low for immediate use (Claude doesn't use it), but high for future semantic tool discovery, auto-generated API documentation, and interoperability with other MCP servers.

---

## Part 5: Documentation Generation from RDF

### Production-Ready Tools

| Tool | Input | Output | Best For |
|------|-------|--------|----------|
| **WIDOCO** (Widely Integrated Documentation Generator) | OWL/RDF ontologies | Static HTML + Markdown | Ontology documentation with auto-generated diagrams |
| **LODE** (Live OWL Documentation Environment) | OWL ontologies | HTML (online service) | Quick, interactive ontology visualization |
| **Pandoc** + custom Lua filters | Markdown + RDF metadata | HTML/PDF/Epub | Hybrid: human-written content + RDF-generated sections |
| **mkdocs-material** + plugins | YAML frontmatter + RDF | Static site | Searchable documentation portal (like docs.rs) |

### Recommended Approach for CLAUDE.md & Architecture Docs

1. **Define rocket-craft ontology** (OWL + SHACL):
   ```turtle
   @prefix rc: <http://rocket-craft.local/ontology#> .
   @prefix owl: <http://www.w3.org/2002/07/owl#> .

   rc:Workspace a owl:Class ; rdfs:comment "A Rust workspace containing multiple crates" .
   rc:Crate a owl:Class ; rdfs:comment "A Rust crate with dependencies" .
   rc:Dependency a owl:ObjectProperty ; rdfs:domain rc:Crate ; rdfs:range rc:Crate .
   ```

2. **Populate RDF** via Phase 1–3 tooling (dependency export, AST codegen).

3. **Use WIDOCO** to auto-generate architecture diagrams:
   - SPARQL: `SELECT ?workspace ?crate WHERE { ?workspace rc:contains ?crate }` → render dependency matrix.
   - Query results become mermaid/plantuml diagrams in generated HTML.

4. **Integrate with mkdocs**:
   ```yaml
   # mkdocs.yml
   plugins:
     - rdf-autodoc:
         ontology: rocket-craft.ttl
         sparql-queries:
           - name: "Crate Dependencies"
             query: |
               SELECT ?crate ?depends_on WHERE {
                 ?crate a rc:Crate . ?crate rc:depends ?depends_on
               }
   ```

5. **Output**: Auto-generated CLAUDE.md sections (Workspace Overview, Crate Dependency Graphs, Architecture Diagrams).

**Estimated Effort**: High (3–4 weeks to model monorepo as OWL, integrate WIDOCO/mkdocs pipeline).

---

## Part 6: Rust Monorepo Documentation Best Practices (Ecosystem Gap)

### Key Finding
Large Rust projects (tokio, serde, actix-web) have **no standard unified documentation aggregator**. Each crate publishes independently to docs.rs. Monorepos rely on manual CI scripts or `cargo doc --open`.

### Rocket-Craft's Unique Opportunity
With unify-rs (RDF, SPARQL) + rustdoc-json + custom tooling, rocket-craft can **establish a reference implementation** for semantic monorepo documentation:

**Vision**: 
```bash
./rocket docs                          # Auto-generate all docs from RDF source
./rocket docs --format html            # Generate searchable HTML portal
./rocket docs --format markdown        # Generate CLAUDE.md + README
./rocket docs --query "list-all-types" # SPARQL queries over docs
```

**ROI**: Position rocket-craft as an exemplar in the Rust ecosystem for documentation automation. Could attract OSS contributors and establish patterns for broader adoption.

---

## Summary: Actionable Implementation Roadmap

### **Phase 1 (Weeks 1–2): Dependency Graph Export** ⭐ Priority: HIGH
- **Tools**: cargo metadata, unify-rdf
- **Output**: All 44 crates + workspace topology as PROV/DCAT RDF
- **ROI**: Medium effort, high immediate value; enables all downstream phases
- **Deliverable**: nexus-engine-deps.ttl, blueprint-rs-deps.ttl, etc.

### **Phase 2 (Weeks 3–4): Standards-Compliant RDF Storage** ⭐ Priority: HIGH
- **Tools**: oxrdf
- **Output**: unify-rs uses production-grade RDF 1.1 storage
- **ROI**: Medium effort, high architectural clarity; enables external SPARQL tools
- **Deliverable**: unify-rdf refactoring, oxrdf integration tests

### **Phase 3 (Weeks 5–10): Rust AST → RDF Code Generator** ⭐ Priority: MEDIUM
- **Tools**: syn 2.0, rustdoc-json, custom codegen
- **Output**: `./rocket codegen` generates RDF from source + compiled metadata
- **ROI**: High effort, very high value; closes the Rust-to-RDF tooling gap
- **Deliverable**: nexus-codegen crate, type/trait RDF schemas, SHACL shape validation

### **Phase 4 (Weeks 11–14): Documentation Generation** ⭐ Priority: MEDIUM
- **Tools**: WIDOCO, mkdocs-material, SPARQL templating
- **Output**: Auto-generated CLAUDE.md, README, architecture diagrams
- **ROI**: High effort, very high value; positions rocket-craft as exemplar
- **Deliverable**: `./rocket docs`, searchable HTML portal, architecture mermaid diagrams

### **Phase 5 (Future): MCP Semantic Augmentation** ⭐ Priority: LOW
- **Tools**: Custom RDF attributes, MCP spec extensions
- **Output**: unify-mcp tools annotated with RDF metadata
- **ROI**: Low for immediate use, high for future interoperability
- **Deliverable**: Optional RDF context in MCP tool schema

---

## Gaps & Considerations

### Gaps in Public Tooling
1. **No production Rust AST → RDF converter** exists (must build in Phase 3).
2. **Rustdoc JSON** is experimental on stable (requires RUSTC_BOOTSTRAP=1).
3. **MCP protocol** doesn't natively support RDF discovery (Anthropic/MCP TC could adopt in v2026+).

### Technical Challenges
1. **Modeling Phantom Types in RDF**: PhantomData<S> has zero runtime representation. SHACL property shapes can express the constraint, but validation is static (compile-time, not runtime).
2. **Generic Type Parameters**: Rust's `Machine<L: Law, P>` generic syntax is hard to represent in OWL. ShEx is more expressive here.
3. **Lifetime Parameters**: RDF/OWL have no native concept of Rust lifetimes. Model them as temporal constraints (PROV temporal properties).

### Integration Points with Existing Rocket-Craft Systems
- **unify-rdf**: Replace custom store with oxrdf (Phase 2).
- **unify-mcp**: Augment tool registry with RDF metadata (Phase 5, optional).
- **rocket-cmd**: Add `./rocket codegen` and `./rocket docs` subcommands (Phases 1, 4).
- **CI (github/workflows)**: Integrate rustdoc JSON export and RDF validation into CI (Phase 3).

---

## Conclusion

Rocket-craft has a **rare, high-impact opportunity** to pioneer semantic monorepo documentation by:

1. Leveraging public ontologies (PROV, DCAT, SHACL, Schema.org) for standards-based RDF generation.
2. Closing the Rust-to-RDF tooling gap with a custom syn-based code generator.
3. Establishing a reusable reference implementation for the broader Rust ecosystem.

**Total Effort**: 10–14 weeks across 4 phases (Phase 5 optional).  
**Expected Outcome**: Fully automated, machine-readable, search-engine-discoverable documentation for all 44 crates; elimination of manual CLAUDE.md maintenance; auto-generation of architecture diagrams via SPARQL queries.

---

## References

- W3C PROV: https://www.w3.org/TR/prov-overview/
- W3C DCAT: https://www.w3.org/TR/vocab-dcat-2/
- W3C SHACL: https://www.w3.org/TR/shacl/
- W3C ShEx: https://shex.io/
- Schema.org CodeRepository: https://schema.org/CodeRepository
- WIDOCO: https://github.com/dgarijo/Widoco
- oxrdf (Oxigraph): https://github.com/oxigraph/oxrdf
- rustdoc JSON RFC 2963: https://rust-lang.github.io/rfcs/2963-doc-cfg.html
- MCP Specification: https://modelcontextprotocol.io/
