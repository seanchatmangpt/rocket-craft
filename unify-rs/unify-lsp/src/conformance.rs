/// Per-dimension delta between two conformance scores.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConformanceDelta {
    pub fitness_delta: f64,
    pub precision_delta: f64,
    pub generalization_delta: f64,
    pub simplicity_delta: f64,
}

/// Process-conformance score (fitness/precision/generalization/simplicity).
///
/// Values are in [0.0, 1.0] where 1.0 is perfect conformance.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConformanceScore {
    pub fitness: f64,
    pub precision: f64,
    pub generalization: f64,
    pub simplicity: f64,
}

impl ConformanceScore {
    /// Construct a score from all four dimensions.
    pub fn new(fitness: f64, precision: f64, generalization: f64, simplicity: f64) -> Self {
        Self {
            fitness,
            precision,
            generalization,
            simplicity,
        }
    }

    /// Harmonic mean of fitness and precision.
    ///
    /// Returns 0.0 when both are 0.0 to avoid division by zero.
    pub fn f_measure(&self) -> f64 {
        let f = self.fitness;
        let p = self.precision;
        if f + p == 0.0 {
            return 0.0;
        }
        2.0 * f * p / (f + p)
    }

    /// Returns `true` when `f_measure()` is strictly above `threshold`.
    pub fn is_above_threshold(&self, threshold: f64) -> bool {
        self.f_measure() > threshold
    }

    /// Compute the signed delta between `self` and `other` (self − other).
    pub fn delta(&self, other: &Self) -> ConformanceDelta {
        ConformanceDelta {
            fitness_delta: self.fitness - other.fitness,
            precision_delta: self.precision - other.precision,
            generalization_delta: self.generalization - other.generalization,
            simplicity_delta: self.simplicity - other.simplicity,
        }
    }

    /// All dimensions at 1.0.
    pub fn perfect() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }

    /// All dimensions at 0.0.
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_score_f_measure_is_one() {
        let s = ConformanceScore::perfect();
        assert!((s.f_measure() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn zero_score_f_measure_is_zero() {
        let s = ConformanceScore::zero();
        assert_eq!(s.f_measure(), 0.0);
    }

    #[test]
    fn f_measure_harmonic_mean() {
        let s = ConformanceScore::new(1.0, 0.5, 0.0, 0.0);
        let expected = 2.0 * 1.0 * 0.5 / (1.0 + 0.5);
        assert!((s.f_measure() - expected).abs() < 1e-9);
    }

    #[test]
    fn is_above_threshold_true_when_f_above() {
        let s = ConformanceScore::perfect();
        assert!(s.is_above_threshold(0.9));
    }

    #[test]
    fn is_above_threshold_false_when_f_below() {
        let s = ConformanceScore::zero();
        assert!(!s.is_above_threshold(0.0)); // 0.0 is not strictly above 0.0
    }

    #[test]
    fn delta_computes_signed_difference() {
        let a = ConformanceScore::new(0.9, 0.8, 0.7, 0.6);
        let b = ConformanceScore::new(0.5, 0.3, 0.2, 0.1);
        let d = a.delta(&b);
        assert!((d.fitness_delta - 0.4).abs() < 1e-9);
        assert!((d.precision_delta - 0.5).abs() < 1e-9);
    }

    #[test]
    fn equal_scores_have_zero_delta() {
        let s = ConformanceScore::new(0.7, 0.7, 0.7, 0.7);
        let d = s.delta(&s.clone());
        assert_eq!(d.fitness_delta, 0.0);
    }
}
