use ib4_core::types::{MagicType, StatusEffect};

pub struct MagicResult {
    pub damage: f32,
    pub effect: Option<StatusEffect>,
    pub effect_turns: u32,
    pub burn_dpt: f32,
    pub is_heal: bool,
    pub heal_amount: f32,
}

pub fn resolve_magic(
    magic: MagicType,
    magic_stat: u32,
    magic_bonus_mult: f32,
    mana_cost_mult: f32,
) -> (MagicResult, f32) {
    let magic_bonus = magic_stat as f32 * 10.0 * magic_bonus_mult;

    match magic {
        MagicType::Fire => {
            let damage = 30.0 + magic_bonus;
            let dpt = 5.0 + magic_bonus * 0.1;
            let mana_cost = 20.0 * mana_cost_mult;
            (
                MagicResult {
                    damage,
                    effect: Some(StatusEffect::Burn),
                    effect_turns: 3,
                    burn_dpt: dpt,
                    is_heal: false,
                    heal_amount: 0.0,
                },
                mana_cost,
            )
        }
        MagicType::Lightning => {
            let damage = 50.0 + magic_bonus;
            let mana_cost = 30.0 * mana_cost_mult;
            (
                MagicResult {
                    damage,
                    effect: Some(StatusEffect::Stun),
                    effect_turns: 1,
                    burn_dpt: 0.0,
                    is_heal: false,
                    heal_amount: 0.0,
                },
                mana_cost,
            )
        }
        MagicType::Ice => {
            let damage = 35.0 + magic_bonus;
            let mana_cost = 25.0 * mana_cost_mult;
            (
                MagicResult {
                    damage,
                    effect: Some(StatusEffect::Freeze),
                    effect_turns: 2,
                    burn_dpt: 0.0,
                    is_heal: false,
                    heal_amount: 0.0,
                },
                mana_cost,
            )
        }
        MagicType::Dark => {
            let damage = 60.0 + magic_bonus;
            let mana_cost = 35.0 * mana_cost_mult;
            (
                MagicResult {
                    damage,
                    effect: Some(StatusEffect::Dark),
                    effect_turns: 2,
                    burn_dpt: 0.0,
                    is_heal: false,
                    heal_amount: 0.0,
                },
                mana_cost,
            )
        }
        MagicType::Light => {
            let heal = 40.0 + magic_bonus;
            let mana_cost = 25.0 * mana_cost_mult;
            (
                MagicResult {
                    damage: 0.0,
                    effect: Some(StatusEffect::Healed),
                    effect_turns: 0,
                    burn_dpt: 0.0,
                    is_heal: true,
                    heal_amount: heal,
                },
                mana_cost,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ib4_core::types::{MagicType, StatusEffect};

    // ── Fire ─────────────────────────────────────────────────────────────────

    #[test]
    fn fire_base_damage_at_zero_magic_stat() {
        let (r, cost) = resolve_magic(MagicType::Fire, 0, 1.0, 1.0);
        assert_eq!(r.damage, 30.0);
        assert_eq!(r.effect, Some(StatusEffect::Burn));
        assert_eq!(r.effect_turns, 3);
        assert!(!r.is_heal);
        assert_eq!(cost, 20.0);
    }

    #[test]
    fn fire_scales_with_magic_stat() {
        let (r, _) = resolve_magic(MagicType::Fire, 5, 1.0, 1.0);
        // base 30 + 5 * 10 * 1.0 = 80
        assert_eq!(r.damage, 80.0);
    }

    #[test]
    fn fire_burn_dpt_scales_with_magic_bonus() {
        let (r, _) = resolve_magic(MagicType::Fire, 10, 1.0, 1.0);
        // burn_dpt = 5 + (10 * 10 * 1.0) * 0.1 = 5 + 10 = 15
        assert_eq!(r.burn_dpt, 15.0);
    }

    // ── Lightning ─────────────────────────────────────────────────────────────

    #[test]
    fn lightning_stuns_for_one_turn() {
        let (r, cost) = resolve_magic(MagicType::Lightning, 0, 1.0, 1.0);
        assert_eq!(r.damage, 50.0);
        assert_eq!(r.effect, Some(StatusEffect::Stun));
        assert_eq!(r.effect_turns, 1);
        assert_eq!(cost, 30.0);
    }

    // ── Ice ───────────────────────────────────────────────────────────────────

    #[test]
    fn ice_freezes_for_two_turns() {
        let (r, cost) = resolve_magic(MagicType::Ice, 0, 1.0, 1.0);
        assert_eq!(r.damage, 35.0);
        assert_eq!(r.effect, Some(StatusEffect::Freeze));
        assert_eq!(r.effect_turns, 2);
        assert_eq!(cost, 25.0);
    }

    // ── Dark ─────────────────────────────────────────────────────────────────

    #[test]
    fn dark_has_highest_base_damage() {
        let (fire, _)  = resolve_magic(MagicType::Fire,      0, 1.0, 1.0);
        let (light, _) = resolve_magic(MagicType::Lightning, 0, 1.0, 1.0);
        let (ice, _)   = resolve_magic(MagicType::Ice,       0, 1.0, 1.0);
        let (dark, _)  = resolve_magic(MagicType::Dark,      0, 1.0, 1.0);
        assert!(dark.damage > fire.damage);
        assert!(dark.damage > light.damage);
        assert!(dark.damage > ice.damage);
    }

    // ── Light ─────────────────────────────────────────────────────────────────

    #[test]
    fn light_is_heal_not_damage() {
        let (r, cost) = resolve_magic(MagicType::Light, 0, 1.0, 1.0);
        assert!(r.is_heal);
        assert_eq!(r.damage, 0.0);
        assert_eq!(r.heal_amount, 40.0);
        assert_eq!(r.effect, Some(StatusEffect::Healed));
        assert_eq!(r.effect_turns, 0);
        assert_eq!(cost, 25.0);
    }

    #[test]
    fn light_heal_scales_with_magic_stat() {
        let (r, _) = resolve_magic(MagicType::Light, 3, 1.0, 1.0);
        // 40 + 3 * 10 * 1.0 = 70
        assert_eq!(r.heal_amount, 70.0);
    }

    // ── Multipliers ───────────────────────────────────────────────────────────

    #[test]
    fn magic_bonus_mult_scales_damage() {
        let (r1, _) = resolve_magic(MagicType::Fire, 5, 1.0, 1.0);
        let (r2, _) = resolve_magic(MagicType::Fire, 5, 2.0, 1.0);
        assert!(r2.damage > r1.damage, "2× magic_bonus_mult must increase damage");
    }

    #[test]
    fn mana_cost_mult_scales_cost() {
        let (_, cost1) = resolve_magic(MagicType::Lightning, 0, 1.0, 1.0);
        let (_, cost2) = resolve_magic(MagicType::Lightning, 0, 1.0, 2.0);
        assert!((cost2 / cost1 - 2.0).abs() < 0.01, "2× mana_cost_mult must double cost");
    }
}
