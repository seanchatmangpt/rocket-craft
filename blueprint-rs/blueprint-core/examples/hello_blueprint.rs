//! Hello Blueprint — demonstrates creating a simple UE4 Blueprint in Rust
//!
//! This example creates an Actor Blueprint that prints "Hello from Blueprint-RS!"
//! when the game starts (BeginPlay event), then chains a second PrintString.
//!
//! To use the output in UE4:
//! 1. Open the UE4 Blueprint editor
//! 2. Open the Event Graph
//! 3. Copy the T3D output below
//! 4. Paste (Ctrl+V) into the Blueprint editor
//! 5. The nodes appear wired and ready to use!
//!
//! Run with:
//!     cargo run --example hello_blueprint -p blueprint-core

use blueprint_core::builder::{BlueprintBuilder, VarType};

fn main() {
    // ── Build the Blueprint ──────────────────────────────────────────────────
    let mut builder = BlueprintBuilder::new("HelloWorldActor", "Actor");

    // Add variables (Health: int, PlayerName: string)
    builder.add_variable_mut("Health", VarType::Int, Some("100".to_string()));
    builder.add_variable_mut("PlayerName", VarType::String, Some("BlueprintPlayer".to_string()));

    // Wire up: BeginPlay → PrintString #1 → PrintString #2
    let begin_play = builder.begin_play_node();
    let print1 = builder.print_string("Hello from Blueprint-RS!");
    builder.exec_connect(&begin_play, &print1);

    let print2 = builder.print_string("Blueprint transpilation works!");
    builder.exec_connect(&print1, &print2);

    // ── T3D output ──────────────────────────────────────────────────────────
    let t3d = builder.to_t3d();

    println!("=== T3D Output (paste into UE4 Blueprint Editor) ===");
    println!("{}", t3d);
    println!("=== End T3D Output ===");
    println!();

    // ── JSON output ─────────────────────────────────────────────────────────
    match builder.to_json() {
        Ok(json) => {
            println!("=== JSON Representation ===");
            println!("{}", json);
            println!("=== End JSON ===");
        }
        Err(e) => eprintln!("JSON serialization error: {}", e),
    }
}
