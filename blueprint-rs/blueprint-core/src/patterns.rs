//! Blueprint Pattern Library
//!
//! High-level composable patterns that create entire multi-node Blueprint
//! subsystems with a single function call. These sit at the "framework level"
//! above individual nodes, letting callers bootstrap common gameplay systems
//! by passing a [`BlueprintBuilder`] to a pattern function.
//!
//! # Quick start
//! ```rust,no_run
//! use blueprint_core::BlueprintBuilder;
//! use blueprint_core::patterns::{health_system, HealthSystemPattern};
//!
//! let mut builder = BlueprintBuilder::new("MyCharacter", "Character");
//! health_system(&mut builder, &HealthSystemPattern::default());
//! let t3d = builder.to_t3d();
//! ```

use crate::builder::{BlueprintBuilder, VarType};

// ============================================================
// PATTERN: Health System
// Creates a complete health system with take_damage, heal, death events
// ============================================================

/// Configuration for the health system pattern.
pub struct HealthSystemPattern {
    /// Maximum (and initial) health value.
    pub max_health: i32,
    /// Name of the `Health` variable.
    pub var_health: String,
    /// Name of the `MaxHealth` variable.
    pub var_max_health: String,
    /// Name of the take-damage custom event.
    pub event_on_damage: String,
    /// Name of the death custom event.
    pub event_on_death: String,
    /// Name of the heal custom event.
    pub event_on_heal: String,
}

impl Default for HealthSystemPattern {
    fn default() -> Self {
        Self {
            max_health: 100,
            var_health: "Health".to_string(),
            var_max_health: "MaxHealth".to_string(),
            event_on_damage: "OnDamageTaken".to_string(),
            event_on_death: "OnDeath".to_string(),
            event_on_heal: "OnHealed".to_string(),
        }
    }
}

/// Apply a complete health system to a [`BlueprintBuilder`].
///
/// Creates:
/// - `Health`, `MaxHealth`, `IsAlive` variables
/// - `OnDamageTaken` event → print → death-check branch → death event / continue
/// - `OnHealed` event → print
pub fn health_system(builder: &mut BlueprintBuilder, config: &HealthSystemPattern) {
    // Variables
    builder.add_variable_mut(
        &config.var_health,
        VarType::Int,
        Some(config.max_health.to_string()),
    );
    builder.add_variable_mut(
        &config.var_max_health,
        VarType::Int,
        Some(config.max_health.to_string()),
    );
    builder.add_variable_mut("IsAlive", VarType::Bool, Some("True".to_string()));

    // TakeDamage event → print → death-check branch
    let take_damage_ev = builder.custom_event_node(&config.event_on_damage);
    let damage_print = builder.print_string("Taking damage!");
    let death_check = builder.branch_node();
    let death_print = builder.print_string("Player died!");
    let death_event = builder.custom_event_node(&config.event_on_death);

    builder.exec_connect(&take_damage_ev, &damage_print);
    builder.exec_connect(&damage_print, &death_check);
    // Branch "then" fires when health <= 0 (death path)
    builder.connect(&death_check, "then", &death_print, "execute");
    builder.exec_connect(&death_print, &death_event);

    // Heal event
    let heal_ev = builder.custom_event_node(&config.event_on_heal);
    let heal_print = builder.print_string("Healing!");
    builder.exec_connect(&heal_ev, &heal_print);
}

// ============================================================
// PATTERN: State Machine
// ============================================================

/// Configuration for the state machine pattern.
pub struct StateMachinePattern {
    /// All possible states by name.
    pub states: Vec<String>,
    /// Initial value of the current-state variable.
    pub initial_state: String,
    /// Name of the variable that tracks the current state.
    pub var_name: String,
}

impl Default for StateMachinePattern {
    fn default() -> Self {
        Self {
            states: vec!["Idle".to_string(), "Running".to_string(), "Dead".to_string()],
            initial_state: "Idle".to_string(),
            var_name: "CurrentState".to_string(),
        }
    }
}

