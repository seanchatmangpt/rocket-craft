
//! Authority Byte Fields — SoA layout.
//! ⚠️ GENERATED FILE — do NOT edit by hand.
//! Source of truth: ontology/mechbirth.ttl
//! Generator:       ggen/templates/authority_state.rs.tera
//! SPARQL:          ggen/sparql/extract_authority_fields.sparql

use crate::error::RefusalReason;

/// Maximum admitted class value for any authority byte field.
/// Sourced from mb:MAX_CLASS mb:value in ontology.
pub const MAX_CLASS: u8 = 15;

/// Sentinel value indicating refusal — never admitted to live state.
/// Sourced from mb:REFUSED_CLASS mb:value in ontology.
pub const REFUSED_CLASS: u8 = 255;

/// SoA authority state. Every field is a parallel `Vec<u8>` of length `count`.
/// Invariant: all Vecs have identical length at all times.
///
/// Fields generated from extract_authority_fields.sparql:
/// - `damage` (soaIndex=0): per-part damage accumulation class
/// - `heat` (soaIndex=1): per-part thermal accumulation class
/// - `stress` (soaIndex=2): per-part structural stress class
/// - `grip` (soaIndex=3): per-part grip class — 7 is nominal
/// - `socket_health` (soaIndex=4): per-socket health class — MAX_CLASS means fully healthy
/// - `lod` (soaIndex=5): LOD tier: 0=PRIMARY(CROWN), 1=SECONDARY, 2=TERTIARY

#[derive(Debug, Clone, Default)]
pub struct AuthorityState {

    /// per-part damage accumulation class

    pub damage: Vec<u8>,

    /// per-part thermal accumulation class

    pub heat: Vec<u8>,

    /// per-part structural stress class

    pub stress: Vec<u8>,

    /// per-part grip class — 7 is nominal

    pub grip: Vec<u8>,

    /// per-socket health class — MAX_CLASS means fully healthy

    pub socket_health: Vec<u8>,

    /// LOD tier: 0=PRIMARY(CROWN), 1=SECONDARY, 2=TERTIARY

    pub lod: Vec<u8>,

}

impl AuthorityState {
    /// Construct a zero-initialised authority state for `count` parts
    /// with ontology-specified default values per field.
    pub fn new(count: usize) -> Self {
        Self {

            damage: vec![0u8; count],

            heat: vec![0u8; count],

            stress: vec![0u8; count],

            grip: vec![7u8; count],

            socket_health: vec![15u8; count],

            lod: vec![2u8; count],

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

    /// Validates that all class values fall within `[0, MAX_CLASS]`.
    pub fn validate_classes(&self) -> Vec<RefusalReason> {
        let mut errs = Vec::new();

        for (i, &v) in self.damage.iter().enumerate() {
            if v > MAX_CLASS {
                errs.push(RefusalReason::InvalidAuthorityClass {
                    field: format!("damage[{}]", i),
                    class: v,
                });
            }
        }

        for (i, &v) in self.heat.iter().enumerate() {
            if v > MAX_CLASS {
                errs.push(RefusalReason::InvalidAuthorityClass {
                    field: format!("heat[{}]", i),
                    class: v,
                });
            }
        }

        for (i, &v) in self.stress.iter().enumerate() {
            if v > MAX_CLASS {
                errs.push(RefusalReason::InvalidAuthorityClass {
                    field: format!("stress[{}]", i),
                    class: v,
                });
            }
        }

        for (i, &v) in self.grip.iter().enumerate() {
            if v > MAX_CLASS {
                errs.push(RefusalReason::InvalidAuthorityClass {
                    field: format!("grip[{}]", i),
                    class: v,
                });
            }
        }

        for (i, &v) in self.socket_health.iter().enumerate() {
            if v > MAX_CLASS {
                errs.push(RefusalReason::InvalidAuthorityClass {
                    field: format!("socket_health[{}]", i),
                    class: v,
                });
            }
        }

        for (i, &v) in self.lod.iter().enumerate() {
            if v > MAX_CLASS {
                errs.push(RefusalReason::InvalidAuthorityClass {
                    field: format!("lod[{}]", i),
                    class: v,
                });
            }
        }

        errs
    }
}