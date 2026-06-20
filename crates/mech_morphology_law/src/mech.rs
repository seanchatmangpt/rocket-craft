//! The `MeasuredMech` aggregate and its safe builder — the sole input to the
//! `Measured` phase.

use crate::axis::UpAxis;
use crate::part::MeasuredPart;
use crate::refusal::MorphologyError;
use crate::units::Meters;
use alloc::vec::Vec;

/// A fully measured mech ready for law validation.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MeasuredMech {
    /// Measured parts (kept sorted by id for canonical hashing).
    pub parts: Vec<MeasuredPart>,
    /// The up-axis declared by the source spec.
    pub declared_up: UpAxis,
    /// The up-axis observed from actual part stacking.
    pub actual_up: UpAxis,
    /// Law-defined skeletal body height (head-top to leg-bottom), NOT the
    /// accessory-inclusive union.
    pub body_height: Meters,
}

impl MeasuredMech {
    /// Find a part by id.
    pub fn part(&self, id: &str) -> Option<&MeasuredPart> {
        self.parts.iter().find(|p| p.id.as_str() == id)
    }
}

/// Safe builder; rejects empty mechs and duplicate ids.
#[derive(Debug, Default)]
pub struct MeasuredMechBuilder {
    parts: Vec<MeasuredPart>,
    declared_up: Option<UpAxis>,
    actual_up: Option<UpAxis>,
    body_height: Option<Meters>,
}

impl MeasuredMechBuilder {
    /// New empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a part.
    pub fn part(mut self, p: MeasuredPart) -> Self {
        self.parts.push(p);
        self
    }

    /// Set both up-axes (declared, actual).
    pub fn up_axes(mut self, declared: UpAxis, actual: UpAxis) -> Self {
        self.declared_up = Some(declared);
        self.actual_up = Some(actual);
        self
    }

    /// Set the skeletal body height.
    pub fn body_height(mut self, h: Meters) -> Self {
        self.body_height = Some(h);
        self
    }

    /// Finalize, validating non-emptiness and id-uniqueness. Parts are sorted by
    /// id for deterministic canonical serialization.
    pub fn build(mut self) -> Result<MeasuredMech, MorphologyError> {
        if self.parts.is_empty() {
            return Err(MorphologyError::Build("mech has no parts"));
        }
        self.parts.sort_by(|a, b| a.id.cmp(&b.id));
        for w in self.parts.windows(2) {
            if w[0].id == w[1].id {
                return Err(MorphologyError::Build("duplicate part id"));
            }
        }
        let declared_up = self
            .declared_up
            .ok_or(MorphologyError::Build("declared up-axis missing"))?;
        let actual_up = self
            .actual_up
            .ok_or(MorphologyError::Build("actual up-axis missing"))?;
        let body_height = self
            .body_height
            .ok_or(MorphologyError::Build("body height missing"))?;
        Ok(MeasuredMech {
            parts: self.parts,
            declared_up,
            actual_up,
            body_height,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axis::{Axis, Bbox};
    use crate::part::{PartClass, PartId};

    fn p(id: &str) -> MeasuredPart {
        let z = |v: f64| Meters::new(v).unwrap();
        MeasuredPart::new(
            PartId::new(id),
            PartClass::TorsoSegment,
            Bbox::new([z(0.0), z(0.0), z(0.0)], [z(1.0), z(1.0), z(1.0)]),
            false,
        )
    }

    #[test]
    fn rejects_empty_and_dup() {
        let r = MeasuredMechBuilder::new().build();
        assert!(r.is_err());
        let r = MeasuredMechBuilder::new()
            .part(p("A"))
            .part(p("A"))
            .up_axes(UpAxis::new(Axis::Y), UpAxis::new(Axis::Y))
            .body_height(Meters::new(1.0).unwrap())
            .build();
        assert!(r.is_err());
    }
}
