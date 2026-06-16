use ib4_core::player::PlayerState;

#[derive(Debug, Clone)]
pub struct PerkDef {
    pub id: &'static str,
    pub name: &'static str,
    pub tier: u8,
    pub prerequisite: Option<&'static str>,
    pub description: &'static str,
    // Aggregate effect fields
    pub attack_bonus: f32,          // additive percentage (0.10 = +10%)
    pub defense_bonus: f32,
    pub magic_bonus: f32,
    pub health_bonus: f32,
    pub xp_gain: f32,
    pub gold_find: f32,
    pub crit_chance: f32,
    pub magic_cost_reduction: f32,
    pub combo_window_bonus: u32,    // extra turns before combo reset
    pub grants_parry_bonus: bool,   // QIPResonance
}

/// Computed aggregate from all selected perks.
#[derive(Debug, Clone, Default)]
pub struct PerkAggregate {
    pub attack_mult: f32,       // 1.0 + sum of attack_bonus
    pub defense_mult: f32,
    pub magic_mult: f32,
    pub health_mult: f32,
    pub xp_mult: f32,
    pub gold_mult: f32,
    pub crit_bonus: f32,
    pub magic_cost_mult: f32,   // 1.0 - sum of magic_cost_reduction
    pub combo_extra_turns: u32,
    pub has_parry_bonus: bool,
}

pub struct PerkTree {
    perks: Vec<PerkDef>,
}

