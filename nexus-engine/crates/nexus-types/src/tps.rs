//! TPS Branchless Mech Parts Generator for Gundam Nexus

use sha2::{Sha256, Digest};

/// StateVector representing the ontology state coordinates O*
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StateVector {
    pub civilization_id: u64,
    pub frame_id: u64,
    pub armor_profile: u64,
    pub joint_profile: u64,
    pub mass_profile: u64,
}

/// Part representing the manufactured component specs projected from the state vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Part {
    pub geometry: u64,
    pub socket_fit: u64,
    pub motion_clearance: u64,
    pub collision_volume: u64,
    pub mass_balance: u64,
    pub physics_role: u64,
    pub assembly_compatibility: u64,
}

// Bounded lookup tables/matrices (sizes are powers of 2 to allow branchless masking)
pub const GEOMETRY_BASE_TABLE: [u64; 8] = [100, 200, 300, 400, 500, 600, 700, 800];
pub const PHYSICS_ROLE_TABLE: [u64; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
pub const ASSEMBLY_COMPATIBILITY_TABLE: [u64; 8] = [
    0x000F, 0x00F0, 0x0F00, 0xF000, 0x00FF, 0xFF00, 0x0FFF, 0xFFFF,
];

/// The branchless transformation function Part = μ(O*)
/// Uses bitwise masks and mathematical operations without if/else branching.
#[allow(non_snake_case)]
pub fn μ(state: StateVector) -> Part {
    // Indexes are masked to guarantee they are within bounds [0, 7], avoiding bounds-checking branch overheads.
    let geom_idx = (state.civilization_id ^ state.frame_id) & 7;
    let role_idx = (state.armor_profile ^ state.joint_profile) & 7;
    let compat_idx = (state.mass_profile ^ state.frame_id) & 7;

    let geometry = GEOMETRY_BASE_TABLE[geom_idx as usize];
    let physics_role = PHYSICS_ROLE_TABLE[role_idx as usize];
    let assembly_compatibility = ASSEMBLY_COMPATIBILITY_TABLE[compat_idx as usize];

    // Compute remaining specifications branchlessly
    let socket_fit = (state.frame_id.wrapping_add(state.joint_profile) & 0xFF) | 1; // force non-zero
    let motion_clearance = state.armor_profile.wrapping_sub(state.joint_profile) & 0xFF;
    let collision_volume = ((state.armor_profile.wrapping_mul(state.mass_profile)) & 0xFFFF) | 1; // force non-zero
    let mass_balance = ((state.mass_profile ^ 0xAA55AA55AA55AA55) & 0xFFFF) | 1; // force non-zero

    Part {
        geometry,
        socket_fit,
        motion_clearance,
        collision_volume,
        mass_balance,
        physics_role,
        assembly_compatibility,
    }
}

/// Poka-yoke compile-time/const verification function.
pub const fn const_validate_part(part: &Part) -> bool {
    part.geometry > 0
        && part.physics_role > 0
        && part.socket_fit > 0
        && part.collision_volume > 0
        && part.mass_balance > 0
}

/// Macro for compile-time assertion (Poka-yoke)
///
/// Ensures a [`Part`] is valid at compile time.
///
/// # Examples
///
/// ```
/// use nexus_types::tps::Part;
/// use nexus_types::assert_part_poka_yoke;
///
/// const VALID_PART: Part = Part {
///     geometry: 1,
///     socket_fit: 1,
///     motion_clearance: 1,
///     collision_volume: 1,
///     mass_balance: 1,
///     physics_role: 1,
///     assembly_compatibility: 1,
/// };
///
/// assert_part_poka_yoke!(VALID_PART);
/// ```
#[macro_export]
macro_rules! assert_part_poka_yoke {
    ($part:expr) => {
        const _: () = {
            let p: $crate::Part = $part;
            if !$crate::const_validate_part(&p) {
                panic!("Poka-yoke compile-time verification failed for Part!");
            }
        };
    };
}

/// MechAssembly representing a complete robot frame with structural and kinematic properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MechAssembly {
    pub head: Part,
    pub torso: Part,
    pub left_arm: Part,
    pub right_arm: Part,
    pub legs: Part,
}

/// Validation errors that can be detected during assembly inspection.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TpsValidationError {
    #[error("Socket mating failure: {slot_a} and {slot_b} sockets are incompatible")]
    SocketMatingFailure { slot_a: &'static str, slot_b: &'static str },

    #[error("Collision intersection between {slot_a} and {slot_b}: distance is {distance:.4}, but combined radius is {combined_radius:.4}")]
    CollisionIntersection {
        slot_a: &'static str,
        slot_b: &'static str,
        distance: f32,
        combined_radius: f32,
    },

    #[error("Load capacity exceeded: legs capacity is {capacity}, but total assembly mass is {total_mass}")]
    LoadCapacityExceeded { capacity: u64, total_mass: u64 },

    #[error("Horizontal center of mass deviation {deviation:.4} exceeds limit {limit:.4}")]
    CenterOfMassDeviation { deviation: f32, limit: f32 },
}