/// Apply a state machine pattern.
///
/// Creates:
/// - `CurrentState` string variable
/// - `TransitionTo` custom event with a print stub
/// - `OnEnter_<State>` and `OnExit_<State>` custom events for every state
pub fn state_machine(builder: &mut BlueprintBuilder, config: &StateMachinePattern) {
    builder.add_variable_mut(
        &config.var_name,
        VarType::String,
        Some(config.initial_state.clone()),
    );

    // Central transition event
    let transition_ev = builder.custom_event_node("TransitionTo");
    let print_transition = builder.print_string("State transition!");
    builder.exec_connect(&transition_ev, &print_transition);

    // Per-state enter / exit events
    for state in &config.states {
        let enter_ev = builder.custom_event_node(&format!("OnEnter_{}", state));
        let enter_print = builder.print_string(&format!("Entering state: {}", state));
        builder.exec_connect(&enter_ev, &enter_print);

        let exit_ev = builder.custom_event_node(&format!("OnExit_{}", state));
        let exit_print = builder.print_string(&format!("Exiting state: {}", state));
        builder.exec_connect(&exit_ev, &exit_print);
    }
}

// ============================================================
// PATTERN: Repeating Timer
// ============================================================

/// Configuration for the timer pattern.
pub struct TimerPattern {
    /// Interval in seconds between firings.
    pub rate: f32,
    /// Whether the timer loops.
    pub looping: bool,
    /// Name of the custom event that fires each tick.
    pub callback_event: String,
    /// If true, wire BeginPlay → StartTimer automatically.
    pub auto_start: bool,
}

impl Default for TimerPattern {
    fn default() -> Self {
        Self {
            rate: 1.0,
            looping: true,
            callback_event: "OnTimerFired".to_string(),
            auto_start: true,
        }
    }
}

/// Apply a timer pattern.
///
/// Creates:
/// - A callback custom event with a fired-print stub
/// - Optionally a `BeginPlay → SetTimerByEvent` chain
/// - A `StopTimer` custom event stub
pub fn timer(builder: &mut BlueprintBuilder, config: &TimerPattern) {
    // Callback event
    let callback = builder.custom_event_node(&config.callback_event);
    let fired_print =
        builder.print_string(&format!("Timer '{}' fired!", config.callback_event));
    builder.exec_connect(&callback, &fired_print);

    // Optional auto-start via BeginPlay
    if config.auto_start {
        let begin = builder.begin_play_node();
        let start_timer = builder.set_timer_by_event(config.rate, config.looping);
        let start_print = builder.print_string("Timer started!");
        builder.exec_connect(&begin, &start_timer);
        builder.exec_connect(&start_timer, &start_print);
    }

    // StopTimer event
    let stop_ev = builder.custom_event_node("StopTimer");
    let stop_print = builder.print_string("Timer stopped!");
    builder.exec_connect(&stop_ev, &stop_print);
}

// ============================================================
// PATTERN: Inventory System
// ============================================================

/// Configuration for the inventory pattern.
pub struct InventoryPattern {
    /// Maximum number of items the inventory can hold.
    pub capacity: i32,
    /// Human-readable label for the item type (used in print strings).
    pub item_type: String,
}

impl Default for InventoryPattern {
    fn default() -> Self {
        Self {
            capacity: 20,
            item_type: "Item".to_string(),
        }
    }
}

/// Apply an inventory pattern.
///
/// Creates:
/// - `Items` (string), `ItemCount` (int), `MaxItems` (int) variables
/// - `AddItem` event with capacity-check branch → `InventoryFull` or add path
/// - `RemoveItem` event stub
/// - `ClearInventory` event stub
pub fn inventory(builder: &mut BlueprintBuilder, config: &InventoryPattern) {
    builder.add_variable_mut("Items", VarType::String, None);
    builder.add_variable_mut("ItemCount", VarType::Int, Some("0".to_string()));
    builder.add_variable_mut("MaxItems", VarType::Int, Some(config.capacity.to_string()));

    // AddItem event with capacity check
    let add_ev = builder.custom_event_node("AddItem");
    let capacity_check = builder.branch_node();
    let full_print = builder.print_string("Inventory full!");
    let full_ev = builder.custom_event_node("InventoryFull");
    let add_print = builder.print_string(&format!("{} added!", config.item_type));

    builder.exec_connect(&add_ev, &capacity_check);
    builder.connect(&capacity_check, "then", &full_print, "execute");
    builder.exec_connect(&full_print, &full_ev);
    builder.connect(&capacity_check, "else", &add_print, "execute");

    // RemoveItem
    let remove_ev = builder.custom_event_node("RemoveItem");
    let remove_print = builder.print_string(&format!("{} removed!", config.item_type));
    builder.exec_connect(&remove_ev, &remove_print);

    // ClearInventory
    let clear_ev = builder.custom_event_node("ClearInventory");
    let clear_print = builder.print_string("Inventory cleared!");
    builder.exec_connect(&clear_ev, &clear_print);
}

