use chicago_tdd_tools::discover_games;

#[test]
fn should_discover_infinity_blade_and_gundam_nexus() {
    let discovered = discover_games();
    assert_eq!(discovered.len(), 2);

    let ib4 = discovered
        .iter()
        .find(|g| g.crate_name == "ib4-mud")
        .expect("IB4");
    assert_eq!(ib4.name, "Infinity Blade 4 MUD");
    assert_eq!(
        ib4.details,
        "Discovered player session for: DiscoveryTester"
    ); // I need the correct string. Let's see.

    let gundam = discovered
        .iter()
        .find(|g| g.crate_name == "nexus-session")
        .expect("Gundam");
    assert_eq!(gundam.name, "Gundam Nexus");
    assert_eq!(
        gundam.details,
        "Discovered Gundam pilot session: NexusTester (ID: 1001)"
    );
}