impl PerkTree {
    pub fn new() -> Self {
        Self {
            perks: vec![
                // ── Tier 1 (no prerequisites) ──────────────────────────────────────
                PerkDef {
                    id: "BloodyResolve",
                    name: "Bloody Resolve",
                    tier: 1,
                    prerequisite: None,
                    description: "Centuries of bloodshed have sharpened your lineage. Gain +10% attack damage.",
                    attack_bonus: 0.10,
                    defense_bonus: 0.0,
                    magic_bonus: 0.0,
                    health_bonus: 0.0,
                    xp_gain: 0.0,
                    gold_find: 0.0,
                    crit_chance: 0.0,
                    magic_cost_reduction: 0.0,
                    combo_window_bonus: 0,
                    grants_parry_bonus: false,
                },
                PerkDef {
                    id: "IronHide",
                    name: "Iron Hide",
                    tier: 1,
                    prerequisite: None,
                    description: "Your bloodline has endured countless blows. Gain +10% damage reduction.",
                    attack_bonus: 0.0,
                    defense_bonus: 0.10,
                    magic_bonus: 0.0,
                    health_bonus: 0.0,
                    xp_gain: 0.0,
                    gold_find: 0.0,
                    crit_chance: 0.0,
                    magic_cost_reduction: 0.0,
                    combo_window_bonus: 0,
                    grants_parry_bonus: false,
                },
                PerkDef {
                    id: "SwiftStrikes",
                    name: "Swift Strikes",
                    tier: 1,
                    prerequisite: None,
                    description: "Ancestral reflexes extend the combo input window by 0.1 seconds.",
                    attack_bonus: 0.0,
                    defense_bonus: 0.0,
                    magic_bonus: 0.0,
                    health_bonus: 0.0,
                    xp_gain: 0.0,
                    gold_find: 0.0,
                    crit_chance: 0.0,
                    magic_cost_reduction: 0.0,
                    combo_window_bonus: 1,
                    grants_parry_bonus: false,
                },
                PerkDef {
                    id: "MagicSensitivity",
                    name: "Magic Sensitivity",
                    tier: 1,
                    prerequisite: None,
                    description: "An awakened bloodline resonates with the QIP. Gain +15% magic damage.",
                    attack_bonus: 0.0,
                    defense_bonus: 0.0,
                    magic_bonus: 0.15,
                    health_bonus: 0.0,
                    xp_gain: 0.0,
                    gold_find: 0.0,
                    crit_chance: 0.0,
                    magic_cost_reduction: 0.0,
                    combo_window_bonus: 0,
                    grants_parry_bonus: false,
                },
                PerkDef {
                    id: "Scavenger",
                    name: "Scavenger",
                    tier: 1,
                    prerequisite: None,
                    description: "A survivor's instinct — your lineage knows where enemies hide their gold. +20% gold found.",
                    attack_bonus: 0.0,
                    defense_bonus: 0.0,
                    magic_bonus: 0.0,
                    health_bonus: 0.0,
                    xp_gain: 0.0,
                    gold_find: 0.20,
                    crit_chance: 0.0,
                    magic_cost_reduction: 0.0,
                    combo_window_bonus: 0,
                    grants_parry_bonus: false,
                },

                // ── Tier 2 (require BL 5+) ─────────────────────────────────────────
                PerkDef {
                    id: "DeadlyPrecision",
                    name: "Deadly Precision",
                    tier: 2,
                    prerequisite: Some("BloodyResolve"),
                    description: "Your attacks find chinks in every defence. Gain +5% critical hit chance.",
                    attack_bonus: 0.0,
                    defense_bonus: 0.0,
                    magic_bonus: 0.0,
                    health_bonus: 0.0,
                    xp_gain: 0.0,
                    gold_find: 0.0,
                    crit_chance: 0.05,
                    magic_cost_reduction: 0.0,
                    combo_window_bonus: 0,
                    grants_parry_bonus: false,
                },
                PerkDef {
                    id: "FortressStance",
                    name: "Fortress Stance",
                    tier: 2,
                    prerequisite: Some("IronHide"),
                    description: "A mountain cannot be moved. Your maximum health is increased by 15%.",
                    attack_bonus: 0.0,
                    defense_bonus: 0.0,
                    magic_bonus: 0.0,
                    health_bonus: 0.15,
                    xp_gain: 0.0,
                    gold_find: 0.0,
                    crit_chance: 0.0,
                    magic_cost_reduction: 0.0,
                    combo_window_bonus: 0,
                    grants_parry_bonus: false,
                },
                PerkDef {
                    id: "ComboMaster",
                    name: "Combo Master",
                    tier: 2,
                    prerequisite: Some("SwiftStrikes"),
                    description: "Your bloodline has mastered the flow of battle. Extend the combo window by 0.15 s and gain +5% attack damage.",
                    attack_bonus: 0.05,
                    defense_bonus: 0.0,
                    magic_bonus: 0.0,
                    health_bonus: 0.0,
                    xp_gain: 0.0,
                    gold_find: 0.0,
                    crit_chance: 0.0,
                    magic_cost_reduction: 0.0,
                    combo_window_bonus: 1,
                    grants_parry_bonus: false,
                },
                PerkDef {
                    id: "ArcaneChanneling",
                    name: "Arcane Channeling",
                    tier: 2,
                    prerequisite: Some("MagicSensitivity"),
                    description: "Bloodline resonance reduces the QIP energy required to cast spells by 20%.",
                    attack_bonus: 0.0,
                    defense_bonus: 0.0,
                    magic_bonus: 0.0,
                    health_bonus: 0.0,
                    xp_gain: 0.0,
                    gold_find: 0.0,
                    crit_chance: 0.0,
                    magic_cost_reduction: 0.20,
                    combo_window_bonus: 0,
                    grants_parry_bonus: false,
                },
                PerkDef {
                    id: "TreasureHunter",
                    name: "Treasure Hunter",
                    tier: 2,
                    prerequisite: Some("Scavenger"),
                    description: "Centuries of looting have honed your bloodline's nose for wealth. +30% gold found.",
                    attack_bonus: 0.0,
                    defense_bonus: 0.0,
                    magic_bonus: 0.0,
                    health_bonus: 0.0,
                    xp_gain: 0.0,
                    gold_find: 0.30,
                    crit_chance: 0.0,
                    magic_cost_reduction: 0.0,
                    combo_window_bonus: 0,
                    grants_parry_bonus: false,
                },

                // ── Tier 3 (require BL 10+) ────────────────────────────────────────
                PerkDef {
                    id: "AusarLegacy",
                    name: "Ausar's Legacy",
                    tier: 3,
                    prerequisite: Some("DeadlyPrecision"),
                    description: "Channel the fury of the Deathless King himself. +25% attack damage and +10% critical hit chance.",
                    attack_bonus: 0.25,
                    defense_bonus: 0.0,
                    magic_bonus: 0.0,
                    health_bonus: 0.0,
                    xp_gain: 0.0,
                    gold_find: 0.0,
                    crit_chance: 0.10,
                    magic_cost_reduction: 0.0,
                    combo_window_bonus: 0,
                    grants_parry_bonus: false,
                },
                PerkDef {
                    id: "DeathlessResilience",
                    name: "Deathless Resilience",
                    tier: 3,
                    prerequisite: Some("FortressStance"),
                    description: "Your bloodline defies death itself. +30% maximum health and +15% damage reduction.",
                    attack_bonus: 0.0,
                    defense_bonus: 0.15,
                    magic_bonus: 0.0,
                    health_bonus: 0.30,
                    xp_gain: 0.0,
                    gold_find: 0.0,
                    crit_chance: 0.0,
                    magic_cost_reduction: 0.0,
                    combo_window_bonus: 0,
                    grants_parry_bonus: false,
                },
                PerkDef {
                    id: "QIPResonance",
                    name: "QIP Resonance",
                    tier: 3,
                    prerequisite: Some("ComboMaster"),
                    description: "Perfect harmony with the QIP crystal slows your perception of time. Parry +0.05 s and combo window +0.20 s.",
                    attack_bonus: 0.0,
                    defense_bonus: 0.0,
                    magic_bonus: 0.0,
                    health_bonus: 0.0,
                    xp_gain: 0.0,
                    gold_find: 0.0,
                    crit_chance: 0.0,
                    magic_cost_reduction: 0.0,
                    combo_window_bonus: 1,
                    grants_parry_bonus: true,
                },
                PerkDef {
                    id: "WorkerOfSecretsGift",
                    name: "Worker of Secrets' Gift",
                    tier: 3,
                    prerequisite: Some("ArcaneChanneling"),
                    description: "The Worker's ancient knowledge flows through your bloodline. +40% magic damage and -30% magic energy cost.",
                    attack_bonus: 0.0,
                    defense_bonus: 0.0,
                    magic_bonus: 0.40,
                    health_bonus: 0.0,
                    xp_gain: 0.0,
                    gold_find: 0.0,
                    crit_chance: 0.0,
                    magic_cost_reduction: 0.30,
                    combo_window_bonus: 0,
                    grants_parry_bonus: false,
                },
                PerkDef {
                    id: "InfinitySeeker",
                    name: "Infinity Seeker",
                    tier: 3,
                    prerequisite: Some("TreasureHunter"),
                    description: "Your bloodline has transcended the cycle. Earn 50% more experience from all sources.",
                    attack_bonus: 0.0,
                    defense_bonus: 0.0,
                    magic_bonus: 0.0,
                    health_bonus: 0.0,
                    xp_gain: 0.50,
                    gold_find: 0.0,
                    crit_chance: 0.0,
                    magic_cost_reduction: 0.0,
                    combo_window_bonus: 0,
                    grants_parry_bonus: false,
                },
            ],
        }
    }

