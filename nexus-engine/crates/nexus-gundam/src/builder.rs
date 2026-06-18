use crate::generated_gundam::{
    PlanetCategory, MobilityTypeCategory, Earth, Mars, Venus,
    Frame, Power, Armor, Weapon, Sensor, UtilitySystem, Joint, AABB
};
use serde::{Serialize, Deserialize};

pub struct Unset;
pub struct Set<T>(pub T);

// --- Mech Struct & Verification ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mech<Mob> {
    pub frame: Frame,
    pub mobility: Mob,
    pub power: Power,
    pub armor: Armor,
    pub weapons: Vec<Weapon>,
    pub sensors: Vec<Sensor>,
    pub utility: Vec<UtilitySystem>,
    pub joints: Vec<Joint>,
    pub class: String,
}

/// A builder for constructing a custom Gundam/Mech assembly with typestate safety.
///
/// Only mechs with a set mobility component can be built.
///
/// # Examples
///
/// ```
/// use nexus_gundam::builder::{MechBuilder, Set, Unset};
/// use nexus_gundam::generated_gundam::{Frame, Power, Armor, Flight, Mobility, AABB};
///
/// let frame = Frame::default();
/// let power = Power::default();
/// let armor = Armor::default();
/// let mobility = Flight {
///     physical: Mobility {
///         id: "FlightModule".to_string(),
///         mass: 100.0,
///         occupancy: AABB::default(),
///         clearance: AABB::default(),
///         load_capacity: 1000.0,
///         max_speed: 150.0,
///     },
///     wing_span: 12.0,
/// };
///
/// let mech = MechBuilder::new()
///     .with_frame(frame)
///     .with_power(power)
///     .with_armor(armor)
///     .with_mobility(mobility)
///     .with_class("AerialGuardian")
///     .build();
///
/// assert_eq!(mech.class, "AerialGuardian");
/// ```
#[derive(Debug, Clone)]
pub struct MechBuilder<MobilityState> {
    pub frame: Option<Frame>,
    pub mobility: MobilityState,
    pub power: Option<Power>,
    pub armor: Option<Armor>,
    pub weapons: Vec<Weapon>,
    pub sensors: Vec<Sensor>,
    pub utility: Vec<UtilitySystem>,
    pub joints: Vec<Joint>,
    pub class: Option<String>,
}

impl MechBuilder<Unset> {
    /// Create a new empty `MechBuilder` where mobility is unset.
    pub fn new() -> Self {
        Self {
            frame: None,
            mobility: Unset,
            power: None,
            armor: None,
            weapons: Vec::new(),
            sensors: Vec::new(),
            utility: Vec::new(),
            joints: Vec::new(),
            class: None,
        }
    }
}

impl Default for MechBuilder<Unset> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M> MechBuilder<M> {
    pub fn with_frame(mut self, frame: Frame) -> Self {
        self.frame = Some(frame);
        self
    }

    pub fn with_mobility<Mob: MobilityTypeCategory>(self, mobility: Mob) -> MechBuilder<Set<Mob>> {
        MechBuilder {
            frame: self.frame,
            mobility: Set(mobility),
            power: self.power,
            armor: self.armor,
            weapons: self.weapons,
            sensors: self.sensors,
            utility: self.utility,
            joints: self.joints,
            class: self.class,
        }
    }

    pub fn with_power(mut self, power: Power) -> Self {
        self.power = Some(power);
        self
    }

    pub fn with_armor(mut self, armor: Armor) -> Self {
        self.armor = Some(armor);
        self
    }

    pub fn add_weapon(mut self, weapon: Weapon) -> Self {
        self.weapons.push(weapon);
        self
    }

    pub fn add_sensor(mut self, sensor: Sensor) -> Self {
        self.sensors.push(sensor);
        self
    }

    pub fn add_utility(mut self, utility: UtilitySystem) -> Self {
        self.utility.push(utility);
        self
    }

    pub fn add_joint(mut self, joint: Joint) -> Self {
        self.joints.push(joint);
        self
    }

    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into());
        self
    }
}

impl<Mob: MobilityTypeCategory> MechBuilder<Set<Mob>> {
    pub fn build(self) -> Mech<Mob> {
        Mech {
            frame: self.frame.unwrap_or_default(),
            mobility: self.mobility.0,
            power: self.power.unwrap_or_default(),
            armor: self.armor.unwrap_or_default(),
            weapons: self.weapons,
            sensors: self.sensors,
            utility: self.utility,
            joints: self.joints,
            class: self.class.unwrap_or_else(|| "Warrior".to_string()),
        }
    }
}