impl MechAssembly {
    /// Inspect the assembly and run Poka-yoke checks for compatibility, clearances, loads, and balance.
    pub fn validate(&self) -> Result<(), TpsValidationError> {
        // 1. Socket mating checks (bitwise matching pattern check)
        if (self.head.socket_fit & 0x0F) != (self.torso.socket_fit & 0x0F) {
            return Err(TpsValidationError::SocketMatingFailure { slot_a: "head", slot_b: "torso" });
        }
        if (self.left_arm.socket_fit & 0xF0) != (self.torso.socket_fit & 0xF0) {
            return Err(TpsValidationError::SocketMatingFailure { slot_a: "left_arm", slot_b: "torso" });
        }
        if (self.right_arm.socket_fit & 0xF0) != (self.torso.socket_fit & 0xF0) {
            return Err(TpsValidationError::SocketMatingFailure { slot_a: "right_arm", slot_b: "torso" });
        }
        if (self.legs.socket_fit & 0x0F) != (self.torso.socket_fit & 0x0F) {
            return Err(TpsValidationError::SocketMatingFailure { slot_a: "legs", slot_b: "torso" });
        }

        // Helper to get part radius based on collision volume and motion clearance.
        let get_radius = |part: &Part| -> f32 {
            ((part.collision_volume & 0xFF) as f32) / 100.0 + ((part.motion_clearance & 0xFF) as f32) / 200.0
        };

        // 2. Collision intersections check (relative coordinates layout)
        let slots = [
            ("head", (0.0f32, 0.0f32, 2.0f32), &self.head),
            ("torso", (0.0f32, 0.0f32, 0.0f32), &self.torso),
            ("left_arm", (-2.0f32, 0.0f32, 1.0f32), &self.left_arm),
            ("right_arm", (2.0f32, 0.0f32, 1.0f32), &self.right_arm),
            ("legs", (0.0f32, 0.0f32, -2.0f32), &self.legs),
        ];

        for i in 0..slots.len() {
            for j in (i + 1)..slots.len() {
                let (name_a, pos_a, part_a) = slots[i];
                let (name_b, pos_b, part_b) = slots[j];
                let dx = pos_a.0 - pos_b.0;
                let dy = pos_a.1 - pos_b.1;
                let dz = pos_a.2 - pos_b.2;
                let distance = (dx * dx + dy * dy + dz * dz).sqrt();
                let radius_a = get_radius(part_a);
                let radius_b = get_radius(part_b);
                let combined_radius = radius_a + radius_b;
                if combined_radius > distance {
                    return Err(TpsValidationError::CollisionIntersection {
                        slot_a: name_a,
                        slot_b: name_b,
                        distance,
                        combined_radius,
                    });
                }
            }
        }

        // 3. Load capacity check: legs capacity is legs.geometry * 10
        let capacity = self.legs.geometry.wrapping_mul(10);
        let total_mass = self.head.mass_balance
            .wrapping_add(self.torso.mass_balance)
            .wrapping_add(self.left_arm.mass_balance)
            .wrapping_add(self.right_arm.mass_balance)
            .wrapping_add(self.legs.mass_balance);

        if total_mass > capacity {
            return Err(TpsValidationError::LoadCapacityExceeded {
                capacity,
                total_mass,
            });
        }

        // 4. Horizontal center of mass deviations check
        if total_mass > 0 {
            let left_mass = self.left_arm.mass_balance as f32;
            let right_mass = self.right_arm.mass_balance as f32;
            let com_x = (right_mass * 2.0 - left_mass * 2.0) / (total_mass as f32);
            let deviation = com_x.abs();
            let limit = 0.5f32;
            if deviation > limit {
                return Err(TpsValidationError::CenterOfMassDeviation {
                    deviation,
                    limit,
                });
            }
        }

        Ok(())
    }
}

/// Jidoka check: automation with a human touch, stops the line immediately on any defect.
pub fn jidoka_check(assembly: &MechAssembly) {
    if let Err(err) = assembly.validate() {
        panic!("JIDOKA STOP: Defect detected in MechAssembly: {}", err);
    }
}

/// TpsAssemblyReceipt with lineage hash, timestamp, passed gates, and final decision.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TpsAssemblyReceipt {
    pub lineage_hash: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub passed_gates: Vec<String>,
    pub final_decision: String,
}

