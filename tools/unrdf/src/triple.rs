/// An RDF term: Named node (IRI), blank node, or literal value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Term {
    /// An IRI (Internationalized Resource Identifier).
    Named(String),
    /// A blank node with an internal identifier.
    Blank(String),
    /// A literal value with optional datatype and language tag.
    Literal {
        value: String,
        datatype: Option<String>,
        lang: Option<String>,
    },
}

impl From<&str> for Term {
    fn from(s: &str) -> Self {
        Term::Named(s.into())
    }
}

impl From<String> for Term {
    fn from(s: String) -> Self {
        Term::Named(s)
    }
}

/// An RDF triple consisting of subject, predicate, and object terms.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Triple {
    pub subject: Term,
    pub predicate: Term,
    pub object: Term,
}

impl Triple {
    pub fn new(s: impl Into<Term>, p: impl Into<Term>, o: impl Into<Term>) -> Self {
        Triple {
            subject: s.into(),
            predicate: p.into(),
            object: o.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triple_new_creates_correctly() {
        let t = Triple::new(
            "http://example.org/s",
            "http://example.org/p",
            "http://example.org/o",
        );
        assert_eq!(t.subject, Term::Named("http://example.org/s".into()));
        assert_eq!(t.predicate, Term::Named("http://example.org/p".into()));
        assert_eq!(t.object, Term::Named("http://example.org/o".into()));
    }

    #[test]
    fn test_term_from_str_creates_named() {
        let term: Term = "http://example.org/foo".into();
        assert_eq!(term, Term::Named("http://example.org/foo".into()));
    }

    #[test]
    fn test_term_literal_variant() {
        let t = Term::Literal {
            value: "hello".into(),
            datatype: Some("xsd:string".into()),
            lang: None,
        };
        if let Term::Literal {
            value,
            datatype,
            lang,
        } = &t
        {
            assert_eq!(value, "hello");
            assert_eq!(datatype.as_deref(), Some("xsd:string"));
            assert!(lang.is_none());
        } else {
            panic!("Expected Literal variant");
        }
    }

    #[test]
    fn test_term_blank_variant() {
        let t = Term::Blank("b0".into());
        assert_eq!(t, Term::Blank("b0".into()));
    }
}
