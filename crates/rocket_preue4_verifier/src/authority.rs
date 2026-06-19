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
