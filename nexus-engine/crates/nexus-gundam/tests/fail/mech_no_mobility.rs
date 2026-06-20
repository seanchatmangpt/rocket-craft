use nexus_gundam::builder::MechBuilder;
use nexus_gundam::mech_primitives::GundamFrame;

fn main() {
    // Attempting to build without calling with_mobility, with_power, with_armor, or with_sensor.
    // This should fail because `.build()` is not implemented for `MechBuilder<Set<GundamFrame>, Unset, Unset, Unset, Unset>`.
    let _mech = MechBuilder::new()
        .with_frame(GundamFrame)
        .build();
}
