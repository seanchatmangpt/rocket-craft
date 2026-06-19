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
