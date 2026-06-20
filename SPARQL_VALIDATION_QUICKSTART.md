# SPARQL Validation Quick Start Guide — Rocket Craft unify-rs

## Overview

This guide shows how to implement and use SPARQL-based validation in the Rocket Craft `unify-rs` workspace, specifically extending the RDF triple store with advanced validation patterns.

## Current Implementation Status

**Location:** `/home/user/rocket-craft/unify-rs/unify-rdf/src/`

### Existing Components

```
unify-rdf/
├── triple.rs         # RDF Term, Triple definitions
├── store.rs          # TripleStore (in-memory RDF graph)
├── sparql.rs         # Basic SPARQL executor (SELECT/ASK patterns)
├── shacl.rs          # SHACL shapes and constraint validation
├── pipeline.rs       # 5-stage ontology pipeline (μ₁–μ₅)
├── manifest.rs       # Project manifest parsing
└── lib.rs            # Public API exports
```

### Current Capabilities

- **TripleStore**: Basic pattern queries (subject/predicate/object matching)
- **SPARQL**: Limited SELECT/ASK support for `?s ?p ?o` patterns
- **SHACL**: MinCount, MaxCount, Datatype, NodeKind constraints
- **Pipeline**: Turtle RDF loading and type extraction

## Recommended Enhancement Path

### Phase 1: Extend SPARQL Executor (Immediate)

Implement support for:
1. FILTER expressions
2. OPTIONAL patterns
3. UNION queries
4. NOT EXISTS / EXISTS

**File:** `/home/user/rocket-craft/unify-rs/unify-rdf/src/sparql.rs`

```rust
// Enhanced SparqlExecutor
pub trait SparqlExecutor {
    type Error: std::error::Error;
    
    // Existing
    fn select(&self, query: &str) -> Result<Vec<Binding>, Self::Error>;
    fn ask(&self, query: &str) -> Result<bool, Self::Error>;
    
    // New in Phase 1
    fn select_with_filter(&self, pattern: &str, filter: &str) 
        -> Result<Vec<Binding>, Self::Error>;
    fn select_optional(&self, required: &str, optional: &str) 
        -> Result<Vec<Binding>, Self::Error>;
}
```

### Phase 2: Cardinality Constraints (Short-term)

Implement COUNT aggregates:

```rust
// New in shacl.rs
pub enum ShaclConstraint {
    // Existing
    MinCount { path: Term, count: usize },
    MaxCount { path: Term, count: usize },
    Datatype { path: Term, datatype: String },
    NodeKind { path: Term, kind: String },
    
    // New
    ExactCount { path: Term, count: usize },
    HasValue { path: Term, value: Term },
    InList { path: Term, values: Vec<Term> },
}
```

### Phase 3: Property Paths and Advanced Patterns (Medium-term)

Add support for:
- Property path expressions (`^property`, `property1/property2`)
- Complex FILTER expressions
- Aggregate functions (SUM, AVG, GROUP_CONCAT)
- SPARQL CONSTRUCT (for transformation)

### Phase 4: Full SPARQL 1.1 Alignment (Long-term)

Integrate with full-featured SPARQL library (e.g., `oxigraph`):

```toml
# Cargo.toml
[dependencies]
oxigraph = "0.3"
```

---

## Common Validation Patterns in unify-rs

### Pattern 1: Required Property Validation

**Current Implementation (SHACL):**
```rust
let shapes = vec![ShaclShape {
    target_class: Term::Named("http://example.org/Person".into()),
    constraints: vec![
        ShaclConstraint::MinCount {
            path: Term::Named("http://example.org/email".into()),
            count: 1,
        }
    ],
}];

let result = shacl::validate(&store, &shapes);
```

**Extension (SPARQL):**
```rust
let query = r#"
SELECT ?person WHERE {
  ?person rdf:type <http://example.org/Person> .
  FILTER NOT EXISTS { ?person <http://example.org/email> ?email }
}
"#;

let violations = executor.select(query)?;
```

### Pattern 2: Cardinality Range Validation

**SHACL Approach:**
```rust
ShaclConstraint::MinCount { path: email_prop, count: 1 },
ShaclConstraint::MaxCount { path: ssn_prop, count: 1 },
```

**SPARQL Approach (to implement):**
```sparql
SELECT ?person (COUNT(?email) as ?emailCount) WHERE {
  ?person rdf:type ex:Person .
  OPTIONAL { ?person ex:email ?email }
}
GROUP BY ?person
HAVING (COUNT(?email) BETWEEN 1 AND 10)
```

### Pattern 3: Type-Dependent Validation

**Current Issue:** SHACL shapes don't support conditional constraints.

**Solution with Extended SPARQL:**
```rust
pub struct ConditionalConstraint {
    pub condition: String,      // SPARQL WHERE clause
    pub constraint: ShaclConstraint,
}

// If person is Employee, they must have workEmail
let conditional = ConditionalConstraint {
    condition: "?person rdf:type ex:Employee".into(),
    constraint: ShaclConstraint::MinCount {
        path: Term::Named("http://example.org/workEmail".into()),
        count: 1,
    }
};
```

