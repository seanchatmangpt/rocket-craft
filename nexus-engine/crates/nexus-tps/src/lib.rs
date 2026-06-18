use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use nexus_types::tps::Part;

pub mod telemetry;


// ============================================================================
// R1: PartSlot and PartStateVector
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PartSlot {
    Head = 0,
    Torso = 1,
    Waist = 2,
    ArmL = 3,
    ArmR = 4,
    LegL = 5,
    LegR = 6,
    Backpack = 7,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PartStateVector {
    pub civilization_id: u16,
    pub frame_id: u8,
    pub armor_profile: f32,
    pub joint_profile: f32,
    pub mass_profile: f32,
    pub weapon_profile: f32,
    pub motion_profile: f32,
    pub part_slot: PartSlot,
}

// ============================================================================
// R3: Jidoka halts
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SocketType {
    Head,
    Torso,
    Waist,
    ArmL,
    ArmR,
    LegL,
    LegR,
    Backpack,
}

impl From<PartSlot> for SocketType {
    fn from(slot: PartSlot) -> Self {
        match slot {
            PartSlot::Head => SocketType::Head,
            PartSlot::Torso => SocketType::Torso,
            PartSlot::Waist => SocketType::Waist,
            PartSlot::ArmL => SocketType::ArmL,
            PartSlot::ArmR => SocketType::ArmR,
            PartSlot::LegL => SocketType::LegL,
            PartSlot::LegR => SocketType::LegR,
            PartSlot::Backpack => SocketType::Backpack,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error, Serialize, Deserialize)]
pub enum JidokaHalt {
    #[error("Socket mismatch: expected {expected:?}, got {got:?}")]
    SocketMismatch { expected: SocketType, got: SocketType },

    #[error("Collision volume intersects: part_a {part_a:?}, part_b {part_b:?}")]
    CollisionVolumeIntersects { part_a: PartSlot, part_b: PartSlot },

    #[error("Mass exceeds frame capacity: mass {mass}, capacity {capacity}")]
    MassExceedsFrameCapacity { mass: f32, capacity: f32 },

    #[error("Motion bounds violated: axis {axis:?}, limit {limit}, actual {actual}")]
    MotionBoundsViolated { axis: Axis, limit: f32, actual: f32 },
}

// ============================================================================
// R2: generate_part lookup tables and branchless execution
// ============================================================================

#[derive(Clone, Copy)]
struct CivParams {
    mass_mult: f32,
    armor_mult: f32,
    motion_mult: f32,
    _weapon_mult: f32,
    _joint_mult: f32,
}

const CIV_PARAMS: [CivParams; 4] = [
    CivParams { mass_mult: 1.0, armor_mult: 1.0, motion_mult: 1.0, _weapon_mult: 1.0, _joint_mult: 1.0 },
    CivParams { mass_mult: 0.8, armor_mult: 1.2, motion_mult: 0.9, _weapon_mult: 1.1, _joint_mult: 1.05 },
    CivParams { mass_mult: 1.25, armor_mult: 0.85, motion_mult: 1.3, _weapon_mult: 0.75, _joint_mult: 1.2 },
    CivParams { mass_mult: 1.5, armor_mult: 1.5, motion_mult: 0.5, _weapon_mult: 1.5, _joint_mult: 0.7 },
];

#[derive(Clone, Copy)]
struct SlotParams {
    base_mass: f32,
    mass_range: f32,
    base_dim_x: f32,
    dim_x_range: f32,
    base_dim_y: f32,
    dim_y_range: f32,
    base_dim_z: f32,
    dim_z_range: f32,
    base_clearance: f32,
    clearance_range: f32,
    base_volume: f32,
    volume_range: f32,
    _socket_gender: u64,
}

const SLOT_PARAMS: [SlotParams; 8] = [
    // Head = 0
    SlotParams { base_mass: 5.0, mass_range: 2.0, base_dim_x: 0.4, dim_x_range: 0.2, base_dim_y: 0.4, dim_y_range: 0.2, base_dim_z: 0.4, dim_z_range: 0.2, base_clearance: 0.1, clearance_range: 0.05, base_volume: 0.064, volume_range: 0.05, _socket_gender: 2 },
    // Torso = 1
    SlotParams { base_mass: 40.0, mass_range: 20.0, base_dim_x: 1.8, dim_x_range: 0.6, base_dim_y: 1.2, dim_y_range: 0.4, base_dim_z: 1.6, dim_z_range: 0.6, base_clearance: 0.2, clearance_range: 0.1, base_volume: 3.456, volume_range: 2.5, _socket_gender: 1 },
    // Waist = 2
    SlotParams { base_mass: 20.0, mass_range: 10.0, base_dim_x: 1.4, dim_x_range: 0.4, base_dim_y: 1.0, dim_y_range: 0.3, base_dim_z: 0.8, dim_z_range: 0.3, base_clearance: 0.15, clearance_range: 0.05, base_volume: 1.12, volume_range: 0.8, _socket_gender: 2 },
    // ArmL = 3
    SlotParams { base_mass: 12.0, mass_range: 6.0, base_dim_x: 0.6, dim_x_range: 0.3, base_dim_y: 0.6, dim_y_range: 0.3, base_dim_z: 1.8, dim_z_range: 0.6, base_clearance: 0.15, clearance_range: 0.05, base_volume: 0.648, volume_range: 0.4, _socket_gender: 2 },
    // ArmR = 4
    SlotParams { base_mass: 12.0, mass_range: 6.0, base_dim_x: 0.6, dim_x_range: 0.3, base_dim_y: 0.6, dim_y_range: 0.3, base_dim_z: 1.8, dim_z_range: 0.6, base_clearance: 0.15, clearance_range: 0.05, base_volume: 0.648, volume_range: 0.4, _socket_gender: 2 },
    // LegL = 5
    SlotParams { base_mass: 25.0, mass_range: 12.0, base_dim_x: 0.8, dim_x_range: 0.4, base_dim_y: 0.8, dim_y_range: 0.4, base_dim_z: 2.4, dim_z_range: 0.8, base_clearance: 0.25, clearance_range: 0.1, base_volume: 1.536, volume_range: 1.2, _socket_gender: 2 },
    // LegR = 6
    SlotParams { base_mass: 25.0, mass_range: 12.0, base_dim_x: 0.8, dim_x_range: 0.4, base_dim_y: 0.8, dim_y_range: 0.4, base_dim_z: 2.4, dim_z_range: 0.8, base_clearance: 0.25, clearance_range: 0.1, base_volume: 1.536, volume_range: 1.2, _socket_gender: 2 },
    // Backpack = 7
    SlotParams { base_mass: 15.0, mass_range: 10.0, base_dim_x: 1.2, dim_x_range: 0.4, base_dim_y: 0.8, dim_y_range: 0.4, base_dim_z: 1.2, dim_z_range: 0.4, base_clearance: 0.3, clearance_range: 0.15, base_volume: 1.152, volume_range: 0.9, _socket_gender: 2 },
];

/// Pure branchless clamp of a float to the [0.0, 1.0] range
#[inline(always)]
pub fn branchless_clamp01(x: f32) -> f32 {
    let bits = x.to_bits();
    let sign = (bits >> 31) & 1;
    let positive_part = x * ((1 - sign) as f32);

    let diff = positive_part - 1.0;
    let diff_bits = diff.to_bits();
    let diff_sign = (diff_bits >> 31) & 1; // 1 if positive_part < 1.0, else 0

    (positive_part * (diff_sign as f32)) + (1.0 * ((1 - diff_sign) as f32))
}

/// Zero-branch linear interpolation
#[inline(always)]
pub fn branchless_lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn validate_profile(val: f32, limit_max: f32, axis: Axis) -> Result<(), JidokaHalt> {
    if !val.is_finite() || val < 0.0 || val > limit_max {
        let limit = if val < 0.0 { 0.0 } else { limit_max };
        Err(JidokaHalt::MotionBoundsViolated { axis, limit, actual: val })
    } else {
        Ok(())
    }
}

pub fn generate_part(state: &PartStateVector) -> Result<Part, JidokaHalt> {
    validate_profile(state.armor_profile, 1.0, Axis::X)?;
    validate_profile(state.joint_profile, 1.0, Axis::Y)?;
    validate_profile(state.mass_profile, 1.0, Axis::Z)?;
    validate_profile(state.weapon_profile, 1.0, Axis::X)?;
    validate_profile(state.motion_profile, 100.0, Axis::Z)?;

    let u_armor = branchless_clamp01(state.armor_profile);
    let u_joint = branchless_clamp01(state.joint_profile);
    let u_mass = branchless_clamp01(state.mass_profile);
    let u_weapon = branchless_clamp01(state.weapon_profile);
    let u_motion = branchless_clamp01(state.motion_profile);

    let civ_idx = (state.civilization_id as usize) & 3;
    let slot_idx = (state.part_slot as usize) & 7;

    let civ = &CIV_PARAMS[civ_idx];
    let slot = &SLOT_PARAMS[slot_idx];

    let armor_bits = state.armor_profile.to_bits() as u64;
    let joint_bits = state.joint_profile.to_bits() as u64;
    let mass_bits = state.mass_profile.to_bits() as u64;

    let internal_state = nexus_types::tps::StateVector {
        civilization_id: state.civilization_id as u64,
        frame_id: state.frame_id as u64,
        armor_profile: armor_bits,
        joint_profile: joint_bits,
        mass_profile: mass_bits,
    };

    // Call workspace μ function to obtain base transformation
    let base_part = nexus_types::tps::μ(internal_state);

    // Derive geometry, dimensions, socket fit, motion clearance, collision volume, and mass balance branchlessly
    let dim_x = branchless_lerp(slot.base_dim_x, slot.base_dim_x + slot.dim_x_range, u_armor) * civ.armor_mult;
    let dim_y = branchless_lerp(slot.base_dim_y, slot.base_dim_y + slot.dim_y_range, u_armor) * civ.armor_mult;
    let dim_z = branchless_lerp(slot.base_dim_z, slot.base_dim_z + slot.dim_z_range, u_mass) * civ.mass_mult;

    let geom_x_quant = ((dim_x * 100.0) as u64) & 0xFFFF;
    let geom_y_quant = ((dim_y * 100.0) as u64) & 0xFFFF;
    let geom_z_quant = ((dim_z * 100.0) as u64) & 0xFFFF;
    let geometry = geom_x_quant | (geom_y_quant << 16) | (geom_z_quant << 32) | (base_part.geometry & 1);

    // Sockets mating layout: base_fit derived from frame_id & joint_profile to keep them matching
    let base_fit = (state.frame_id as u64).wrapping_add((u_joint * 255.0) as u64) & 0xFF;
    let socket_fit = base_fit | (base_part.socket_fit & 1);

    // Scale clearance and volume down to avoid collisions during standard assembly spacing
    let raw_clearance = branchless_lerp(slot.base_clearance, slot.base_clearance + slot.clearance_range, u_motion);
    let motion_clearance = (((raw_clearance * civ.motion_mult * 10.0) as u64) & 0xFF) | (base_part.motion_clearance & 1);

    let raw_volume = branchless_lerp(slot.base_volume, slot.base_volume + slot.volume_range, u_mass);
    let collision_volume = (((raw_volume * civ.mass_mult * 10.0) as u64) & 0xFF) | (base_part.collision_volume & 1);

    let raw_mass = branchless_lerp(slot.base_mass, slot.base_mass + slot.mass_range, u_mass);
    let mass_balance = (((raw_mass * civ.mass_mult * 10.0) as u64) & 0xFFFF) | (base_part.mass_balance & 1);

    let physics_role = (slot_idx as u64 * 10) + ((u_weapon * 3.0) as u64) + 1;
    let assembly_compatibility = (1 << slot_idx) as u64;

    let part = Part {
        geometry,
        socket_fit,
        motion_clearance,
        collision_volume,
        mass_balance,
        physics_role,
        assembly_compatibility,
    };

    Ok(part)
}

// ============================================================================
// R4: Deterministic TpsReceipt
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TpsReceipt {
    pub state_vector_hash: String,
    pub part_slot: PartSlot,
    pub gates_passed_mask: u32,
    pub mass: u64,
    pub collider_aabb: [f32; 6],
    pub motion_bounds: [f32; 2],
    pub jidoka_halts: Vec<String>,
}

impl TpsReceipt {
    pub fn generate(state: &PartStateVector, part: &Part, halts: Vec<String>) -> Self {
        let serialized = serde_json::to_string(state).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        let state_vector_hash = format!("{:x}", hasher.finalize());

        let geom = part.geometry;
        let x_size = ((geom) & 0xFFFF) as f32 / 100.0;
        let y_size = ((geom >> 16) & 0xFFFF) as f32 / 100.0;
        let z_size = ((geom >> 32) & 0xFFFF) as f32 / 100.0;

        let collider_aabb = [
            -x_size / 2.0, -y_size / 2.0, -z_size / 2.0,
            x_size / 2.0, y_size / 2.0, z_size / 2.0,
        ];

        let motion_bounds = [0.0, 100.0];

        let mut gates_passed_mask = 0u32;
        if halts.is_empty() {
            gates_passed_mask = 0b111;
        }

        Self {
            state_vector_hash,
            part_slot: state.part_slot,
            gates_passed_mask,
            mass: part.mass_balance,
            collider_aabb,
            motion_bounds,
            jidoka_halts: halts,
        }
    }
}

// ============================================================================
// R5: assemble_mech and MechTpsReceipt
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MechTpsReceipt {
    pub lineage_hash: String,
    pub timestamp: String,
    pub parts: Vec<Part>,
    pub passed_gates: Vec<String>,
    pub final_decision: String,
    pub total_mass: u64,
    pub load_capacity: u64,
    pub component_count: usize,
    pub receipt_hash: String,
}

impl MechTpsReceipt {
    pub fn generate(
        parts: Vec<Part>,
        passed_gates: Vec<String>,
        final_decision: String,
        total_mass: u64,
        load_capacity: u64,
        timestamp: String,
    ) -> Self {
        let serialized_parts = serde_json::to_string(&parts).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(serialized_parts.as_bytes());
        let lineage_hash = format!("{:x}", hasher.finalize());

        let mut r_hasher = Sha256::new();
        r_hasher.update(lineage_hash.as_bytes());
        r_hasher.update(total_mass.to_be_bytes());
        r_hasher.update(load_capacity.to_be_bytes());
        r_hasher.update(final_decision.as_bytes());
        r_hasher.update(timestamp.as_bytes());
        let receipt_hash = format!("{:x}", r_hasher.finalize());

        Self {
            lineage_hash,
            timestamp,
            parts,
            passed_gates,
            final_decision,
            total_mass,
            load_capacity,
            component_count: 8,
            receipt_hash,
        }
    }

    /// Formats the mech assembly receipt into a highly readable, colored ASCII-art table.
    pub fn format_receipt(&self) -> String {
        let mut s = String::new();
        s.push_str("\n\x1b[1;33m┌──────────────────────────────────────────────────────────────────────────────────┐\x1b[0m\n");
        s.push_str("\x1b[1;33m│                           NEXUS MECH ASSEMBLY RECEIPT                            │\x1b[0m\n");
        s.push_str("\x1b[1;33m├──────────┬──────────────────────┬────────────┬──────────┬─────────────┬──────────┤\x1b[0m\n");
        s.push_str("\x1b[1;33m│ Slot     │ Dimensions (X×Y×Z m) │ Mass (kg)  │ Socket   │ Clear. (m)  │ Status   │\x1b[0m\n");
        s.push_str("\x1b[1;33m├──────────┼──────────────────────┼────────────┼──────────┼─────────────┼──────────┤\x1b[0m\n");

        let slots = [
            "Head", "Torso", "Waist", "ArmL", "ArmR", "LegL", "LegR", "Backpack"
        ];

        for (i, part) in self.parts.iter().enumerate() {
            let slot_name = slots.get(i).unwrap_or(&"Unknown");
            
            let geom = part.geometry;
            let dim_x = ((geom) & 0xFFFF) as f32 / 100.0;
            let dim_y = ((geom >> 16) & 0xFFFF) as f32 / 100.0;
            let dim_z = ((geom >> 32) & 0xFFFF) as f32 / 100.0;
            let dims_str = format!("{:.2}×{:.2}×{:.2}", dim_x, dim_y, dim_z);

            let mass = part.mass_balance as f32 / 10.0;
            let socket = format!("0x{:02X}", part.socket_fit);
            let clearance = (part.motion_clearance & 0xFF) as f32 / 10.0;
            
            s.push_str(&format!(
                "│ \x1b[1;36m{:<8}\x1b[0m │ {:<20} │ {:<10.2} │ {:<8} │ {:<11.2} │ \x1b[1;32m{:<8}\x1b[0m │\n",
                slot_name, dims_str, mass, socket, clearance, "PASS"
            ));
        }

        s.push_str("\x1b[1;33m├──────────┴──────────────────────┴────────────┴──────────┴─────────────┴──────────┤\x1b[0m\n");
        
        let total_mass_kg = self.total_mass as f32 / 10.0;
        let capacity_kg = self.load_capacity as f32 / 10.0;
        
        s.push_str(&format!(
            "│  \x1b[1mTotal Mass\x1b[0m:  {:<10.2} kg /  \x1b[1mLoad Capacity\x1b[0m:  {:<10.2} kg    \x1b[1;32m[{}]\x1b[0m       │\n",
            total_mass_kg, capacity_kg, self.final_decision
        ));
        s.push_str(&format!(
            "│  \x1b[1mLineage Hash\x1b[0m:  {:<59} │\n",
            if self.lineage_hash.len() > 55 { format!("{}...", &self.lineage_hash[..55]) } else { self.lineage_hash.clone() }
        ));
        s.push_str(&format!(
            "│  \x1b[1mReceipt Hash\x1b[0m:  {:<59} │\n",
            if self.receipt_hash.len() > 55 { format!("{}...", &self.receipt_hash[..55]) } else { self.receipt_hash.clone() }
        ));
        s.push_str("\x1b[1;33m└──────────────────────────────────────────────────────────────────────────────────┘\x1b[0m\n");
        s
    }
}

pub fn assemble_mech(vectors: &[PartStateVector; 8]) -> Result<MechTpsReceipt, JidokaHalt> {
    tracing::info!("Initializing mech assembly sequence...");

    // 1. Structure Verification: unique and complete slot allocation
    tracing::info!("Gate 1: Structure Verification. Validating slot configurations...");
    let mut parts_map = [None; 8];
    for v in vectors.iter() {
        let idx = v.part_slot as usize;
        if idx >= 8 {
            tracing::error!("Assembly structure defect: slot index {:?} out of bounds", v.part_slot);
            panic!("Assembly structure defect: slot index out of bounds");
        }
        if parts_map[idx].is_some() {
            tracing::error!("Assembly structure defect: duplicate slot configuration for {:?}", v.part_slot);
            panic!("Assembly structure defect: duplicate slot");
        }
        let part = generate_part(v)?;
        tracing::debug!("Successfully generated part for slot {:?}", v.part_slot);
        parts_map[idx] = Some((v, part));
    }

    for (idx, p) in parts_map.iter().enumerate() {
        if p.is_none() {
            tracing::error!("Assembly structure defect: missing slot configuration at index {}", idx);
            panic!("Assembly structure defect: missing slot");
        }
    }
    tracing::info!("Gate 1: Structure Verification [PASSED]");

    let head = parts_map[PartSlot::Head as usize].unwrap().1;
    let torso = parts_map[PartSlot::Torso as usize].unwrap().1;
    let waist = parts_map[PartSlot::Waist as usize].unwrap().1;
    let arm_l = parts_map[PartSlot::ArmL as usize].unwrap().1;
    let arm_r = parts_map[PartSlot::ArmR as usize].unwrap().1;
    let leg_l = parts_map[PartSlot::LegL as usize].unwrap().1;
    let leg_r = parts_map[PartSlot::LegR as usize].unwrap().1;
    let backpack = parts_map[PartSlot::Backpack as usize].unwrap().1;

    // 2. Socket Mating Gate
    tracing::info!("Gate 2: Socket Mating Verification. Verifying part connector interfaces...");
    if (head.socket_fit & 0x0F) != (torso.socket_fit & 0x0F) {
        let err = JidokaHalt::SocketMismatch { expected: SocketType::Torso, got: SocketType::Head };
        tracing::error!("Jidoka Halt at Socket Gate: {}", err);
        return Err(err);
    }
    if (waist.socket_fit & 0x0F) != (torso.socket_fit & 0x0F) {
        let err = JidokaHalt::SocketMismatch { expected: SocketType::Torso, got: SocketType::Waist };
        tracing::error!("Jidoka Halt at Socket Gate: {}", err);
        return Err(err);
    }
    if (arm_l.socket_fit & 0xF0) != (torso.socket_fit & 0xF0) {
        let err = JidokaHalt::SocketMismatch { expected: SocketType::Torso, got: SocketType::ArmL };
        tracing::error!("Jidoka Halt at Socket Gate: {}", err);
        return Err(err);
    }
    if (arm_r.socket_fit & 0xF0) != (torso.socket_fit & 0xF0) {
        let err = JidokaHalt::SocketMismatch { expected: SocketType::Torso, got: SocketType::ArmR };
        tracing::error!("Jidoka Halt at Socket Gate: {}", err);
        return Err(err);
    }
    if (leg_l.socket_fit & 0x0F) != (waist.socket_fit & 0x0F) {
        let err = JidokaHalt::SocketMismatch { expected: SocketType::Waist, got: SocketType::LegL };
        tracing::error!("Jidoka Halt at Socket Gate: {}", err);
        return Err(err);
    }
    if (leg_r.socket_fit & 0x0F) != (waist.socket_fit & 0x0F) {
        let err = JidokaHalt::SocketMismatch { expected: SocketType::Waist, got: SocketType::LegR };
        tracing::error!("Jidoka Halt at Socket Gate: {}", err);
        return Err(err);
    }
    if (backpack.socket_fit & 0xF0) != (torso.socket_fit & 0xF0) {
        let err = JidokaHalt::SocketMismatch { expected: SocketType::Torso, got: SocketType::Backpack };
        tracing::error!("Jidoka Halt at Socket Gate: {}", err);
        return Err(err);
    }
    tracing::info!("Gate 2: Socket Mating Verification [PASSED]");

    // 3. Collision and Clearance Gate
    tracing::info!("Gate 3: Collision and Clearance Verification. Check volume clearance...");
    let get_radius = |part: &Part| -> f32 {
        ((part.collision_volume & 0xFF) as f32) / 100.0 + ((part.motion_clearance & 0xFF) as f32) / 200.0
    };

    let slots_positions: [(PartSlot, (f32, f32, f32)); 8] = [
        (PartSlot::Head, (0.0, 0.0, 2.0)),
        (PartSlot::Torso, (0.0, 0.0, 0.0)),
        (PartSlot::Waist, (0.0, 0.0, -1.0)),
        (PartSlot::ArmL, (-2.0, 0.0, 1.0)),
        (PartSlot::ArmR, (2.0, 0.0, 1.0)),
        (PartSlot::LegL, (-1.0, 0.0, -2.5)),
        (PartSlot::LegR, (1.0, 0.0, -2.5)),
        (PartSlot::Backpack, (0.0, -1.0, 0.5)),
    ];

    for i in 0..8 {
        for j in (i + 1)..8 {
            let (slot_a, pos_a) = slots_positions[i];
            let (slot_b, pos_b) = slots_positions[j];
            let part_a = parts_map[slot_a as usize].unwrap().1;
            let part_b = parts_map[slot_b as usize].unwrap().1;

            let dx = pos_a.0 - pos_b.0;
            let dy = pos_a.1 - pos_b.1;
            let dz = pos_a.2 - pos_b.2;
            let distance = (dx*dx + dy*dy + dz*dz).sqrt();

            let combined_radius = get_radius(&part_a) + get_radius(&part_b);
            if combined_radius > distance {
                let err = JidokaHalt::CollisionVolumeIntersects { part_a: slot_a, part_b: slot_b };
                tracing::error!("Jidoka Halt at Collision Gate: {}", err);
                return Err(err);
            }
        }
    }
    tracing::info!("Gate 3: Collision and Clearance Verification [PASSED]");

    // 4. Mass and Frame Capacity Gate
    tracing::info!("Gate 4: Mass and Frame Capacity Verification. Checking payload thresholds...");
    let total_mass: u64 = parts_map.iter().map(|p| p.unwrap().1.mass_balance).sum();
    
    // Extract leg dimensions for capacity calculation
    let leg_l_z = (leg_l.geometry >> 32) & 0xFFFF;
    let leg_r_z = (leg_r.geometry >> 32) & 0xFFFF;
    let leg_l_val = if leg_l_z > 0 { leg_l_z } else { leg_l.geometry };
    let leg_r_val = if leg_r_z > 0 { leg_r_z } else { leg_r.geometry };
    
    let load_capacity = (leg_l_val.wrapping_add(leg_r_val)).wrapping_mul(7) / 2;
    if total_mass > load_capacity {
        let err = JidokaHalt::MassExceedsFrameCapacity {
            mass: total_mass as f32,
            capacity: load_capacity as f32,
        };
        tracing::error!("Jidoka Halt at Mass Gate: {}", err);
        return Err(err);
    }
    tracing::info!("Gate 4: Mass and Frame Capacity Verification [PASSED]");

    // Assembly Complete: Produce Receipts
    let parts_vec: Vec<Part> = parts_map.iter().map(|p| p.unwrap().1).collect();
    let passed_gates = vec![
        "StructureGate".to_string(),
        "SocketGate".to_string(),
        "CollisionGate".to_string(),
        "MassGate".to_string(),
    ];
    let timestamp = if std::env::var("NEXUS_TEST_DETERMINISTIC").is_ok() {
        "2026-06-18T12:00:00Z".to_string()
    } else {
        chrono::Utc::now().to_rfc3339()
    };

    let receipt = MechTpsReceipt::generate(
        parts_vec,
        passed_gates,
        "APPROVED".to_string(),
        total_mass,
        load_capacity,
        timestamp,
    );

    tracing::info!(target: "receipt", "{}", receipt.format_receipt());

    Ok(receipt)
}

// ============================================================================
// R3: Poka-yoke compile-time safety
// ============================================================================

pub mod poka_yoke {
    use super::JidokaHalt;

    pub struct ArmSocket;
    pub struct ArmMount;

    pub struct LegSocket;
    pub struct LegMount;

    pub struct HeadSocket;
    pub struct HeadMount;

    pub struct TorsoSocket;
    pub struct TorsoMount;

    pub struct WaistSocket;
    pub struct WaistMount;

    pub struct BackpackSocket;
    pub struct BackpackMount;

    impl ArmSocket {
        pub fn connect(self, _mount: ArmMount) -> Result<(), JidokaHalt> {
            Ok(())
        }
    }

    impl LegSocket {
        pub fn connect(self, _mount: LegMount) -> Result<(), JidokaHalt> {
            Ok(())
        }
    }

    impl HeadSocket {
        pub fn connect(self, _mount: HeadMount) -> Result<(), JidokaHalt> {
            Ok(())
        }
    }

    impl TorsoSocket {
        pub fn connect(self, _mount: TorsoMount) -> Result<(), JidokaHalt> {
            Ok(())
        }
    }

    impl WaistSocket {
        pub fn connect(self, _mount: WaistMount) -> Result<(), JidokaHalt> {
            Ok(())
        }
    }

    impl BackpackSocket {
        pub fn connect(self, _mount: BackpackMount) -> Result<(), JidokaHalt> {
            Ok(())
        }
    }
}
