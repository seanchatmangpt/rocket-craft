//! Deterministic canonical byte serialization for hashing. SINGLE tolerance:
//! every float is rendered at exactly 6 decimal places, once. There is no second
//! `1e-3` rounding pass (closes the double-rounding med-defect).

use crate::axis::Bbox;
use crate::law::LawOutcome;
use crate::mech::MeasuredMech;
use crate::refusal::{RefusalCode, Standing};
use alloc::string::String;
use core::fmt::Write;

/// Fixed precision used everywhere a float is canonicalized.
const PRECISION: usize = 6;

fn push_f64(buf: &mut String, v: f64) {
    // core::fmt with explicit precision is deterministic across platforms.
    let _ = write!(buf, "{:.*}", PRECISION, v);
}

fn push_bbox(buf: &mut String, b: &Bbox) {
    for m in &b.min {
        push_f64(buf, m.get());
        buf.push(',');
    }
    buf.push('|');
    for m in &b.max {
        push_f64(buf, m.get());
        buf.push(',');
    }
}

/// Canonicalize a measured mech. Parts are already sorted by id at build time;
/// `has_rotation` is included so a rotated leaf changes the hash.
pub fn canonical_mech(mech: &MeasuredMech) -> String {
    let mut buf = String::new();
    buf.push_str("MECH\n");
    let _ = writeln!(buf, "up_declared={}", mech.declared_up.axis().token());
    let _ = writeln!(buf, "up_actual={}", mech.actual_up.axis().token());
    buf.push_str("body_height=");
    push_f64(&mut buf, mech.body_height.get());
    buf.push('\n');
    for p in &mech.parts {
        let _ = write!(buf, "PART {} {} ", p.id.as_str(), p.class.token());
        push_bbox(&mut buf, &p.bbox);
        let _ = writeln!(buf, " rot={}", p.has_rotation as u8);
    }
    buf
}

fn push_refusal(buf: &mut String, c: &RefusalCode) {
    match c {
        RefusalCode::AnatomyParadox => buf.push_str("ANATOMY_PARADOX"),
        RefusalCode::RefuseDefaultShieldProportion => {
            buf.push_str("REFUSE_DEFAULT_SHIELD_PROPORTION")
        }
        RefusalCode::RefusePartHeightBand { part, ratio, band } => {
            let _ = write!(buf, "REFUSE_PART_HEIGHT_BAND({})[", part.as_str());
            push_f64(buf, ratio.get());
            buf.push(';');
            push_f64(buf, band.lo.get());
            buf.push(';');
            push_f64(buf, band.hi.get());
            buf.push(']');
        }
        RefusalCode::OrientationDefect => buf.push_str("ORIENTATION_DEFECT"),
        RefusalCode::UnmeasurablePart { part } => {
            let _ = write!(buf, "UNMEASURABLE_PART({})", part.as_str());
        }
    }
}

/// Canonicalize a law outcome (standing + sorted refusals + orientation).
pub fn canonical_outcome(o: &LawOutcome) -> String {
    let mut buf = String::new();
    buf.push_str("OUTCOME\nstanding=");
    match &o.standing {
        Standing::Admitted => buf.push_str("ADMITTED"),
        Standing::PartialAlive { .. } => buf.push_str("PARTIAL_ALIVE"),
        Standing::Refused { .. } => buf.push_str("REFUSED"),
        Standing::Unknown => buf.push_str("UNKNOWN"),
    }
    buf.push('\n');
    for c in &o.refusals {
        buf.push_str("R ");
        push_refusal(&mut buf, c);
        buf.push('\n');
    }
    if let Some(of) = &o.orientation {
        let _ = writeln!(
            buf,
            "ORIENT declared={} actual={}",
            of.declared.axis().token(),
            of.actual.axis().token()
        );
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axis::{Axis, UpAxis};
    use crate::mech::MeasuredMechBuilder;
    use crate::part::{MeasuredPart, PartClass, PartId};
    use crate::units::Meters;

    #[test]
    fn canonical_is_deterministic() {
        let z = |v: f64| Meters::new(v).unwrap();
        let mk = || {
            MeasuredMechBuilder::new()
                .part(MeasuredPart::new(
                    PartId::new("SM_Head"),
                    PartClass::MechaCrown,
                    Bbox::new([z(0.0), z(0.0), z(14.0)], [z(1.0), z(1.0), z(16.0)]),
                    false,
                ))
                .up_axes(UpAxis::new(Axis::Y), UpAxis::new(Axis::Y))
                .body_height(z(18.0))
                .build()
                .unwrap()
        };
        assert_eq!(canonical_mech(&mk()), canonical_mech(&mk()));
    }
}
