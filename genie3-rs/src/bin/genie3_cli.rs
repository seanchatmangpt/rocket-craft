use genie3_rs::{
    types::{Bounds3D, Rotation3D, Transform, Vector3},
    world::{Actor, Environment, Object, Place, Weather, WorldState},
    simulation::{SimulationCommand, SimulationEngine},
};
use std::collections::HashMap;
use std::io::{self, Write};

fn print_help() {
    println!("Available Commands:");
    println!("  w / forward      - Move bot_1 forward (Y += 5.0)");
    println!("  s / backward     - Move bot_1 backward (Y -= 5.0)");
    println!("  a / left         - Move bot_1 left (X -= 5.0)");
    println!("  d / right        - Move bot_1 right (X += 5.0)");
    println!("  spawn actor <id> at <x> <y> <z>");
    println!("                   - Spawn a new actor at coordinates");
    println!("  spawn object <id> at <x> <y> <z>");
    println!("                   - Spawn a new object at coordinates");
    println!("  weather <sunny|cloudy|stormy|rainy>");
    println!("                   - Set the environment weather");
    println!("  time <hour>      - Set the environment time of day (0.0 to 24.0)");
    println!("  status / show    - Display complete current world state");
    println!("  help             - Show this help menu");
    println!("  exit / quit      - Exit the world simulation");
}

