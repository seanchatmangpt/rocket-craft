//! Integration tests: UFO-disc negative fixture, in-band admission, boundary
//! seams, real-mech honesty, measurability, freshness, and determinism. Mirrors
//! `scripts/verify_metric_morphology.py`.

use mech_morphology_law::prelude::*;

fn m(v: f64) -> Meters {
    Meters::new(v).unwrap()
}

fn part(id: &str, class: PartClass, zmin: f64, zmax: f64) -> MeasuredPart {
    MeasuredPart::new(
        PartId::new(id),
        class,
        Bbox::new([m(0.0), m(0.0), m(zmin)], [m(1.0), m(1.0), m(zmax)]),
        false,
    )
}

/// Canonical in-band bipedal mech (archetype bands from ontology 116): head 0.130
/// (band 0.08-0.15), torso 0.370 (0.30-0.45), leg 0.630 (0.55-0.85).
/// Anatomy: head above torso, leg below torso. body_height 18.
fn in_band_mech() -> MeasuredMech {
    MeasuredMechBuilder::new()
        .part(part("SM_Head", PartClass::MechaCrown, 30.0, 32.34))
        .part(part("SM_Torso", PartClass::TorsoSegment, 20.0, 26.66))
        .part(part("SM_Limb_Left", PartClass::BipedalLeg, 0.0, 11.34))
        .up_axes(UpAxis::new(Axis::Z), UpAxis::new(Axis::Z))
        .body_height(m(18.0))
        .build()
        .unwrap()
}

#[test]
fn in_band_fixture_admits() {
    let machine = Machine::<BipedalMetricEnvelopeLaw, Measured>::new(in_band_mech());
    let validated = machine.validate();
    assert_eq!(validated.standing(), &Standing::Admitted);
    let admitted = validated.admit();
    assert!(admitted.is_ok(), "in-band mech must reach Admitted phase");
}

#[test]
fn ufo_disc_negative_fixture_refused() {
    // head Y_min (0..2) below torso Y_max (6..12) -> ANATOMY_PARADOX;
    // shield ratio 1.0 with no archetype -> REFUSE_DEFAULT_SHIELD_PROPORTION.
    let mech = MeasuredMechBuilder::new()
        .part(part("SM_Head", PartClass::MechaCrown, 0.0, 2.0))
        .part(part("SM_Torso", PartClass::TorsoSegment, 6.0, 12.84))
        .part(part("SM_Disc", PartClass::Shield, 0.0, 18.0)) // span 18 / 18 = 1.0
        .up_axes(UpAxis::new(Axis::Z), UpAxis::new(Axis::Z))
        .body_height(m(18.0))
        .build()
        .unwrap();

    let out = BipedalMetricEnvelopeLaw::new().validate(&mech);
    match &out.standing {
        Standing::Refused { codes } => {
            assert!(codes.contains(&RefusalCode::AnatomyParadox));
            assert!(codes.contains(&RefusalCode::RefuseDefaultShieldProportion));
        }
        other => panic!("UFO disc must be Refused, got {:?}", other),
    }

    // admit() unreachable: it does not exist for Measured; from Validated -> ClaimHold.
    let machine = Machine::<BipedalMetricEnvelopeLaw, Measured>::with_law(
        BipedalMetricEnvelopeLaw::new(),
        mech,
    );
    let claim = machine.validate().admit();
    assert!(claim.is_err(), "refused mech must CLAIM_HOLD, never admit");
}

#[test]
fn band_edges_inclusive_and_exclusive() {
    // Head band [0.0800, 0.1500] (archetype 116) with body_height 1000 ->
    // span = ratio*1000. Torso at 0.370 (in band), placed below the head.
    let mk = |span: f64| {
        MeasuredMechBuilder::new()
            .part(part("SM_Head", PartClass::MechaCrown, 800.0, 800.0 + span))
            .part(part("SM_Torso", PartClass::TorsoSegment, 300.0, 670.0)) // 0.370, below head
            .up_axes(UpAxis::new(Axis::Z), UpAxis::new(Axis::Z))
            .body_height(m(1000.0))
            .build()
            .unwrap()
    };
    let law = BipedalMetricEnvelopeLaw::new();
    assert_eq!(law.validate(&mk(80.0)).standing, Standing::Admitted); // 0.0800 inclusive
    assert_eq!(law.validate(&mk(150.0)).standing, Standing::Admitted); // 0.1500 inclusive
    assert!(matches!(
        law.validate(&mk(79.9)).standing,
        Standing::PartialAlive { .. }
    )); // 0.0799 outside
    assert!(matches!(
        law.validate(&mk(150.1)).standing,
        Standing::PartialAlive { .. }
    )); // 0.1501 outside
}