// --- Validation and Receipt Types ---

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Joint '{name}' has no rotation limits defined")]
    MissingJointLimits { name: String },

    #[error("Joint '{name}' has invalid rotation limits: min exceeds max on axis {axis}")]
    InvalidJointLimits { name: String, axis: String },

    #[error("Collision detected: component '{first}' occupancy overlaps with '{second}'")]
    CollisionDetected { first: String, second: String },

    #[error("Clearance violation: component '{component}' occupancy overlaps with clearance zone of '{other}'")]
    ClearanceViolation { component: String, other: String },

    #[error("Total mech mass ({total_mass} kg) exceeds mobility load capacity ({load_capacity} kg)")]
    LoadCapacityExceeded { total_mass: f32, load_capacity: f32 },

    #[error("Planetary incompatibility: {message}")]
    PlanetaryIncompatibility { message: String },

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyReceipt {
    pub lineage_hash: String,
    pub total_mass: f32,
    pub load_capacity: f32,
    pub mobility_type: String,
    pub component_count: usize,
    pub timestamp: String,
}

pub struct ComponentMetadata {
    pub id: String,
    pub mass: f32,
    pub occupancy: AABB,
    pub clearance: AABB,
}

impl<Mob: MobilityTypeCategory> Mech<Mob> {
    pub fn get_all_components(&self) -> Vec<ComponentMetadata> {
        let mut components = vec![
            ComponentMetadata {
                id: self.frame.id.clone(),
                mass: self.frame.mass,
                occupancy: self.frame.occupancy,
                clearance: self.frame.clearance,
            },
            ComponentMetadata {
                id: self.mobility.physical().id.clone(),
                mass: self.mobility.physical().mass,
                occupancy: self.mobility.physical().occupancy,
                clearance: self.mobility.physical().clearance,
            },
            ComponentMetadata {
                id: self.power.id.clone(),
                mass: self.power.mass,
                occupancy: self.power.occupancy,
                clearance: self.power.clearance,
            },
            ComponentMetadata {
                id: self.armor.id.clone(),
                mass: self.armor.mass,
                occupancy: self.armor.occupancy,
                clearance: self.armor.clearance,
            },
        ];

        for w in &self.weapons {
            components.push(ComponentMetadata {
                id: w.id.clone(),
                mass: w.mass,
                occupancy: w.occupancy,
                clearance: w.clearance,
            });
        }
        for s in &self.sensors {
            components.push(ComponentMetadata {
                id: s.id.clone(),
                mass: s.mass,
                occupancy: s.occupancy,
                clearance: s.clearance,
            });
        }
        for u in &self.utility {
            components.push(ComponentMetadata {
                id: u.id.clone(),
                mass: u.mass,
                occupancy: u.occupancy,
                clearance: u.clearance,
            });
        }
        components
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        // 1. Joint limits check
        for j in &self.joints {
            let limits = j.limits.ok_or_else(|| ValidationError::MissingJointLimits {
                name: j.name.clone(),
            })?;
            if limits.min_yaw > limits.max_yaw {
                return Err(ValidationError::InvalidJointLimits { name: j.name.clone(), axis: "yaw".to_string() });
            }
            if limits.min_pitch > limits.max_pitch {
                return Err(ValidationError::InvalidJointLimits { name: j.name.clone(), axis: "pitch".to_string() });
            }
            if limits.min_roll > limits.max_roll {
                return Err(ValidationError::InvalidJointLimits { name: j.name.clone(), axis: "roll".to_string() });
            }
        }

        let comps = self.get_all_components();

        // 2. Collision and clearance check (n^2 pairwise)
        for i in 0..comps.len() {
            for j in (i + 1)..comps.len() {
                let a = &comps[i];
                let b = &comps[j];

                if a.occupancy.intersects(&b.occupancy) {
                    return Err(ValidationError::CollisionDetected {
                        first: a.id.clone(),
                        second: b.id.clone(),
                    });
                }
                if a.occupancy.intersects(&b.clearance) {
                    return Err(ValidationError::ClearanceViolation {
                        component: a.id.clone(),
                        other: b.id.clone(),
                    });
                }
                if b.occupancy.intersects(&a.clearance) {
                    return Err(ValidationError::ClearanceViolation {
                        component: b.id.clone(),
                        other: a.id.clone(),
                    });
                }
            }
        }

        // 3. Load capacity check
        let total_mass: f32 = comps.iter().map(|c| c.mass).sum::<f32>() 
            + self.joints.iter().map(|j| j.mass).sum::<f32>();
        
        let load_capacity = self.mobility.physical().load_capacity;
        if total_mass > load_capacity {
            return Err(ValidationError::LoadCapacityExceeded {
                total_mass,
                load_capacity,
            });
        }

        Ok(())
    }

