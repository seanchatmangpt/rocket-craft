use std::env;
use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Actor {
    name: String,
    #[serde(rename = "class")]
    class_name: Option<String>,
    label: Option<String>,
    location: Vec3,
    rotation: Option<Vec3>,
    scale: Vec3,
    mesh: Option<String>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filepath = if args.len() > 1 {
        &args[1]
    } else {
        "Brm-HTML5-Shipping.data"
    };

    println!("Reading and parsing data from: {}", filepath);
    let mut file = match File::open(filepath) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening file: {}", e);
            std::process::exit(1);
        }
    };

    let mut data = String::new();
    if let Err(e) = file.read_to_string(&mut data) {
        eprintln!("Error reading file: {}", e);
        std::process::exit(1);
    }

    let actors: Vec<Actor> = match serde_json::from_str(&data) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error parsing JSON: {}", e);
            std::process::exit(1);
        }
    };

    println!("Successfully parsed {} actors.", actors.len());

    // Detect active zones
    let mut zones = Vec::new();
    for actor in &actors {
        if actor.name.starts_with("Place_") {
            let half_w = actor.scale.x * 50.0;
            let half_h = actor.scale.y * 50.0;
            let x_min = actor.location.x - half_w;
            let x_max = actor.location.x + half_w;
            let y_min = actor.location.y - half_h;
            let y_max = actor.location.y + half_h;
            println!(
                "Zone detected: {} ({}) at x:[{}, {}], y:[{}, {}]",
                actor.name,
                actor.label.as_deref().unwrap_or(""),
                x_min, x_max, y_min, y_max
            );
            zones.push(actor);
        }
    }

    if zones.is_empty() {
        println!("Warning: No zones detected starting with 'Place_'");
    } else {
        println!("Detected {} zones.", zones.len());
    }

    // Run basic verification checks
    let mut passed = true;
    if actors.is_empty() {
        eprintln!("Verification failed: Actors list is empty!");
        passed = false;
    }

    // Check if we have at least one zone and one bot/other actor
    let has_bot = actors.iter().any(|a| a.name.to_lowercase().contains("bot") || a.class_name.as_deref().unwrap_or("").to_lowercase().contains("bot"));
    if !has_bot {
        println!("Note: No actor with 'bot' in name or class found.");
    }

    if passed {
        println!("VERIFICATION SUCCESSFUL.");
    } else {
        std::process::exit(1);
    }
}