impl TpsAssemblyReceipt {
    /// Generate a cryptographic receipt proving manufacturing compliance.
    pub fn generate(assembly: &MechAssembly, passed_gates: Vec<String>, final_decision: String) -> Self {
        let serialized = serde_json::to_string(assembly).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        let hash_result = hasher.finalize();
        let lineage_hash = format!("{:x}", hash_result);

        TpsAssemblyReceipt {
            lineage_hash,
            timestamp: chrono::Utc::now(),
            passed_gates,
            final_decision,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branchless_transformation() {
        let state = StateVector {
            civilization_id: 1,
            frame_id: 2,
            armor_profile: 3,
            joint_profile: 4,
            mass_profile: 5,
        };
        let part = μ(state);

        assert!(part.geometry > 0);
        assert!(part.physics_role > 0);
        assert!(part.socket_fit > 0);
        assert!(part.collision_volume > 0);
        assert!(part.mass_balance > 0);
        assert!(part.assembly_compatibility > 0);

        // Verify correctness for a specific expected transformation path:
        // geom_idx = (1 ^ 2) & 7 = 3 & 7 = 3. GEOMETRY_BASE_TABLE[3] = 400.
        assert_eq!(part.geometry, 400);

        // role_idx = (3 ^ 4) & 7 = 7 & 7 = 7. PHYSICS_ROLE_TABLE[7] = 8.
        assert_eq!(part.physics_role, 8);

        // compat_idx = (5 ^ 2) & 7 = 7 & 7 = 7. ASSEMBLY_COMPATIBILITY_TABLE[7] = 0xFFFF.
        assert_eq!(part.assembly_compatibility, 0xFFFF);
    }

    #[test]
    fn test_poka_yoke_const_validation() {
        let valid_part = Part {
            geometry: 100,
            socket_fit: 5,
            motion_clearance: 10,
            collision_volume: 50,
            mass_balance: 120,
            physics_role: 1,
            assembly_compatibility: 0xFFFF,
        };
        assert!(const_validate_part(&valid_part));

        let invalid_part = Part {
            geometry: 0,
            socket_fit: 5,
            motion_clearance: 10,
            collision_volume: 50,
            mass_balance: 120,
            physics_role: 1,
            assembly_compatibility: 0xFFFF,
        };
        assert!(!const_validate_part(&invalid_part));
    }

    #[test]
    fn test_assert_part_poka_yoke_macro() {
        // Since macro runs in a const context, let's declare a const Part.
        const VALID_PART: Part = Part {
            geometry: 100,
            socket_fit: 5,
            motion_clearance: 10,
            collision_volume: 50,
            mass_balance: 120,
            physics_role: 1,
            assembly_compatibility: 0xFFFF,
        };
        assert_part_poka_yoke!(VALID_PART);
    }

    #[test]
    fn test_mech_assembly_validation_success() {
        // Let's create parts that satisfy all checks.
        // Geometry: 500 for legs gives legs_capacity = 5000.
        // Mass balance: 100 for each part. Total mass = 500.
        // Sockets: socket_fit must mate.
        // Torso socket: 0xFF.
        // Head socket: 0xFF (head & 0x0F = 0x0F, torso & 0x0F = 0x0F).
        // Left arm socket: 0xFF (left_arm & 0xF0 = 0xF0, torso & 0xF0 = 0xF0).
        // Right arm socket: 0xFF (right_arm & 0xF0 = 0xF0, torso & 0xF0 = 0xF0).
        // Legs socket: 0xFF (legs & 0x0F = 0x0F, torso & 0x0F = 0x0F).
        // Collision: keep collision volume low (e.g. 1) and motion clearance low (e.g. 1).
        // Radius of each part = 1/100 + 1/200 = 0.015.
        // Left mass = 100, Right mass = 100. com_x = (200 - 200)/500 = 0.
        let compliant_part = Part {
            geometry: 500,
            socket_fit: 0xFF,
            motion_clearance: 1,
            collision_volume: 1,
            mass_balance: 100,
            physics_role: 1,
            assembly_compatibility: 0xFFFF,
        };

        let assembly = MechAssembly {
            head: compliant_part,
            torso: compliant_part,
            left_arm: compliant_part,
            right_arm: compliant_part,
            legs: compliant_part,
        };

        assert!(assembly.validate().is_ok());
        jidoka_check(&assembly);
    }

    #[test]
    fn test_mech_assembly_validation_socket_mismatch() {
        let torso_part = Part {
            geometry: 500,
            socket_fit: 0xFF,
            motion_clearance: 1,
            collision_volume: 1,
            mass_balance: 100,
            physics_role: 1,
            assembly_compatibility: 0xFFFF,
        };

        // Head socket ends in 0x00, torso ends in 0x0F. This mismatch fails validation.
        let bad_head = Part {
            socket_fit: 0xF0,
            ..torso_part
        };

        let assembly = MechAssembly {
            head: bad_head,
            torso: torso_part,
            left_arm: torso_part,
            right_arm: torso_part,
            legs: torso_part,
        };

        let result = assembly.validate();
        assert!(matches!(result, Err(TpsValidationError::SocketMatingFailure { slot_a: "head", slot_b: "torso" })));
    }

    #[test]
    fn test_mech_assembly_validation_collision() {
        let torso_part = Part {
            geometry: 500,
            socket_fit: 0xFF,
            motion_clearance: 1,
            collision_volume: 1,
            mass_balance: 100,
            physics_role: 1,
            assembly_compatibility: 0xFFFF,
        };

        // If head and torso overlap because they have giant collision radius.
        // Distance Head-Torso is 2.0.
        // If head has collision_volume = 200 (radius_a = 2.0), they will overlap.
        let huge_head = Part {
            collision_volume: 200, // radius = 2.0
            ..torso_part
        };

        let assembly = MechAssembly {
            head: huge_head,
            torso: torso_part,
            left_arm: torso_part,
            right_arm: torso_part,
            legs: torso_part,
        };

        let result = assembly.validate();
        assert!(matches!(result, Err(TpsValidationError::CollisionIntersection { .. })));
    }

    #[test]
    fn test_mech_assembly_validation_overload() {
        let torso_part = Part {
            geometry: 10, // low geometry makes legs capacity = 10 * 10 = 100
            socket_fit: 0xFF,
            motion_clearance: 1,
            collision_volume: 1,
            mass_balance: 100, // total mass of 5 parts is 500, exceeding capacity of 100
            physics_role: 1,
            assembly_compatibility: 0xFFFF,
        };

        let assembly = MechAssembly {
            head: torso_part,
            torso: torso_part,
            left_arm: torso_part,
            right_arm: torso_part,
            legs: torso_part,
        };

        let result = assembly.validate();
        assert!(matches!(result, Err(TpsValidationError::LoadCapacityExceeded { .. })));
    }

    #[test]
    fn test_mech_assembly_validation_com_deviation() {
        let compliant_part = Part {
            geometry: 500,
            socket_fit: 0xFF,
            motion_clearance: 1,
            collision_volume: 1,
            mass_balance: 10,
            physics_role: 1,
            assembly_compatibility: 0xFFFF,
        };

        // Left arm has massive mass, right arm has minimal. This creates a horizontal center of mass deviation.
        let heavy_left = Part {
            mass_balance: 300,
            ..compliant_part
        };

        let assembly = MechAssembly {
            head: compliant_part,
            torso: compliant_part,
            left_arm: heavy_left,
            right_arm: compliant_part,
            legs: compliant_part,
        };

        let result = assembly.validate();
        assert!(matches!(result, Err(TpsValidationError::CenterOfMassDeviation { .. })));
    }

    #[test]
    #[should_panic(expected = "JIDOKA STOP")]
    fn test_jidoka_panic_on_defect() {
        let bad_part = Part {
            geometry: 10,
            socket_fit: 0xFF,
            motion_clearance: 1,
            collision_volume: 1,
            mass_balance: 500,
            physics_role: 1,
            assembly_compatibility: 0xFFFF,
        };
        let assembly = MechAssembly {
            head: bad_part,
            torso: bad_part,
            left_arm: bad_part,
            right_arm: bad_part,
            legs: bad_part,
        };
        jidoka_check(&assembly);
    }

    #[test]
    fn test_receipt_generation() {
        let compliant_part = Part {
            geometry: 500,
            socket_fit: 0xFF,
            motion_clearance: 1,
            collision_volume: 1,
            mass_balance: 100,
            physics_role: 1,
            assembly_compatibility: 0xFFFF,
        };

        let assembly = MechAssembly {
            head: compliant_part,
            torso: compliant_part,
            left_arm: compliant_part,
            right_arm: compliant_part,
            legs: compliant_part,
        };

        let passed_gates = vec!["GATE 0".to_string(), "GATE 1".to_string()];
        let receipt = TpsAssemblyReceipt::generate(&assembly, passed_gates, "APPROVED".to_string());

        assert!(!receipt.lineage_hash.is_empty());
        assert_eq!(receipt.final_decision, "APPROVED");
        assert_eq!(receipt.passed_gates.len(), 2);
    }
}
