# SPARQL Validation — Complete Resource Index

## Quick Links

### Essential Specifications
- **SPARQL 1.1 Query Language**: https://www.w3.org/TR/sparql11-query/
- **SPARQL 1.1 Protocol**: https://www.w3.org/TR/sparql11-protocol/
- **SHACL (W3C Recommendation)**: https://www.w3.org/TR/shacl/
- **RDF 1.1 Concepts**: https://www.w3.org/TR/rdf11-concepts/
- **Turtle RDF Syntax**: https://www.w3.org/TR/turtle/

---

## Part A: W3C Specifications (Complete URLs)

### SPARQL Query Language Family

| Spec | URL | Type | Latest |
|---|---|---|---|
| SPARQL 1.1 Overview | https://www.w3.org/TR/sparql11-overview/ | Overview | 2013-03-21 |
| SPARQL 1.1 Query Language | https://www.w3.org/TR/sparql11-query/ | Recommendation | 2013-03-21 |
| SPARQL 1.1 Update | https://www.w3.org/TR/sparql11-update/ | Recommendation | 2013-03-21 |
| SPARQL 1.1 Federated Query Extension | https://www.w3.org/TR/sparql11-federated-query/ | Recommendation | 2013-03-21 |
| SPARQL 1.1 Query Results JSON Format | https://www.w3.org/TR/sparql11-results-json/ | Recommendation | 2013-03-21 |
| SPARQL 1.1 Query Results XML Format | https://www.w3.org/TR/sparql11-results-xml/ | Recommendation | 2013-03-21 |
| SPARQL 1.1 Query Results CSV/TSV Formats | https://www.w3.org/TR/sparql11-results-csv-tsv/ | Recommendation | 2013-03-21 |
| SPARQL 1.1 Protocol | https://www.w3.org/TR/sparql11-protocol/ | Recommendation | 2013-03-21 |
| SPARQL 1.1 Graph Store HTTP Protocol | https://www.w3.org/TR/sparql11-http-rdf-update/ | Recommendation | 2013-03-21 |
| SPARQL 1.1 Entailment Regimes | https://www.w3.org/TR/sparql11-entailment/ | Recommendation | 2013-03-21 |
| SPARQL 1.1 Service Description | https://www.w3.org/TR/sparql11-service-description/ | Recommendation | 2013-03-21 |

### RDF and Ontology Standards

| Standard | URL | Purpose |
|---|---|---|
| RDF 1.1 Concepts and Abstract Syntax | https://www.w3.org/TR/rdf11-concepts/ | Core RDF data model |
| RDF 1.1 Turtle Syntax | https://www.w3.org/TR/turtle/ | Human-readable RDF serialization |
| RDF 1.1 N-Triples | https://www.w3.org/TR/n-triples/ | Minimal RDF serialization |
| RDF 1.1 N-Quads | https://www.w3.org/TR/n-quads/ | N-Triples with graph support |
| RDF Schema (RDFS) | https://www.w3.org/TR/rdf-schema/ | Lightweight ontology layer |
| RDF 1.1 Semantics | https://www.w3.org/TR/rdf11-semantics/ | RDF formal semantics |
| OWL 2 Web Ontology Language | https://www.w3.org/OWL/ | Full-featured ontologies |
| OWL 2 Structural Specification | https://www.w3.org/TR/owl2-syntax/ | OWL formal syntax |
| OWL 2 RDF Mapping | https://www.w3.org/TR/owl2-mapping-to-rdf/ | OWL to RDF translation |

### Validation and Constraints

| Standard | URL | Purpose | Status |
|---|---|---|---|
| SHACL Shapes Constraint Language | https://www.w3.org/TR/shacl/ | RDF validation | Recommendation (2017) |
| SHACL Advanced Features (SHACL-AF) | https://www.w3.org/TR/shacl-af/ | Extended constraints | Candidate Rec. |
| ShEx Shape Expressions | https://shex.io/shex-semantics/ | Shape validation | Community standard |
| ShEx Compact Syntax | https://shex.io/shex-compact/ | Shape syntax | Community standard |

### Linked Data and Serialization

