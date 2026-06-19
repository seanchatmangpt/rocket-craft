use crate::generated_gundam::*;

pub struct BranchlessPartsGenerator;

impl BranchlessPartsGenerator {
    pub fn generate_part(part_type: &str, state_vector: &[f32; 3]) -> Result<Weapon, String> {
        // Branchless generation logic (mapping state vector directly to physical properties)
        let mass = state_vector[0] * 10.0;
        let damage = state_vector[1] * 5.0;
        let range = state_vector[2] * 20.0;

        // Jidoka: Reject impossible assemblies immediately
        if damage > 100.0 {
            return Err("Invalid damage calculation".into());
        }

        Ok(Weapon {
            id: format!("{}_part", part_type),
            mass,
            occupancy: AABB::default(),
            clearance: AABB::default(),
            damage,
            range,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen(part: &str, sv: [f32; 3]) -> Result<Weapon, String> {
        BranchlessPartsGenerator::generate_part(part, &sv)
    }

    // ── success path ──────────────────────────────────────────────────────────

    #[test]
    fn generate_part_returns_weapon_with_scaled_fields() {
        let w = gen("beam", [1.0, 2.0, 3.0]).unwrap();
        assert!((w.mass - 10.0).abs() < 0.001);   // 1.0 * 10
        assert!((w.damage - 10.0).abs() < 0.001); // 2.0 * 5
        assert!((w.range - 60.0).abs() < 0.001);  // 3.0 * 20
    }

    #[test]
    fn generate_part_id_includes_part_type() {
        let w = gen("sword", [0.5, 0.5, 0.5]).unwrap();
        assert!(w.id.contains("sword"));
    }

    #[test]
    fn zero_state_vector_produces_zero_mass_damage_range() {
        let w = gen("null", [0.0, 0.0, 0.0]).unwrap();
        assert_eq!(w.mass, 0.0);
        assert_eq!(w.damage, 0.0);
        assert_eq!(w.range, 0.0);
    }

    // ── Jidoka gate: damage > 100.0 is rejected ───────────────────────────────

    #[test]
    fn damage_above_100_returns_error() {
        // state_vector[1] * 5 = 20.1 * 5 = 100.5 > 100
        let result = gen("broken", [1.0, 20.1, 1.0]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid damage"));
    }

    #[test]
    fn damage_exactly_100_is_accepted() {
        // state_vector[1] * 5 = 20.0 * 5 = 100.0 — not > 100
        let result = gen("edge", [1.0, 20.0, 1.0]);
        assert!(result.is_ok());
    }

    #[test]
    fn damage_just_above_100_is_rejected() {
        let result = gen("edge2", [1.0, 20.0001, 1.0]);
        assert!(result.is_err());
    }

    // ── field independence ────────────────────────────────────────────────────

    #[test]
    fn large_mass_and_range_do_not_trigger_jidoka() {
        // Only damage is gated; mass and range have no limit
        let result = gen("heavy", [100.0, 1.0, 100.0]);
        assert!(result.is_ok());
        let w = result.unwrap();
        assert!((w.mass - 1000.0).abs() < 0.01);
        assert!((w.range - 2000.0).abs() < 0.01);
    }

    #[test]
    fn negative_state_vector_values_produce_negative_properties() {
        let w = gen("anti", [-1.0, -1.0, -1.0]).unwrap();
        assert!(w.mass < 0.0);
        assert!(w.damage < 0.0);
        assert!(w.range < 0.0);
    }
}
