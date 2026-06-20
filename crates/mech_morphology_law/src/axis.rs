//! Axis / up-axis / bounding-box geometry. The up-axis disagreement defect
//! (parts stack along Z while USD declares `upAxis=Y`) is a first-class
//! [`OrientationFinding`], never silently resolved.

use crate::units::Meters;

/// A cartesian axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Axis {
    /// X axis.
    X,
    /// Y axis.
    Y,
    /// Z axis.
    Z,
}

impl Axis {
    /// Array index into a `[T; 3]` ordered `[X, Y, Z]`.
    pub fn index(self) -> usize {
        match self {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        }
    }

    /// Stable token used in canonical serialization.
    pub fn token(self) -> &'static str {
        match self {
            Axis::X => "X",
            Axis::Y => "Y",
            Axis::Z => "Z",
        }
    }
}

/// An up-axis, distinguishing a spec-declared axis from an actual-stacking axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UpAxis(pub Axis);

impl UpAxis {
    /// Construct from an axis.
    pub fn new(a: Axis) -> Self {
        UpAxis(a)
    }

    /// The underlying axis.
    pub fn axis(self) -> Axis {
        self.0
    }
}

/// An axis-aligned bounding box in meters.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bbox {
    /// Minimum corner `[x, y, z]`.
    pub min: [Meters; 3],
    /// Maximum corner `[x, y, z]`.
    pub max: [Meters; 3],
}

impl Bbox {
    /// Construct from corners.
    pub fn new(min: [Meters; 3], max: [Meters; 3]) -> Self {
        Bbox { min, max }
    }

    /// Extent (max - min) along `axis`.
    pub fn span(&self, axis: Axis) -> f64 {
        let i = axis.index();
        self.max[i].get() - self.min[i].get()
    }

    /// `(min, max)` along `axis`.
    pub fn extent_along(&self, axis: Axis) -> (f64, f64) {
        let i = axis.index();
        (self.min[i].get(), self.max[i].get())
    }
}

/// The orientation defect made explicit: the declared up-axis disagrees with the
/// observed part-stacking axis. Surfacing this is law R5 / gap-3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OrientationFinding {
    /// The up-axis declared by spec (authoritative-by-spec).
    pub declared: UpAxis,
    /// The actual axis parts stack along (observed).
    pub actual: UpAxis,
}

impl OrientationFinding {
    /// True when declared and actual disagree (a defect).
    pub fn is_defect(&self) -> bool {
        self.declared.axis() != self.actual.axis()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orientation_defect_detected() {
        let f = OrientationFinding {
            declared: UpAxis::new(Axis::Y),
            actual: UpAxis::new(Axis::Z),
        };
        assert!(f.is_defect());
        let ok = OrientationFinding {
            declared: UpAxis::new(Axis::Y),
            actual: UpAxis::new(Axis::Y),
        };
        assert!(!ok.is_defect());
    }
}