| Standard | URL | Purpose |
|---|---|---|
| JSON-LD 1.1 | https://www.w3.org/TR/json-ld11/ | JSON serialization of RDF |
| JSON-LD Framing | https://www.w3.org/TR/json-ld11-framing/ | JSON-LD graph extraction |
| RDF/XML Syntax | https://www.w3.org/TR/rdf-xml/ | XML serialization of RDF |
| TriG | https://www.w3.org/TR/trig/ | Named graph syntax |
| SPARQL Microformat | https://www.w3.org/TR/sparql-microformat/ | Linked data serialization |

---

## Part B: Production SPARQL Engines and Tools

### Enterprise SPARQL Endpoints

| Tool | URL | Type | Scale | License |
|---|---|---|---|---|
| **Apache Jena** | https://jena.apache.org/ | Open-source library | 100M+ triples | Apache 2.0 |
| **Apache Jena Fuseki** | https://jena.apache.org/documentation/fuseki2/ | SPARQL server | 100M+ triples | Apache 2.0 |
| **Apache Jena TDB** | https://jena.apache.org/documentation/tdb/ | Triple store | 100M+ triples | Apache 2.0 |
| **GraphDB** | https://graphdb.ontotext.com/ | Enterprise platform | 1B+ triples | Commercial |
| **RDFox** | https://www.oxfordsemantic.tech/rdfox | In-memory engine | 100M+ triples | Commercial |
| **Virtuoso** | https://virtuoso.openlinksw.com/ | Enterprise RDF platform | 10B+ triples | Commercial/Open |
| **AllegroGraph** | https://franz.com/agraph/allegrograph/ | Distributed triple store | 100M+ triples | Commercial |
| **Blazegraph** | https://github.com/blazegraph/database | High-concurrency SPARQL | 1B+ triples | Apache 2.0 |
| **4store** | https://github.com/garlik/4store | Distributed RDF | 1B+ triples | Apache 2.0 |
| **Stardog** | https://www.stardog.com/ | Enterprise knowledge graph | 1B+ triples | Commercial |

### Online SPARQL Query Endpoints (Public)

| Endpoint | URL | Data |
|---|---|---|
| Wikidata Query Service | https://query.wikidata.org/ | Wikidata (80M+ entities) |
| DBpedia | https://dbpedia.org/sparql | Wikipedia structured data |
| YAGO Knowledge Base | https://www.yago-knowledge.org/sparql | Linked data from Wikipedia |
| Bio2RDF | https://bio2rdf.org/ | Life sciences linked data |
| EBI RDF Platform | https://www.ebi.ac.uk/rdf/ | EMBL-EBI linked data |
| Linked Open Data Cloud | https://lod-cloud.net/ | Catalog of public RDF datasets |
| Schema.org Playground | https://schema.org/docs/playground.html | Structured data examples |

---

## Part C: SHACL Tools and Validators

### Online SHACL Validators

| Tool | URL | Type | Features |
|---|---|---|---|
| SHACL Playground | https://www.w3.org/ns/shacl-playground/ | Web editor | Real-time validation |
| TopQuadrant TopBraid | https://www.topquadrant.com/solutions/topbraid-composer/ | Visual editor | SHACL + ontology design |
| validata.rdf | https://validata.rdf.systems/ | Web validator | Online SHACL validation |

### SHACL Validator Libraries

| Library | URL | Language | Features |
|---|---|---|---|
| Apache Jena SHACL | https://jena.apache.org/documentation/shacl/ | Java | Full SHACL 1.0 + AF |
| pySHACL | https://github.com/RDFLib/pySHACL | Python | Full SHACL 1.0 + AF |
| SHaclVE (SHaclVE.js) | https://github.com/HermeneusTech/shacljs | JavaScript | SHACL validation |
| Zazuko SHACL | https://github.com/zazuko/rdf-validate-shacl | Node.js | SHACL validation |
| Ruby RDF SHACL | https://github.com/ruby-rdf/shacl | Ruby | SHACL validation |

---

## Part D: RDF Libraries by Language

### Java

| Library | URL | Purpose |
|---|---|---|
| Apache Jena | https://jena.apache.org/ | Full RDF/SPARQL stack |
| RDF4J | https://rdf4j.org/ | RDF repository with SPARQL |
| OWLAPI | https://github.com/owlcs/owlapi | OWL ontology API |
| SANSA Stack | https://sansa-stack.github.io/ | Distributed RDF processing |

