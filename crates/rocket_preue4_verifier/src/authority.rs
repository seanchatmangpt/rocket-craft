//! GC-MECHBIRTH-002: Authority Byte Fields — SoA layout.
//! All authority state is held in Structure-of-Arrays buffers.
//! No mutations from prediction layer are permitted.

use crate::error::RefusalReason;

/// Maximum admitted class value for any authority byte field.
pub const MAX_CLASS: u8 = 15;

/// Sentinel value indicating refusal — never admitted to live state.
pub const REFUSED_CLASS: u8 = 255;

/// SoA authority state. Every field is a parallel `Vec<u8>` of length `count`.
/// Invariant: all Vecs have identical length at all times.
#[derive(Debug, Clone, Default)]
pub struct AuthorityState {
    /// Per-part damage class [0, MAX_CLASS].
    pub damage: Vec<u8>,
    /// Per-part thermal class [0, MAX_CLASS].
    pub heat: Vec<u8>,
    /// Per-part structural stress class [0, MAX_CLASS].
    pub stress: Vec<u8>,
    /// Per-part grip class [0, MAX_CLASS]. Default = 7 (nominal).
    pub grip: Vec<u8>,
    /// Per-socket health class [0, MAX_CLASS]. Default = MAX_CLASS (healthy).
    pub socket_health: Vec<u8>,
    /// Per-part LOD class: 0=PRIMARY(CROWN), 1=SECONDARY, 2=TERTIARY.
    pub lod: Vec<u8>,
}

impl AuthorityState {
    /// Construct a new zero-initialised authority state for `count` parts,
    /// with nominal grip and full socket health.
    pub fn new(count: usize) -> Self {
        Self {
            damage: vec![0u8; count],
            heat: vec![0u8; count],
            stress: vec![0u8; count],
            grip: vec![7u8; count],                // nominal grip class
            socket_health: vec![MAX_CLASS; count], // fully healthy
            lod: vec![2u8; count],                 // default SECONDARY
        }
    }

    /// Returns the number of parts tracked by this state.
    #[inline]
    pub fn len(&self) -> usize {
        self.damage.len()
    }

    /// Returns `true` when no parts are tracked.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.damage.is_empty()
    }

    /// Validates that all SoA buffers have identical length.
    /// Returns `Err(InvalidAuthorityClass { field: "lengths", class: 0 })` on mismatch.
    pub fn validate_lengths(&self) -> Result<(), RefusalReason> {
        let n = self.damage.len();
        let ok = self.heat.len() == n
            && self.stress.len() == n
            && self.grip.len() == n
            && self.socket_health.len() == n
            && self.lod.len() == n;
        if ok {
            Ok(())
        } else {
            Err(RefusalReason::InvalidAuthorityClass {
                field: "lengths".into(),
                class: 0,
            })
        }
    }

    /// Validates that all class values in every buffer fall within `[0, MAX_CLASS]`.
    /// Returns a `Vec` of all violations found (may be empty for a clean state).
    pub fn validate_classes(&self) -> Vec<RefusalReason> {
        let mut errs = Vec::new();
        for (i, &v) in self.damage.iter().enumerate() {
            if v > MAX_CLASS {
                errs.push(RefusalReason::InvalidAuthorityClass {
                    field: format!("damage[{i}]"),
                    class: v,
                });
            }
        }
        for (i, &v) in self.heat.iter().enumerate() {
            if v > MAX_CLASS {
                errs.push(RefusalReason::InvalidAuthorityClass {
                    field: format!("heat[{i}]"),
                    class: v,
                });
            }
        }
        for (i, &v) in self.stress.iter().enumerate() {
            if v > MAX_CLASS {
                errs.push(RefusalReason::InvalidAuthorityClass {
                    field: format!("stress[{i}]"),
                    class: v,
                });
            }
        }
        for (i, &v) in self.grip.iter().enumerate() {
            if v > MAX_CLASS {
                errs.push(RefusalReason::InvalidAuthorityClass {
                    field: format!("grip[{i}]"),
                    class: v,
                });
            }
        }
        for (i, &v) in self.socket_health.iter().enumerate() {
            if v > MAX_CLASS {
                errs.push(RefusalReason::InvalidAuthorityClass {
                    field: format!("socket_health[{i}]"),
                    class: v,
                });
            }
        }
        errs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_has_correct_count() {
        let s = AuthorityState::new(5);
        assert_eq!(s.len(), 5);
        assert!(!s.is_empty());
    }

    #[test]
    fn new_state_is_empty_for_zero_parts() {
        let s = AuthorityState::new(0);
        assert!(s.is_empty());
    }

    #[test]
    fn new_state_default_grip_is_7() {
        let s = AuthorityState::new(3);
        assert!(s.grip.iter().all(|&g| g == 7));
    }

    #[test]
    fn new_state_default_socket_health_is_max_class() {
        let s = AuthorityState::new(4);
        assert!(s.socket_health.iter().all(|&h| h == MAX_CLASS));
    }

    #[test]
    fn new_state_default_damage_heat_stress_are_zero() {
        let s = AuthorityState::new(3);
        assert!(s.damage.iter().all(|&v| v == 0));
        assert!(s.heat.iter().all(|&v| v == 0));
        assert!(s.stress.iter().all(|&v| v == 0));
    }

    #[test]
    fn validate_lengths_passes_for_consistent_state() {
        let s = AuthorityState::new(4);
        assert!(s.validate_lengths().is_ok());
    }

    #[test]
    fn validate_lengths_fails_when_buffers_mismatched() {
        let mut s = AuthorityState::new(4);
        s.heat.push(0); // now heat.len() == 5, others == 4
        assert!(s.validate_lengths().is_err());
    }

    #[test]
    fn validate_classes_is_empty_for_default_state() {
        let s = AuthorityState::new(5);
        assert!(s.validate_classes().is_empty());
    }

    #[test]
    fn validate_classes_catches_out_of_range_damage() {
        let mut s = AuthorityState::new(3);
        s.damage[1] = 16; // > MAX_CLASS=15
        let errs = s.validate_classes();
        assert_eq!(errs.len(), 1);
        assert!(matches!(&errs[0],
            RefusalReason::InvalidAuthorityClass { field, .. } if field.contains("damage[1]")
        ));
    }

    #[test]
    fn validate_classes_catches_multiple_violations() {
        let mut s = AuthorityState::new(3);
        s.heat[0] = 255;
        s.stress[2] = 100;
        let errs = s.validate_classes();
        assert_eq!(errs.len(), 2);
    }
}
