use crate::store::TripleStore;
use crate::triple::Term;

/// A SHACL shape targeting a particular RDF class.
pub struct ShaclShape {
    pub target_class: Term,
    pub constraints: Vec<ShaclConstraint>,
}

/// Individual SHACL constraints supported by this facade.
pub enum ShaclConstraint {
    MinCount { path: Term, count: usize },
    MaxCount { path: Term, count: usize },
    Datatype { path: Term, datatype: String },
    NodeKind { path: Term, kind: String },
}

/// Aggregate result of a SHACL validation run.
pub struct ShaclResult {
    pub conforms: bool,
    pub violations: Vec<ShaclViolation>,
}

/// A single SHACL violation.
pub struct ShaclViolation {
    pub node: Term,
    pub path: Option<Term>,
    pub message: String,
}

/// Validate a TripleStore against a slice of SHACL shapes.
pub fn validate(store: &TripleStore, shapes: &[ShaclShape]) -> ShaclResult {
    let mut violations: Vec<ShaclViolation> = Vec::new();
    let rdf_type = Term::Named("rdf:type".into());

    for shape in shapes {
        // Collect all nodes that are instances of shape.target_class
        let target_nodes: Vec<Term> = store
            .query_predicate(&rdf_type)
            .into_iter()
            .filter(|t| t.object == shape.target_class)
            .map(|t| t.subject.clone())
            .collect();

        for node in &target_nodes {
            for constraint in &shape.constraints {
                match constraint {
                    ShaclConstraint::MinCount { path, count } => {
                        let actual = store.objects_for(node, path).len();
                        if actual < *count {
                            violations.push(ShaclViolation {
                                node: node.clone(),
                                path: Some(path.clone()),
                                message: format!(
                                    "MinCount violation: expected >= {}, got {}",
                                    count, actual
                                ),
                            });
                        }
                    }
                    ShaclConstraint::MaxCount { path, count } => {
                        let actual = store.objects_for(node, path).len();
                        if actual > *count {
                            violations.push(ShaclViolation {
                                node: node.clone(),
                                path: Some(path.clone()),
                                message: format!(
                                    "MaxCount violation: expected <= {}, got {}",
                                    count, actual
                                ),
                            });
                        }
                    }
                    ShaclConstraint::Datatype { path, datatype } => {
                        for obj in store.objects_for(node, path) {
                            let conforms = match obj {
                                Term::Literal {
                                    datatype: Some(dt), ..
                                } => dt == datatype,
                                _ => false,
                            };
                            if !conforms {
                                violations.push(ShaclViolation {
                                    node: node.clone(),
                                    path: Some(path.clone()),
                                    message: format!("Datatype violation: expected {}", datatype),
                                });
                            }
                        }
                    }
                    ShaclConstraint::NodeKind { path, kind } => {
                        for obj in store.objects_for(node, path) {
                            let actual_kind = match obj {
                                Term::Named(_) => "IRI",
                                Term::Blank(_) => "BlankNode",
                                Term::Literal { .. } => "Literal",
                            };
                            if actual_kind != kind.as_str() {
                                violations.push(ShaclViolation {
                                    node: node.clone(),
                                    path: Some(path.clone()),
                                    message: format!(
                                        "NodeKind violation: expected {}, got {}",
                                        kind, actual_kind
                                    ),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    ShaclResult {
        conforms: violations.is_empty(),
        violations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::TripleStore;
    use crate::triple::Triple;

    fn make_store_with_instance() -> TripleStore {
        let mut store = TripleStore::new();
        // Declare <http://ex/alice> rdf:type <http://ex/Person>
        store.add(Triple {
            subject: Term::Named("http://ex/alice".into()),
            predicate: Term::Named("rdf:type".into()),
            object: Term::Named("http://ex/Person".into()),
        });
        // Give her a name
        store.add(Triple {
            subject: Term::Named("http://ex/alice".into()),
            predicate: Term::Named("http://ex/name".into()),
            object: Term::Literal {
                value: "Alice".into(),
                datatype: Some("xsd:string".into()),
                lang: None,
            },
        });
        store
    }

    #[test]
    fn test_shacl_validate_passes_on_conforming_store() {
        let store = make_store_with_instance();
        let shapes = vec![ShaclShape {
            target_class: Term::Named("http://ex/Person".into()),
            constraints: vec![ShaclConstraint::MinCount {
                path: Term::Named("http://ex/name".into()),
                count: 1,
            }],
        }];
        let result = validate(&store, &shapes);
        assert!(result.conforms);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_shacl_validate_fails_with_violation_when_constraint_violated() {
        let store = make_store_with_instance();
        let shapes = vec![ShaclShape {
            target_class: Term::Named("http://ex/Person".into()),
            constraints: vec![ShaclConstraint::MinCount {
                path: Term::Named("http://ex/email".into()), // Alice has no email
                count: 1,
            }],
        }];
        let result = validate(&store, &shapes);
        assert!(!result.conforms);
        assert_eq!(result.violations.len(), 1);
        assert!(result.violations[0].message.contains("MinCount"));
    }

    #[test]
    fn test_shacl_validate_max_count_violation() {
        let mut store = make_store_with_instance();
        // Add a second name to alice (should fail MaxCount 1)
        store.add(Triple {
            subject: Term::Named("http://ex/alice".into()),
            predicate: Term::Named("http://ex/name".into()),
            object: Term::Literal {
                value: "Alice B.".into(),
                datatype: Some("xsd:string".into()),
                lang: None,
            },
        });
        let shapes = vec![ShaclShape {
            target_class: Term::Named("http://ex/Person".into()),
            constraints: vec![ShaclConstraint::MaxCount {
                path: Term::Named("http://ex/name".into()),
                count: 1,
            }],
        }];
        let result = validate(&store, &shapes);
        assert!(!result.conforms);
        assert!(result.violations[0].message.contains("MaxCount"));
    }
}