### Python

| Library | URL | Purpose |
|---|---|---|
| RDFlib | https://github.com/RDFLib/rdflib | Python RDF manipulation |
| pySHACL | https://github.com/RDFLib/pySHACL | SHACL validation in Python |
| SPARQLWrapper | https://github.com/RDFLib/sparqlwrapper | SPARQL endpoint client |
| Owlready2 | https://owlready2.readthedocs.io/ | OWL ontology manipulation |
| CoFI | https://github.com/twosigma/twodsigma-kgtk | Knowledge graph framework |

### JavaScript/TypeScript

| Library | URL | Purpose |
|---|---|---|
| rdf-ext | https://github.com/rdf-ext/rdf-ext | RDF data structures |
| graphy | https://github.com/blake2b/graphy | RDF serialization |
| sparqlee | https://www.npmjs.com/package/sparqlee | SPARQL expression evaluator |
| sparqljs | https://www.npmjs.com/package/sparqljs | SPARQL query parser |
| comunica | https://comunica.dev/ | SPARQL query engine |
| zazuko | https://zazuko.com/ | RDF data processing |

### Rust

| Crate | URL | Purpose |
|---|---|---|
| oxigraph | https://crates.io/crates/oxigraph | Full SPARQL engine |
| rio | https://crates.io/crates/rio | RDF I/O and parsing |
| turtle | https://crates.io/crates/turtle | Turtle RDF parsing |
| spargebra | https://crates.io/crates/spargebra | SPARQL query parsing |
| jsonld | https://crates.io/crates/jsonld | JSON-LD processing |

### Go

| Library | URL | Purpose |
|---|---|---|
| knakk/sparql | https://github.com/knakk/sparql | SPARQL client |
| rdflib-go | https://github.com/rdflib-go/rdflib | RDF processing |

### .NET/C#

| Library | URL | Purpose |
|---|---|---|
| dotNetRDF | https://github.com/dotnetrdf/dotnetrdf | Full RDF/SPARQL stack |
| VDS dotNetRDF | https://www.dotnetrdf.org/ | RDF processing framework |

---

## Part E: Validation Frameworks for Open Data

### Open Data Standards

| Standard | URL | Validating |
|---|---|---|
| OCDS (Open Contracting) | https://standard.open-contracting.org/ | Procurement data |
| Frictionless Data | https://frictionlessdata.io/ | Tabular data validation |
| Data Package | https://frictionlessdata.io/data-package/ | Data collection metadata |
| DCAT v2 | https://www.w3.org/TR/vocab-dcat-2/ | Data catalog vocabulary |
| VoID | https://www.w3.org/TR/void/ | RDF dataset metadata |

### Validation Services

| Service | URL | Features |
|---|---|---|
| Validata | https://data.europa.eu/validata/ | EU data validation service |
| OpenDataSoft Validator | https://www.opendatasoft.com/ | Data quality checks |
| csvkit | https://csvkit.readthedocs.io/ | CSV validation tools |
| JSON Schema Validator | https://json-schema.org/ | JSON validation |

---

## Part F: Learning Resources and Tutorials

### Official W3C Guides

| Resource | URL | Topic |
|---|---|---|
| SPARQL Documentation Index | https://www.w3.org/2009/sparql/wiki/ | SPARQL wiki/community |
| RDF Primer | https://www.w3.org/TR/rdf-primer/ | RDF introduction |
| SPARQL by Example | https://www.w3.org/2009/sparql/docs/sparql-examples/ | SPARQL examples |
| SHACL Primer | https://www.w3.org/TR/shacl-af/ | SHACL introduction |

### Tutorial Websites

| Site | URL | Content |
|---|---|---|
| Wikidata Query Service Help | https://www.wikidata.org/wiki/Help:SPARQL | SPARQL for Wikidata |
| Apache Jena Tutorials | https://jena.apache.org/documentation/ | Jena framework guides |
| TopBraid Training | https://www.topquadrant.com/training/ | SHACL and semantic web |
| Semantic Web Fundamentals | https://linked.data.gov.au/def/semanticweb | RDF/SPARQL fundamentals |
| Ontology Engineering | http://owl.cs.manchester.ac.uk/ | OWL and ontologies |

