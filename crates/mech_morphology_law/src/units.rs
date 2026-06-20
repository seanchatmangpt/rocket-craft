//! Strongly-typed metric domain. No bare `f64` escapes this boundary; NaN/inf are
//! banned at construction.

use core::fmt;

/// Construction-time metric error. NaN/inf are never representable downstream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MetricError {
    /// A non-finite value (NaN or ±inf) was supplied.
    NonFinite,
    /// A negative value was supplied where only `[0, inf)` is lawful.
    Negative,
}

impl fmt::Display for MetricError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricError::NonFinite => f.write_str("non-finite metric value (NaN/inf banned)"),
            MetricError::Negative => f.write_str("negative metric value (only [0,inf) lawful)"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for MetricError {}

/// A finite length in meters. NaN/inf are rejected at construction.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Meters(f64);

impl Meters {
    /// Construct, guarding finiteness.
    pub fn new(v: f64) -> Result<Self, MetricError> {
        if v.is_finite() {
            Ok(Meters(v))
        } else {
            Err(MetricError::NonFinite)
        }
    }

    /// The raw value (only at the boundary).
    pub fn get(self) -> f64 {
        self.0
    }
}

/// A unit-free ratio in `[0, inf)` — typically ratio-of-body-height.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ratio(f64);

impl Ratio {
    /// Construct a ratio, banning NaN/inf and negatives.
    pub fn new(v: f64) -> Result<Self, MetricError> {
        if !v.is_finite() {
            return Err(MetricError::NonFinite);
        }
        if v < 0.0 {
            return Err(MetricError::Negative);
        }
        Ok(Ratio(v))
    }

    /// The raw value (only at the boundary).
    pub fn get(self) -> f64 {
        self.0
    }

    /// Ratio of two lengths; a zero denominator yields `0.0` (caller treats a
    /// zero body-height as a degenerate measurement upstream).
    pub fn of(part: Meters, body_height: Meters) -> Result<Self, MetricError> {
        let denom = body_height.0;
        let v = if denom == 0.0 { 0.0 } else { part.0 / denom };
        Ratio::new(v.abs())
    }
}

/// An inclusive ratio band `[lo, hi]` mirrored from ontology 116.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Band {
    /// Inclusive lower bound.
    pub lo: Ratio,
    /// Inclusive upper bound.
    pub hi: Ratio,
}

impl Band {
    /// A degenerate `[0,0]` band (total fallback for infallible literal builders).
    pub const ZERO: Band = Band {
        lo: Ratio(0.0),
        hi: Ratio(0.0),
    };

    /// Build a band from raw bounds (infallible for valid literals used by the law).
    pub fn new(lo: f64, hi: f64) -> Result<Self, MetricError> {
        Ok(Band {
            lo: Ratio::new(lo)?,
            hi: Ratio::new(hi)?,
        })
    }

    /// Inclusive containment: `lo <= r <= hi`.
    pub fn contains(&self, r: Ratio) -> bool {
        r.0 >= self.lo.0 && r.0 <= self.hi.0
    }

    /// True when `r` lies exactly on either seam.
    pub fn is_boundary(&self, r: Ratio) -> bool {
        r.0 == self.lo.0 || r.0 == self.hi.0
    }
}

/// Unit-to-meter conversion factor (USD cm units -> meters is `0.01`).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MetersPerUnit(f64);

impl MetersPerUnit {
    /// Construct, guarding finiteness and non-negativity.
    pub fn new(v: f64) -> Result<Self, MetricError> {
        if !v.is_finite() {
            return Err(MetricError::NonFinite);
        }
        if v < 0.0 {
            return Err(MetricError::Negative);
        }
        Ok(MetersPerUnit(v))
    }

    /// The canonical centimeter scale (`0.01`).
    pub fn centimeters() -> Self {
        MetersPerUnit(0.01)
    }

    /// Convert raw units to meters.
    pub fn apply(self, units: f64) -> Result<Meters, MetricError> {
        Meters::new(units * self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bans_non_finite() {
        assert_eq!(Meters::new(f64::NAN), Err(MetricError::NonFinite));
        assert_eq!(Ratio::new(f64::INFINITY), Err(MetricError::NonFinite));
        assert_eq!(Ratio::new(-0.1), Err(MetricError::Negative));
    }

    #[test]
    fn band_inclusive_and_boundary() {
        let b = Band::new(0.08, 0.15).unwrap();
        assert!(b.contains(Ratio::new(0.08).unwrap()));
        assert!(b.contains(Ratio::new(0.15).unwrap()));
        assert!(!b.contains(Ratio::new(0.0799).unwrap()));
        assert!(!b.contains(Ratio::new(0.1501).unwrap()));
        assert!(b.is_boundary(Ratio::new(0.08).unwrap()));
        assert!(!b.is_boundary(Ratio::new(0.10).unwrap()));
    }
}
