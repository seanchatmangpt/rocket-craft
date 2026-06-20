use crate::mech_primitives::{
    BecomeMythology, Build, CreateHistory, Discover, Expand, Explore, Preserve,
};
use anyhow::{anyhow, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExperiencePhase {
    #[default]
    Explore,
    Discover,
    Build,
    Preserve,
    Expand,
    CreateHistory,
    BecomeMythology,
}

#[derive(Debug, Clone, Default)]
pub struct SimulationState {
    pub current_phase: ExperiencePhase,
    pub active_mechs: usize,
    pub civilization_count: usize,
    pub preserved_count: usize,
    pub history_log: Vec<String>,
}

pub trait SimulationInterface {
    fn step_phase(&mut self) -> Result<ExperiencePhase>;
    fn run_simulation_cycle(&mut self) -> Result<SimulationState>;
    fn spawn_mech(&mut self, name: &str, class: &str) -> Result<()>;
    fn form_civilization(&mut self, name: &str, planet: &str) -> Result<()>;
    fn get_state(&self) -> SimulationState;
}

// --- ExperiencePhaseTrait Definition ---

pub trait ExperiencePhaseTrait {
    fn step(&self) -> Box<dyn ExperiencePhaseTrait>;
    fn run(&self, sim: &mut GundamNexusSimulation) -> Result<()>;
    fn name(&self) -> &'static str;
}

// --- ExperiencePhaseTrait Implementations for ZSTs ---

impl ExperiencePhaseTrait for Explore {
    fn step(&self) -> Box<dyn ExperiencePhaseTrait> {
        Box::new(Discover)
    }
    fn run(&self, sim: &mut GundamNexusSimulation) -> Result<()> {
        sim.history_log
            .push("Exploring surrounding solar systems and charting planets.".to_string());
        Ok(())
    }
    fn name(&self) -> &'static str {
        "Explore"
    }
}

impl ExperiencePhaseTrait for Discover {
    fn step(&self) -> Box<dyn ExperiencePhaseTrait> {
        Box::new(Build)
    }
    fn run(&self, sim: &mut GundamNexusSimulation) -> Result<()> {
        sim.history_log
            .push("Discovered ancient ruins of a long-lost mech civilization.".to_string());
        Ok(())
    }
    fn name(&self) -> &'static str {
        "Discover"
    }
}

impl ExperiencePhaseTrait for Build {
    fn step(&self) -> Box<dyn ExperiencePhaseTrait> {
        Box::new(Preserve)
    }
    fn run(&self, sim: &mut GundamNexusSimulation) -> Result<()> {
        sim.history_log
            .push("Constructing new orbital platforms and manufacturing frames.".to_string());
        Ok(())
    }
    fn name(&self) -> &'static str {
        "Build"
    }
}

impl ExperiencePhaseTrait for Preserve {
    fn step(&self) -> Box<dyn ExperiencePhaseTrait> {
        Box::new(Expand)
    }
    fn run(&self, sim: &mut GundamNexusSimulation) -> Result<()> {
        sim.preserved_count += 1;
        sim.history_log.push(format!(
            "Preserved historical game engine state. Total: {}.",
            sim.preserved_count
        ));
        Ok(())
    }
    fn name(&self) -> &'static str {
        "Preserve"
    }
}

impl ExperiencePhaseTrait for Expand {
    fn step(&self) -> Box<dyn ExperiencePhaseTrait> {
        Box::new(CreateHistory)
    }
    fn run(&self, sim: &mut GundamNexusSimulation) -> Result<()> {
        sim.history_log.push(
            "Civilizations expanding across boundaries, establishing new colonies.".to_string(),
        );
        Ok(())
    }
    fn name(&self) -> &'static str {
        "Expand"
    }
}

impl ExperiencePhaseTrait for CreateHistory {
    fn step(&self) -> Box<dyn ExperiencePhaseTrait> {
        Box::new(BecomeMythology)
    }
    fn run(&self, sim: &mut GundamNexusSimulation) -> Result<()> {
        sim.history_log
            .push("A legendary pilot has established a new record in battle.".to_string());
        Ok(())
    }
    fn name(&self) -> &'static str {
        "CreateHistory"
    }
}