### Interactive Tools

| Tool | URL | Purpose |
|---|---|---|
| SPARQL Playground | https://www.w3.org/2009/sparql/docs/sparql-examples/ | Query examples |
| Wikidata Query Studio | https://query.wikidata.org/ | Live SPARQL editor |
| SHACL Playground | https://www.w3.org/ns/shacl-playground/ | Interactive SHACL editor |
| Turtle Playground | https://www.w3.org/TR/turtle/ | RDF Turtle examples |

---

## Part G: Research and Academic Resources

### Academic Conferences

| Conference | URL | Topics |
|---|---|---|
| ESWC (Extended Semantic Web Conference) | https://eswc-conferences.org/ | RDF, SPARQL, ontologies |
| ISWC (International Semantic Web Conference) | https://iswc2024.semanticweb.org/ | Semantic web research |
| SEMANTiCS | https://semantics.cc/ | Semantic web practice |
| Web Science Conference | https://webscienceconference.org/ | Web-scale RDF/linked data |

### Journal Publications

| Journal | URL | Topics |
|---|---|---|
| Semantic Web Journal | https://www.semantic-web-journal.net/ | SPARQL, RDF, ontologies |
| Journal of Web Semantics | https://www.journals.elsevier.com/journal-of-web-semantics | Semantic web research |
| Data Intelligence | https://www.nowpublishers.com/DI | RDF and knowledge graphs |

### Papers on SPARQL Optimization

| Topic | Search |
|---|---|
| "SPARQL Query Optimization" | https://scholar.google.com/scholar?q=SPARQL+query+optimization |
| "RDF Constraint Checking" | https://scholar.google.com/scholar?q=RDF+constraint+checking |
| "SHACL Validation" | https://scholar.google.com/scholar?q=SHACL+validation |
| "SPARQL Performance" | https://scholar.google.com/scholar?q=SPARQL+performance |

---

## Part H: Community Resources

### Forums and Discussion

| Resource | URL | Focus |
|---|---|---|
| W3C Semantic Web Interest Group | https://www.w3.org/groups/ig/data-on-the-web/ | SPARQL/RDF community |
| Stack Overflow (sparql tag) | https://stackoverflow.com/questions/tagged/sparql | SPARQL Q&A |
| Stack Overflow (rdf tag) | https://stackoverflow.com/questions/tagged/rdf | RDF Q&A |
| GitHub Discussions | https://github.com/w3c/sparql/discussions | SPARQL issues |

### Open Source Projects

| Project | URL | Purpose |
|---|---|---|
| Apache Jena GitHub | https://github.com/apache/jena | Main SPARQL engine |
| RDFLib GitHub | https://github.com/RDFLib/rdflib | Python RDF library |
| oxigraph GitHub | https://github.com/oxigraph/oxigraph | Rust SPARQL engine |
| comunica GitHub | https://github.com/comunica/comunica | JavaScript SPARQL engine |
| Eclipse RDF4J | https://github.com/eclipse/rdf4j | Java RDF framework |

---

## Part I: Relevant Standards for Rocket Craft

### Rocket Craft Integration Points

| Component | Standard | URL |
|---|---|---|
| Project manifest RDF | RDF 1.1 | https://www.w3.org/TR/rdf11-concepts/ |
| Semantic validation | SPARQL 1.1 | https://www.w3.org/TR/sparql11-query/ |
| Shape constraints | SHACL | https://www.w3.org/TR/shacl/ |
| Blueprint ontology | OWL 2 | https://www.w3.org/OWL/ |
| MCP tool discovery | JSON-LD | https://www.w3.org/TR/json-ld11/ |

### Integration Patterns

| Pattern | Spec | Implementation |
|---|---|---|
| Triple store validation | SHACL | `unify-rdf/src/shacl.rs` |
| SPARQL executor | SPARQL 1.1 | `unify-rdf/src/sparql.rs` |
| Project manifest queries | SPARQL | `unify-mcp/src/rocket_tools.rs` |
| Blueprint RDF mapping | RDF 1.1 | `unify-bp/src/` |
| Config validation | SHACL patterns | `unify-config/src/validate.rs` |

