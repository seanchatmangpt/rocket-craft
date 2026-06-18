#![allow(clippy::type_complexity)]
use nexus_gundam::generated_gundam::{Mars, PlanetCategory, RotationLimits, Venus, AABB};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::marker::PhantomData;
use thiserror::Error;

// --- Marker States & Typestates ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unvalidated;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validated;

pub struct Unset;
pub struct Set<T>(pub T);

// --- Assembly Specification Typestates (R1) ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    pub id: String,
    pub size: [f32; 3],
    pub scale: f32,
    pub load_capacity: f32,
    pub center_of_mass: [f32; 3],
    pub mobility_class: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Joint {
    pub rotation_limits: Option<RotationLimits>,
    pub extension_limits: [f32; 2],
    pub attachment_limits: [f32; 2],
    pub compatibility_rules: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotionProfile {
    Walk,
    Run,
    Flight,
    Hover,
    Construction,
    Mining,
    Combat,
    Repair,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Power {
    pub mass: f32,
    pub energy_capacity: f32,
    pub output: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollisionVolume {
    pub physical_occupancy: AABB,
    pub interaction_zones: Vec<AABB>,
    pub damage_zones: Vec<AABB>,
    pub clearance_volumes: AABB,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialSpec {
    pub structural: String,
    pub armor: String,
    pub visual: String,
    pub wear_state: f32,
    pub environmental: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanetaryValues {
    pub faith: f32,
    pub ambition: f32,
    pub beauty: f32,
    pub community: f32,
    pub order: f32,
    pub knowledge: f32,
}

#[derive(Serialize, Deserialize)]
#[serde(bound = "")]
pub struct CulturalProfile<P = nexus_gundam::generated_gundam::Earth> {
    pub planetary_values: PlanetaryValues,
    #[serde(skip)]
    pub _marker: PhantomData<P>,
}

impl<P> Clone for CulturalProfile<P> {
    fn clone(&self) -> Self {
        Self {
            planetary_values: self.planetary_values.clone(),
            _marker: PhantomData,
        }
    }
}

impl<P> PartialEq for CulturalProfile<P> {
    fn eq(&self, other: &Self) -> bool {
        self.planetary_values == other.planetary_values
    }
}

impl<P> std::fmt::Debug for CulturalProfile<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CulturalProfile")
            .field("planetary_values", &self.planetary_values)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FunctionalRole {
    Worker,
    Builder,
    Miner,
    Explorer,
    Transport,
    Guardian,
    Warrior,
    Ark,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct MechAssemblySpec<State = Unvalidated, P = nexus_gundam::generated_gundam::Earth> {
    pub frame: Frame,
    pub joints: Vec<Joint>,
    pub power: Power,
    pub motion_profile: MotionProfile,
    pub collision_volume: CollisionVolume,
    pub material_spec: MaterialSpec,
    pub cultural_profile: CulturalProfile<P>,
    pub functional_role: FunctionalRole,
    pub equipment_mass: f32,
    #[serde(skip)]
    pub _state: PhantomData<State>,
}

impl<State, P> Clone for MechAssemblySpec<State, P> {
    fn clone(&self) -> Self {
        Self {
            frame: self.frame.clone(),
            joints: self.joints.clone(),
            power: self.power.clone(),
            motion_profile: self.motion_profile,
            collision_volume: self.collision_volume.clone(),
            material_spec: self.material_spec.clone(),
            cultural_profile: self.cultural_profile.clone(),
            functional_role: self.functional_role,
            equipment_mass: self.equipment_mass,
            _state: PhantomData,
        }
    }
}

impl<State, P> PartialEq for MechAssemblySpec<State, P> {
    fn eq(&self, other: &Self) -> bool {
        self.frame == other.frame
            && self.joints == other.joints
            && self.power == other.power
            && self.motion_profile == other.motion_profile
            && self.collision_volume == other.collision_volume
            && self.material_spec == other.material_spec
            && self.cultural_profile == other.cultural_profile
            && self.functional_role == other.functional_role
            && self.equipment_mass == other.equipment_mass
    }
}

// --- Builder Pattern with Typestates ---

pub struct MechAssemblySpecBuilder<
    P,
    FrameState = Unset,
    JointsState = Unset,
    PowerState = Unset,
    MotionProfileState = Unset,
    CollisionVolumeState = Unset,
    MaterialSpecState = Unset,
    CulturalProfileState = Unset,
    FunctionalRoleState = Unset,
> {
    frame: Option<Frame>,
    joints: Option<Vec<Joint>>,
    power: Option<Power>,
    motion_profile: Option<MotionProfile>,
    collision_volume: Option<CollisionVolume>,
    material_spec: Option<MaterialSpec>,
    cultural_profile: Option<CulturalProfile<P>>,
    functional_role: Option<FunctionalRole>,
    equipment_mass: f32,
    _marker: PhantomData<P>,
    _states: PhantomData<(
        FrameState,
        JointsState,
        PowerState,
        MotionProfileState,
        CollisionVolumeState,
        MaterialSpecState,
        CulturalProfileState,
        FunctionalRoleState,
    )>,
}

impl<P> MechAssemblySpecBuilder<P, Unset, Unset, Unset, Unset, Unset, Unset, Unset, Unset> {
    pub fn new() -> Self {
        Self {
            frame: None,
            joints: None,
            power: None,
            motion_profile: None,
            collision_volume: None,
            material_spec: None,
            cultural_profile: None,
            functional_role: None,
            equipment_mass: 0.0,
            _marker: PhantomData,
            _states: PhantomData,
        }
    }
}

impl<P> Default
    for MechAssemblySpecBuilder<P, Unset, Unset, Unset, Unset, Unset, Unset, Unset, Unset>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<P, F, J, Po, Mo, C, Ma, Cu, Fu> MechAssemblySpecBuilder<P, F, J, Po, Mo, C, Ma, Cu, Fu> {
    pub fn frame(
        self,
        frame: Frame,
    ) -> MechAssemblySpecBuilder<P, Set<Frame>, J, Po, Mo, C, Ma, Cu, Fu> {
        MechAssemblySpecBuilder {
            frame: Some(frame),
            joints: self.joints,
            power: self.power,
            motion_profile: self.motion_profile,
            collision_volume: self.collision_volume,
            material_spec: self.material_spec,
            cultural_profile: self.cultural_profile,
            functional_role: self.functional_role,
            equipment_mass: self.equipment_mass,
            _marker: PhantomData,
            _states: PhantomData,
        }
    }

    pub fn joints(
        self,
        joints: Vec<Joint>,
    ) -> MechAssemblySpecBuilder<P, F, Set<Vec<Joint>>, Po, Mo, C, Ma, Cu, Fu> {
        MechAssemblySpecBuilder {
            frame: self.frame,
            joints: Some(joints),
            power: self.power,
            motion_profile: self.motion_profile,
            collision_volume: self.collision_volume,
            material_spec: self.material_spec,
            cultural_profile: self.cultural_profile,
            functional_role: self.functional_role,
            equipment_mass: self.equipment_mass,
            _marker: PhantomData,
            _states: PhantomData,
        }
    }

    pub fn power(
        self,
        power: Power,
    ) -> MechAssemblySpecBuilder<P, F, J, Set<Power>, Mo, C, Ma, Cu, Fu> {
        MechAssemblySpecBuilder {
            frame: self.frame,
            joints: self.joints,
            power: Some(power),
            motion_profile: self.motion_profile,
            collision_volume: self.collision_volume,
            material_spec: self.material_spec,
            cultural_profile: self.cultural_profile,
            functional_role: self.functional_role,
            equipment_mass: self.equipment_mass,
            _marker: PhantomData,
            _states: PhantomData,
        }
    }

    pub fn motion_profile(
        self,
        motion_profile: MotionProfile,
    ) -> MechAssemblySpecBuilder<P, F, J, Po, Set<MotionProfile>, C, Ma, Cu, Fu> {
        MechAssemblySpecBuilder {
            frame: self.frame,
            joints: self.joints,
            power: self.power,
            motion_profile: Some(motion_profile),
            collision_volume: self.collision_volume,
            material_spec: self.material_spec,
            cultural_profile: self.cultural_profile,
            functional_role: self.functional_role,
            equipment_mass: self.equipment_mass,
            _marker: PhantomData,
            _states: PhantomData,
        }
    }

    pub fn collision_volume(
        self,
        collision_volume: CollisionVolume,
    ) -> MechAssemblySpecBuilder<P, F, J, Po, Mo, Set<CollisionVolume>, Ma, Cu, Fu> {
        MechAssemblySpecBuilder {
            frame: self.frame,
            joints: self.joints,
            power: self.power,
            motion_profile: self.motion_profile,
            collision_volume: Some(collision_volume),
            material_spec: self.material_spec,
            cultural_profile: self.cultural_profile,
            functional_role: self.functional_role,
            equipment_mass: self.equipment_mass,
            _marker: PhantomData,
            _states: PhantomData,
        }
    }

    pub fn material_spec(
        self,
        material_spec: MaterialSpec,
    ) -> MechAssemblySpecBuilder<P, F, J, Po, Mo, C, Set<MaterialSpec>, Cu, Fu> {
        MechAssemblySpecBuilder {
            frame: self.frame,
            joints: self.joints,
            power: self.power,
            motion_profile: self.motion_profile,
            collision_volume: self.collision_volume,
            material_spec: Some(material_spec),
            cultural_profile: self.cultural_profile,
            functional_role: self.functional_role,
            equipment_mass: self.equipment_mass,
            _marker: PhantomData,
            _states: PhantomData,
        }
    }

    pub fn cultural_profile(
        self,
        cultural_profile: CulturalProfile<P>,
    ) -> MechAssemblySpecBuilder<P, F, J, Po, Mo, C, Ma, Set<CulturalProfile<P>>, Fu> {
        MechAssemblySpecBuilder {
            frame: self.frame,
            joints: self.joints,
            power: self.power,
            motion_profile: self.motion_profile,
            collision_volume: self.collision_volume,
            material_spec: self.material_spec,
            cultural_profile: Some(cultural_profile),
            functional_role: self.functional_role,
            equipment_mass: self.equipment_mass,
            _marker: PhantomData,
            _states: PhantomData,
        }
    }

    pub fn functional_role(
        self,
        functional_role: FunctionalRole,
    ) -> MechAssemblySpecBuilder<P, F, J, Po, Mo, C, Ma, Cu, Set<FunctionalRole>> {
        MechAssemblySpecBuilder {
            frame: self.frame,
            joints: self.joints,
            power: self.power,
            motion_profile: self.motion_profile,
            collision_volume: self.collision_volume,
            material_spec: self.material_spec,
            cultural_profile: self.cultural_profile,
            functional_role: Some(functional_role),
            equipment_mass: self.equipment_mass,
            _marker: PhantomData,
            _states: PhantomData,
        }
    }

    pub fn equipment_mass(mut self, equipment_mass: f32) -> Self {
        self.equipment_mass = equipment_mass;
        self
    }
}

impl<P>
    MechAssemblySpecBuilder<
        P,
        Set<Frame>,
        Set<Vec<Joint>>,
        Set<Power>,
        Set<MotionProfile>,
        Set<CollisionVolume>,
        Set<MaterialSpec>,
        Set<CulturalProfile<P>>,
        Set<FunctionalRole>,
    >
{
    pub fn build(self) -> MechAssemblySpec<Unvalidated, P> {
        MechAssemblySpec {
            frame: self.frame.unwrap(),
            joints: self.joints.unwrap(),
            power: self.power.unwrap(),
            motion_profile: self.motion_profile.unwrap(),
            collision_volume: self.collision_volume.unwrap(),
            material_spec: self.material_spec.unwrap(),
            cultural_profile: self.cultural_profile.unwrap(),
            functional_role: self.functional_role.unwrap(),
            equipment_mass: self.equipment_mass,
            _state: PhantomData,
        }
    }
}

// --- Motion & Collision Validation Gates (R2) ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Error)]
pub enum Gate {
    #[error("Gate 1: Load capacity check failed")]
    Gate1,
    #[error("Gate 2: Joint rotation limits check failed")]
    Gate2,
    #[error("Gate 3: Collision volumes check failed")]
    Gate3,
    #[error("Gate 4: Motion profile compatibility check failed")]
    Gate4,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
#[error("Assembly refusal at {gate}: {reason}")]
pub struct AssemblyRefusal {
    pub gate: Gate,
    pub reason: String,
}

fn validate_aabb(aabb: &AABB) -> Result<(), String> {
    for i in 0..3 {
        let min_val = aabb.min[i];
        let max_val = aabb.max[i];
        if min_val.is_nan() || min_val.is_infinite() || max_val.is_nan() || max_val.is_infinite() {
            return Err("AABB contains NaN or Infinite coordinate".to_string());
        }
        if min_val > max_val {
            return Err(format!(
                "AABB min ({}) is greater than max ({}) on axis {}",
                min_val, max_val, i
            ));
        }
    }
    Ok(())
}

impl<P: PlanetCategory + 'static> MechAssemblySpec<Unvalidated, P> {
    pub fn validate(self) -> Result<MechAssemblySpec<Validated, P>, AssemblyRefusal> {
        // Gate 1: Load capacity check
        // Reject NaN, Infinite, or negative masses for equipment_mass and power.mass
        if self.equipment_mass.is_nan()
            || self.equipment_mass.is_infinite()
            || self.equipment_mass < 0.0
        {
            return Err(AssemblyRefusal {
                gate: Gate::Gate1,
                reason: format!("Invalid equipment mass: {}", self.equipment_mass),
            });
        }
        if self.power.mass.is_nan() || self.power.mass.is_infinite() || self.power.mass < 0.0 {
            return Err(AssemblyRefusal {
                gate: Gate::Gate1,
                reason: format!("Invalid power mass: {}", self.power.mass),
            });
        }
        // Validate load_capacity: reject NaN, infinite, or negative values
        if self.frame.load_capacity.is_nan()
            || self.frame.load_capacity.is_infinite()
            || self.frame.load_capacity < 0.0
        {
            return Err(AssemblyRefusal {
                gate: Gate::Gate1,
                reason: format!("Invalid frame load capacity: {}", self.frame.load_capacity),
            });
        }
        // Validate scale: reject NaN, infinite, or <= 0.0 values
        if self.frame.scale.is_nan() || self.frame.scale.is_infinite() || self.frame.scale <= 0.0 {
            return Err(AssemblyRefusal {
                gate: Gate::Gate1,
                reason: format!("Invalid frame scale: {}", self.frame.scale),
            });
        }
        // Validate size: reject any coordinate that is NaN, infinite, or <= 0.0
        for (i, &coord) in self.frame.size.iter().enumerate() {
            if coord.is_nan() || coord.is_infinite() || coord <= 0.0 {
                return Err(AssemblyRefusal {
                    gate: Gate::Gate1,
                    reason: format!("Invalid frame size coordinate at index {}: {}", i, coord),
                });
            }
        }
        // Validate energy_capacity: reject NaN, infinite, or negative values
        if self.power.energy_capacity.is_nan()
            || self.power.energy_capacity.is_infinite()
            || self.power.energy_capacity < 0.0
        {
            return Err(AssemblyRefusal {
                gate: Gate::Gate1,
                reason: format!(
                    "Invalid power energy capacity: {}",
                    self.power.energy_capacity
                ),
            });
        }
        // Validate output: reject NaN, infinite, or negative values
        if self.power.output.is_nan() || self.power.output.is_infinite() || self.power.output < 0.0
        {
            return Err(AssemblyRefusal {
                gate: Gate::Gate1,
                reason: format!("Invalid power output: {}", self.power.output),
            });
        }

        // Validate center_of_mass: reject if any coordinate is NaN or infinite
        for (i, &coord) in self.frame.center_of_mass.iter().enumerate() {
            if coord.is_nan() || coord.is_infinite() {
                return Err(AssemblyRefusal {
                    gate: Gate::Gate1,
                    reason: format!(
                        "Invalid frame center of mass coordinate at index {}: {}",
                        i, coord
                    ),
                });
            }
        }

        // Validate wear_state: reject if NaN, infinite, or negative
        if self.material_spec.wear_state.is_nan()
            || self.material_spec.wear_state.is_infinite()
            || self.material_spec.wear_state < 0.0
        {
            return Err(AssemblyRefusal {
                gate: Gate::Gate1,
                reason: format!(
                    "Invalid material wear state: {}",
                    self.material_spec.wear_state
                ),
            });
        }

        let total_load = self.equipment_mass + self.power.mass;
        if total_load > self.frame.load_capacity {
            return Err(AssemblyRefusal {
                gate: Gate::Gate1,
                reason: format!(
                    "Attached equipment mass ({}) + power mass ({}) exceeds frame load capacity ({})",
                    self.equipment_mass, self.power.mass, self.frame.load_capacity
                ),
            });
        }

        // Gate 2: Joint rotation limits check
        // Reject NaN/Infinite rotation limits, and reject any limit where min > max
        for (i, joint) in self.joints.iter().enumerate() {
            let limits = joint.rotation_limits.ok_or_else(|| AssemblyRefusal {
                gate: Gate::Gate2,
                reason: format!("Joint at index {} has no defined rotation limits", i),
            })?;

            let check_val = |val: f32| val.is_nan() || val.is_infinite();
            if check_val(limits.min_yaw)
                || check_val(limits.max_yaw)
                || check_val(limits.min_pitch)
                || check_val(limits.max_pitch)
                || check_val(limits.min_roll)
                || check_val(limits.max_roll)
            {
                return Err(AssemblyRefusal {
                    gate: Gate::Gate2,
                    reason: format!("Joint at index {} has NaN or Infinite rotation limits", i),
                });
            }

            if limits.min_yaw > limits.max_yaw
                || limits.min_pitch > limits.max_pitch
                || limits.min_roll > limits.max_roll
            {
                return Err(AssemblyRefusal {
                    gate: Gate::Gate2,
                    reason: format!("Joint at index {} has invalid limits where min > max", i),
                });
            }

            // Validate extension_limits and attachment_limits
            let ext_min = joint.extension_limits[0];
            let ext_max = joint.extension_limits[1];
            if ext_min.is_nan()
                || ext_min.is_infinite()
                || ext_max.is_nan()
                || ext_max.is_infinite()
            {
                return Err(AssemblyRefusal {
                    gate: Gate::Gate2,
                    reason: format!("Joint at index {} has NaN or Infinite extension limits", i),
                });
            }
            if ext_min > ext_max {
                return Err(AssemblyRefusal {
                    gate: Gate::Gate2,
                    reason: format!(
                        "Joint at index {} has invalid extension limits where min ({}) > max ({})",
                        i, ext_min, ext_max
                    ),
                });
            }

            let att_min = joint.attachment_limits[0];
            let att_max = joint.attachment_limits[1];
            if att_min.is_nan()
                || att_min.is_infinite()
                || att_max.is_nan()
                || att_max.is_infinite()
            {
                return Err(AssemblyRefusal {
                    gate: Gate::Gate2,
                    reason: format!("Joint at index {} has NaN or Infinite attachment limits", i),
                });
            }
            if att_min > att_max {
                return Err(AssemblyRefusal {
                    gate: Gate::Gate2,
                    reason: format!(
                        "Joint at index {} has invalid attachment limits where min ({}) > max ({})",
                        i, att_min, att_max
                    ),
                });
            }
        }

        // Gate 3: Collision volumes check
        // Reject NaN/Infinite coordinates for all AABBs, and reject any AABB where min > max on any axis
        validate_aabb(&self.collision_volume.physical_occupancy).map_err(|e| AssemblyRefusal {
            gate: Gate::Gate3,
            reason: format!("Physical occupancy AABB: {}", e),
        })?;
        validate_aabb(&self.collision_volume.clearance_volumes).map_err(|e| AssemblyRefusal {
            gate: Gate::Gate3,
            reason: format!("Clearance volume AABB: {}", e),
        })?;
        for (i, zone) in self.collision_volume.interaction_zones.iter().enumerate() {
            validate_aabb(zone).map_err(|e| AssemblyRefusal {
                gate: Gate::Gate3,
                reason: format!("Interaction zone {} AABB: {}", i, e),
            })?;
        }
        for (i, zone) in self.collision_volume.damage_zones.iter().enumerate() {
            validate_aabb(zone).map_err(|e| AssemblyRefusal {
                gate: Gate::Gate3,
                reason: format!("Damage zone {} AABB: {}", i, e),
            })?;
        }

        // 1. clearance_volumes completely contains physical_occupancy
        for i in 0..3 {
            if self.collision_volume.clearance_volumes.min[i]
                > self.collision_volume.physical_occupancy.min[i]
                || self.collision_volume.clearance_volumes.max[i]
                    < self.collision_volume.physical_occupancy.max[i]
            {
                return Err(AssemblyRefusal {
                    gate: Gate::Gate3,
                    reason: format!("Clearance volume does not completely contain physical occupancy on axis {}", i),
                });
            }
        }

        // 2. clearance_volumes does not intersect any of the interaction or damage zones
        for (i, zone) in self.collision_volume.interaction_zones.iter().enumerate() {
            if self.collision_volume.clearance_volumes.intersects(zone) {
                return Err(AssemblyRefusal {
                    gate: Gate::Gate3,
                    reason: format!(
                        "Clearance volume intersects with interaction zone at index {}",
                        i
                    ),
                });
            }
        }
        for (i, zone) in self.collision_volume.damage_zones.iter().enumerate() {
            if self.collision_volume.clearance_volumes.intersects(zone) {
                return Err(AssemblyRefusal {
                    gate: Gate::Gate3,
                    reason: format!(
                        "Clearance volume intersects with damage zone at index {}",
                        i
                    ),
                });
            }
        }

        // 3. Physical occupancy intersects with interaction zones
        for (i, zone) in self.collision_volume.interaction_zones.iter().enumerate() {
            if self.collision_volume.physical_occupancy.intersects(zone) {
                return Err(AssemblyRefusal {
                    gate: Gate::Gate3,
                    reason: format!(
                        "Physical occupancy intersects with interaction zone at index {}",
                        i
                    ),
                });
            }
        }
        // 4. Physical occupancy intersects with damage zones
        for (i, zone) in self.collision_volume.damage_zones.iter().enumerate() {
            if self.collision_volume.physical_occupancy.intersects(zone) {
                return Err(AssemblyRefusal {
                    gate: Gate::Gate3,
                    reason: format!(
                        "Physical occupancy intersects with damage zone at index {}",
                        i
                    ),
                });
            }
        }
        // 5. Interaction zones intersect with damage zones
        for (i, iz) in self.collision_volume.interaction_zones.iter().enumerate() {
            for (j, dz) in self.collision_volume.damage_zones.iter().enumerate() {
                if iz.intersects(dz) {
                    return Err(AssemblyRefusal {
                        gate: Gate::Gate3,
                        reason: format!(
                            "Interaction zone at index {} intersects with damage zone at index {}",
                            i, j
                        ),
                    });
                }
            }
        }
        // 6. Interaction zones intersecting with other interaction zones
        for i in 0..self.collision_volume.interaction_zones.len() {
            for j in (i + 1)..self.collision_volume.interaction_zones.len() {
                if self.collision_volume.interaction_zones[i]
                    .intersects(&self.collision_volume.interaction_zones[j])
                {
                    return Err(AssemblyRefusal {
                        gate: Gate::Gate3,
                        reason: format!(
                            "Interaction zone {} intersects with interaction zone {}",
                            i, j
                        ),
                    });
                }
            }
        }
        // 7. Damage zones intersecting with other damage zones
        for i in 0..self.collision_volume.damage_zones.len() {
            for j in (i + 1)..self.collision_volume.damage_zones.len() {
                if self.collision_volume.damage_zones[i]
                    .intersects(&self.collision_volume.damage_zones[j])
                {
                    return Err(AssemblyRefusal {
                        gate: Gate::Gate3,
                        reason: format!("Damage zone {} intersects with damage zone {}", i, j),
                    });
                }
            }
        }

        // Gate 4: Motion profile compatibility check
        let is_compatible = match (
            self.frame.mobility_class.to_lowercase().as_str(),
            self.motion_profile,
        ) {
            (
                "walking" | "bipedal" | "legs",
                MotionProfile::Walk
                | MotionProfile::Run
                | MotionProfile::Combat
                | MotionProfile::Construction
                | MotionProfile::Mining
                | MotionProfile::Repair,
            ) => true,
            (
                "flight" | "aerial",
                MotionProfile::Flight
                | MotionProfile::Hover
                | MotionProfile::Combat
                | MotionProfile::Repair,
            ) => true,
            (
                "hover",
                MotionProfile::Hover
                | MotionProfile::Walk
                | MotionProfile::Combat
                | MotionProfile::Repair,
            ) => true,
            _ => false,
        };
        if !is_compatible {
            return Err(AssemblyRefusal {
                gate: Gate::Gate4,
                reason: format!(
                    "Motion profile {:?} is incompatible with mobility class '{}'",
                    self.motion_profile, self.frame.mobility_class
                ),
            });
        }

        // Planetary constraints check (Gate 4)
        let is_mars = std::any::TypeId::of::<P>() == std::any::TypeId::of::<Mars>();
        let is_venus = std::any::TypeId::of::<P>() == std::any::TypeId::of::<Venus>();

        if is_mars {
            match self.functional_role {
                FunctionalRole::Warrior
                | FunctionalRole::Guardian
                | FunctionalRole::Miner
                | FunctionalRole::Worker
                | FunctionalRole::Ark => {}
                _ => {
                    return Err(AssemblyRefusal {
                        gate: Gate::Gate4,
                        reason: format!(
                            "Mars mechs cannot have functional role {:?}",
                            self.functional_role
                        ),
                    });
                }
            }
        }

        if is_venus {
            match self.motion_profile {
                MotionProfile::Flight
                | MotionProfile::Hover
                | MotionProfile::Combat
                | MotionProfile::Repair => {}
                _ => {
                    return Err(AssemblyRefusal {
                        gate: Gate::Gate4,
                        reason: format!("Venus mechs must use Flight or Hover mobility (got motion profile {:?})", self.motion_profile),
                    });
                }
            }
        }

        Ok(MechAssemblySpec {
            frame: self.frame,
            joints: self.joints,
            power: self.power,
            motion_profile: self.motion_profile,
            collision_volume: self.collision_volume,
            material_spec: self.material_spec,
            cultural_profile: self.cultural_profile,
            functional_role: self.functional_role,
            equipment_mass: self.equipment_mass,
            _state: PhantomData,
        })
    }
}

// --- Cultural & Functional Generation (R3) ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerationWeights {
    pub angelic_wing_binder_probability: f32,
    pub white_gold_material_bias: bool,
    pub symmetric_joint_layout_enforced: bool,
    pub heavy_weapon_mount_probability: f32,
    pub sensor_array_hardpoint_required: bool,
}

impl<P> CulturalProfile<P> {
    pub fn compute_weights(&self) -> GenerationWeights {
        let faith = self.planetary_values.faith;
        let order = self.planetary_values.order;
        let ambition = self.planetary_values.ambition;
        let knowledge = self.planetary_values.knowledge;

        GenerationWeights {
            angelic_wing_binder_probability: if faith >= 0.8 { 0.9 } else { 0.1 },
            white_gold_material_bias: faith >= 0.8,
            symmetric_joint_layout_enforced: order >= 0.8,
            heavy_weapon_mount_probability: if ambition >= 0.8 { 0.8 } else { 0.1 },
            sensor_array_hardpoint_required: knowledge >= 0.8,
        }
    }
}

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_f32(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.state >> 32) as u32) as f32 / u32::MAX as f32
    }
}

pub fn generate_spec<P: PlanetCategory + 'static>(
    cultural_profile: CulturalProfile<P>,
    functional_role: FunctionalRole,
) -> MechAssemblySpec<Unvalidated, P> {
    let faith = cultural_profile.planetary_values.faith;
    let ambition = cultural_profile.planetary_values.ambition;
    let order = cultural_profile.planetary_values.order;
    let knowledge = cultural_profile.planetary_values.knowledge;

    // 1. Mobility class & motion profile mapping based on role
    let (mobility_class, motion_profile) = match functional_role {
        FunctionalRole::Ark => ("Flight".to_string(), MotionProfile::Flight),
        FunctionalRole::Worker => ("Walking".to_string(), MotionProfile::Walk),
        FunctionalRole::Builder => ("Walking".to_string(), MotionProfile::Construction),
        FunctionalRole::Miner => ("Walking".to_string(), MotionProfile::Mining),
        FunctionalRole::Explorer => ("Hover".to_string(), MotionProfile::Hover),
        FunctionalRole::Transport => ("Walking".to_string(), MotionProfile::Run),
        FunctionalRole::Guardian => ("Walking".to_string(), MotionProfile::Combat),
        FunctionalRole::Warrior => ("Walking".to_string(), MotionProfile::Combat),
    };

    // 2. Frame generation
    let load_capacity = match functional_role {
        FunctionalRole::Ark => 5000.0,
        FunctionalRole::Worker | FunctionalRole::Builder | FunctionalRole::Miner => 2000.0,
        _ => 1000.0,
    };
    let size = match functional_role {
        FunctionalRole::Ark => [30.0, 30.0, 30.0],
        _ => [5.0, 5.0, 5.0],
    };
    let frame = Frame {
        id: "Standard Frame".to_string(),
        size,
        scale: 1.0,
        load_capacity,
        center_of_mass: [0.0, 1.0, 0.0],
        mobility_class,
    };

    // 3. CollisionVolume generation (non-overlapping zones)
    let p_min = [-size[0] / 2.0, 0.0, -size[2] / 2.0];
    let p_max = [size[0] / 2.0, size[1], size[2] / 2.0];
    let physical_occupancy = AABB::new(p_min, p_max);

    let clearance_volumes = AABB::new(
        [p_min[0] - 1.0, p_min[1] - 1.0, p_min[2] - 1.0],
        [p_max[0] + 1.0, p_max[1] + 1.0, p_max[2] + 1.0],
    );

    let interaction_zones = vec![AABB::new(
        [p_max[0] + 5.0, 0.0, p_max[2] + 5.0],
        [p_max[0] + 6.0, 1.0, p_max[2] + 6.0],
    )];
    let damage_zones = vec![AABB::new(
        [p_min[0] - 6.0, 0.0, p_min[2] - 6.0],
        [p_min[0] - 5.0, 1.0, p_min[2] - 5.0],
    )];

    let collision_volume = CollisionVolume {
        physical_occupancy,
        interaction_zones,
        damage_zones,
        clearance_volumes,
    };

    // 4. MaterialSpec mapping
    let visual = if faith >= 0.8 {
        "White/Gold".to_string()
    } else {
        "Industrial Gray".to_string()
    };
    let material_spec = MaterialSpec {
        structural: "Titanium-Alloy".to_string(),
        armor: "Ceramic Composite".to_string(),
        visual,
        wear_state: 0.0,
        environmental: "Vacuum Rated".to_string(),
    };

    // 5. Joint layout generation with LCG
    let mut seed = 0u64;
    for val in &[
        faith,
        ambition,
        cultural_profile.planetary_values.beauty,
        cultural_profile.planetary_values.community,
        order,
        knowledge,
    ] {
        let bits = val.to_bits() as u64;
        seed = seed.wrapping_mul(31).wrapping_add(bits);
    }
    let mut rng = Lcg::new(seed);

    let mut joints = Vec::new();

    let rotation_limits = Some(RotationLimits::default());
    let extension_limits = [0.0, 2.0];
    let attachment_limits = [0.0, 1.0];

    let enforce_symmetric = order >= 0.8;

    let add_joint_helper = |joints: &mut Vec<Joint>, name: &str| {
        if enforce_symmetric {
            joints.push(Joint {
                rotation_limits,
                extension_limits,
                attachment_limits,
                compatibility_rules: vec![format!("Left_{}_rule", name)],
            });
            joints.push(Joint {
                rotation_limits,
                extension_limits,
                attachment_limits,
                compatibility_rules: vec![format!("Right_{}_rule", name)],
            });
        } else {
            joints.push(Joint {
                rotation_limits,
                extension_limits,
                attachment_limits,
                compatibility_rules: vec![format!("{}_rule", name)],
            });
        }
    };

    add_joint_helper(&mut joints, "WaistJoint");

    let faith_prob = if faith >= 0.8 { 0.9 } else { 0.1 };
    if rng.next_f32() < faith_prob {
        add_joint_helper(&mut joints, "AngelicWingBinderJoint");
    }

    let ambition_prob = if ambition >= 0.8 { 0.8 } else { 0.1 };
    if rng.next_f32() < ambition_prob {
        add_joint_helper(&mut joints, "HeavyWeaponMountJoint");
    }

    if knowledge >= 0.8 {
        add_joint_helper(&mut joints, "SensorArrayHardpointJoint");
    }

    let power = Power {
        mass: 20.0,
        energy_capacity: 1000.0,
        output: 100.0,
    };

    MechAssemblySpec {
        frame,
        joints,
        power,
        motion_profile,
        collision_volume,
        material_spec,
        cultural_profile,
        functional_role,
        equipment_mass: 100.0,
        _state: PhantomData,
    }
}

// --- Deterministic Assembly Receipt (R4) ---

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct AssemblyReceipt<P> {
    pub spec_hash: [u8; 32],
    pub frame_id: String,
    pub motion_class: MotionProfile,
    pub gates_passed: [bool; 4],
    pub cultural_weights: CulturalProfile<P>,
    pub functional_role: FunctionalRole,
    pub timestamp: u64,
}

impl<P> Clone for AssemblyReceipt<P> {
    fn clone(&self) -> Self {
        Self {
            spec_hash: self.spec_hash,
            frame_id: self.frame_id.clone(),
            motion_class: self.motion_class,
            gates_passed: self.gates_passed,
            cultural_weights: self.cultural_weights.clone(),
            functional_role: self.functional_role,
            timestamp: self.timestamp,
        }
    }
}

impl<P> PartialEq for AssemblyReceipt<P> {
    fn eq(&self, other: &Self) -> bool {
        self.spec_hash == other.spec_hash
            && self.frame_id == other.frame_id
            && self.motion_class == other.motion_class
            && self.gates_passed == other.gates_passed
            && self.cultural_weights == other.cultural_weights
            && self.functional_role == other.functional_role
            && self.timestamp == other.timestamp
    }
}

impl<P: PlanetCategory + 'static> MechAssemblySpec<Validated, P> {
    pub fn generate_receipt(&self, timestamp: u64) -> Result<AssemblyReceipt<P>, AssemblyRefusal> {
        let serialized = serde_json::to_vec(self).map_err(|e| AssemblyRefusal {
            gate: Gate::Gate1,
            reason: format!("Serialization failure: {}", e),
        })?;

        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        let hash_result = hasher.finalize();
        let mut spec_hash = [0u8; 32];
        spec_hash.copy_from_slice(&hash_result);

        Ok(AssemblyReceipt {
            spec_hash,
            frame_id: self.frame.id.clone(),
            motion_class: self.motion_profile,
            gates_passed: [true; 4],
            cultural_weights: self.cultural_profile.clone(),
            functional_role: self.functional_role,
            timestamp,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_gundam::generated_gundam::Earth;

    #[test]
    fn test_generating_valid_ark_class() {
        let cp = CulturalProfile {
            planetary_values: PlanetaryValues {
                faith: 0.9,
                ambition: 0.5,
                beauty: 0.5,
                community: 0.5,
                order: 0.8,
                knowledge: 0.5,
            },
            _marker: PhantomData::<Earth>,
        };

        let spec = generate_spec(cp, FunctionalRole::Ark);

        let validated_spec = spec.validate().expect("Validation failed");

        assert_eq!(validated_spec.material_spec.visual, "White/Gold");
        assert!(validated_spec.joints.len() % 2 == 0);

        let receipt = validated_spec.generate_receipt(123456789).unwrap();
        assert_eq!(receipt.timestamp, 123456789);
        assert_eq!(receipt.gates_passed, [true; 4]);
    }

    #[test]
    fn test_generating_valid_worker_class() {
        let cp = CulturalProfile {
            planetary_values: PlanetaryValues {
                faith: 0.5,
                ambition: 0.5,
                beauty: 0.5,
                community: 0.8,
                order: 0.5,
                knowledge: 0.9,
            },
            _marker: PhantomData::<Earth>,
        };

        let spec = generate_spec(cp, FunctionalRole::Worker);

        let validated_spec = spec.validate().expect("Validation failed");

        let has_sensor_hardpoint = validated_spec.joints.iter().any(|j| {
            j.compatibility_rules
                .iter()
                .any(|r| r.contains("SensorArrayHardpoint"))
        });
        assert!(
            has_sensor_hardpoint,
            "Should have a sensor array hardpoint joint"
        );
    }

    #[test]
    fn test_refusal_unbounded_joint_rotation_fails_gate2() {
        let cp = CulturalProfile {
            planetary_values: PlanetaryValues {
                faith: 0.5,
                ambition: 0.5,
                beauty: 0.5,
                community: 0.5,
                order: 0.5,
                knowledge: 0.5,
            },
            _marker: PhantomData::<Earth>,
        };

        let mut spec = generate_spec(cp, FunctionalRole::Warrior);
        spec.joints[0].rotation_limits = None;

        let res = spec.validate();
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err.gate, Gate::Gate2);
        assert!(err.reason.contains("has no defined rotation limits"));
    }

    #[test]
    fn test_refusal_self_intersecting_collision_volumes_fails_gate3() {
        let cp = CulturalProfile {
            planetary_values: PlanetaryValues {
                faith: 0.5,
                ambition: 0.5,
                beauty: 0.5,
                community: 0.5,
                order: 0.5,
                knowledge: 0.5,
            },
            _marker: PhantomData::<Earth>,
        };

        let mut spec = generate_spec(cp, FunctionalRole::Warrior);
        spec.collision_volume.interaction_zones[0] = spec.collision_volume.physical_occupancy;

        let res = spec.validate();
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err.gate, Gate::Gate3);
        assert!(err.reason.contains("intersects with interaction zone"));
    }

    #[test]
    fn test_determinism_identical_state_produces_byte_identical_receipt() {
        let cp1 = CulturalProfile {
            planetary_values: PlanetaryValues {
                faith: 0.9,
                ambition: 0.8,
                beauty: 0.7,
                community: 0.6,
                order: 0.5,
                knowledge: 0.4,
            },
            _marker: PhantomData::<Earth>,
        };
        let cp2 = cp1.clone();

        let spec1 = generate_spec(cp1, FunctionalRole::Guardian);
        let spec2 = generate_spec(cp2, FunctionalRole::Guardian);

        let validated1 = spec1.validate().unwrap();
        let validated2 = spec2.validate().unwrap();

        assert_eq!(validated1, validated2);

        let receipt1 = validated1.generate_receipt(987654321).unwrap();
        let receipt2 = validated2.generate_receipt(987654321).unwrap();

        assert_eq!(receipt1, receipt2);

        let bytes1 = serde_json::to_vec(&receipt1).unwrap();
        let bytes2 = serde_json::to_vec(&receipt2).unwrap();
        assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn test_gate1_refusal_on_load_capacity_exceeded() {
        let cp = CulturalProfile {
            planetary_values: PlanetaryValues {
                faith: 0.5,
                ambition: 0.5,
                beauty: 0.5,
                community: 0.5,
                order: 0.5,
                knowledge: 0.5,
            },
            _marker: PhantomData::<Earth>,
        };

        let mut spec = generate_spec(cp, FunctionalRole::Warrior);
        spec.equipment_mass = spec.frame.load_capacity + 1.0;

        let res = spec.validate();
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err.gate, Gate::Gate1);
        assert!(err.reason.contains("exceeds frame load capacity"));
    }

    #[test]
    fn test_gate4_refusal_on_incompatible_mobility_profile() {
        let cp = CulturalProfile {
            planetary_values: PlanetaryValues {
                faith: 0.5,
                ambition: 0.5,
                beauty: 0.5,
                community: 0.5,
                order: 0.5,
                knowledge: 0.5,
            },
            _marker: PhantomData::<Earth>,
        };

        let mut spec = generate_spec(cp, FunctionalRole::Warrior);
        spec.motion_profile = MotionProfile::Flight;

        let res = spec.validate();
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err.gate, Gate::Gate4);
        assert!(err.reason.contains("is incompatible with mobility class"));
    }
}
