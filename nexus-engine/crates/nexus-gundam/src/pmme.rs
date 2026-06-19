use crate::generated_gundam::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssemblySpec {
    pub frame: Frame,
    pub mobility: Mobility,
    pub power: Power,
    pub armor: Armor,
    pub weapons: Vec<Weapon>,
    pub sensors: Sensor,
}

pub struct ProceduralMechManufacturingEngine;

impl ProceduralMechManufacturingEngine {
    pub fn manufacture(spec: AssemblySpec) -> Result<String, String> {
        // Validate Collision Volume (Poka-yoke)
        let total_volume =
            spec.frame.occupancy.max[0] * spec.frame.occupancy.max[1] * spec.frame.occupancy.max[2];
        if total_volume > 100.0 {
            return Err("Collision volume exceeded".into());
        }

        // Deterministic assembly receipt generation
        let receipt = format!(
            "ASSEMBLY_RECEIPT:{}",
            blake3::hash(serde_json::to_string(&spec).unwrap().as_bytes()).to_hex()
        );
        Ok(receipt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated_gundam::{AABB, Armor, Frame, Mobility, Power, Sensor, Weapon};

    fn small_aabb() -> AABB {
        AABB::new([0.0, 0.0, 0.0], [0.5, 0.5, 0.5])
    }

    fn valid_spec() -> AssemblySpec {
        // Frame occupancy max = [1,1,1] → volume = 1.0 < 100.0 ✓
        AssemblySpec {
            frame: Frame::default(),
            mobility: Mobility::default(),
            power: Power::default(),
            armor: Armor::default(),
            weapons: vec![Weapon {
                id: "Beam Saber".into(), mass: 5.0,
                occupancy: small_aabb(), clearance: small_aabb(),
                damage: 80.0, range: 1.5,
            }],
            sensors: Sensor {
                id: "Minovsky Radar".into(), mass: 2.0,
                occupancy: small_aabb(), clearance: small_aabb(),
                detection_range: 500.0,
            },
        }
    }

    // ── manufacture success path ──────────────────────────────────────────────

    #[test]
    fn manufacture_valid_spec_returns_ok_with_receipt_prefix() {
        let result = ProceduralMechManufacturingEngine::manufacture(valid_spec());
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with("ASSEMBLY_RECEIPT:"));
    }

    #[test]
    fn receipt_is_64_hex_chars_after_prefix() {
        let receipt = ProceduralMechManufacturingEngine::manufacture(valid_spec()).unwrap();
        let hash_part = receipt.strip_prefix("ASSEMBLY_RECEIPT:").unwrap();
        assert_eq!(hash_part.len(), 64); // BLAKE3 = 32 bytes = 64 hex chars
        assert!(hash_part.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn receipt_is_deterministic_for_same_spec() {
        let r1 = ProceduralMechManufacturingEngine::manufacture(valid_spec()).unwrap();
        let r2 = ProceduralMechManufacturingEngine::manufacture(valid_spec()).unwrap();
        assert_eq!(r1, r2);
    }

    #[test]
    fn different_specs_produce_different_receipts() {
        let mut spec2 = valid_spec();
        spec2.frame.mass = 999.0; // different content
        let r1 = ProceduralMechManufacturingEngine::manufacture(valid_spec()).unwrap();
        let r2 = ProceduralMechManufacturingEngine::manufacture(spec2).unwrap();
        assert_ne!(r1, r2);
    }

    // ── collision volume gate ─────────────────────────────────────────────────

    #[test]
    fn manufacture_rejects_frame_with_volume_over_100() {
        let mut spec = valid_spec();
        // max = [5, 5, 5] → volume = 125 > 100
        spec.frame.occupancy = AABB::new([0.0, 0.0, 0.0], [5.0, 5.0, 5.0]);
        let result = ProceduralMechManufacturingEngine::manufacture(spec);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Collision volume exceeded"));
    }

    #[test]
    fn manufacture_accepts_frame_exactly_at_volume_100() {
        // max = [4.64, 4.64, 4.64] ≈ 100  (4.64^3 ≈ 99.9) — within threshold
        let v = 4.64_f32;
        let mut spec = valid_spec();
        spec.frame.occupancy = AABB::new([0.0, 0.0, 0.0], [v, v, v]);
        // 4.64^3 ≈ 99.8 < 100
        assert!(ProceduralMechManufacturingEngine::manufacture(spec).is_ok());
    }

    // ── empty weapons list ────────────────────────────────────────────────────

    #[test]
    fn manufacture_succeeds_with_no_weapons() {
        let mut spec = valid_spec();
        spec.weapons = vec![];
        assert!(ProceduralMechManufacturingEngine::manufacture(spec).is_ok());
    }

    // ── multiple weapons ──────────────────────────────────────────────────────

    #[test]
    fn manufacture_succeeds_with_multiple_weapons() {
        let mut spec = valid_spec();
        spec.weapons.push(Weapon {
            id: "Shield".into(), mass: 8.0,
            occupancy: small_aabb(), clearance: small_aabb(),
            damage: 0.0, range: 0.0,
        });
        assert!(ProceduralMechManufacturingEngine::manufacture(spec).is_ok());
    }

    // ── AssemblySpec serde ────────────────────────────────────────────────────

    #[test]
    fn assembly_spec_serializes_and_deserializes() {
        let spec = valid_spec();
        let json = serde_json::to_string(&spec).unwrap();
        let back: AssemblySpec = serde_json::from_str(&json).unwrap();
        assert_eq!(back, spec);
    }
}
