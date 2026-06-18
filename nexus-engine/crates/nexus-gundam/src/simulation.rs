use anyhow::{anyhow, Result};
use crate::generated_gundam::{
    BecomeMythology, Build, CreateHistory, Discover, Expand, Explore, Preserve
};

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
        sim.history_log.push("Exploring surrounding solar systems and charting planets.".to_string());
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
        sim.history_log.push("Discovered ancient ruins of a long-lost mech civilization.".to_string());
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
        sim.history_log.push("Constructing new orbital platforms and manufacturing frames.".to_string());
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
        sim.history_log.push(format!("Preserved historical game engine state. Total: {}.", sim.preserved_count));
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
        sim.history_log.push("Civilizations expanding across boundaries, establishing new colonies.".to_string());
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
        sim.history_log.push("A legendary pilot has established a new record in battle.".to_string());
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
        sim.history_log.push("Historic battles are recorded as mythology for future cycles.".to_string());
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
        self.history_log.push(format!("Simulation phase shifted to {:?}", phase));
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
        let valid_classes = ["Worker", "Explorer", "Builder", "Miner", "Trader", "Guardian", "Warrior", "Ark"];
        if !valid_classes.contains(&class) {
            return Err(anyhow!("Invalid Mech class: {}", class));
        }
        self.mechs.push(name.to_string());
        self.history_log.push(format!("Spawned new Mech of class {}: {}", class, name));
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
        self.civilizations.push((name.to_string(), planet.to_string()));
        self.history_log.push(format!("Civilization '{}' formed on sentient planet '{}'", name, planet));
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
