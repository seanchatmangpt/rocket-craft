//! Property-based invariants: any ratio strictly inside a class band admits that
//! part's contribution; strictly outside refuses.

use mech_morphology_law::prelude::*;
use proptest::prelude::*;

fn m(v: f64) -> Meters {
    Meters::new(v).unwrap()
}

// Build a minimal mech with a single head whose span yields `ratio` (body=100).
fn head_mech(ratio: f64) -> MeasuredMech {
    let span = ratio * 100.0;
    MeasuredMechBuilder::new()
        .part(MeasuredPart::new(
            PartId::new("SM_Head"),
            PartClass::MechaCrown,
            Bbox::new(
                [m(0.0), m(0.0), m(200.0)],
                [m(1.0), m(1.0), m(200.0 + span)],
            ),
            false,
        ))
        .part(MeasuredPart::new(
            PartId::new("SM_Torso"),
            PartClass::TorsoSegment,
            // torso 0.130 (band 0.1190-0.1390), clearly below head (head min z=200).
            Bbox::new([m(0.0), m(0.0), m(30.0)], [m(1.0), m(1.0), m(43.0)]),
            false,
        ))
        .up_axes(UpAxis::new(Axis::Z), UpAxis::new(Axis::Z))
        .body_height(m(100.0))
        .build()
        .unwrap()
}

proptest! {
    // Head band is [0.1190, 0.1390] (ontology 116). Strictly inside -> Admitted.
    #[test]
    fn inside_head_band_admits(r in 0.1191f64..0.1389f64) {
        let out = BipedalMetricEnvelopeLaw::new().validate(&head_mech(r));
        prop_assert_eq!(out.standing, Standing::Admitted);
    }

    // Strictly below -> a height-band refusal and not Admitted.
    #[test]
    fn below_head_band_refuses(r in 0.0f64..0.1189f64) {
        let out = BipedalMetricEnvelopeLaw::new().validate(&head_mech(r));
        prop_assert_ne!(out.standing.clone(), Standing::Admitted);
        let has_band = out.refusals.iter().any(|c| matches!(c, RefusalCode::RefusePartHeightBand { .. }));
        prop_assert!(has_band);
    }

    // Strictly above -> refusal and not Admitted.
    #[test]
    fn above_head_band_refuses(r in 0.1391f64..2.0f64) {
        let out = BipedalMetricEnvelopeLaw::new().validate(&head_mech(r));
        prop_assert_ne!(out.standing.clone(), Standing::Admitted);
        let has_band = out.refusals.iter().any(|c| matches!(c, RefusalCode::RefusePartHeightBand { .. }));
        prop_assert!(has_band);
    }
}

proptest! {
    // Shield exception band [0.85,1.10] with an archetype always classifies Ok.
    #[test]
    fn exception_with_archetype_ok(r in 0.85f64..=1.10f64) {
        let tier = classify_shield(Ratio::new(r).unwrap(), Some(ArchetypeException::Siege));
        prop_assert!(tier.is_ok());
    }

    // Same band with no archetype always refuses.
    #[test]
    fn exception_without_archetype_refuses(r in 0.8501f64..=1.10f64) {
        let res = classify_shield(Ratio::new(r).unwrap(), None);
        prop_assert_eq!(res, Err(RefusalCode::RefuseDefaultShieldProportion));
    }
}
