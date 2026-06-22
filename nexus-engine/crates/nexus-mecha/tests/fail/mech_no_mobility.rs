use nexus_mecha::builder::MechBuilder;
use nexus_mecha::mech_primitives::MechaFrame;

fn main() {
    // Attempting to build without calling with_mobility, with_power, with_armor, or with_sensor.
    // This should fail because `.build()` is not implemented for `MechBuilder<Set<MechaFrame>, Unset, Unset, Unset, Unset>`.
    let _mech = MechBuilder::new()
        .with_frame(MechaFrame)
        .build();
}