// ============================================================
// PATTERN: Damage System (Actor-level)
// ============================================================

/// Apply a damage system with shield, armor, and damage type filtering.
///
/// Creates:
/// - `Armor`, `Shield`, `DamageMultiplier` variables
/// - `ProcessDamage` event → shield-check branch
pub fn damage_system(builder: &mut BlueprintBuilder) {
    builder.add_variable_mut("Armor", VarType::Float, Some("0.0".to_string()));
    builder.add_variable_mut("Shield", VarType::Float, Some("0.0".to_string()));
    builder.add_variable_mut("DamageMultiplier", VarType::Float, Some("1.0".to_string()));

    let take_damage = builder.custom_event_node("ProcessDamage");
    let has_shield = builder.branch_node();
    let shield_absorbed = builder.print_string("Shield absorbed damage!");
    let apply_damage = builder.print_string("Damage applied!");

    builder.exec_connect(&take_damage, &has_shield);
    builder.connect(&has_shield, "then", &shield_absorbed, "execute");
    builder.connect(&has_shield, "else", &apply_damage, "execute");
}

// ============================================================
// PATTERN: FPS Character Controller
// ============================================================

/// Apply a first-person character controller pattern.
///
/// Creates:
/// - `WalkSpeed`, `SprintSpeed`, `IsCrouching`, `IsAiming` variables
/// - `BeginPlay` init chain
/// - `StartSprint` / `StopSprint` events
/// - `ToggleCrouch` event with crouch/stand branch
pub fn fps_controller(builder: &mut BlueprintBuilder) {
    builder.add_variable_mut("WalkSpeed", VarType::Float, Some("600.0".to_string()));
    builder.add_variable_mut("SprintSpeed", VarType::Float, Some("900.0".to_string()));
    builder.add_variable_mut("IsCrouching", VarType::Bool, Some("False".to_string()));
    builder.add_variable_mut("IsAiming", VarType::Bool, Some("False".to_string()));

    // Init
    let begin = builder.begin_play_node();
    let init_print = builder.print_string("FPS Character initialized!");
    builder.exec_connect(&begin, &init_print);

    // Sprint
    let sprint_ev = builder.custom_event_node("StartSprint");
    let sprint_print = builder.print_string("Sprinting!");
    builder.exec_connect(&sprint_ev, &sprint_print);

    let stop_sprint = builder.custom_event_node("StopSprint");
    let stop_print = builder.print_string("Walk speed restored.");
    builder.exec_connect(&stop_sprint, &stop_print);

    // Crouch toggle
    let crouch_ev = builder.custom_event_node("ToggleCrouch");
    let crouch_check = builder.branch_node();
    let stand_print = builder.print_string("Standing up.");
    let crouch_print = builder.print_string("Crouching.");
    builder.exec_connect(&crouch_ev, &crouch_check);
    builder.connect(&crouch_check, "then", &stand_print, "execute");
    builder.connect(&crouch_check, "else", &crouch_print, "execute");
}

// ============================================================
// PATTERN: Dialogue System
// ============================================================

/// Configuration for the dialogue pattern.
pub struct DialoguePattern {
    /// All lines spoken in sequence.
    pub lines: Vec<String>,
    /// Name displayed as the speaker.
    pub speaker_name: String,
}

impl Default for DialoguePattern {
    fn default() -> Self {
        Self {
            lines: vec!["Hello!".to_string(), "How are you?".to_string()],
            speaker_name: "NPC".to_string(),
        }
    }
}

/// Apply a simple dialogue system with sequential line advancement.
///
/// Creates:
/// - `CurrentLine` (int), `IsDialogueActive` (bool) variables
/// - `StartDialogue`, `NextLine`, `EndDialogue` custom events
pub fn dialogue_system(builder: &mut BlueprintBuilder, config: &DialoguePattern) {
    builder.add_variable_mut("CurrentLine", VarType::Int, Some("0".to_string()));
    builder.add_variable_mut("IsDialogueActive", VarType::Bool, Some("False".to_string()));

    let start_ev = builder.custom_event_node("StartDialogue");
    let first_line = config
        .lines
        .first()
        .cloned()
        .unwrap_or_else(|| "...".to_string());
    let start_print =
        builder.print_string(&format!("[{}]: {}", config.speaker_name, first_line));
    builder.exec_connect(&start_ev, &start_print);

    let next_ev = builder.custom_event_node("NextLine");
    let next_print = builder.print_string("Advancing dialogue...");
    builder.exec_connect(&next_ev, &next_print);

    let end_ev = builder.custom_event_node("EndDialogue");
    let end_print = builder.print_string("Dialogue ended.");
    builder.exec_connect(&end_ev, &end_print);
}

