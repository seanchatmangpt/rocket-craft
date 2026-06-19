// GENERATED FILE - DO NOT EDIT MANUALLY
// Source: O* = ontology/ggen-packs/mechbirth/schema/mechbirth_lod_geom_motion.ttl
// Pipeline: ggen -> extract_motion_families.sparql -> motion.rs.tera
// Synthesis: van der Aalst (Petri-net precedence ordering) + Carmack (no allocation, positional Vec scan)
//
// GC-MECHBIRTH-002 Motion Surrogate
// Counterfactual laws encoded in O*:
//   - CF-1: PlantFeet must appear at index BEFORE Fire (positional, not containment)
//   - CF-2: VentHeat must appear at index BEFORE Fire when heat_class >= 12
//   - CF-4: heat_class/stress_class/leg_damage_class must be in [0, 15]

use crate::error::RefusalReason;

// --- MOTION FAMILY ---
// Derived from O* MotionFamily taxonomy via extract_motion_families.sparql

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MotionFamily {

    Brace,

    Collapse,

    FireWeapon,

    Recover,

    Repair,

    Run,

    Walk,

}

// --- MOTION PHASE ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MotionPhase {
    AcquireTarget,
    RotateTorso,
    PlantFeet,
    LockHip,
    StabilizeShoulder,
    ChargeWeapon,
    VentHeat,
    Fire,
    AbsorbRecoil,
    RecoverBalance,
    CoolSocket,
    ReturnToReady,
    Stride,
    Brace,
    Collapse,
    DiagnoseSocket,
    ApplyRepair,
    VerifyRepair,
}

// --- MOTION TRACE ---

#[derive(Debug, Clone)]
pub struct MotionTrace {
    pub family: MotionFamily,
    pub phases: Vec<MotionPhase>,
    pub socket_available: bool,
    pub heat_class: u8,
    pub stress_class: u8,
    pub leg_damage_class: u8,
}

impl MotionTrace {
    /// Validate trace and compute authority-state deltas.
    /// Returns Ok((delta_heat, delta_stress)) or Err(RefusalReason).
    /// Laws encoded in O* - do not derive from this code.
    pub fn validate_and_compute_effects(&self) -> Result<(i8, i8), RefusalReason> {
        match self.family {


            MotionFamily::Brace => Ok((0, 0)),



            MotionFamily::Collapse => Ok((0, 0)),



            MotionFamily::FireWeapon => self.validate_fire_weapon(),



            MotionFamily::Recover => Ok((0, 0)),



            MotionFamily::Repair => Ok((0, 0)),



            MotionFamily::Run => self.validate_walk_run(),



            MotionFamily::Walk => self.validate_walk_run(),


        }
    }

    fn validate_fire_weapon(&self) -> Result<(i8, i8), RefusalReason> {
        // CF-4 guard (O* MotionInputBoundsLaw): class fields must be in [0, 15].
        const MAX_CLASS: u8 = 15;
        if self.heat_class > MAX_CLASS
            || self.stress_class > MAX_CLASS
            || self.leg_damage_class > MAX_CLASS
        {
            return Err(RefusalReason::MotionClearanceViolation {
                detail: format!(
                    "class value out of range: heat={} stress={} leg_damage={}",
                    self.heat_class, self.stress_class, self.leg_damage_class
                ),
            });
        }

        let fire_idx = self.phases.iter().position(|p| p == &MotionPhase::Fire);
        let plant_idx = self.phases.iter().position(|p| p == &MotionPhase::PlantFeet);
        let has_fire = fire_idx.is_some();

        // CF-1 (O* PlantFeetBeforeFire constraint):
        // PlantFeet must positionally precede Fire - containment alone is insufficient.
        if has_fire {
            match (plant_idx, fire_idx) {
                (None, _) => {
                    return Err(RefusalReason::MotionClearanceViolation {
                        detail: "Fire phase requires PlantFeet".into(),
                    });
                }
                (Some(pi), Some(fi)) if pi >= fi => {
                    return Err(RefusalReason::MotionClearanceViolation {
                        detail: format!(
                            "PlantFeet (idx {}) must precede Fire (idx {})",
                            pi, fi
                        ),
                    });
                }
                _ => {}
            }
        }

        // Socket required for weapon actuation.
        if has_fire && !self.socket_available {
            return Err(RefusalReason::MissingSocket {
                dependent: "FireWeapon".into(),
            });
        }

        // CF-2 (O* VentHeatBeforeFire constraint, heatThreshold=12):
        // When heat_class >= 12, VentHeat must positionally precede Fire.
        if has_fire && self.heat_class >= 12 {
            let vent_idx = self.phases.iter().position(|p| p == &MotionPhase::VentHeat);
            match (vent_idx, fire_idx) {
                (None, _) => {
                    return Err(RefusalReason::MotionClearanceViolation {
                        detail: format!(
                            "heat_class {} requires VentHeat before Fire",
                            self.heat_class
                        ),
                    });
                }
                (Some(vi), Some(fi)) if vi >= fi => {
                    return Err(RefusalReason::MotionClearanceViolation {
                        detail: format!(
                            "VentHeat (idx {}) must precede Fire (idx {}); heat_class={}",
                            vi, fi, self.heat_class
                        ),
                    });
                }
                _ => {}
            }
        }

        // Effects: Fire +heat; AbsorbRecoil +stress.
        let delta_heat: i8 = if has_fire { 2 } else { 0 };
        let delta_stress: i8 = if self.phases.contains(&MotionPhase::AbsorbRecoil) { 1 } else { 0 };
        Ok((delta_heat, delta_stress))
    }

    fn validate_walk_run(&self) -> Result<(i8, i8), RefusalReason> {
        // CF-4 guard (O* MotionInputBoundsLaw).
        const MAX_CLASS: u8 = 15;
        if self.heat_class > MAX_CLASS
            || self.stress_class > MAX_CLASS
            || self.leg_damage_class > MAX_CLASS
        {
            return Err(RefusalReason::MotionClearanceViolation {
                detail: format!(
                    "class value out of range: heat={} stress={} leg_damage={}",
                    self.heat_class, self.stress_class, self.leg_damage_class
                ),
            });
        }
        // O* RunFamily law: leg_damage_class >= 12 -> refused for Run; admitted (degraded) for Walk.
        if self.family == MotionFamily::Run && self.leg_damage_class >= 12 {
            return Err(RefusalReason::MotionClearanceViolation {
                detail: format!("leg_damage_class {} too high for Run", self.leg_damage_class),
            });
        }
        Ok((0, 0))
    }
}