use nexus_gundam::mech_primitives::{
    check_integration_grammar, reference_fabric_grammar, AppendageIntegration,
    DestructionRelationship, MaterialAssignment, MountSocket, RotationLimits, AABB,
};

fn valid_appendage(id: &str) -> AppendageIntegration {
    AppendageIntegration {
        appendage_id: id.into(),
        root_mount: MountSocket {
            name: format!("socket_{id}"),
            translate: [0.0, 0.0, 0.0],
            rotate_xyz: [0.0, 0.0, 0.0],
        },
        mechanical_transition: RotationLimits::default(),
        armor_cover: AABB::new([-10.0, -10.0, -10.0], [10.0, 10.0, 10.0]),
        child_socket: None,
        is_load_bearing: true,
        material: MaterialAssignment {
            slot: "primary".into(),
            material_id: "M_WhiteArmor".into(),
        },
        destruction: DestructionRelationship {
            detaches_from: "Torso".into(),
            damage_threshold: 50.0,
        },
    }
}

// ── Grammar passes ───────────────────────────────────────────────────────────

#[test]
fn reference_fabric_grammar_passes() {
    let grammar = reference_fabric_grammar();
    assert!(
        check_integration_grammar(&grammar).is_ok(),
        "reference_fabric_grammar() must satisfy its own grammar"
    );
}

#[test]
fn empty_appendage_list_passes() {
    assert!(check_integration_grammar(&[]).is_ok());
}

#[test]
fn single_valid_appendage_passes() {
    let a = valid_appendage("Head");
    assert!(check_integration_grammar(&[a]).is_ok());
}

#[test]
fn reference_grammar_has_five_appendages() {
    let grammar = reference_fabric_grammar();
    assert_eq!(grammar.len(), 5, "Head + 2 wings + 2 blades");
}

#[test]
fn reference_grammar_includes_all_expected_parts() {
    let grammar = reference_fabric_grammar();
    let ids: Vec<&str> = grammar.iter().map(|a| a.appendage_id.as_str()).collect();
    assert!(ids.contains(&"Head"));
    assert!(ids.contains(&"WingArray_Left"));
    assert!(ids.contains(&"WingArray_Right"));
    assert!(ids.contains(&"Blade_Left"));
    assert!(ids.contains(&"Blade_Right"));
}

// ── Grammar violations — property 1 (root_mount.name) ───────────────────────

#[test]
fn empty_root_mount_name_is_violation() {
    let mut a = valid_appendage("Arm");
    a.root_mount.name = String::new();
    let result = check_integration_grammar(&[a]);
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert_eq!(violations[0].missing_property, "root_mount.name");
}

// ── Grammar violations — property 3 (armor_cover) ───────────────────────────

#[test]
fn degenerate_armor_cover_is_violation() {
    let mut a = valid_appendage("Wing");
    a.armor_cover = AABB::default(); // min == max == [0,0,0]
    let result = check_integration_grammar(&[a]);
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert!(violations[0].missing_property.contains("armor_cover"));
}

// ── Grammar violations — property 6 (material) ──────────────────────────────

#[test]
fn empty_material_slot_is_violation() {
    let mut a = valid_appendage("Blade");
    a.material.slot = String::new();
    let result = check_integration_grammar(&[a]);
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert!(violations.iter().any(|v| v.missing_property.contains("material")));
}

#[test]
fn empty_material_id_is_violation() {
    let mut a = valid_appendage("Blade");
    a.material.material_id = String::new();
    let result = check_integration_grammar(&[a]);
    assert!(result.is_err());
}

// ── Grammar violations — property 7 (destruction) ───────────────────────────

#[test]
fn empty_destruction_detaches_from_is_violation() {
    let mut a = valid_appendage("Blade");
    a.destruction.detaches_from = String::new();
    let result = check_integration_grammar(&[a]);
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert!(violations.iter().any(|v| v.missing_property.contains("destruction")));
}

// ── Multiple violations accumulate ───────────────────────────────────────────

#[test]
fn multiple_bad_appendages_all_reported() {
    let mut a1 = valid_appendage("PartA");
    a1.root_mount.name = String::new(); // violation 1

    let mut a2 = valid_appendage("PartB");
    a2.material.slot = String::new(); // violation 2

    let result = check_integration_grammar(&[a1, a2]);
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert_eq!(violations.len(), 2);
    assert_eq!(violations[0].appendage_id, "PartA");
    assert_eq!(violations[1].appendage_id, "PartB");
}

// ── Socket layout correctness for reference fabric ───────────────────────────

#[test]
fn wing_arrays_are_symmetric_on_x_axis() {
    let grammar = reference_fabric_grammar();
    let wl = grammar.iter().find(|a| a.appendage_id == "WingArray_Left").unwrap();
    let wr = grammar.iter().find(|a| a.appendage_id == "WingArray_Right").unwrap();
    // X coordinates must be equal-and-opposite
    let lx = wl.root_mount.translate[0];
    let rx = wr.root_mount.translate[0];
    assert!((lx + rx).abs() < 0.01, "wing X mounts must be symmetric: {lx} + {rx} ≠ 0");
    // Y and Z must match
    assert!((wl.root_mount.translate[1] - wr.root_mount.translate[1]).abs() < 0.01);
    assert!((wl.root_mount.translate[2] - wr.root_mount.translate[2]).abs() < 0.01);
}

#[test]
fn blades_are_symmetric_on_x_axis() {
    let grammar = reference_fabric_grammar();
    let bl = grammar.iter().find(|a| a.appendage_id == "Blade_Left").unwrap();
    let br = grammar.iter().find(|a| a.appendage_id == "Blade_Right").unwrap();
    let lx = bl.root_mount.translate[0];
    let rx = br.root_mount.translate[0];
    assert!((lx + rx).abs() < 0.01, "blade X mounts must be symmetric: {lx} + {rx} ≠ 0");
}

#[test]
fn head_is_above_torso_origin() {
    let grammar = reference_fabric_grammar();
    let head = grammar.iter().find(|a| a.appendage_id == "Head").unwrap();
    assert!(head.root_mount.translate[1] > 100.0, "head must be at least 100 cm above origin");
}

#[test]
fn wings_are_above_blades_in_y() {
    let grammar = reference_fabric_grammar();
    let wing = grammar.iter().find(|a| a.appendage_id == "WingArray_Left").unwrap();
    let blade = grammar.iter().find(|a| a.appendage_id == "Blade_Left").unwrap();
    assert!(
        wing.root_mount.translate[1] > blade.root_mount.translate[1],
        "shoulder socket must be above arm socket"
    );
}

#[test]
fn all_appendages_attach_to_torso() {
    let grammar = reference_fabric_grammar();
    for a in &grammar {
        assert_eq!(
            a.destruction.detaches_from, "Torso",
            "{} must detach from Torso", a.appendage_id
        );
    }
}
