use crate::triple::{Term, Triple};

/// An in-memory RDF graph (triple store).
#[derive(Debug, Default, Clone)]
pub struct TripleStore {
    triples: Vec<Triple>,
}

impl TripleStore {
    pub fn new() -> Self {
        TripleStore::default()
    }

    /// Add a triple to the store.
    pub fn add(&mut self, triple: Triple) {
        self.triples.push(triple);
    }

    /// Remove a triple from the store. Returns `true` if it was present.
    pub fn remove(&mut self, triple: &Triple) -> bool {
        let before = self.triples.len();
        self.triples.retain(|t| t != triple);
        self.triples.len() < before
    }

    /// Check whether the store contains a given triple.
    pub fn contains(&self, triple: &Triple) -> bool {
        self.triples.contains(triple)
    }

    /// Number of triples in the store.
    pub fn len(&self) -> usize {
        self.triples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.triples.is_empty()
    }

    /// Return all triples whose subject matches.
    pub fn query_subject(&self, subject: &Term) -> Vec<&Triple> {
        self.triples.iter().filter(|t| &t.subject == subject).collect()
    }

    /// Return all triples whose predicate matches.
    pub fn query_predicate(&self, predicate: &Term) -> Vec<&Triple> {
        self.triples.iter().filter(|t| &t.predicate == predicate).collect()
    }

    /// Return all triples whose object matches.
    pub fn query_object(&self, object: &Term) -> Vec<&Triple> {
        self.triples.iter().filter(|t| &t.object == object).collect()
    }

    /// Pattern-based lookup. `None` acts as a wildcard.
    pub fn query_pattern(
        &self,
        s: Option<&Term>,
        p: Option<&Term>,
        o: Option<&Term>,
    ) -> Vec<&Triple> {
        self.triples
            .iter()
            .filter(|t| {
                s.map_or(true, |sv| &t.subject == sv)
                    && p.map_or(true, |pv| &t.predicate == pv)
                    && o.map_or(true, |ov| &t.object == ov)
            })
            .collect()
    }

    /// All distinct subjects in the store.
    pub fn subjects(&self) -> Vec<&Term> {
        let mut seen: Vec<&Term> = Vec::new();
        for t in &self.triples {
            if !seen.contains(&&t.subject) {
                seen.push(&t.subject);
            }
        }
        seen
    }

    /// All distinct predicates in the store.
    pub fn predicates(&self) -> Vec<&Term> {
        let mut seen: Vec<&Term> = Vec::new();
        for t in &self.triples {
            if !seen.contains(&&t.predicate) {
                seen.push(&t.predicate);
            }
        }
        seen
    }

    /// All objects for a given (subject, predicate) pair.
    pub fn objects_for(&self, subject: &Term, predicate: &Term) -> Vec<&Term> {
        self.triples
            .iter()
            .filter(|t| &t.subject == subject && &t.predicate == predicate)
            .map(|t| &t.object)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::triple::Term;

    fn make_triple(s: &str, p: &str, o: &str) -> Triple {
        Triple::new(s, p, o)
    }

    #[test]
    fn test_store_add_contains_len() {
        let mut store = TripleStore::new();
        assert!(store.is_empty());
        let t = make_triple("http://s", "http://p", "http://o");
        store.add(t.clone());
        assert_eq!(store.len(), 1);
        assert!(store.contains(&t));
    }

    #[test]
    fn test_store_remove() {
        let mut store = TripleStore::new();
        let t = make_triple("http://s", "http://p", "http://o");
        store.add(t.clone());
        assert!(store.remove(&t));
        assert_eq!(store.len(), 0);
        assert!(!store.remove(&t)); // second remove returns false
    }

    #[test]
    fn test_query_subject_returns_only_matching() {
        let mut store = TripleStore::new();
        let t1 = make_triple("http://s1", "http://p", "http://o");
        let t2 = make_triple("http://s2", "http://p", "http://o");
        store.add(t1.clone());
        store.add(t2.clone());
        let s1 = Term::Named("http://s1".into());
        let results = store.query_subject(&s1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], &t1);
    }

    #[test]
    fn test_query_pattern_none_wildcards_returns_all() {
        let mut store = TripleStore::new();
        store.add(make_triple("http://s1", "http://p1", "http://o1"));
        store.add(make_triple("http://s2", "http://p2", "http://o2"));
        let results = store.query_pattern(None, None, None);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_pattern_with_subject_filter() {
        let mut store = TripleStore::new();
        store.add(make_triple("http://s1", "http://p", "http://o1"));
        store.add(make_triple("http://s2", "http://p", "http://o2"));
        let s = Term::Named("http://s1".into());
        let results = store.query_pattern(Some(&s), None, None);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_objects_for_returns_correct_values() {
        let mut store = TripleStore::new();
        let subj = Term::Named("http://s".into());
        let pred = Term::Named("http://p".into());
        let obj1 = Term::Named("http://o1".into());
        let obj2 = Term::Named("http://o2".into());
        store.add(Triple { subject: subj.clone(), predicate: pred.clone(), object: obj1.clone() });
        store.add(Triple { subject: subj.clone(), predicate: pred.clone(), object: obj2.clone() });
        // different subject triple
        store.add(make_triple("http://other", "http://p", "http://o3"));
        let objs = store.objects_for(&subj, &pred);
        assert_eq!(objs.len(), 2);
        assert!(objs.contains(&&obj1));
        assert!(objs.contains(&&obj2));
    }
}