### Pattern 4: Cross-Property Referential Integrity

**Example:** All referenced managers must exist as Persons

```rust
// To implement
pub struct ReferentialConstraint {
    pub subject_property: Term,
    pub target_class: Term,
}

// Usage:
let constraint = ReferentialConstraint {
    subject_property: Term::Named("ex:manager".into()),
    target_class: Term::Named("ex:Person".into()),
};
```

---

## Integration Examples

### Example 1: Validating a Project Manifest

**File Path:** `/home/user/rocket-craft/unify-rs/unify-rdf/src/project_bridge.rs`

Current typestate: `ProjectManifest<Pending>` → `Ingested` → `Validated`

**Enhancement:** Add semantic validation to `Validated` state:

```rust
impl ProjectManifest<Validated> {
    /// Run SPARQL validation rules against the manifest.
    pub fn validate_semantics(&self, executor: &impl SparqlExecutor) 
        -> Result<ValidationReport, ValidationError> {
        
        let mut violations = Vec::new();
        
        // Check 1: Each project has a name
        let q = "SELECT ?project WHERE { 
            ?project rdf:type ue4:Project . 
            FILTER NOT EXISTS { ?project ue4:name ?name }
        }";
        
        for binding in executor.select(q)? {
            violations.push(Violation {
                subject: binding["project"].clone(),
                message: "Project missing name".into(),
            });
        }
        
        Ok(ValidationReport {
            conforms: violations.is_empty(),
            violations,
        })
    }
}
```

### Example 2: Validating UE4 Blueprint Consistency

**Path:** `/home/user/rocket-craft/unify-rs/unify-bp/src/`

**Use Case:** Ensure all Blueprint nodes reference valid parent graphs.

```rust
pub fn validate_blueprint_graph(
    store: &TripleStore,
    blueprint_id: &Term,
) -> ValidationResult {
    // Check: All nodes have parent graph
    let query = format!(
        "SELECT ?node WHERE {{ 
            <{}> rdf:type bp:Blueprint . 
            <{}> bp:node ?node . 
            FILTER NOT EXISTS {{ ?node bp:parentGraph ?graph }} 
        }}",
        blueprint_id, blueprint_id
    );
    
    // Execute and collect violations
    let executor = PatternExecutor(&store);
    let orphan_nodes = executor.select(&query)?;
    
    Ok(ValidationResult {
        conforms: orphan_nodes.is_empty(),
        violations: orphan_nodes.into_iter()
            .map(|b| Violation {
                node: b["node"].clone(),
                message: "Blueprint node missing parent graph".into(),
            })
            .collect(),
    })
}
```

### Example 3: Validating the Ontology Pipeline

**Path:** `/home/user/rocket-craft/unify-rs/unify-rdf/src/pipeline.rs`

**Enhancement:** Add validation after each pipeline stage:

```rust
impl OntologyPipeline {
    /// Validate loaded ontology against SHACL shapes.
    pub fn validate_ontology(&self, shapes: &[ShaclShape]) -> ShaclResult {
        shacl::validate(&self.store, shapes)
    }
    
    /// Run semantic validation queries.
    pub fn validate_semantics(&self, queries: &[&str]) -> Vec<ValidationResult> {
        let executor = PatternExecutor(&self.store);
        queries.iter()
            .map(|q| {
                match executor.select(q) {
                    Ok(bindings) => ValidationResult {
                        conforms: bindings.is_empty(),
                        violations: bindings.into_iter()
                            .map(|b| Violation {
                                node: b.get("subject").cloned()
                                    .unwrap_or(Term::Blank("unknown".into())),
                                message: format!("Constraint violated: {}", q),
                            })
                            .collect(),
                    },
                    Err(e) => ValidationResult {
                        conforms: false,
                        violations: vec![Violation {
                            node: Term::Blank("error".into()),
                            message: format!("Query error: {}", e),
                        }],
                    },
                }
            })
            .collect()
    }
}
```

---

## Testing SPARQL Validation

### Unit Test Template

```rust
#[cfg(test)]
mod validation_tests {
    use super::*;

    fn setup_test_store() -> TripleStore {
        let mut store = TripleStore::new();
        
        // Add test data
        store.add(Triple::new(
            "http://example.org/alice",
            "rdf:type",
            "http://example.org/Person"
        ));
        
        store.add(Triple::new(
            "http://example.org/alice",
            "http://example.org/email",
            "alice@example.org"
        ));
        
        store
    }

    #[test]
    fn test_person_without_email_violates_mincount() {
        let mut store = setup_test_store();
        
        // Add a person without email
        store.add(Triple::new(
            "http://example.org/bob",
            "rdf:type",
            "http://example.org/Person"
        ));
        
        let shapes = vec![ShaclShape {
            target_class: Term::Named("http://example.org/Person".into()),
            constraints: vec![ShaclConstraint::MinCount {
                path: Term::Named("http://example.org/email".into()),
                count: 1,
            }],
        }];
        
        let result = shacl::validate(&store, &shapes);
        assert!(!result.conforms);
        assert_eq!(result.violations.len(), 1);
    }

    #[test]
    fn test_all_persons_with_emails_pass_validation() {
        let store = setup_test_store();
        
        let shapes = vec![ShaclShape {
            target_class: Term::Named("http://example.org/Person".into()),
            constraints: vec![ShaclConstraint::MinCount {
                path: Term::Named("http://example.org/email".into()),
                count: 1,
            }],
        }];
        
        let result = shacl::validate(&store, &shapes);
        assert!(result.conforms);
    }
}
```