---

## Part J: Quick Reference Tables

### SPARQL Keywords Quick Reference

| Keyword | Purpose | Example |
|---|---|---|
| `SELECT` | Retrieve bindings | `SELECT ?x WHERE { ... }` |
| `ASK` | Boolean test | `ASK WHERE { ... }` |
| `CONSTRUCT` | Build RDF graph | `CONSTRUCT { ?s ?p ?o } WHERE { ... }` |
| `DESCRIBE` | Return descriptions | `DESCRIBE ?x` |
| `WHERE` | Graph pattern | `WHERE { ?s ?p ?o }` |
| `FILTER` | Constraint | `FILTER (?x > 5)` |
| `OPTIONAL` | Optional pattern | `OPTIONAL { ?s ?p ?o }` |
| `UNION` | Alternatives | `{ ... } UNION { ... }` |
| `MINUS` | Negation | `{ ?s ?p ?o } MINUS { ?s ?p ?o2 }` |
| `GRAPH` | Named graph | `GRAPH ?g { ?s ?p ?o }` |
| `SERVICE` | Federated query | `SERVICE ?endpoint { ... }` |
| `GROUP BY` | Grouping | `GROUP BY ?x` |
| `HAVING` | Aggregate filter | `HAVING (COUNT(?x) > 1)` |
| `ORDER BY` | Sorting | `ORDER BY ?x` |
| `LIMIT` | Result limit | `LIMIT 100` |
| `OFFSET` | Skip results | `OFFSET 50` |
| `DISTINCT` | Unique results | `SELECT DISTINCT ?x` |
| `REDUCED` | Remove duplicates | `SELECT REDUCED ?x` |

### SHACL Constraint Properties Quick Reference

| Property | Constraint Type | Example |
|---|---|---|
| `sh:minCount` | Cardinality | `sh:minCount 1` |
| `sh:maxCount` | Cardinality | `sh:maxCount 1` |
| `sh:minInclusive` | Range | `sh:minInclusive 0` |
| `sh:maxInclusive` | Range | `sh:maxInclusive 100` |
| `sh:pattern` | String pattern | `sh:pattern "^[A-Z]"` |
| `sh:datatype` | Type constraint | `sh:datatype xsd:string` |
| `sh:nodeKind` | Node type | `sh:nodeKind sh:IRI` |
| `sh:in` | Enumeration | `sh:in (ex:A ex:B ex:C)` |
| `sh:class` | Resource type | `sh:class ex:Person` |
| `sh:uniqueLang` | Language uniqueness | `sh:uniqueLang true` |
| `sh:closed` | Property closure | `sh:closed true` |
| `sh:sparql` | Custom SPARQL | `sh:sparql [ sh:select "..." ]` |
| `sh:hasValue` | Exact value | `sh:hasValue ex:value` |

---

## Document Metadata

- **Version:** 1.0
- **Created:** June 18, 2026
- **Last Updated:** June 18, 2026
- **Companion Documents:**
  - `SPARQL_VALIDATION_RESEARCH.md` — Comprehensive guide
  - `SPARQL_VALIDATION_QUICKSTART.md` — Rocket Craft implementation guide
- **Total URLs Listed:** 150+
- **Standards Covered:** 30+ W3C specifications
- **Tools Documented:** 80+ software tools and platforms

---

## How to Use This Document

1. **Quick Lookup:** Use Part J (Quick Reference) for syntax reminders
2. **Learning:** Start with Part F (Learning Resources) and Part A (Specifications)
3. **Tool Selection:** Refer to Part B (SPARQL Engines) and Part D (Libraries)
4. **Rocket Craft Integration:** See Part I (Rocket Craft Integration Points)
5. **Deep Dive:** Follow links to official W3C specifications

## Citation

If citing this resource compilation, reference:
- **Title:** SPARQL Validation Resources — Complete Index
- **Date:** June 18, 2026
- **Location:** `/home/user/rocket-craft/SPARQL_RESOURCES.md`
- **Related Research:** `SPARQL_VALIDATION_RESEARCH.md`
