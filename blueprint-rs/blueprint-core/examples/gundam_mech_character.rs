//! gundam_mech_character — Gundam-themed UE4 Character Blueprint
//!
//! Generates a GundamMechCharacter Blueprint with:
//!   - A "GundamMesh" string variable (proxy for SkeletalMeshComponent ref)
//!   - A "PilotName" string variable
//!   - An "ArmorPoints" integer variable
//!   - A "BeamSaberActive" boolean variable
//!   - BeginPlay: initialises the mech and prints cockpit status
//!   - EventTick: stub system update
//!   - Custom event "ActivateBeamSaber": arms the beam saber → sets BeamSaberActive=true
//!   - Custom event "DeactivateBeamSaber": disarms the beam saber → sets BeamSaberActive=false
//!   - Custom event "TakeMechDamage": damage handler with Branch → destroyed / absorbing
//!
//! Run: cargo run --example gundam_mech_character -p blueprint-core > /tmp/GundamMechCharacter.T3D

use blueprint_core::ast::Pin;
use blueprint_core::builder::{BlueprintBuilder, NodeHandle, VarType};
use blueprint_core::nodes;
use blueprint_core::types::{PinDirection, PinType};
use blueprint_core::serializer::T3dSerializer;

fn main() {
    // -----------------------------------------------------------------------
    // Build the Blueprint using the fluent builder.
    // All nodes are added to the default EventGraph.
    // -----------------------------------------------------------------------
    let mut b = BlueprintBuilder::new("GundamMechCharacter", "Character");

    // ------------------------------------------------------------------
    // Variables
    // ------------------------------------------------------------------
    // GundamMesh — declared as a String variable; in UE4 set the type to
    // SkeletalMeshComponent in the Blueprint editor after import.
    b.add_variable_mut("GundamMesh",      VarType::String, Some("None".to_string()));
    b.add_variable_mut("PilotName",       VarType::String, Some("Amuro Ray".to_string()));
    b.add_variable_mut("ArmorPoints",     VarType::Int,    Some("1000".to_string()));
    b.add_variable_mut("MaxArmorPoints",  VarType::Int,    Some("1000".to_string()));
    b.add_variable_mut("BeamSaberActive", VarType::Bool,   Some("false".to_string()));
    b.add_variable_mut("IsDestroyed",     VarType::Bool,   Some("false".to_string()));

    // ------------------------------------------------------------------
    // BeginPlay: cockpit initialisation
    //   BeginPlay → "Gundam systems online!" → "Cockpit pressurised"
    // ------------------------------------------------------------------
    let begin_play   = b.begin_play_node();
    let init_msg     = b.print_string("Gundam systems online! RX-78-2 initiating...");
    b.exec_connect(&begin_play, &init_msg);
    let cockpit_msg  = b.print_string("GundamMech: Cockpit pressurised. Pilot ready for launch.");
    b.exec_connect(&init_msg, &cockpit_msg);

    // ------------------------------------------------------------------
    // EventTick: stub system update
    // ------------------------------------------------------------------
    let tick         = b.tick_node();
    let tick_msg     = b.print_string("[GundamMech] Tick: all systems nominal.");
    b.exec_connect(&tick, &tick_msg);

    // ------------------------------------------------------------------
    // Custom event: ActivateBeamSaber
    //   ActivateBeamSaber → "BEAM SABER ACTIVATED"
    //   (SetVariable node wired at AST level below)
    // ------------------------------------------------------------------
    let beam_activate  = b.custom_event_node("ActivateBeamSaber");
    let beam_on_msg    = b.print_string("BEAM SABER ACTIVATED — En garde!");
    b.exec_connect(&beam_activate, &beam_on_msg);

    // ------------------------------------------------------------------
    // Custom event: DeactivateBeamSaber
    //   DeactivateBeamSaber → "Beam Saber deactivated"
    //   (SetVariable node wired at AST level below)
    // ------------------------------------------------------------------
    let beam_deactivate = b.custom_event_node("DeactivateBeamSaber");
    let beam_off_msg    = b.print_string("Beam Saber deactivated. Returning to standby.");
    b.exec_connect(&beam_deactivate, &beam_off_msg);

    // ------------------------------------------------------------------
    // Custom event: TakeMechDamage
    //   TakeMechDamage → "IMPACT DETECTED" → Branch
    //                                          ↳ true  → "CRITICAL — destroyed!"
    //                                          ↳ false → "Armor holding"
    // ------------------------------------------------------------------
    let take_damage    = b.custom_event_node("TakeMechDamage");
    let hit_msg        = b.print_string("IMPACT DETECTED — Mech sustaining damage!");
    b.exec_connect(&take_damage, &hit_msg);

    let branch         = b.branch_node();
    b.exec_connect(&hit_msg, &branch);

    let destroyed_msg  = b.print_string("CRITICAL — GundamMech destroyed! Ejecting pilot...");
    b.connect(&branch, "true", &destroyed_msg, "execute");

    let absorb_msg     = b.print_string("Armor holding. Continuing combat operations.");
    b.connect(&branch, "false", &absorb_msg, "execute");

    // -----------------------------------------------------------------------
    // Consume builder → get the Blueprint AST, then add raw AST nodes
    // -----------------------------------------------------------------------
    let mut bp = b.build();

    {
        let eg = bp.event_graph();

        // SetBeamSaberActive = true  (activated after beam_on_msg prints)
        let set_saber_on = nodes::set_variable(
            "SetBeamSaberActive_On",
            "BeamSaberActive",
            PinType::bool(),
        )
        .at(2000, 200)
        // Set the input pin default to true
        .with_pin({
            // Pin already added by set_variable; we patch after add_node below.
            // Use a dummy pin here — we'll patch after insertion instead.
            // (Using a workaround: set_variable already adds the pins; we just
            //  set the default on the node after adding it to the graph.)
            Pin::data_input("_dummy", PinType::bool())  // placeholder; removed via patch
        });

        // Actually, just create the node cleanly without the dummy pin:
        // Re-create without the dummy trick.
        let mut set_saber_on = nodes::set_variable(
            "SetBeamSaberActive_On",
            "BeamSaberActive",
            PinType::bool(),
        )
        .at(2000, 200);
        if let Some(p) = set_saber_on.find_pin_mut("BeamSaberActive") {
            if p.direction == PinDirection::Input {
                p.default_value = Some("true".to_string());
            }
        }
        eg.add_node(set_saber_on);

        // SetBeamSaberActive = false (deactivated after beam_off_msg prints)
        let mut set_saber_off = nodes::set_variable(
            "SetBeamSaberActive_Off",
            "BeamSaberActive",
            PinType::bool(),
        )
        .at(2000, 500);
        if let Some(p) = set_saber_off.find_pin_mut("BeamSaberActive") {
            if p.direction == PinDirection::Input {
                p.default_value = Some("false".to_string());
            }
        }
        eg.add_node(set_saber_off);

        // Wire: beam_on_msg.then → SetBeamSaberActive_On.execute
        // beam_on_msg node name = beam_on_msg.name (from builder handle)
        eg.connect(&beam_on_msg.name, "then", "SetBeamSaberActive_On", "execute");

        // Wire: beam_off_msg.then → SetBeamSaberActive_Off.execute
        eg.connect(&beam_off_msg.name, "then", "SetBeamSaberActive_Off", "execute");
    }

    // -----------------------------------------------------------------------
    // Serialise to T3D and print to stdout
    // -----------------------------------------------------------------------
    let t3d = T3dSerializer::serialize(&bp);
    print!("{}", t3d);
}