fn print_state(state: &WorldState) {
    println!("\n--- World State (Step {}) ---", state.step_index);
    println!("Time: {:.1}h | Weather: {:?}", state.environment.time_of_day, state.environment.weather);
    println!("Places:");
    for p in &state.places {
        let bounds = &p.bounds;
        println!("  - Place '{}' ({}) bounds center: {:?}, half_extents: {:?}", p.id, p.name, bounds.center, bounds.half_extents);
    }
    println!("Actors:");
    for a in &state.actors {
        println!("  - Actor '{}' ({}) position: {:?} in Place: {:?}", a.id, a.name, a.position, a.place_id);
    }
    println!("Objects:");
    for o in &state.objects {
        println!("  - Object '{}' ({}) position: {:?} in Place: {:?}", o.id, o.name, o.transform.position, o.place_id);
    }
    println!("-----------------------------\n");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("====================================================");
    println!("      Genie 3 World Builder Interactive CLI");
    println!("====================================================");

    // Initialize Default WorldState
    let mut state = WorldState::new();
    state.environment = Environment::new(Weather::Sunny, 12.0);

    // Default place
    let room_bounds = Bounds3D::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(50.0, 50.0, 50.0));
    let mut room = Place::new("room_1", "Control Room", room_bounds);
    room.properties.insert("hard_containment".to_string(), serde_json::Value::Bool(true));
    state.places.push(room);

    // Default actor
    let mut bot = Actor::new("bot_1", "Welder Bot", "Robot", Vector3::new(0.0, 0.0, 0.0));
    bot.place_id = Some("room_1".to_string());
    let mut bot_props = HashMap::new();
    bot_props.insert("half_extents".to_string(), serde_json::json!({"x": 1.0, "y": 1.0, "z": 2.0}));
    bot_props.insert("max_speed".to_string(), serde_json::json!(15.0));
    bot.properties = bot_props;
    state.actors.push(bot);

    // Default object
    let cnc_transform = Transform::new(Vector3::new(10.0, 10.0, 0.0), Rotation3D::default(), Vector3::new(1.0, 1.0, 1.0));
    let mut cnc = Object::new("cnc_1", "CNC Alpha", "Machine", cnc_transform);
    cnc.place_id = Some("room_1".to_string());
    let mut cnc_props = HashMap::new();
    cnc_props.insert("half_extents".to_string(), serde_json::json!({"x": 2.0, "y": 2.0, "z": 2.0}));
    cnc.properties = cnc_props;
    state.objects.push(cnc);

    let engine = SimulationEngine::default();

    print_state(&state);
    print_help();

    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        print!("genie3> ");
        io::stdout().flush()?;
        input.clear();
        if stdin.read_line(&mut input)? == 0 {
            break;
        }

        let cmd_line = input.trim();
        if cmd_line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = cmd_line.split_whitespace().collect();
        let cmd = parts[0].to_lowercase();

        match cmd.as_str() {
            "exit" | "quit" | "q" => {
                println!("Exiting World Simulation. Goodbye!");
                break;
            }
            "help" | "h" | "?" => {
                print_help();
            }
            "status" | "show" => {
                print_state(&state);
            }
            "w" | "forward" => {
                let sim_cmd = SimulationCommand::MoveActor {
                    actor_id: "bot_1".to_string(),
                    movement: Vector3::new(0.0, 5.0, 0.0),
                    rotation: Rotation3D::default(),
                };
                match engine.execute_command(&state, &sim_cmd, 0.1) {
                    Ok(next) => {
                        state = next;
                        println!("Success: Moved bot_1 forward.");
                        print_state(&state);
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
            "s" | "backward" => {
                let sim_cmd = SimulationCommand::MoveActor {
                    actor_id: "bot_1".to_string(),
                    movement: Vector3::new(0.0, -5.0, 0.0),
                    rotation: Rotation3D::default(),
                };
                match engine.execute_command(&state, &sim_cmd, 0.1) {
                    Ok(next) => {
                        state = next;
                        println!("Success: Moved bot_1 backward.");
                        print_state(&state);
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
            "a" | "left" => {
                let sim_cmd = SimulationCommand::MoveActor {
                    actor_id: "bot_1".to_string(),
                    movement: Vector3::new(-5.0, 0.0, 0.0),
                    rotation: Rotation3D::default(),
                };
                match engine.execute_command(&state, &sim_cmd, 0.1) {
                    Ok(next) => {
                        state = next;
                        println!("Success: Moved bot_1 left.");
                        print_state(&state);
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
            "d" | "right" => {
                let sim_cmd = SimulationCommand::MoveActor {
                    actor_id: "bot_1".to_string(),
                    movement: Vector3::new(5.0, 0.0, 0.0),
                    rotation: Rotation3D::default(),
                };
                match engine.execute_command(&state, &sim_cmd, 0.1) {
                    Ok(next) => {
                        state = next;
                        println!("Success: Moved bot_1 right.");
                        print_state(&state);
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
            "spawn" => {
                if parts.len() < 7 || parts[3] != "at" {
                    println!("Usage: spawn [actor|object] <id> at <x> <y> <z>");
                    continue;
                }
                let spawn_type = parts[1].to_lowercase();
                let id = parts[2].to_string();
                let x: f32 = parts[4].parse().unwrap_or(0.0);
                let y: f32 = parts[5].parse().unwrap_or(0.0);
                let z: f32 = parts[6].parse().unwrap_or(0.0);
                let position = Vector3::new(x, y, z);

                if spawn_type == "actor" {
                    let mut props = HashMap::new();
                    props.insert("half_extents".to_string(), serde_json::json!({"x": 0.5, "y": 0.5, "z": 1.0}));
                    let sim_cmd = SimulationCommand::SpawnActor {
                        id: id.clone(),
                        name: format!("Actor {}", id),
                        actor_type: "Generic".to_string(),
                        position,
                        rotation: None,
                        properties: props,
                    };
                    match engine.execute_command(&state, &sim_cmd, 0.1) {
                        Ok(next) => {
                            state = next;
                            println!("Success: Spawned actor '{}' at {:?}", id, position);
                            print_state(&state);
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                } else if spawn_type == "object" {
                    let mut props = HashMap::new();
                    props.insert("half_extents".to_string(), serde_json::json!({"x": 1.0, "y": 1.0, "z": 1.0}));
                    let sim_cmd = SimulationCommand::SpawnObject {
                        id: id.clone(),
                        name: format!("Object {}", id),
                        class: "Generic".to_string(),
                        transform: Transform::new(position, Rotation3D::default(), Vector3::new(1.0, 1.0, 1.0)),
                        properties: props,
                    };
                    match engine.execute_command(&state, &sim_cmd, 0.1) {
                        Ok(next) => {
                            state = next;
                            println!("Success: Spawned object '{}' at {:?}", id, position);
                            print_state(&state);
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                } else {
                    println!("Invalid spawn type. Use 'actor' or 'object'.");
                }
            }
            "weather" => {
                if parts.len() < 2 {
                    println!("Usage: weather <sunny|cloudy|stormy|rainy>");
                    continue;
                }
                let weather_str = parts[1].to_lowercase();
                let weather = match weather_str.as_str() {
                    "sunny" => Weather::Sunny,
                    "cloudy" => Weather::Cloudy,
                    "stormy" => Weather::Stormy,
                    "rainy" => Weather::Rainy,
                    _ => {
                        println!("Unknown weather type. Defaulting to Sunny.");
                        Weather::Sunny
                    }
                };
                let sim_cmd = SimulationCommand::ChangeWeather { weather };
                match engine.execute_command(&state, &sim_cmd, 0.1) {
                    Ok(next) => {
                        state = next;
                        println!("Success: Weather changed to {:?}", weather_str);
                        print_state(&state);
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
            "time" => {
                if parts.len() < 2 {
                    println!("Usage: time <hour>");
                    continue;
                }
                let time_val: f32 = parts[1].parse().unwrap_or(12.0);
                let sim_cmd = SimulationCommand::ChangeTime { time_of_day: time_val };
                match engine.execute_command(&state, &sim_cmd, 0.0) {
                    Ok(next) => {
                        state = next;
                        println!("Success: Time set to {:.2}", time_val);
                        print_state(&state);
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
            _ => {
                println!("Unknown command. Type 'help' for options.");
            }
        }
    }

    Ok(())
}