    pub fn all_perks(&self) -> &[PerkDef] {
        &self.perks
    }

    pub fn get_perk(&self, id: &str) -> Option<&PerkDef> {
        self.perks.iter().find(|p| p.id == id)
    }

    /// Select a perk for the player.
    ///
    /// Returns `Err` if:
    /// - the perk id is unknown,
    /// - the player has no perk points,
    /// - the perk is already selected,
    /// - the prerequisite perk has not been selected,
    /// - the perk's tier bloodline requirement is not met (T2: BL≥5, T3: BL≥10).
    pub fn select_perk(&self, player: &mut PlayerState, perk_id: &str) -> Result<String, String> {
        let perk = self
            .get_perk(perk_id)
            .ok_or_else(|| format!("Unknown perk: {}", perk_id))?;

        if player.perk_points == 0 {
            return Err("No perk points available.".to_string());
        }

        if player.selected_perks.contains(&perk_id.to_string()) {
            return Err(format!("{} already selected.", perk.name));
        }

        if let Some(prereq) = perk.prerequisite {
            if !player.selected_perks.contains(&prereq.to_string()) {
                return Err(format!("Requires {} first.", prereq));
            }
        }

        let tier_bloodline: i32 = match perk.tier {
            2 => 5,
            3 => 10,
            _ => 0,
        };
        if player.bloodline < tier_bloodline {
            return Err(format!(
                "Tier {} perks require Bloodline {}+.",
                perk.tier, tier_bloodline
            ));
        }

        player.selected_perks.push(perk_id.to_string());
        player.perk_points -= 1;
        Ok(format!("'{}' acquired: {}", perk.name, perk.description))
    }

    /// Compute aggregate effects from all selected perks.
    pub fn compute_aggregate(&self, selected: &[String]) -> PerkAggregate {
        let mut agg = PerkAggregate {
            attack_mult: 1.0,
            defense_mult: 1.0,
            magic_mult: 1.0,
            health_mult: 1.0,
            xp_mult: 1.0,
            gold_mult: 1.0,
            crit_bonus: 0.0,
            magic_cost_mult: 1.0,
            combo_extra_turns: 0,
            has_parry_bonus: false,
        };

        for id in selected {
            if let Some(p) = self.get_perk(id) {
                agg.attack_mult += p.attack_bonus;
                agg.defense_mult += p.defense_bonus;
                agg.magic_mult += p.magic_bonus;
                agg.health_mult += p.health_bonus;
                agg.xp_mult += p.xp_gain;
                agg.gold_mult += p.gold_find;
                agg.crit_bonus += p.crit_chance;
                agg.magic_cost_mult -= p.magic_cost_reduction;
                agg.combo_extra_turns += p.combo_window_bonus;
                if p.grants_parry_bonus {
                    agg.has_parry_bonus = true;
                }
            }
        }

        // Never drop magic cost below 10%
        agg.magic_cost_mult = agg.magic_cost_mult.max(0.1);
        agg
    }
}

impl Default for PerkTree {
    fn default() -> Self {
        Self::new()
    }
}
