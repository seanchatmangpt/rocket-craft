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