impl ExperiencePhaseTrait for BecomeMythology {
    fn step(&self) -> Box<dyn ExperiencePhaseTrait> {
        Box::new(Explore)
    }
    fn run(&self, sim: &mut GundamNexusSimulation) -> Result<()> {
        sim.history_log
            .push("Historic battles are recorded as mythology for future cycles.".to_string());
        Ok(())
    }
    fn name(&self) -> &'static str {
        "BecomeMythology"
    }
}

// --- GundamNexusSimulation ---

pub struct GundamNexusSimulation {
    pub current_phase: Box<dyn ExperiencePhaseTrait>,
    pub mechs: Vec<String>,
    pub civilizations: Vec<(String, String)>, // (name, planet)
    pub preserved_count: usize,
    pub history_log: Vec<String>,
}

impl GundamNexusSimulation {
    pub fn new() -> Self {
        Self {
            current_phase: Box::new(Explore),
            mechs: Vec::new(),
            civilizations: Vec::new(),
            preserved_count: 0,
            history_log: vec!["Simulation initialized. Planetary intelligence online.".to_string()],
        }
    }
}

impl Default for GundamNexusSimulation {
    fn default() -> Self {
        Self::new()
    }
}

impl SimulationInterface for GundamNexusSimulation {
    fn step_phase(&mut self) -> Result<ExperiencePhase> {
        let next_phase = self.current_phase.step();
        let name = next_phase.name();
        self.current_phase = next_phase;

        let phase = match name {
            "Explore" => ExperiencePhase::Explore,
            "Discover" => ExperiencePhase::Discover,
            "Build" => ExperiencePhase::Build,
            "Preserve" => ExperiencePhase::Preserve,
            "Expand" => ExperiencePhase::Expand,
            "CreateHistory" => ExperiencePhase::CreateHistory,
            "BecomeMythology" => ExperiencePhase::BecomeMythology,
            _ => ExperiencePhase::Explore,
        };
        self.history_log
            .push(format!("Simulation phase shifted to {:?}", phase));
        Ok(phase)
    }

    fn run_simulation_cycle(&mut self) -> Result<SimulationState> {
        // Temporarily take ownership of current_phase to satisfy borrow checker
        let phase = std::mem::replace(&mut self.current_phase, Box::new(Explore));
        let res = phase.run(self);
        self.current_phase = phase;
        res?;
        Ok(self.get_state())
    }

    fn spawn_mech(&mut self, name: &str, class: &str) -> Result<()> {
        if name.trim().is_empty() {
            return Err(anyhow!("Mech name cannot be empty"));
        }
        let valid_classes = [
            "Worker", "Explorer", "Builder", "Miner", "Trader", "Guardian", "Warrior", "Ark",
        ];
        if !valid_classes.contains(&class) {
            return Err(anyhow!("Invalid Mech class: {}", class));
        }
        self.mechs.push(name.to_string());
        self.history_log
            .push(format!("Spawned new Mech of class {}: {}", class, name));
        Ok(())
    }

    fn form_civilization(&mut self, name: &str, planet: &str) -> Result<()> {
        if name.trim().is_empty() {
            return Err(anyhow!("Civilization name cannot be empty"));
        }
        let valid_planets = ["Earth", "Mars", "Venus", "Sentinel"];
        if !valid_planets.contains(&planet) {
            return Err(anyhow!("Unknown planet: {}", planet));
        }
        self.civilizations
            .push((name.to_string(), planet.to_string()));
        self.history_log.push(format!(
            "Civilization '{}' formed on sentient planet '{}'",
            name, planet
        ));
        Ok(())
    }

