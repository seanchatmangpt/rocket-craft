use mech_factory_mud::world::Simulation;

#[test]
fn test_refused_missing_socket() {
    let sim = Simulation::run("refused_missing_socket");
    assert_eq!(sim.report.status, "REFUSED");
    
    // ensure no admitted weapon mount row
    let weapon_mounts: Vec<_> = sim.projections.iter().filter(|p| p.projection_type == "WeaponMount").collect();
    assert!(weapon_mounts.is_empty());
}
