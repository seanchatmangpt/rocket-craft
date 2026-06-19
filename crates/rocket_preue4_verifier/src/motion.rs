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
        // CF-4 guard: class values must be in [0, 15].
        let max_class: u8 = 15;
        if self.heat_class > max_class || self.stress_class > max_class || self.leg_damage_class > max_class {
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

        // CF-1: PlantFeet must positionally precede Fire — not merely be present.
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
                            "PlantFeet (idx {pi}) must precede Fire (idx {fi}) in phase sequence"
                        ),
                    });
                }
                _ => {}
            }
        }
        // Socket required for WeaponMount actuation.
        if has_fire && !self.socket_available {
            return Err(RefusalReason::MissingSocket {
                dependent: "FireWeapon".into(),
            });
        }
        // CF-2: High heat — VentHeat must positionally precede Fire.
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
                            "VentHeat (idx {vi}) must precede Fire (idx {fi}); heat_class={}",
                            self.heat_class
                        ),
                    });
                }
                _ => {}
            }
        }
        // Effects: Fire increases heat; AbsorbRecoil increases stress.
        let delta_heat: i8 = if has_fire { 2 } else { 0 };
        let delta_stress: i8 = if self.phases.contains(&MotionPhase::AbsorbRecoil) { 1 } else { 0 };
        Ok((delta_heat, delta_stress))
    }

    fn validate_walk_run(&self) -> Result<(i8, i8), RefusalReason> {
        // CF-4 guard: class values must be in [0, 15].
        let max_class: u8 = 15;
        if self.heat_class > max_class || self.stress_class > max_class || self.leg_damage_class > max_class {
            return Err(RefusalReason::MotionClearanceViolation {
                detail: format!(
                    "class value out of range: heat={} stress={} leg_damage={}",
                    self.heat_class, self.stress_class, self.leg_damage_class
                ),
            });
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn fire_trace(phases: Vec<MotionPhase>, socket: bool, heat: u8) -> MotionTrace {
        MotionTrace {
            family: MotionFamily::FireWeapon,
            phases,
            socket_available: socket,
            heat_class: heat,
            stress_class: 0,
            leg_damage_class: 0,
        }
    }

    // ── FireWeapon validation ─────────────────────────────────────────────────

    #[test]
    fn fire_weapon_valid_with_plant_and_socket() {
        let t = fire_trace(
            vec![MotionPhase::PlantFeet, MotionPhase::Fire],
            true, 0,
        );
        let (dh, ds) = t.validate_and_compute_effects().unwrap();
        assert_eq!(dh, 2); // fire +2 heat
        assert_eq!(ds, 0); // no AbsorbRecoil in trace
    }

    #[test]
    fn fire_weapon_without_plant_feet_returns_error() {
        let t = fire_trace(vec![MotionPhase::Fire], true, 0);
        assert!(matches!(
            t.validate_and_compute_effects().unwrap_err(),
            crate::error::RefusalReason::MotionClearanceViolation { .. }
        ));
    }

    #[test]
    fn fire_weapon_without_socket_returns_missing_socket() {
        let t = fire_trace(
            vec![MotionPhase::PlantFeet, MotionPhase::Fire],
            false, 0,
        );
        assert!(matches!(
            t.validate_and_compute_effects().unwrap_err(),
            crate::error::RefusalReason::MissingSocket { .. }
        ));
    }

    #[test]
    fn fire_weapon_high_heat_without_vent_returns_error() {
        let t = fire_trace(
            vec![MotionPhase::PlantFeet, MotionPhase::Fire],
            true, 12, // heat_class >= 12 requires VentHeat
        );
        assert!(matches!(
            t.validate_and_compute_effects().unwrap_err(),
            crate::error::RefusalReason::MotionClearanceViolation { .. }
        ));
    }

    #[test]
    fn fire_weapon_high_heat_with_vent_passes() {
        let t = fire_trace(
            vec![MotionPhase::PlantFeet, MotionPhase::VentHeat, MotionPhase::Fire],
            true, 12,
        );
        assert!(t.validate_and_compute_effects().is_ok());
    }

    #[test]
    fn absorb_recoil_adds_stress_delta() {
        let t = fire_trace(
            vec![MotionPhase::PlantFeet, MotionPhase::Fire, MotionPhase::AbsorbRecoil],
            true, 0,
        );
        let (dh, ds) = t.validate_and_compute_effects().unwrap();
        assert_eq!(dh, 2);
        assert_eq!(ds, 1);
    }

    // ── Walk / Run validation ─────────────────────────────────────────────────

    #[test]
    fn run_with_low_leg_damage_passes() {
        let t = MotionTrace {
            family: MotionFamily::Run,
            phases: vec![MotionPhase::Stride],
            socket_available: true,
            heat_class: 0,
            stress_class: 0,
            leg_damage_class: 5,
        };
        assert!(t.validate_and_compute_effects().is_ok());
    }

    #[test]
    fn run_with_leg_damage_12_or_higher_returns_error() {
        let t = MotionTrace {
            family: MotionFamily::Run,
            phases: vec![MotionPhase::Stride],
            socket_available: true,
            heat_class: 0,
            stress_class: 0,
            leg_damage_class: 12,
        };
        assert!(matches!(
            t.validate_and_compute_effects().unwrap_err(),
            crate::error::RefusalReason::MotionClearanceViolation { .. }
        ));
    }

    #[test]
    fn walk_with_high_leg_damage_passes_degraded() {
        let t = MotionTrace {
            family: MotionFamily::Walk,
            phases: vec![MotionPhase::Stride],
            socket_available: true,
            heat_class: 0,
            stress_class: 0,
            leg_damage_class: 15, // high but Walk is admitted
        };
        let (dh, ds) = t.validate_and_compute_effects().unwrap();
        assert_eq!(dh, 0);
        assert_eq!(ds, 0);
    }

    #[test]
    fn other_motion_families_return_zero_effects() {
        for family in [MotionFamily::Brace, MotionFamily::Recover, MotionFamily::Repair] {
            let t = MotionTrace {
                family,
                phases: vec![],
                socket_available: true,
                heat_class: 0,
                stress_class: 0,
                leg_damage_class: 0,
            };
            let (dh, ds) = t.validate_and_compute_effects().unwrap();
            assert_eq!((dh, ds), (0, 0));
        }
    }
}