    fn get_state(&self) -> SimulationState {
        let phase = match self.current_phase.name() {
            "Explore" => ExperiencePhase::Explore,
            "Discover" => ExperiencePhase::Discover,
            "Build" => ExperiencePhase::Build,
            "Preserve" => ExperiencePhase::Preserve,
            "Expand" => ExperiencePhase::Expand,
            "CreateHistory" => ExperiencePhase::CreateHistory,
            "BecomeMythology" => ExperiencePhase::BecomeMythology,
            _ => ExperiencePhase::Explore,
        };
        SimulationState {
            current_phase: phase,
            active_mechs: self.mechs.len(),
            civilization_count: self.civilizations.len(),
            preserved_count: self.preserved_count,
            history_log: self.history_log.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sim() -> GundamNexusSimulation {
        GundamNexusSimulation::new()
    }

    // ── initial state ─────────────────────────────────────────────────────────

    #[test]
    fn new_sim_starts_in_explore_phase() {
        let s = sim();
        assert_eq!(s.get_state().current_phase, ExperiencePhase::Explore);
    }

    #[test]
    fn new_sim_has_no_mechs_or_civilizations() {
        let s = sim();
        let state = s.get_state();
        assert_eq!(state.active_mechs, 0);
        assert_eq!(state.civilization_count, 0);
    }

    #[test]
    fn new_sim_history_log_has_one_init_entry() {
        let s = sim();
        assert_eq!(s.history_log.len(), 1);
        assert!(s.history_log[0].contains("initialized"));
    }

    // ── step_phase ────────────────────────────────────────────────────────────

    #[test]
    fn step_phase_explore_to_discover() {
        let mut s = sim();
        let next = s.step_phase().unwrap();
        assert_eq!(next, ExperiencePhase::Discover);
    }

    #[test]
    fn step_phase_full_cycle_returns_to_explore() {
        let mut s = sim();
        // Explore→Discover→Build→Preserve→Expand→CreateHistory→BecomeMythology→Explore
        for _ in 0..7 {
            s.step_phase().unwrap();
        }
        assert_eq!(s.get_state().current_phase, ExperiencePhase::Explore);
    }

    #[test]
    fn step_phase_appends_to_history_log() {
        let mut s = sim();
        let before = s.history_log.len();
        s.step_phase().unwrap();
        assert!(s.history_log.len() > before);
    }

    // ── run_simulation_cycle ──────────────────────────────────────────────────

    #[test]
    fn run_cycle_returns_current_state() {
        let mut s = sim();
        let state = s.run_simulation_cycle().unwrap();
        assert_eq!(state.current_phase, ExperiencePhase::Explore);
    }

    #[test]
    fn run_cycle_in_preserve_phase_increments_preserved_count() {
        let mut s = sim();
        // Advance to Preserve (3 steps: Explore→Discover→Build→Preserve)
        for _ in 0..3 { s.step_phase().unwrap(); }
        assert_eq!(s.get_state().current_phase, ExperiencePhase::Preserve);
        let before = s.preserved_count;
        s.run_simulation_cycle().unwrap();
        assert_eq!(s.preserved_count, before + 1);
    }

    // ── spawn_mech ────────────────────────────────────────────────────────────

    #[test]
    fn spawn_valid_mech_succeeds() {
        let mut s = sim();
        s.spawn_mech("Nu Gundam", "Warrior").unwrap();
        assert_eq!(s.get_state().active_mechs, 1);
    }

    #[test]
    fn spawn_mech_empty_name_is_rejected() {
        let mut s = sim();
        assert!(s.spawn_mech("", "Warrior").is_err());
        assert!(s.spawn_mech("   ", "Warrior").is_err());
    }

    #[test]
    fn spawn_mech_invalid_class_is_rejected() {
        let mut s = sim();
        assert!(s.spawn_mech("Unicorn", "GodKing").is_err());
    }

    #[test]
    fn spawn_mech_all_valid_classes_succeed() {
        let mut s = sim();
        for class in ["Worker", "Explorer", "Builder", "Miner", "Trader", "Guardian", "Warrior", "Ark"] {
            s.spawn_mech(class, class).unwrap();
        }
        assert_eq!(s.get_state().active_mechs, 8);
    }

    // ── form_civilization ─────────────────────────────────────────────────────

    #[test]
    fn form_civilization_on_valid_planet_succeeds() {
        let mut s = sim();
        s.form_civilization("Zanscare Empire", "Mars").unwrap();
        assert_eq!(s.get_state().civilization_count, 1);
    }

    #[test]
    fn form_civilization_empty_name_is_rejected() {
        let mut s = sim();
        assert!(s.form_civilization("", "Earth").is_err());
    }

    #[test]
    fn form_civilization_unknown_planet_is_rejected() {
        let mut s = sim();
        assert!(s.form_civilization("Britannia", "Jupiter").is_err());
    }

    #[test]
    fn form_civilization_all_valid_planets_succeed() {
        let mut s = sim();
        for planet in ["Earth", "Mars", "Venus", "Sentinel"] {
            s.form_civilization(&format!("Civ-{planet}"), planet).unwrap();
        }
        assert_eq!(s.get_state().civilization_count, 4);
    }
}