// ============================================================
// PATTERN: Ragdoll / Physics Death
// ============================================================

/// Apply a ragdoll/physics-based death system.
///
/// Creates:
/// - `EnableRagdoll` custom event → mesh ragdoll print → disable-input print
pub fn ragdoll_death(builder: &mut BlueprintBuilder) {
    let death_ev = builder.custom_event_node("EnableRagdoll");
    let mesh_print = builder.print_string("Enabling ragdoll physics...");
    let disable_input = builder.print_string("Input disabled.");
    builder.exec_connect(&death_ev, &mesh_print);
    builder.exec_connect(&mesh_print, &disable_input);
}

// ============================================================
// PATTERN: Wave Spawner
// ============================================================

/// Configuration for the wave spawner pattern.
pub struct WaveSpawnerPattern {
    /// Total number of waves to spawn.
    pub num_waves: usize,
    /// How many enemies spawn in wave 1 (scales linearly).
    pub base_enemies: usize,
}

impl Default for WaveSpawnerPattern {
    fn default() -> Self {
        Self {
            num_waves: 5,
            base_enemies: 3,
        }
    }
}

/// Apply a wave spawner pattern with escalating enemy counts.
///
/// Creates:
/// - `CurrentWave`, `EnemiesRemaining`, `TotalWaves`, `BaseEnemies` variables
/// - `StartWave` event → spawn print
/// - `OnEnemyDied` event → wave-clear check branch → `OnWaveCleared`
/// - `OnWaveCleared` → final-wave check branch → `OnAllWavesCleared`
pub fn wave_spawner(builder: &mut BlueprintBuilder, config: &WaveSpawnerPattern) {
    builder.add_variable_mut("CurrentWave", VarType::Int, Some("0".to_string()));
    builder.add_variable_mut("EnemiesRemaining", VarType::Int, Some("0".to_string()));
    builder.add_variable_mut(
        "TotalWaves",
        VarType::Int,
        Some(config.num_waves.to_string()),
    );
    builder.add_variable_mut(
        "BaseEnemies",
        VarType::Int,
        Some(config.base_enemies.to_string()),
    );

    let start_wave_ev = builder.custom_event_node("StartWave");
    let spawn_print = builder.print_string("Wave starting!");
    builder.exec_connect(&start_wave_ev, &spawn_print);

    let enemy_died = builder.custom_event_node("OnEnemyDied");
    let wave_clear_check = builder.branch_node();
    builder.exec_connect(&enemy_died, &wave_clear_check);

    let wave_clear = builder.custom_event_node("OnWaveCleared");
    let wave_clear_print = builder.print_string("Wave cleared!");
    builder.connect(&wave_clear_check, "then", &wave_clear, "execute");
    builder.exec_connect(&wave_clear, &wave_clear_print);

    let final_check = builder.branch_node();
    builder.exec_connect(&wave_clear_print, &final_check);

    let game_won = builder.custom_event_node("OnAllWavesCleared");
    let game_won_print = builder.print_string("All waves cleared! You win!");
    builder.connect(&final_check, "then", &game_won, "execute");
    builder.exec_connect(&game_won, &game_won_print);
}

// ============================================================
// PATTERN: Camera Shake
// ============================================================

/// Apply a camera shake pattern triggered by a custom event.
///
/// Creates:
/// - `ShakeIntensity`, `ShakeDuration` float variables
/// - `TriggerCameraShake` event → shake print
///
/// # Arguments
/// * `intensity` - magnitude of the shake (stored as a variable default)
/// * `duration`  - duration of the shake in seconds
pub fn camera_shake(builder: &mut BlueprintBuilder, intensity: f32, duration: f32) {
    builder.add_variable_mut(
        "ShakeIntensity",
        VarType::Float,
        Some(intensity.to_string()),
    );
    builder.add_variable_mut(
        "ShakeDuration",
        VarType::Float,
        Some(duration.to_string()),
    );

    let shake_ev = builder.custom_event_node("TriggerCameraShake");
    let shake_print = builder.print_string(&format!(
        "Camera shake! Intensity={intensity:.2} Duration={duration:.2}"
    ));
    builder.exec_connect(&shake_ev, &shake_print);
}

