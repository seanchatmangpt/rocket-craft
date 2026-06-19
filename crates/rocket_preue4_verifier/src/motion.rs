use crate::error::RefusalReason;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MotionFamily {
    Walk,
    Run,
    Brace,
    FireWeapon,
    Recover,
    Repair,
    Collapse,
}

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
    // Walk/Run
    Stride,
    Brace,
    Collapse,
    // Repair
    DiagnoseSocket,
    ApplyRepair,
    VerifyRepair,
}

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
    /// Returns `Ok((delta_heat, delta_stress))` or `Err(RefusalReason)`.
    /// Effects represent the delta to apply to authority state after the trace.
    pub fn validate_and_compute_effects(&self) -> Result<(i8, i8), RefusalReason> {
        match self.family {
            MotionFamily::FireWeapon => self.validate_fire_weapon(),
            MotionFamily::Walk | MotionFamily::Run => self.validate_walk_run(),
            _ => Ok((0, 0)),
        }
    }

    fn validate_fire_weapon(&self) -> Result<(i8, i8), RefusalReason> {
        let has_plant = self.phases.contains(&MotionPhase::PlantFeet);
        let has_fire = self.phases.contains(&MotionPhase::Fire);

        // PlantFeet must precede Fire.
        if has_fire && !has_plant {
            return Err(RefusalReason::MotionClearanceViolation {
                detail: "Fire phase requires PlantFeet".into(),
            });
        }
        // Socket required for WeaponMount actuation.
        if has_fire && !self.socket_available {
            return Err(RefusalReason::MissingSocket {
                dependent: "FireWeapon".into(),
            });
        }
        // High heat forces VentHeat or refuses the trace.
        if has_fire && self.heat_class >= 12 && !self.phases.contains(&MotionPhase::VentHeat) {
            return Err(RefusalReason::MotionClearanceViolation {
                detail: format!(
                    "heat_class {} requires VentHeat before Fire",
                    self.heat_class
                ),
            });
        }
        // Effects: Fire increases heat; AbsorbRecoil increases stress.
        let delta_heat: i8 = if has_fire { 2 } else { 0 };
        let delta_stress: i8 = if self.phases.contains(&MotionPhase::AbsorbRecoil) {
            1
        } else {
            0
        };
        Ok((delta_heat, delta_stress))
    }

    fn validate_walk_run(&self) -> Result<(i8, i8), RefusalReason> {
        // Run is refused when leg damage is too severe.
        if self.family == MotionFamily::Run && self.leg_damage_class >= 12 {
            return Err(RefusalReason::MotionClearanceViolation {
                detail: format!(
                    "leg_damage_class {} too high for Run",
                    self.leg_damage_class
                ),
            });
        }
        // Walk with high leg damage is degraded but admitted.
        Ok((0, 0))
    }
}
