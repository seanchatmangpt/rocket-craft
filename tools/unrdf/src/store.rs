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
        self.triples
            .iter()
            .filter(|t| &t.subject == subject)
            .collect()
    }

    /// Return all triples whose predicate matches.
    pub fn query_predicate(&self, predicate: &Term) -> Vec<&Triple> {
        self.triples
            .iter()
            .filter(|t| &t.predicate == predicate)
            .collect()
    }

    /// Return all triples whose object matches.
    pub fn query_object(&self, object: &Term) -> Vec<&Triple> {
        self.triples
            .iter()
            .filter(|t| &t.object == object)
            .collect()
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
                s.is_none_or(|sv| &t.subject == sv)
                    && p.is_none_or(|pv| &t.predicate == pv)
                    && o.is_none_or(|ov| &t.object == ov)
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
    use crate::triple::{Term, Triple};

    fn s(iri: &str) -> Term { Term::Named(iri.into()) }

    fn t(subj: &str, pred: &str, obj: &str) -> Triple {
        Triple::new(s(subj), s(pred), s(obj))
    }

    // ── basic add / contains / len ────────────────────────────────────────────

    #[test]
    fn new_store_is_empty() {
        let store = TripleStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn add_makes_contains_return_true() {
        let mut store = TripleStore::new();
        let triple = t(":Alice", ":knows", ":Bob");
        store.add(triple.clone());
        assert!(store.contains(&triple));
    }

    #[test]
    fn len_reflects_number_of_triples() {
        let mut store = TripleStore::new();
        store.add(t(":Alice", ":knows", ":Bob"));
        store.add(t(":Bob", ":knows", ":Carol"));
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn duplicate_add_increases_len() {
        // Vec-based store does not deduplicate
        let mut store = TripleStore::new();
        let triple = t(":Alice", ":knows", ":Bob");
        store.add(triple.clone());
        store.add(triple);
        assert_eq!(store.len(), 2);
    }

    // ── remove ────────────────────────────────────────────────────────────────

    #[test]
    fn remove_existing_returns_true_and_decreases_len() {
        let mut store = TripleStore::new();
        let triple = t(":Alice", ":knows", ":Bob");
        store.add(triple.clone());
        assert!(store.remove(&triple));
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn remove_absent_returns_false() {
        let mut store = TripleStore::new();
        assert!(!store.remove(&t(":X", ":y", ":Z")));
    }

    // ── query_subject / query_predicate / query_object ────────────────────────

    #[test]
    fn query_subject_returns_matching_triples() {
        let mut store = TripleStore::new();
        store.add(t(":Alice", ":knows", ":Bob"));
        store.add(t(":Alice", ":likes", ":Carol"));
        store.add(t(":Bob", ":knows", ":Dave"));
        let results = store.query_subject(&s(":Alice"));
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|t| t.subject == s(":Alice")));
    }

    #[test]
    fn query_predicate_returns_matching_triples() {
        let mut store = TripleStore::new();
        store.add(t(":Alice", ":knows", ":Bob"));
        store.add(t(":Bob", ":knows", ":Carol"));
        store.add(t(":Alice", ":likes", ":Dave"));
        let results = store.query_predicate(&s(":knows"));
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn query_object_returns_matching_triples() {
        let mut store = TripleStore::new();
        store.add(t(":Alice", ":knows", ":Bob"));
        store.add(t(":Carol", ":likes", ":Bob"));
        store.add(t(":Alice", ":likes", ":Dave"));
        let results = store.query_object(&s(":Bob"));
        assert_eq!(results.len(), 2);
    }

    // ── query_pattern ─────────────────────────────────────────────────────────

    #[test]
    fn query_pattern_all_none_returns_all() {
        let mut store = TripleStore::new();
        store.add(t(":A", ":p", ":B"));
        store.add(t(":C", ":q", ":D"));
        let results = store.query_pattern(None, None, None);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn query_pattern_subject_filter() {
        let mut store = TripleStore::new();
        store.add(t(":Alice", ":knows", ":Bob"));
        store.add(t(":Bob", ":knows", ":Carol"));
        let alice = s(":Alice");
        let results = store.query_pattern(Some(&alice), None, None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].subject, alice);
    }

    #[test]
    fn query_pattern_spo_exact_match() {
        let mut store = TripleStore::new();
        store.add(t(":A", ":B", ":C"));
        store.add(t(":X", ":Y", ":Z"));
        let a = s(":A"); let b = s(":B"); let c = s(":C");
        let results = store.query_pattern(Some(&a), Some(&b), Some(&c));
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn query_pattern_no_match_returns_empty() {
        let mut store = TripleStore::new();
        store.add(t(":A", ":B", ":C"));
        let x = s(":NOPE");
        let results = store.query_pattern(Some(&x), None, None);
        assert!(results.is_empty());
    }

    // ── predicates ───────────────────────────────────────────────────────────

    #[test]
    fn predicates_returns_distinct_predicates() {
        let mut store = TripleStore::new();
        store.add(t(":A", ":knows", ":B"));
        store.add(t(":B", ":knows", ":C"));  // same predicate
        store.add(t(":A", ":likes", ":C"));  // different predicate
        let preds = store.predicates();
        assert_eq!(preds.len(), 2);
    }

    // ── objects_for ───────────────────────────────────────────────────────────

    #[test]
    fn objects_for_returns_all_objects_for_sp_pair() {
        let mut store = TripleStore::new();
        store.add(t(":Alice", ":knows", ":Bob"));
        store.add(t(":Alice", ":knows", ":Carol"));
        store.add(t(":Alice", ":likes", ":Dave"));
        let alice = s(":Alice");
        let knows = s(":knows");
        let objects = store.objects_for(&alice, &knows);
        assert_eq!(objects.len(), 2);
        assert!(objects.contains(&&s(":Bob")));
        assert!(objects.contains(&&s(":Carol")));
    }

    #[test]
    fn objects_for_returns_empty_when_no_match() {
        let store = TripleStore::new();
        let alice = s(":Alice");
        let knows = s(":knows");
        assert!(store.objects_for(&alice, &knows).is_empty());
    }

    // ── Term variants ─────────────────────────────────────────────────────────

    #[test]
    fn blank_node_can_be_stored_and_retrieved() {
        let mut store = TripleStore::new();
        let triple = Triple::new(
            Term::Blank("b0".into()),
            Term::Named(":type".into()),
            Term::Named(":Person".into()),
        );
        store.add(triple.clone());
        assert!(store.contains(&triple));
    }

    #[test]
    fn literal_can_be_stored_and_retrieved() {
        let mut store = TripleStore::new();
        let triple = Triple::new(
            Term::Named(":Alice".into()),
            Term::Named(":name".into()),
            Term::Literal {
                value: "Alice Smith".into(),
                datatype: Some("xsd:string".into()),
                lang: None,
            },
        );
        store.add(triple.clone());
        assert!(store.contains(&triple));
    }
}
