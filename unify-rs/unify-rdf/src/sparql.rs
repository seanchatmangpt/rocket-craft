use std::collections::HashMap;
use crate::triple::Term;
use crate::store::TripleStore;

/// A single result row from a SELECT query: variable name → Term.
pub type Binding = HashMap<String, Term>;

/// Trait for anything that can execute SPARQL-like queries.
pub trait SparqlExecutor {
    type Error: std::error::Error;
    fn select(&self, query: &str) -> Result<Vec<Binding>, Self::Error>;
    fn ask(&self, query: &str) -> Result<bool, Self::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum SparqlError {
    #[error("Unsupported query syntax: {0}")]
    Unsupported(String),
    #[error("Parse error: {0}")]
    ParseError(String),
}

/// A simple executor that supports only `SELECT * WHERE { ?s ?p ?o }` over a TripleStore.
pub struct PatternExecutor<'a>(pub &'a TripleStore);

impl<'a> SparqlExecutor for PatternExecutor<'a> {
    type Error = SparqlError;

    fn select(&self, query: &str) -> Result<Vec<Binding>, SparqlError> {
        let q = query.trim().to_lowercase();
        if !q.starts_with("select") {
            return Err(SparqlError::Unsupported(
                "Only SELECT queries are supported".into(),
            ));
        }
        // Detect the simple "SELECT * WHERE { ?s ?p ?o }" pattern
        if q.contains("?s") && q.contains("?p") && q.contains("?o") {
            let bindings = self
                .0
                .query_pattern(None, None, None)
                .into_iter()
                .map(|t| {
                    let mut b = Binding::new();
                    b.insert("s".into(), t.subject.clone());
                    b.insert("p".into(), t.predicate.clone());
                    b.insert("o".into(), t.object.clone());
                    b
                })
                .collect();
            return Ok(bindings);
        }
        Err(SparqlError::Unsupported(
            format!("Cannot parse query: {}", query),
        ))
    }

    fn ask(&self, query: &str) -> Result<bool, SparqlError> {
        let q = query.trim().to_lowercase();
        if !q.starts_with("ask") {
            return Err(SparqlError::Unsupported(
                "Only ASK queries are supported here".into(),
            ));
        }
        // ASK WHERE { ?s ?p ?o } → true when the store is non-empty
        if q.contains("?s") && q.contains("?p") && q.contains("?o") {
            return Ok(!self.0.is_empty());
        }
        Err(SparqlError::Unsupported(
            format!("Cannot parse ASK query: {}", query),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::TripleStore;
    use crate::triple::Triple;

    #[test]
    fn test_pattern_executor_select_all_triples() {
        let mut store = TripleStore::new();
        store.add(Triple::new("http://s1", "http://p1", "http://o1"));
        store.add(Triple::new("http://s2", "http://p2", "http://o2"));
        let exec = PatternExecutor(&store);
        let results = exec.select("SELECT * WHERE { ?s ?p ?o }").unwrap();
        assert_eq!(results.len(), 2);
        // Each binding should have s, p, o keys
        assert!(results[0].contains_key("s"));
        assert!(results[0].contains_key("p"));
        assert!(results[0].contains_key("o"));
    }

    #[test]
    fn test_pattern_executor_select_unsupported_returns_error() {
        let store = TripleStore::new();
        let exec = PatternExecutor(&store);
        let result = exec.select("INSERT DATA { <http://s> <http://p> <http://o> }");
        assert!(result.is_err());
    }

    #[test]
    fn test_pattern_executor_ask_non_empty_store() {
        let mut store = TripleStore::new();
        store.add(Triple::new("http://s", "http://p", "http://o"));
        let exec = PatternExecutor(&store);
        assert_eq!(exec.ask("ASK WHERE { ?s ?p ?o }").unwrap(), true);
    }

    #[test]
    fn test_pattern_executor_ask_empty_store() {
        let store = TripleStore::new();
        let exec = PatternExecutor(&store);
        assert_eq!(exec.ask("ASK WHERE { ?s ?p ?o }").unwrap(), false);
    }
}