---

## Performance Tips

### 1. Filter Early
```rust
// GOOD: Filter on type first (indexed)
let query = "SELECT ?person WHERE {
    ?person rdf:type ex:Person .
    ?person ex:email ?email .
    FILTER (!REGEX(?email, '@'))
}";

// BAD: Filter last (unnecessary work)
let query = "SELECT ?person ?email WHERE {
    ?person ex:email ?email .
    ?person rdf:type ex:Person .
    FILTER (!REGEX(?email, '@'))
}";
```

### 2. Use Aggregate Functions Efficiently
```rust
// Leverage GROUP BY for cardinality checks
let query = "SELECT ?person (COUNT(?email) as ?count) WHERE {
    ?person rdf:type ex:Person .
    OPTIONAL { ?person ex:email ?email }
}
GROUP BY ?person
HAVING (COUNT(?email) < 1)";
```

### 3. Batch Validation
```rust
// Validate multiple constraints in one query
let query = "SELECT ?person ?violation WHERE {
    {
        SELECT ?person ('missing-email' as ?violation) WHERE {
            ?person rdf:type ex:Person .
            FILTER NOT EXISTS { ?person ex:email ?email }
        }
    } UNION {
        SELECT ?person ('invalid-age' as ?violation) WHERE {
            ?person rdf:type ex:Person .
            ?person ex:age ?age .
            FILTER (?age < 0 || ?age > 150)
        }
    }
}";
```

---

## Mapping to W3C Standards

### SHACL Mapping

| unify-rs Type | SHACL Equivalent |
|---|---|
| `ShaclConstraint::MinCount` | `sh:minCount` |
| `ShaclConstraint::MaxCount` | `sh:maxCount` |
| `ShaclConstraint::Datatype` | `sh:datatype` |
| `ShaclConstraint::NodeKind` | `sh:nodeKind` |
| (To implement) `ExactCount` | `sh:count` |
| (To implement) `InList` | `sh:in` |

### SPARQL Mapping

| Operation | SPARQL | unify-rs |
|---|---|---|
| Required property | `FILTER NOT EXISTS { ?s ?p ?o }` | `MinCount { count: 1 }` |
| Max cardinality | `GROUP BY ... HAVING (COUNT() <= n)` | `MaxCount { count: n }` |
| Enum constraint | `FILTER (?x IN (a, b, c))` | (To implement) `InList` |
| Pattern match | `FILTER REGEX(?x, "pattern")` | (To implement) `Pattern` |

---

## Related Resources in Rocket Craft

### RDF/SPARQL Usage
- `/home/user/rocket-craft/unify-rs/unify-mcp/src/rocket_tools.rs` — MCP tools that query project manifest
- `/home/user/rocket-craft/unify-rs/unify-config/src/validate.rs` — Configuration validation patterns
- `/home/user/rocket-craft/unify-rs/unify-lsp/src/diagnostic.rs` — LSP diagnostics (similar violation reporting)

### Semantic Validation
- `/home/user/rocket-craft/tools/rocket-sdk/src/manifest.rs` — Project manifest structure
- `/home/user/rocket-craft/chicago-tdd-tools/tests/suite_validation.rs` — Validation test patterns
- `/home/user/rocket-craft/asset-pipeline/pipeline-core/src/validation.rs` — Asset validation model

---

## Next Steps

1. **Immediate:** Review existing SHACL constraints in `shacl.rs` and identify gaps
2. **Short-term:** Implement FILTER expression parsing in `sparql.rs`
3. **Medium-term:** Add support for COUNT aggregates and GROUP BY
4. **Long-term:** Consider integration with full SPARQL engine (oxigraph)

## Questions for Implementation

1. Should validation be **eager** (all violations) or **lazy** (first violation)?
2. Should violations be **typed** (error code) or **message-only**?
3. Should the RDF store support **partial validation** (incrementally check changed triples)?
4. Should SPARQL queries be **compiled** (for repeated execution) or **interpreted**?

---

## Document Metadata

- **Version:** 1.0
- **Created:** June 18, 2026
- **Related:** `SPARQL_VALIDATION_RESEARCH.md`
- **Location:** `/home/user/rocket-craft/SPARQL_VALIDATION_QUICKSTART.md`
