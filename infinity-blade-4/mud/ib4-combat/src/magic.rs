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