#[test]
fn real_mech_partial_alive_with_orientation_defect() {
    // head off-band + Z-vs-Y up-axis disagreement -> PartialAlive + orientation.
    let mech = MeasuredMechBuilder::new()
        .part(part("SM_Head", PartClass::MechaCrown, 100.0, 105.0)) // 0.05 off-band low
        .part(part("SM_Torso", PartClass::TorsoSegment, 30.0, 68.0)) // 0.38 in band
        .up_axes(UpAxis::new(Axis::Z), UpAxis::new(Axis::Y)) // declared Z, actual Y (disagreement)
        .body_height(m(100.0))
        .build()
        .unwrap();
    let out = BipedalMetricEnvelopeLaw::new().validate(&mech);
    assert!(matches!(out.standing, Standing::PartialAlive { .. }));
    assert!(out.refusals.contains(&RefusalCode::OrientationDefect));
    let of = out.orientation.expect("orientation finding present");
    assert!(of.is_defect());

    let claim = Machine::<BipedalMetricEnvelopeLaw, Measured>::new(mech)
        .validate()
        .admit();
    assert!(claim.is_err(), "partial-alive must CLAIM_HOLD");
}

#[test]
fn rotated_part_blocks_admission() {
    let rotated = MeasuredPart::new(
        PartId::new("SM_Blade_Left"),
        PartClass::MechaWeapon,
        Bbox::new([m(0.0), m(0.0), m(0.0)], [m(1.0), m(1.0), m(3.0)]),
        true, // has_rotation
    );
    let mech = MeasuredMechBuilder::new()
        .part(part("SM_Head", PartClass::MechaCrown, 14.0, 16.0))
        .part(part("SM_Torso", PartClass::TorsoSegment, 6.0, 12.84))
        .part(rotated)
        .up_axes(UpAxis::new(Axis::Z), UpAxis::new(Axis::Z))
        .body_height(m(18.0))
        .build()
        .unwrap();
    let out = BipedalMetricEnvelopeLaw::new().validate(&mech);
    assert!(out.refusals.iter().any(|c| matches!(
        c,
        RefusalCode::UnmeasurablePart { .. }
    )));
    assert_ne!(out.standing, Standing::Admitted);
}

#[test]
fn archetype_widens_shield_band() {
    let mech = MeasuredMechBuilder::new()
        .part(part("SM_Head", PartClass::MechaCrown, 30.0, 32.34)) // 0.130 in band, above torso
        .part(part("SM_Torso", PartClass::TorsoSegment, 20.0, 26.66)) // 0.370 in band
        .part(part("SM_Limb_Left", PartClass::BipedalLeg, 0.0, 11.34)) // 0.630 in band, below torso
        .part(part("SM_Shield", PartClass::Shield, 0.0, 17.1)) // 17.1/18 = 0.95 exception
        .up_axes(UpAxis::new(Axis::Z), UpAxis::new(Axis::Z))
        .body_height(m(18.0))
        .build()
        .unwrap();
    // Without archetype -> refused proportion.
    let no = BipedalMetricEnvelopeLaw::new().validate(&mech);
    assert!(no
        .refusals
        .contains(&RefusalCode::RefuseDefaultShieldProportion));
    // With Tower archetype -> shield ok (admitted).
    let yes =
        BipedalMetricEnvelopeLaw::with_archetype(ArchetypeException::Tower).validate(&mech);
    assert_eq!(yes.standing, Standing::Admitted);
}

#[test]
fn determinism_replay() {
    let a = BipedalMetricEnvelopeLaw::new().validate(&in_band_mech());
    let b = BipedalMetricEnvelopeLaw::new().validate(&in_band_mech());
    assert_eq!(a, b);
}

#[cfg(feature = "blake3hash")]
#[test]
fn freshness_blocks_stale_receipt() {
    let law = BipedalMetricEnvelopeLaw::new();
    let mech = in_band_mech();
    let out = law.validate(&mech);
    let r = ReplayReceipt::build("run", law.id(), &mech, &out, None);
    assert!(r.verify_fresh(&mech).is_ok());

    // mutate the mech (different torso) -> stale.
    let mutated = MeasuredMechBuilder::new()
        .part(part("SM_Head", PartClass::MechaCrown, 14.0, 16.0))
        .part(part("SM_Torso", PartClass::TorsoSegment, 6.0, 13.0))
        .part(part("SM_Limb_Left", PartClass::BipedalLeg, -3.0, 5.64))
        .up_axes(UpAxis::new(Axis::Z), UpAxis::new(Axis::Z))
        .body_height(m(18.0))
        .build()
        .unwrap();
    assert!(r.verify_fresh(&mutated).is_err());
}