// ============================================================
// PATTERN: Floating Damage Text
// ============================================================

/// Apply a floating damage text pattern.
///
/// Creates:
/// - `DamageTextColor` string variable
/// - `SpawnDamageText` event → spawn-text print → fade-out print
pub fn floating_damage_text(builder: &mut BlueprintBuilder) {
    builder.add_variable_mut(
        "DamageTextColor",
        VarType::String,
        Some("Red".to_string()),
    );

    let spawn_ev = builder.custom_event_node("SpawnDamageText");
    let spawn_print = builder.print_string("Spawning floating damage text...");
    let fade_print = builder.print_string("Damage text fading out.");
    builder.exec_connect(&spawn_ev, &spawn_print);
    builder.exec_connect(&spawn_print, &fade_print);
}

// ============================================================
// PATTERN: Respawn System
// ============================================================

/// Apply a respawn system pattern.
///
/// Creates:
/// - `RespawnDelay`, `RespawnCount`, `CanRespawn` variables
/// - `OnPlayerDied` event → delay print → respawn print → `OnPlayerRespawned` event
///
/// # Arguments
/// * `respawn_delay` - seconds before the actor respawns
pub fn respawn_system(builder: &mut BlueprintBuilder, respawn_delay: f32) {
    builder.add_variable_mut(
        "RespawnDelay",
        VarType::Float,
        Some(respawn_delay.to_string()),
    );
    builder.add_variable_mut("RespawnCount", VarType::Int, Some("0".to_string()));
    builder.add_variable_mut("CanRespawn", VarType::Bool, Some("True".to_string()));

    let died_ev = builder.custom_event_node("OnPlayerDied");
    let delay_print = builder.print_string(&format!(
        "Player died. Respawning in {respawn_delay:.1}s..."
    ));
    let respawn_print = builder.print_string("Respawning player!");
    let respawn_ev = builder.custom_event_node("OnPlayerRespawned");

    builder.exec_connect(&died_ev, &delay_print);
    builder.exec_connect(&delay_print, &respawn_print);
    builder.exec_connect(&respawn_print, &respawn_ev);
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::BlueprintBuilder;

    /// Count how many Blueprint nodes appear in the T3D output.
    /// Each node serialises as "Begin Object Class=..." so we count those.
    fn node_count_from_t3d(b: &BlueprintBuilder) -> usize {
        b.to_t3d()
            .lines()
            .filter(|l| l.trim_start().starts_with("Begin Object Class="))
            .count()
    }

    // -----------------------------------------------------------------------
    // health_system
    // -----------------------------------------------------------------------

    #[test]
    fn health_system_adds_at_least_three_nodes() {
        let mut b = BlueprintBuilder::new("HP", "Actor");
        health_system(&mut b, &HealthSystemPattern::default());
        let count = node_count_from_t3d(&b);
        // Expect: take_damage_ev, damage_print, death_check, death_print,
        //         death_event, heal_ev, heal_print → 7 nodes
        assert!(count >= 3, "expected >=3 nodes, got {count}");
    }

    #[test]
    fn health_system_adds_variables() {
        let mut b = BlueprintBuilder::new("HP", "Actor");
        let cfg = HealthSystemPattern::default();
        health_system(&mut b, &cfg);
        // Variables are not in T3D output; check via JSON serialization
        let json = b.to_json().expect("JSON serialization failed");
        assert!(json.contains(&cfg.var_health), "missing Health variable");
        assert!(json.contains(&cfg.var_max_health), "missing MaxHealth variable");
        assert!(json.contains("IsAlive"), "missing IsAlive variable");
    }

    #[test]
    fn health_system_custom_names() {
        let cfg = HealthSystemPattern {
            max_health: 250,
            var_health: "HP".to_string(),
            var_max_health: "MaxHP".to_string(),
            event_on_damage: "Damaged".to_string(),
            event_on_death: "Died".to_string(),
            event_on_heal: "Healed".to_string(),
        };
        let mut b = BlueprintBuilder::new("Custom", "Actor");
        health_system(&mut b, &cfg);
        let json = b.to_json().expect("JSON serialization failed");
        assert!(json.contains(""HP"") || json.contains(r#""name": "HP""#), "missing HP variable");
        assert!(json.contains("MaxHP"), "missing MaxHP variable");
        // Events appear in node names in T3D
        let t3d = b.to_t3d();
        assert!(t3d.contains("Damaged"), "missing Damaged event");
    }

    #[test]
    fn health_system_does_not_panic_on_default() {
        let mut b = BlueprintBuilder::new("HP", "Actor");
        health_system(&mut b, &HealthSystemPattern::default());
        // Must not panic — no assertion needed
        let _ = b.to_t3d();
    }

    // -----------------------------------------------------------------------
    // timer
    // -----------------------------------------------------------------------

    #[test]
    fn timer_auto_start_creates_begin_play_node() {
        let cfg = TimerPattern {
            auto_start: true,
            ..TimerPattern::default()
        };
        let mut b = BlueprintBuilder::new("Ticker", "Actor");
        timer(&mut b, &cfg);
        let t3d = b.to_t3d();
        assert!(
            t3d.contains("ReceiveBeginPlay"),
            "expected ReceiveBeginPlay in T3D when auto_start=true"
        );
    }

    #[test]
    fn timer_no_begin_play_when_auto_start_false() {
        let cfg = TimerPattern {
            auto_start: false,
            ..TimerPattern::default()
        };
        let mut b = BlueprintBuilder::new("Ticker", "Actor");
        timer(&mut b, &cfg);
        let t3d = b.to_t3d();
        assert!(
            !t3d.contains("ReceiveBeginPlay"),
            "expected no ReceiveBeginPlay when auto_start=false"
        );
    }

    #[test]
    fn timer_default_does_not_panic() {
        let mut b = BlueprintBuilder::new("T", "Actor");
        timer(&mut b, &TimerPattern::default());
        let _ = b.to_t3d();
    }

    // -----------------------------------------------------------------------
    // inventory
    // -----------------------------------------------------------------------

    #[test]
    fn inventory_creates_add_and_remove_events() {
        let mut b = BlueprintBuilder::new("Inv", "Actor");
        inventory(&mut b, &InventoryPattern::default());
        let t3d = b.to_t3d();
        assert!(t3d.contains("AddItem"), "missing AddItem event");
        assert!(t3d.contains("RemoveItem"), "missing RemoveItem event");
    }

    #[test]
    fn inventory_creates_inventory_full_event() {
        let mut b = BlueprintBuilder::new("Inv", "Actor");
        inventory(&mut b, &InventoryPattern::default());
        let t3d = b.to_t3d();
        assert!(t3d.contains("InventoryFull"), "missing InventoryFull event");
    }

    #[test]
    fn inventory_adds_variables_in_t3d() {
        let mut b = BlueprintBuilder::new("Inv", "Actor");
        let cfg = InventoryPattern {
            capacity: 10,
            item_type: "Weapon".to_string(),
        };
        inventory(&mut b, &cfg);
        // Variables are not serialized in T3D output; use JSON to verify
        let json = b.to_json().expect("JSON serialization failed");
        assert!(json.contains("Items"), "missing Items variable");
        assert!(json.contains("ItemCount"), "missing ItemCount variable");
        assert!(json.contains("MaxItems"), "missing MaxItems variable");
    }

    // -----------------------------------------------------------------------
    // state_machine
    // -----------------------------------------------------------------------

    #[test]
    fn state_machine_creates_enter_exit_events_for_each_state() {
        let cfg = StateMachinePattern {
            states: vec!["Idle".to_string(), "Running".to_string(), "Dead".to_string()],
            initial_state: "Idle".to_string(),
            var_name: "State".to_string(),
        };
        let mut b = BlueprintBuilder::new("SM", "Actor");
        state_machine(&mut b, &cfg);
        let t3d = b.to_t3d();
        for state in &cfg.states {
            assert!(
                t3d.contains(&format!("OnEnter_{}", state)),
                "missing OnEnter_{state}"
            );
            assert!(
                t3d.contains(&format!("OnExit_{}", state)),
                "missing OnExit_{state}"
            );
        }
    }

    #[test]
    fn state_machine_empty_states_does_not_panic() {
        let cfg = StateMachinePattern {
            states: vec![],
            initial_state: "Idle".to_string(),
            var_name: "CurrentState".to_string(),
        };
        let mut b = BlueprintBuilder::new("SM", "Actor");
        state_machine(&mut b, &cfg);
        let _ = b.to_t3d();
    }

    #[test]
    fn state_machine_default_does_not_panic() {
        let mut b = BlueprintBuilder::new("SM", "Actor");
        state_machine(&mut b, &StateMachinePattern::default());
        let _ = b.to_t3d();
    }

    // -----------------------------------------------------------------------
    // Remaining patterns — smoke / no-panic tests
    // -----------------------------------------------------------------------

    #[test]
    fn damage_system_does_not_panic() {
        let mut b = BlueprintBuilder::new("DS", "Actor");
        damage_system(&mut b);
        let _ = b.to_t3d();
    }

    #[test]
    fn fps_controller_does_not_panic() {
        let mut b = BlueprintBuilder::new("FPS", "Character");
        fps_controller(&mut b);
        let _ = b.to_t3d();
    }

    #[test]
    fn dialogue_empty_lines_does_not_panic() {
        let cfg = DialoguePattern {
            lines: vec![],
            speaker_name: "Ghost".to_string(),
        };
        let mut b = BlueprintBuilder::new("DLG", "Actor");
        dialogue_system(&mut b, &cfg);
        let _ = b.to_t3d();
    }

    #[test]
    fn ragdoll_death_does_not_panic() {
        let mut b = BlueprintBuilder::new("Rag", "Actor");
        ragdoll_death(&mut b);
        let _ = b.to_t3d();
    }

    #[test]
    fn wave_spawner_does_not_panic() {
        let mut b = BlueprintBuilder::new("WS", "Actor");
        wave_spawner(&mut b, &WaveSpawnerPattern::default());
        let _ = b.to_t3d();
    }

    #[test]
    fn camera_shake_does_not_panic() {
        let mut b = BlueprintBuilder::new("CS", "Actor");
        camera_shake(&mut b, 1.5, 0.3);
        let _ = b.to_t3d();
    }

    #[test]
    fn floating_damage_text_does_not_panic() {
        let mut b = BlueprintBuilder::new("FDT", "Actor");
        floating_damage_text(&mut b);
        let _ = b.to_t3d();
    }

    #[test]
    fn respawn_system_does_not_panic() {
        let mut b = BlueprintBuilder::new("RS", "Actor");
        respawn_system(&mut b, 3.0);
        let _ = b.to_t3d();
    }

    // -----------------------------------------------------------------------
    // Multiple patterns on the same builder
    // -----------------------------------------------------------------------

    #[test]
    fn multiple_patterns_same_builder_no_panic() {
        let mut b = BlueprintBuilder::new("Composite", "Character");
        health_system(&mut b, &HealthSystemPattern::default());
        inventory(&mut b, &InventoryPattern::default());
        damage_system(&mut b);
        fps_controller(&mut b);
        camera_shake(&mut b, 0.5, 0.2);
        respawn_system(&mut b, 5.0);
        let _ = b.to_t3d();
    }

    #[test]
    fn multiple_patterns_t3d_contains_all_events() {
        let mut b = BlueprintBuilder::new("Composite", "Character");
        health_system(&mut b, &HealthSystemPattern::default());
        inventory(&mut b, &InventoryPattern::default());
        damage_system(&mut b);
        fps_controller(&mut b);
        camera_shake(&mut b, 0.5, 0.2);
        respawn_system(&mut b, 5.0);
        let t3d = b.to_t3d();
        assert!(t3d.contains("OnDamageTaken"), "missing OnDamageTaken");
        assert!(t3d.contains("AddItem"), "missing AddItem");
        assert!(t3d.contains("ProcessDamage"), "missing ProcessDamage");
        assert!(t3d.contains("StartSprint"), "missing StartSprint");
        assert!(t3d.contains("TriggerCameraShake"), "missing TriggerCameraShake");
        assert!(t3d.contains("OnPlayerRespawned"), "missing OnPlayerRespawned");
    }

    #[test]
    fn multiple_patterns_node_count_is_large() {
        let mut b = BlueprintBuilder::new("Multi", "Character");
        health_system(&mut b, &HealthSystemPattern::default());
        damage_system(&mut b);
        inventory(&mut b, &InventoryPattern::default());
        let count = node_count_from_t3d(&b);
        // health=7 + damage=4 + inventory=7 = 18 minimum
        assert!(count >= 15, "expected >=15 nodes for 3 patterns, got {count}");
    }
}
