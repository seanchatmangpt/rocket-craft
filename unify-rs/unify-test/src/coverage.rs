//! CoverageSurface — tracks which commands/code paths were exercised, modelled
//! after un-test-utils AST surface analysis.

use std::collections::HashSet;

/// Report returned by [`CoverageSurface::report`].
#[derive(Debug, Clone)]
pub struct CoverageReport {
    pub pct: f64,
    pub covered: usize,
    pub total: usize,
    pub uncovered: Vec<String>,
}

impl std::fmt::Display for CoverageReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Coverage: {:.1}% ({}/{}) — uncovered: [{}]",
            self.pct,
            self.covered,
            self.total,
            self.uncovered.join(", ")
        )
    }
}

/// Tracks a declared set of code-surface paths and which ones were exercised.
pub struct CoverageSurface {
    total: HashSet<String>,
    exercised: HashSet<String>,
}

impl CoverageSurface {
    /// Create an empty surface tracker.
    pub fn new() -> Self {
        Self { total: HashSet::new(), exercised: HashSet::new() }
    }

    /// Declare a known path on the code surface (must be called before
    /// `exercise` for accurate percentages).
    pub fn declare(&mut self, path: impl Into<String>) {
        self.total.insert(path.into());
    }

    /// Mark a path as exercised.  If the path was not previously declared it
    /// is silently added to both sets.
    pub fn exercise(&mut self, path: impl Into<String>) {
        let p = path.into();
        self.total.insert(p.clone());
        self.exercised.insert(p);
    }

    /// Percentage of declared paths that were exercised (0.0–100.0).
    /// Returns 0.0 when nothing has been declared.
    pub fn coverage_pct(&self) -> f64 {
        if self.total.is_empty() {
            return 0.0;
        }
        let exercised = self.total.iter().filter(|p| self.exercised.contains(*p)).count();
        (exercised as f64 / self.total.len() as f64) * 100.0
    }

    /// Paths that were declared but never exercised, sorted for determinism.
    pub fn uncovered(&self) -> Vec<&str> {
        let mut v: Vec<&str> = self
            .total
            .iter()
            .filter(|p| !self.exercised.contains(*p))
            .map(|s| s.as_str())
            .collect();
        v.sort_unstable();
        v
    }

    /// Number of declared paths (exercised or not).
    pub fn total_count(&self) -> usize {
        self.total.len()
    }

    /// Number of paths that were exercised.
    pub fn exercised_count(&self) -> usize {
        self.total.iter().filter(|p| self.exercised.contains(*p)).count()
    }

    /// Build a [`CoverageReport`] snapshot.
    pub fn report(&self) -> CoverageReport {
        CoverageReport {
            pct: self.coverage_pct(),
            covered: self.exercised_count(),
            total: self.total_count(),
            uncovered: self.uncovered().iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Default for CoverageSurface {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declare_and_exercise_calculates_correct_pct() {
        let mut cs = CoverageSurface::new();
        cs.declare("cmd/foo");
        cs.declare("cmd/bar");
        cs.declare("cmd/baz");
        cs.exercise("cmd/foo");

        let pct = cs.coverage_pct();
        // 1 out of 3 ≈ 33.33%
        assert!((pct - 33.333).abs() < 0.01, "got {pct}");
        assert_eq!(cs.exercised_count(), 1);
        assert_eq!(cs.total_count(), 3);
    }

    #[test]
    fn uncovered_returns_unexercised_items() {
        let mut cs = CoverageSurface::new();
        cs.declare("a");
        cs.declare("b");
        cs.declare("c");
        cs.exercise("b");

        let unc = cs.uncovered();
        assert_eq!(unc, vec!["a", "c"]);
    }

    #[test]
    fn coverage_report_display_shows_percentage() {
        let mut cs = CoverageSurface::new();
        cs.declare("x");
        cs.declare("y");
        cs.exercise("x");

        let report = cs.report();
        let s = report.to_string();
        assert!(s.contains("50.0%"), "display was: {s}");
        assert!(s.contains("1/2"), "display was: {s}");
    }

    #[test]
    fn full_coverage_gives_100_pct() {
        let mut cs = CoverageSurface::new();
        cs.declare("p1");
        cs.declare("p2");
        cs.exercise("p1");
        cs.exercise("p2");

        assert_eq!(cs.coverage_pct(), 100.0);
        assert!(cs.uncovered().is_empty());
    }

    #[test]
    fn empty_surface_gives_zero_pct() {
        let cs = CoverageSurface::new();
        assert_eq!(cs.coverage_pct(), 0.0);
    }
}
