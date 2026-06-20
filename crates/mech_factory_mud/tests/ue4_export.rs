use mech_factory_mud::world::Simulation;

#[test]
fn test_generated_header_disagrees_with_csv() {
    let sim = Simulation::run("FALSIFY_UE4_HEADER_CSV_MISMATCH");
    assert_eq!(sim.report.status, "REFUSED");
    assert_eq!(sim.report.reason, Some("UE4_HEADER_CSV_MISMATCH".to_string()));
}
