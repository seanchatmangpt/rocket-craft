use nexus_gundam::builder::CivilizationBuilder;

fn main() {
    // Attempting to build without calling with_planet.
    // This should fail because the `.build()` method is not implemented
    // for `CivilizationBuilder<Unset>`, only for `CivilizationBuilder<Set<Plan>>`.
    let _civ = CivilizationBuilder::new()
        .with_name("Principality of Zeon")
        .build();
}
