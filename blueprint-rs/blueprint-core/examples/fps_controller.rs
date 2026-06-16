//! FPS Controller Blueprint — demonstrates a Character Blueprint in Rust
//!
//! Creates a Character Blueprint with:
//! - Health/MaxHealth/WalkSpeed/IsAlive variables
//! - BeginPlay: set initial spawn location and print a message
//! - Tick: add movement input every frame
//! - Custom "OnDamageTaken" event with Branch for death detection
//! - Branch "then" (died) path prints a death message
//!
//! Run with:
//!     cargo run --example fps_controller -p blueprint-core

use blueprint_core::builder::{BlueprintBuilder, VarType};
use blueprint_core::serializer::T3dSerializer;

fn main() {
    let mut builder = BlueprintBuilder::new("FPSCharacter", "Character");

    // ── Variables ────────────────────────────────────────────────────────────
    builder.add_variable_mut("Health", VarType::Int, Some("100".to_string()));
    builder.add_variable_mut("MaxHealth", VarType::Int, Some("100".to_string()));
    builder.add_variable_mut("WalkSpeed", VarType::Float, Some("600.0".to_string()));
    builder.add_variable_mut("IsAlive", VarType::Bool, Some("true".to_string()));

    // ── BeginPlay ────────────────────────────────────────────────────────────
    // Spawn character slightly above ground (Z=90 = half-height of default capsule)
    let begin_play = builder.begin_play_node();
    let init_location = builder.set_actor_location(0.0, 0.0, 90.0);
    builder.exec_connect(&begin_play, &init_location);

    let init_print = builder.print_string("FPS Character initialized!");
    builder.exec_connect(&init_location, &init_print);

    // ── Tick ─────────────────────────────────────────────────────────────────
    // Every frame: add forward movement input
    let tick = builder.tick_node();
    let move_input = builder.add_movement_input();
    builder.exec_connect(&tick, &move_input);

    // ── Custom event: OnDamageTaken ──────────────────────────────────────────
    let damage_event = builder.custom_event_node("OnDamageTaken");
    let damage_print = builder.print_string("Player took damage!");
    builder.exec_connect(&damage_event, &damage_print);

    // Branch: if IsAlive == false → death path
    let death_check = builder.branch_node();
    builder.exec_connect(&damage_print, &death_check);

    // True branch (health depleted → died)
    let death_print = builder.print_string("Player died!");
    builder.connect(&death_check, "then", &death_print, "execute");

    // False branch (still alive)
    let alive_print = builder.print_string("Player survived the hit!");
    builder.connect(&death_check, "else", &alive_print, "execute");

    // ── Serialize ────────────────────────────────────────────────────────────
    let bp = builder.build();

    println!("=== FPS Character Blueprint (T3D) ===");
    println!("{}", T3dSerializer::serialize(&bp));
    println!("=== End ===");
    println!();
    println!("Blueprint summary:");
    println!("  Name:       {}", bp.name);
    println!("  Parent:     {}", bp.parent_class);
    println!("  Variables:  {}", bp.variables.len());
    for v in &bp.variables {
        println!(
            "    {} ({:?}) = {}",
            v.name,
            v.var_type.category,
            v.default_value.as_deref().unwrap_or("<none>")
        );
    }
    println!("  Graphs:     {}", bp.graphs.len());
    for g in &bp.graphs {
        println!("    {} ({} nodes)", g.name, g.nodes.len());
    }
    println!();
    println!("Paste the T3D output above into the UE4 Blueprint Editor Event Graph.");
}
