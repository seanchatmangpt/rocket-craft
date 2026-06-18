//! UI Widget Blueprint — demonstrates a UserWidget Blueprint in Rust
//!
//! Creates a UserWidget Blueprint (e.g., a main-menu screen) with:
//! - MenuTitle / bIsMenuOpen / SelectedOption variables
//! - Construct event (BeginPlay for widgets): bind buttons and print welcome
//! - OnStartButtonClicked custom event: prints selection and starts game
//! - OnQuitButtonClicked custom event: prints farewell
//! - OnSettingsButtonClicked custom event: opens settings sub-menu
//!
//! Run with:
//!     cargo run --example ui_widget -p blueprint-core

use blueprint_core::builder::{BlueprintBuilder, VarType};

fn main() {
    let mut builder = BlueprintBuilder::new("MainMenuWidget", "UserWidget");

    // ── Variables ────────────────────────────────────────────────────────────
    builder.add_variable_mut("MenuTitle", VarType::String, Some("Main Menu".to_string()));
    builder.add_variable_mut("bIsMenuOpen", VarType::Bool, Some("false".to_string()));
    builder.add_variable_mut("SelectedOption", VarType::Int, Some("0".to_string()));

    // ── Construct event ──────────────────────────────────────────────────────
    // UserWidget maps Construct to begin_play_node()
    let construct = builder.begin_play_node();
    let welcome_print = builder.print_string("Main Menu Widget constructed!");
    builder.exec_connect(&construct, &welcome_print);

    // Bind the Start button's OnClicked to our custom event
    let bind_start = builder.bind_event_on_clicked("StartButton");
    builder.exec_connect(&welcome_print, &bind_start);

    // Bind the Quit button
    let bind_quit = builder.bind_event_on_clicked("QuitButton");
    builder.exec_connect(&bind_start, &bind_quit);

    // Bind the Settings button
    let bind_settings = builder.bind_event_on_clicked("SettingsButton");
    builder.exec_connect(&bind_quit, &bind_settings);

    // ── OnStartButtonClicked ─────────────────────────────────────────────────
    let on_start = builder.custom_event_node("OnStartButtonClicked");
    let start_print = builder.print_string("Start button clicked! Loading game...");
    builder.exec_connect(&on_start, &start_print);

    // Branch: check if this is a new game or continue
    let new_game_check = builder.branch_node();
    builder.exec_connect(&start_print, &new_game_check);

    let new_game_print = builder.print_string("Starting new game...");
    builder.connect(&new_game_check, "then", &new_game_print, "execute");

    let continue_print = builder.print_string("Continuing saved game...");
    builder.connect(&new_game_check, "else", &continue_print, "execute");

    // ── OnQuitButtonClicked ──────────────────────────────────────────────────
    let on_quit = builder.custom_event_node("OnQuitButtonClicked");
    let quit_print = builder.print_string("Quit button clicked. Goodbye!");
    builder.exec_connect(&on_quit, &quit_print);

    // ── OnSettingsButtonClicked ──────────────────────────────────────────────
    let on_settings = builder.custom_event_node("OnSettingsButtonClicked");
    let settings_print = builder.print_string("Opening settings menu...");
    builder.exec_connect(&on_settings, &settings_print);

    // ── Serialize ────────────────────────────────────────────────────────────
    let t3d = builder.to_t3d();
    let json = builder.to_json();

    println!("=== UI Widget Blueprint T3D ===");
    println!("{}", t3d);
    println!("=== End T3D ===");
    println!();

    match json {
        Ok(j) => {
            println!("=== UI Widget Blueprint JSON ===");
            println!("{}", j);
            println!("=== End JSON ===");
        }
        Err(e) => println!("JSON serialization error: {}", e),
    }

    // Print a compact summary
    let bp = builder.build();
    println!();
    println!("Summary:");
    println!("  Blueprint: {} (parent: {})", bp.name, bp.parent_class);
    println!("  Variables ({}):", bp.variables.len());
    for v in &bp.variables {
        let cat = v.category.as_deref().unwrap_or("(none)");
        println!(
            "    [{cat}] {name}: {:?} = {}",
            v.var_type.category,
            v.default_value.as_deref().unwrap_or("<none>"),
            name = v.name,
        );
    }
    println!("  Graphs ({}):", bp.graphs.len());
    for g in &bp.graphs {
        println!("    {} — {} nodes", g.name, g.nodes.len());
        for n in &g.nodes {
            let short_class = n.class.rsplit('.').next().unwrap_or(&n.class);
            println!("      {} [{}]", n.name, short_class);
        }
    }
    println!();
    println!("Paste the T3D output into the UE4 Widget Blueprint Event Graph.");
}