    pub fn generate_receipt(&self) -> Result<AssemblyReceipt, ValidationError> {
        self.validate()?;
        
        let serialized = serde_json::to_vec(self)
            .map_err(|e| ValidationError::SerializationError(e.to_string()))?;
        
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(&serialized);
        let lineage_hash = format!("{:x}", hasher.finalize());

        let total_mass: f32 = self.get_all_components().iter().map(|c| c.mass).sum::<f32>() 
            + self.joints.iter().map(|j| j.mass).sum::<f32>();

        Ok(AssemblyReceipt {
            lineage_hash,
            total_mass,
            load_capacity: self.mobility.physical().load_capacity,
            mobility_type: self.mobility.type_name().to_string(),
            component_count: self.get_all_components().len() + self.joints.len(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
}

// --- Planetary Compatibility Trait & SPECIALIZATIONS ---

pub trait PlanetMechCompatibility {
    fn validate_compatibility(&self, class: &str, mobility_name: &str) -> Result<(), ValidationError>;
}

impl PlanetMechCompatibility for Earth {
    fn validate_compatibility(&self, _class: &str, _mobility_name: &str) -> Result<(), ValidationError> {
        Ok(())
    }
}

impl PlanetMechCompatibility for Mars {
    fn validate_compatibility(&self, class: &str, _mobility_name: &str) -> Result<(), ValidationError> {
        if class != "Guardian" && class != "Miner" {
            return Err(ValidationError::PlanetaryIncompatibility {
                message: format!("Mars mechs must be of class Guardian or Miner, found '{}'", class),
            });
        }
        Ok(())
    }
}

impl PlanetMechCompatibility for Venus {
    fn validate_compatibility(&self, _class: &str, mobility_name: &str) -> Result<(), ValidationError> {
        if mobility_name != "Flight" && mobility_name != "Hover" {
            return Err(ValidationError::PlanetaryIncompatibility {
                message: format!("Venus mechs must use Flight or Hover mobility, found '{}'", mobility_name),
            });
        }
        Ok(())
    }
}

// --- Civilization Struct & Specializations ---

#[derive(Debug, Clone)]
pub struct Civilization<Plan> {
    pub planet: Plan,
    pub name: String,
    pub history: Vec<String>,
    pub values: Vec<String>,
    pub environment: String,
    pub resources: Vec<String>,
}

/// A builder for creating custom civilizations on planets with typestate-enforced rules.
///
/// Only civilizations with a declared planet can be built.
///
/// # Examples
///
/// ```
/// use nexus_gundam::builder::{CivilizationBuilder, Set, Unset};
/// use nexus_gundam::generated_gundam::Earth;
///
/// let earth_civ = CivilizationBuilder::new()
///     .with_planet(Earth)
///     .with_name("United Earth Sphere Alliance")
///     .build();
///
/// assert_eq!(earth_civ.name, "United Earth Sphere Alliance");
/// ```
#[derive(Debug, Clone)]
pub struct CivilizationBuilder<PlanetState> {
    pub planet: PlanetState,
    pub name: Option<String>,
    pub history: Vec<String>,
    pub values: Vec<String>,
    pub environment: Option<String>,
    pub resources: Vec<String>,
}

impl CivilizationBuilder<Unset> {
    /// Create a new `CivilizationBuilder` with planet unset.
    pub fn new() -> Self {
        Self {
            planet: Unset,
            name: None,
            history: Vec::new(),
            values: Vec::new(),
            environment: None,
            resources: Vec::new(),
        }
    }
}

impl Default for CivilizationBuilder<Unset> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P> CivilizationBuilder<P> {
    pub fn with_planet<Plan: PlanetCategory>(self, planet: Plan) -> CivilizationBuilder<Set<Plan>> {
        CivilizationBuilder {
            planet: Set(planet),
            name: self.name,
            history: self.history,
            values: self.values,
            environment: self.environment,
            resources: self.resources,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn add_history(mut self, event: impl Into<String>) -> Self {
        self.history.push(event.into());
        self
    }

    pub fn add_value(mut self, value: impl Into<String>) -> Self {
        self.values.push(value.into());
        self
    }

    pub fn with_environment(mut self, env: impl Into<String>) -> Self {
        self.environment = Some(env.into());
        self
    }

    pub fn add_resource(mut self, res: impl Into<String>) -> Self {
        self.resources.push(res.into());
        self
    }
}

impl<Plan: PlanetCategory> CivilizationBuilder<Set<Plan>> {
    pub fn build(self) -> Civilization<Plan> {
        Civilization {
            planet: self.planet.0,
            name: self.name.unwrap_or_else(|| "New Horizon".to_string()),
            history: self.history,
            values: self.values,
            environment: self.environment.unwrap_or_else(|| "Terraformed".to_string()),
            resources: self.resources,
        }
    }
}

impl Civilization<Earth> {
    pub fn spawn_mech_builder(&self) -> MechBuilder<Unset> {
        MechBuilder::new()
            .with_frame(Frame {
                id: "Earth_Balanced_Frame".to_string(),
                mass: 50.0,
                occupancy: AABB::new([-1.0, 0.0, -1.0], [1.0, 3.0, 1.0]),
                clearance: AABB::new([-1.2, 0.0, -1.2], [1.2, 3.2, 1.2]),
                slot_count: 6,
            })
            .with_armor(Armor {
                id: "Luna_Titanium_Armor".to_string(),
                mass: 30.0,
                occupancy: AABB::new([-1.1, 0.0, -1.1], [1.1, 3.1, 1.1]),
                clearance: AABB::new([-1.15, 0.0, -1.15], [1.15, 3.15, 1.15]),
                defense_rating: 150.0,
                material: "Luna Titanium".to_string(),
            })
            .with_class("Warrior")
    }
}

impl Civilization<Mars> {
    pub fn spawn_mech_builder(&self) -> MechBuilder<Unset> {
        MechBuilder::new()
            .with_frame(Frame {
                id: "Mars_Heavy_Frame".to_string(),
                mass: 120.0,
                occupancy: AABB::new([-1.5, 0.0, -1.5], [1.5, 4.0, 1.5]),
                clearance: AABB::new([-1.7, 0.0, -1.7], [1.7, 4.2, 1.7]),
                slot_count: 8,
            })
            .with_armor(Armor {
                id: "Mars_Heavy_Chobham_Armor".to_string(),
                mass: 100.0,
                occupancy: AABB::new([-1.6, 0.0, -1.6], [1.6, 4.1, 1.6]),
                clearance: AABB::new([-1.65, 0.0, -1.65], [1.65, 4.15, 1.65]),
                defense_rating: 300.0,
                material: "Heavy Chobham".to_string(),
            })
            .with_class("Guardian")
    }
}

impl Civilization<Venus> {
    pub fn spawn_mech_builder(&self) -> MechBuilder<Unset> {
        MechBuilder::new()
            .with_frame(Frame {
                id: "Venus_Lightweight_Frame".to_string(),
                mass: 25.0,
                occupancy: AABB::new([-0.8, 0.0, -0.8], [0.8, 2.5, 0.8]),
                clearance: AABB::new([-1.0, 0.0, -1.0], [1.0, 2.7, 1.0]),
                slot_count: 4,
            })
            .with_power(Power {
                id: "Venus_High_Output_Reactor".to_string(),
                mass: 15.0,
                occupancy: AABB::new([-0.4, 0.5, -0.4], [0.4, 1.5, 0.4]),
                clearance: AABB::new([-0.5, 0.4, -0.5], [0.5, 1.6, 0.5]),
                energy_capacity: 1500.0,
                output: 150.0,
            })
            .with_armor(Armor {
                id: "Venus_Aerodynamic_Composite_Armor".to_string(),
                mass: 15.0,
                occupancy: AABB::new([-0.9, 0.0, -0.9], [0.9, 2.6, 0.9]),
                clearance: AABB::new([-0.95, 0.0, -0.95], [0.95, 2.65, 0.95]),
                defense_rating: 60.0,
                material: "Aerogel Composites".to_string(),
            })
            .with_class("Explorer")
    }
}
