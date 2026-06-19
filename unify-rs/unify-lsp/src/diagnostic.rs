use std::collections::HashMap;

/// Severity of an LSP diagnostic.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

/// A position in a text document (LSP-style, zero-based).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// A range in a text document.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// A single LSP diagnostic.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub code: Option<String>,
    pub source: Option<String>,
}

/// A collection of diagnostics keyed by document URI.
pub struct DiagnosticSet {
    by_uri: HashMap<String, Vec<Diagnostic>>,
}

impl DiagnosticSet {
    /// Create an empty diagnostic set.
    pub fn new() -> Self {
        Self {
            by_uri: HashMap::new(),
        }
    }

    /// Add a diagnostic for a URI.
    pub fn add(&mut self, uri: String, diag: Diagnostic) {
        self.by_uri.entry(uri).or_default().push(diag);
    }

    /// Remove all diagnostics for a URI.
    pub fn clear(&mut self, uri: &str) {
        self.by_uri.remove(uri);
    }

    /// Get all diagnostics for a URI.
    pub fn get(&self, uri: &str) -> &[Diagnostic] {
        self.by_uri.get(uri).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Return all error-severity diagnostics across all URIs.
    pub fn all_errors(&self) -> Vec<(&str, &Diagnostic)> {
        self.by_uri
            .iter()
            .flat_map(|(uri, diags)| {
                diags.iter().filter_map(move |d| {
                    if d.severity == DiagnosticSeverity::Error {
                        Some((uri.as_str(), d))
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    /// Count total error diagnostics across all URIs.
    pub fn error_count(&self) -> usize {
        self.by_uri
            .values()
            .flat_map(|v| v.iter())
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .count()
    }

    /// Merge another `DiagnosticSet` into this one (appending per URI).
    pub fn merge(&mut self, other: DiagnosticSet) {
        for (uri, diags) in other.by_uri {
            self.by_uri.entry(uri).or_default().extend(diags);
        }
    }

    /// Number of distinct URIs with diagnostics.
    pub fn uri_count(&self) -> usize {
        self.by_uri.len()
    }
}

impl Default for DiagnosticSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diag(severity: DiagnosticSeverity, msg: &str) -> Diagnostic {
        Diagnostic {
            range: Range {
                start: Position { line: 0, character: 0 },
                end:   Position { line: 0, character: 1 },
            },
            severity,
            message: msg.into(),
            code: None,
            source: None,
        }
    }

    // ── DiagnosticSet basics ──────────────────────────────────────────────────

    #[test]
    fn new_is_empty() {
        let set = DiagnosticSet::new();
        assert_eq!(set.uri_count(), 0);
        assert_eq!(set.error_count(), 0);
    }

    #[test]
    fn add_then_get_returns_diagnostics_for_uri() {
        let mut set = DiagnosticSet::new();
        set.add("file:///a.rs".into(), diag(DiagnosticSeverity::Error, "oops"));
        let diags = set.get("file:///a.rs");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].message, "oops");
    }

    #[test]
    fn get_unknown_uri_returns_empty_slice() {
        let set = DiagnosticSet::new();
        assert!(set.get("file:///nope.rs").is_empty());
    }

    #[test]
    fn add_multiple_to_same_uri_appends() {
        let mut set = DiagnosticSet::new();
        set.add("f".into(), diag(DiagnosticSeverity::Error, "e1"));
        set.add("f".into(), diag(DiagnosticSeverity::Warning, "w1"));
        assert_eq!(set.get("f").len(), 2);
    }

    #[test]
    fn clear_removes_all_for_uri() {
        let mut set = DiagnosticSet::new();
        set.add("f".into(), diag(DiagnosticSeverity::Error, "e"));
        set.clear("f");
        assert!(set.get("f").is_empty());
        assert_eq!(set.uri_count(), 0);
    }

    // ── error_count / all_errors ──────────────────────────────────────────────

    #[test]
    fn error_count_counts_only_errors() {
        let mut set = DiagnosticSet::new();
        set.add("a".into(), diag(DiagnosticSeverity::Error, "e"));
        set.add("a".into(), diag(DiagnosticSeverity::Warning, "w"));
        set.add("b".into(), diag(DiagnosticSeverity::Error, "e2"));
        assert_eq!(set.error_count(), 2);
    }

    #[test]
    fn all_errors_returns_only_error_severity() {
        let mut set = DiagnosticSet::new();
        set.add("a".into(), diag(DiagnosticSeverity::Error, "E"));
        set.add("a".into(), diag(DiagnosticSeverity::Information, "I"));
        let errors = set.all_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].1.message, "E");
    }

    // ── merge ─────────────────────────────────────────────────────────────────

    #[test]
    fn merge_appends_diagnostics_from_other() {
        let mut a = DiagnosticSet::new();
        a.add("f".into(), diag(DiagnosticSeverity::Error, "a"));
        let mut b = DiagnosticSet::new();
        b.add("f".into(), diag(DiagnosticSeverity::Warning, "b"));
        a.merge(b);
        assert_eq!(a.get("f").len(), 2);
    }

    #[test]
    fn merge_adds_new_uris_from_other() {
        let mut a = DiagnosticSet::new();
        a.add("f1".into(), diag(DiagnosticSeverity::Error, "e"));
        let mut b = DiagnosticSet::new();
        b.add("f2".into(), diag(DiagnosticSeverity::Warning, "w"));
        a.merge(b);
        assert_eq!(a.uri_count(), 2);
    }
}
