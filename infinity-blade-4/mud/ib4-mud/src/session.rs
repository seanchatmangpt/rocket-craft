use std::collections::VecDeque;
use anyhow::Result;
use rand::{SeedableRng, Rng};
use rand::rngs::SmallRng;
use serde::{Serialize, Deserialize};
use ib4_core::{
    player::PlayerState,
    enemy::EnemyInstance,
    types::{AttackDir, MagicType, Rarity},
    equipment::weapon_catalog,
};
use crate::command::Command;
use crate::narrative;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SaveData {
    pub player: PlayerState,
    pub bloodline: i32,
    pub arena_queue: Vec<String>,
}

pub struct GameSession {
    pub player: PlayerState,
    pub current_enemy: Option<EnemyInstance>,
    pub arena_queue: VecDeque<String>,
    pub combo_depth: u32,
    pub combo_idle_turns: u32,
    pub combo_reset_threshold: u32,
    pub announced_attack: Option<AttackDir>,
    pub rng: SmallRng,
    pub turn: u32,
    pub loot_bag: Vec<ib4_core::equipment::Weapon>,
    pub is_in_combat: bool,
    pub god_king_ai_turn: u32,
}

impl GameSession {
    pub fn new(name: &str) -> Self {
        let player = PlayerState::new(name);
        let rng = SmallRng::from_entropy();

        // Build default arena queue for bloodline 0
        let queue: VecDeque<String> = vec![
            "LightTitan".to_string(),
            "HeavyTitan".to_string(),
            "DarkKnight".to_string(),
            "CorruptedGalath".to_string(),
        ]
        .into();

        Self {
            player,
            current_enemy: None,
            arena_queue: queue,
            combo_depth: 0,
            combo_idle_turns: 0,
            combo_reset_threshold: 2,
            announced_attack: None,
            rng,
            turn: 0,
            loot_bag: Vec::new(),
            is_in_combat: false,
            god_king_ai_turn: 0,
        }
    }

    pub fn from_json(json: &str) -> Result<Self> {
        let save: SaveData = serde_json::from_str(json)?;
        let mut session = Self::new(&save.player.name);
        session.player = save.player;
        session.arena_queue = save.arena_queue.into();
        Ok(session)
    }

    pub fn to_json(&self) -> String {
        let save = SaveData {
            player: self.player.clone(),
            bloodline: self.player.bloodline,
            arena_queue: self.arena_queue.iter().cloned().collect(),
        };
        serde_json::to_string_pretty(&save).unwrap_or_default()
    }

    /// Dispatch a parsed command. Returns narrative lines to print.
    pub fn dispatch(&mut self, cmd: Command) -> Vec<String> {
        match cmd {
            Command::Look | Command::Explore => self.cmd_look(),
            Command::Attack(dir) => self.cmd_attack(dir),
            Command::Parry => self.cmd_parry(None),
            Command::PerfectParry(dir) => self.cmd_parry(Some(dir)),
            Command::Dodge => self.cmd_dodge(),
            Command::Magic(magic) => self.cmd_magic(magic),
            Command::Status => self.cmd_status(),
            Command::Inventory => self.cmd_inventory(),
            Command::AllocStat(stat) => self.cmd_alloc_stat(&stat),
            Command::Perks => self.cmd_perks(),
            Command::SelectPerk(id) => self.cmd_select_perk(&id),
            Command::Shop => self.cmd_shop(),
            Command::Buy(id) => self.cmd_buy(&id),
            Command::Sell(slot) => self.cmd_sell(&slot),
            Command::Equip(id) => self.cmd_equip(&id),
            Command::Save => {
                let json = self.to_json();
                if let Err(e) = std::fs::write("ib4_save.json", &json) {
                    return vec![format!("[Error] Could not save: {}", e)];
                }
                vec!["Game saved to ib4_save.json.".to_string()]
            }
            Command::Help => vec![narrative::help_text()],
            Command::Quit => {
                vec!["Farewell, Siris. The bloodline remembers.".to_string()]
            }
        }
    }

    fn cmd_look(&mut self) -> Vec<String> {
        let mut lines = Vec::new();
        if let Some(enemy) = &self.current_enemy {
            lines.push(format!("You face: {} [{}]", enemy.name, enemy.phase_label()));
            lines.push(narrative::fmt_enemy_hp(
                &enemy.name,
                enemy.current_hp,
                enemy.base_hp,
                enemy.phase,
            ));
            if let Some(dir) = &self.announced_attack {
                lines.push(narrative::fmt_attack_announce(
                    &enemy.name.clone(),
                    dir,
                    enemy.phase,
                ));
            }
        } else if self.arena_queue.is_empty() {
            lines.push("The arena is empty. Victory!".to_string());
        } else {
            let next = self
                .arena_queue
                .front()
                .map(|s| s.as_str())
                .unwrap_or("unknown");
            lines.push(format!("A {} approaches from the shadows...", next));
            lines.push("Type 'explore' to engage.".to_string());
        }
        lines.push(format!(
            "Siris \u{2014} HP: {:.0}/{:.0} | Mana: {:.0}/{:.0} | Combo: {}x | BL: {}",
            self.player.health,
            self.player.max_health,
            self.player.mana,
            self.player.max_mana,
            self.combo_depth.max(1),
            self.player.bloodline_label()
        ));
        lines
    }

    fn spawn_next_enemy(&mut self) -> Vec<String> {
        let id = match self.arena_queue.pop_front() {
            Some(id) => id,
            None => {
                return vec!["No more enemies. You have conquered this bloodline!".to_string()]
            }
        };

        // TODO: call ib4-ai::roster::spawn_enemy — using inline spawning for now
        let hp_scale = 1.0 + self.player.bloodline.max(0) as f32 * 0.15;
        let enemy = self.make_enemy_inline(&id, hp_scale);
        let name = enemy.name.clone();
        let is_god_king = id == "CorruptedGalath";
        self.current_enemy = Some(enemy);
        self.is_in_combat = true;
        self.announced_attack = None;
        self.god_king_ai_turn = 0;

        let mut lines = vec![format!("A {} emerges!", name)];
        if is_god_king {
            lines.push("Galath's eyes blaze silver. The air freezes.".to_string());
            if let Some(e) = self.current_enemy.as_mut() {
                e.shield_active = true;
            }
            lines.push(
                "[Phase I] Galath raises a hard-light shield. Only PERFECT PARRIES can break it!"
                    .to_string(),
            );
        }

        // Enemy announces first attack
        let dir = self.random_dir();
        if let Some(e) = self.current_enemy.as_ref() {
            lines.push(narrative::fmt_attack_announce(&e.name.clone(), &dir, e.phase));
        }
        self.announced_attack = Some(dir);
        lines
    }

    fn cmd_attack(&mut self, _dir: AttackDir) -> Vec<String> {
        if !self.is_in_combat {
            return self.spawn_next_enemy();
        }
        let enemy = match self.current_enemy.as_ref() {
            Some(e) => e.clone(),
            None => return self.spawn_next_enemy(),
        };

        let mut lines = Vec::new();

        // Player takes damage from pending enemy attack if not parrying/dodging
        if let Some(announced) = self.announced_attack.take() {
            // Player chose to attack instead of parry — take full enemy damage
            let shield_def = self
                .player
                .shield
                .as_ref()
                .map(|s| s.defense_bonus)
                .unwrap_or(0) as f32;
            let defense = self.player.stat_defense as f32 + shield_def;
            let reduction = (defense / (defense + 50.0)).min(0.5);
            let dmg = enemy.attack_damage * (1.0 - reduction);
            self.player.take_damage(dmg);
            lines.push(format!(
                "  \u{1f4a2} {} strikes {} \u{2014} you take {:.0} damage!",
                enemy.name, announced, dmg
            ));
        }

        if !self.player.is_alive() {
            return self.handle_player_death(lines);
        }

        // Deal damage
        // TODO: call ib4-combat::damage::calc_player_damage
        let weapon_atk = self
            .player
            .weapon
            .as_ref()
            .map(|w| w.attack_bonus)
            .unwrap_or(0) as f32;
        let base_dmg = weapon_atk + self.player.stat_attack as f32;
        let multiplier = combo_multiplier(self.combo_depth + 1);
        let crit_chance = self
            .player
            .weapon
            .as_ref()
            .map(|w| w.crit_chance)
            .unwrap_or(0.05);
        let rng_val: f32 = self.rng.gen();
        let is_crit = rng_val < crit_chance;
        let dmg = base_dmg * multiplier * if is_crit { 1.5 } else { 1.0 };

        // GodKing shield check
        let shield_blocked = self
            .current_enemy
            .as_ref()
            .map(|e| e.shield_active)
            .unwrap_or(false);
        if shield_blocked {
            lines.push("  \u{1f6e1}  Your blade sparks off Galath's hard-light shield. No damage!".to_string());
            self.combo_depth = 0;
            self.combo_idle_turns = 0;
            let next_dir = self.random_dir();
            let (ename, ephase) = self
                .current_enemy
                .as_ref()
                .map(|e| (e.name.clone(), e.phase))
                .unwrap_or_default();
            lines.push(narrative::fmt_attack_announce(&ename, &next_dir, ephase));
            self.announced_attack = Some(next_dir);
            return lines;
        }
        if let Some(enemy_mut) = self.current_enemy.as_mut() {
            enemy_mut.take_damage(dmg);
        }

        self.combo_depth += 1;
        self.combo_idle_turns = 0;
        lines.push(narrative::fmt_damage_dealt(dmg, is_crit, self.combo_depth));

        if let Some(enemy_ref) = self.current_enemy.as_ref() {
            lines.push(narrative::fmt_enemy_hp(
                &enemy_ref.name.clone(),
                enemy_ref.current_hp,
                enemy_ref.base_hp,
                enemy_ref.phase,
            ));
        }

        // Check enemy death
        if self
            .current_enemy
            .as_ref()
            .map(|e| !e.is_alive())
            .unwrap_or(false)
        {
            return self.handle_enemy_defeat(lines);
        }

        // Check phase transition
        // TODO: call ib4-combat::titan::TitanAI::check_phase_transition
        if let Some(enemy_mut) = self.current_enemy.as_mut() {
            let hp_pct = enemy_mut.hp_percent();
            let new_phase = if hp_pct <= 30.0 {
                3
            } else if hp_pct <= 60.0 {
                2
            } else {
                1
            };
            if new_phase > enemy_mut.phase {
                enemy_mut.phase = new_phase;
                enemy_mut.attack_damage = match new_phase {
                    2 => enemy_mut.base_attack_damage * 1.25,
                    3 => enemy_mut.base_attack_damage * 1.875,
                    _ => enemy_mut.base_attack_damage,
                };
                lines.push(format!(
                    "  \u{26a1} {} enters {}!",
                    enemy_mut.name,
                    enemy_mut.phase_label()
                ));
            }
        }

        // Enemy announces next attack
        self.announce_next_attack(&mut lines);
        lines
    }

    fn cmd_parry(&mut self, dir: Option<AttackDir>) -> Vec<String> {
        if !self.is_in_combat {
            return vec!["No enemy to parry.".to_string()];
        }

        let announced = match self.announced_attack.take() {
            Some(d) => d,
            None => return vec!["No incoming attack to parry!".to_string()],
        };

        let enemy = match self.current_enemy.as_ref() {
            Some(e) => e.clone(),
            None => return vec!["No enemy present.".to_string()],
        };

        // TODO: call ib4-combat::parry::ParryResolver::resolve
        let perfect = match &dir {
            Some(d) => d == &announced,
            None => false,
        };

        let mut lines = vec![narrative::fmt_parry_result(perfect, &announced)];

        // GodKing shield: perfect parries break it
        if enemy.shield_active && perfect {
            if let Some(e) = self.current_enemy.as_mut() {
                // TODO: call ib4-ai::godking::GodKingAI::register_perfect_parry
                e.perfect_parries_received += 1;
                lines.push(format!(
                    "  \u{2728} Perfect Parry on Galath's shield! ({}/3)",
                    e.perfect_parries_received
                ));
                if e.perfect_parries_received >= 3 {
                    e.shield_active = false;
                    e.phase = 2;
                    e.attack_damage = e.base_attack_damage * 1.25;
                    lines.push(
                        "  \u{1f4a5} GALATH'S SHIELD SHATTERS! He draws dual blades \u{2014} Phase II!"
                            .to_string(),
                    );
                }
            }
        }

        // QIP Scar (GodKing Phase 2 on normal parry)
        if !enemy.shield_active && enemy.id == "CorruptedGalath" && !perfect {
            // TODO: call ib4-ai::godking::GodKingAI::apply_qip_scar
            self.player.qip_scar_stacks += 1;
            lines.push(format!(
                "  \u{2620}  QIP Scar: {}/3 stacks on your soul.",
                self.player.qip_scar_stacks
            ));
            if self.player.qip_scar_stacks >= 3 {
                lines.push("  \u{1f480} THREE QIP SCARS \u{2014} Forced Rebirth!".to_string());
                lines.extend(self.handle_forced_rebirth());
                return lines;
            }
        }

        // Combo reset on parry (unless has ComboMaster)
        let has_combo_master = self.player.has_perk("ComboMaster");
        if !has_combo_master {
            self.combo_depth = 0;
        }

        // Perfect parry: enemy stunned 1 turn
        if perfect {
            if let Some(e) = self.current_enemy.as_mut() {
                e.apply_stun(1);
            }
            lines.push("  The enemy staggers \u{2014} stunned for 1 turn!".to_string());
        }

        self.announce_next_attack(&mut lines);
        lines
    }

    fn cmd_dodge(&mut self) -> Vec<String> {
        if !self.is_in_combat {
            return vec!["Nothing to dodge.".to_string()];
        }
        self.announced_attack = None;
        self.combo_depth = 0;
        let mut lines =
            vec!["  You roll clear \u{2014} dodge successful! Combo lost.".to_string()];
        self.announce_next_attack(&mut lines);
        lines
    }

    fn cmd_magic(&mut self, magic: MagicType) -> Vec<String> {
        if !self.player.has_magic(&magic) {
            return vec![format!("  {} magic not yet unlocked.", magic)];
        }

        // TODO: call ib4-combat::magic::resolve_magic for mana cost and damage
        let mana_cost = match magic {
            MagicType::Fire => 20.0,
            MagicType::Lightning => 30.0,
            MagicType::Ice => 25.0,
            MagicType::Dark => 35.0,
            MagicType::Light => 25.0,
        };

        if !self.player.spend_mana(mana_cost) {
            return vec![format!(
                "  Not enough mana. Need {:.0}, have {:.0}.",
                mana_cost, self.player.mana
            )];
        }

        let magic_bonus = self.player.stat_magic as f32 * 10.0;
        let (damage, heal, effect_text) = match magic {
            MagicType::Fire => (30.0 + magic_bonus, 0.0, "BURNING (3 turns)"),
            MagicType::Lightning => (50.0 + magic_bonus, 0.0, "STUNNED (1 turn)"),
            MagicType::Ice => (35.0 + magic_bonus, 0.0, "FROZEN (2 turns)"),
            MagicType::Dark => (60.0 + magic_bonus * 0.5, 0.0, "DARK CURSED"),
            MagicType::Light => (0.0, 40.0 + magic_bonus, ""),
        };

        let mut lines = vec![narrative::fmt_magic_use(&magic, damage, heal)];

        if magic == MagicType::Light {
            self.player.heal(heal);
            lines.push(format!(
                "  HP restored to {:.0}/{:.0}.",
                self.player.health, self.player.max_health
            ));
        } else {
            let shield_active = self.current_enemy.as_ref().map(|e| e.shield_active).unwrap_or(false);
            if shield_active {
                lines.push(
                    "  \u{26a1} Magic sparks off the hard-light shield!".to_string(),
                );
            } else if let Some(e) = self.current_enemy.as_mut() {
                e.take_damage(damage);
                lines.push(format!("  {} is {}!", e.name, effect_text));
                lines.push(narrative::fmt_enemy_hp(
                    &e.name.clone(),
                    e.current_hp,
                    e.base_hp,
                    e.phase,
                ));
            }
            // check if dead — separate borrow
            if self.current_enemy.as_ref().map(|e| !e.is_alive()).unwrap_or(false) {
                self.combo_depth = 0;
                return self.handle_enemy_defeat(lines);
            }
        }

        self.combo_depth = 0;
        self.announce_next_attack(&mut lines);
        lines
    }

    fn cmd_status(&self) -> Vec<String> {
        let mut lines = vec![
            "\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}".to_string(),
            format!(
                "  {} \u{2014} Level {} | Bloodline {}",
                self.player.name, self.player.level, self.player.bloodline_label()
            ),
            format!(
                "  HP:    {}",
                narrative::fmt_hp_bar(self.player.health, self.player.max_health, 20)
            ),
            format!(
                "  Mana:  {:.0}/{:.0}",
                self.player.mana, self.player.max_mana
            ),
            format!("  Gold:  {} gp", self.player.gold),
            format!(
                "  XP:    {} (next level: {})",
                self.player.xp,
                xp_to_next(&self.player)
            ),
            "  \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}".to_string(),
            format!(
                "  ATK:{} DEF:{} MAG:{} HP_stat:{}",
                self.player.stat_attack,
                self.player.stat_defense,
                self.player.stat_magic,
                self.player.stat_health
            ),
            format!(
                "  Stat Points: {} | Perk Points: {}",
                self.player.stat_points, self.player.perk_points
            ),
            format!(
                "  Combo: {}x ({:.1}\u{d7}) | QIP Scars: {}/3",
                self.combo_depth,
                combo_multiplier(self.combo_depth),
                self.player.qip_scar_stacks
            ),
        ];
        if !self.player.selected_perks.is_empty() {
            lines.push(format!(
                "  Perks: {}",
                self.player.selected_perks.join(", ")
            ));
        }
        let magic: Vec<String> = self
            .player
            .magic_unlocks
            .iter()
            .map(|m| format!("{}", m))
            .collect();
        lines.push(format!(
            "  Magic: {}",
            if magic.is_empty() {
                "Fire".to_string()
            } else {
                magic.join(", ")
            }
        ));
        lines.push(
            "\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}".to_string(),
        );
        lines
    }

    fn cmd_inventory(&self) -> Vec<String> {
        let mut lines =
            vec!["\u{2500}\u{2500} Equipped \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}".to_string()];
        if let Some(w) = &self.player.weapon {
            lines.push(format!(
                "  Weapon: {} (+{} atk, {:?})",
                w.name, w.attack_bonus, w.rarity
            ));
        } else {
            lines.push("  Weapon: (none)".to_string());
        }
        if let Some(s) = &self.player.shield {
            lines.push(format!("  Shield: {} (+{} def)", s.name, s.defense_bonus));
        } else {
            lines.push("  Shield: (none)".to_string());
        }
        if !self.player.loot_bag.is_empty() {
            lines.push("\u{2500}\u{2500} Loot Bag \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}".to_string());
            for w in &self.player.loot_bag {
                lines.push(format!(
                    "  {} [{}] +{} atk | equip {}",
                    w.name, w.id, w.attack_bonus, w.id
                ));
            }
        }
        lines
    }

    fn cmd_alloc_stat(&mut self, stat: &str) -> Vec<String> {
        if self.player.stat_points == 0 {
            return vec!["No stat points available.".to_string()];
        }
        match stat.to_lowercase().as_str() {
            "health" | "hp" => {
                self.player.stat_health += 1;
            }
            "attack" | "atk" => {
                self.player.stat_attack += 1;
            }
            "defense" | "def" => {
                self.player.stat_defense += 1;
            }
            "magic" | "mag" => {
                self.player.stat_magic += 1;
            }
            _ => {
                return vec![format!(
                    "Unknown stat '{}'. Use: health, attack, defense, magic",
                    stat
                )]
            }
        }
        self.player.stat_points -= 1;
        self.player.recalculate_stats();
        self.player.health = self.player.max_health; // restore HP on stat alloc
        vec![format!(
            "  {} increased! ({} points remaining)",
            stat, self.player.stat_points
        )]
    }

    fn cmd_perks(&self) -> Vec<String> {
        let all_perks: &[(&str, u8, Option<&str>, &str)] = &[
            ("BloodyResolve", 1, None, "+10% ATK"),
            ("IronHide", 1, None, "+10% DEF"),
            ("SwiftStrikes", 1, None, "+1 combo turn"),
            ("MagicSensitivity", 1, None, "+15% MAG"),
            ("Scavenger", 1, None, "+20% Gold"),
            (
                "DeadlyPrecision",
                2,
                Some("BloodyResolve"),
                "+5% Crit",
            ),
            ("FortressStance", 2, Some("IronHide"), "+15% MaxHP"),
            (
                "ComboMaster",
                2,
                Some("SwiftStrikes"),
                "+1 combo + 5% ATK",
            ),
            (
                "ArcaneChanneling",
                2,
                Some("MagicSensitivity"),
                "-20% Mana cost",
            ),
            ("TreasureHunter", 2, Some("Scavenger"), "+30% Gold"),
            (
                "AusarLegacy",
                3,
                Some("DeadlyPrecision"),
                "+25% ATK, +10% Crit",
            ),
            (
                "DeathlessResilience",
                3,
                Some("FortressStance"),
                "+30% HP, +15% DEF",
            ),
            (
                "QIPResonance",
                3,
                Some("ComboMaster"),
                "Always PerfectParry + combo",
            ),
            (
                "WorkerOfSecretsGift",
                3,
                Some("ArcaneChanneling"),
                "+40% MAG, -30% Mana",
            ),
            ("InfinitySeeker", 3, Some("TreasureHunter"), "+50% XP"),
        ];

        let mut lines = vec![
            format!(
                "\u{2550}\u{2550}\u{2550} Perk Tree ({} points) \u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}",
                self.player.perk_points
            ),
            format!(
                "  Bloodline: {} (T2 needs BL5, T3 needs BL10)",
                self.player.bloodline_label()
            ),
            "  \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}".to_string(),
        ];

        for (id, tier, prereq, effect) in all_perks {
            let status = if self.player.selected_perks.contains(&id.to_string()) {
                "\u{2713}"
            } else if tier_locked(*tier, self.player.bloodline) {
                "\u{1f512}"
            } else {
                "\u{25cb}"
            };
            let prereq_str = prereq
                .map(|p| format!(" [req: {}]", p))
                .unwrap_or_default();
            lines.push(format!(
                "  {} T{} {} \u{2014} {}{}",
                status, tier, id, effect, prereq_str
            ));
        }
        lines
    }

    fn cmd_select_perk(&mut self, perk_id: &str) -> Vec<String> {
        if self.player.perk_points == 0 {
            return vec![
                "No perk points available. Die and rebirth to earn more.".to_string(),
            ];
        }
        if self.player.selected_perks.contains(&perk_id.to_string()) {
            return vec![format!("{} already selected.", perk_id)];
        }

        let prereqs: &[(&str, Option<&str>, u8)] = &[
            ("BloodyResolve", None, 1),
            ("IronHide", None, 1),
            ("SwiftStrikes", None, 1),
            ("MagicSensitivity", None, 1),
            ("Scavenger", None, 1),
            ("DeadlyPrecision", Some("BloodyResolve"), 2),
            ("FortressStance", Some("IronHide"), 2),
            ("ComboMaster", Some("SwiftStrikes"), 2),
            ("ArcaneChanneling", Some("MagicSensitivity"), 2),
            ("TreasureHunter", Some("Scavenger"), 2),
            ("AusarLegacy", Some("DeadlyPrecision"), 3),
            ("DeathlessResilience", Some("FortressStance"), 3),
            ("QIPResonance", Some("ComboMaster"), 3),
            ("WorkerOfSecretsGift", Some("ArcaneChanneling"), 3),
            ("InfinitySeeker", Some("TreasureHunter"), 3),
        ];

        let entry = prereqs.iter().find(|(id, _, _)| *id == perk_id);
        if entry.is_none() {
            return vec![format!(
                "Unknown perk: {}. Type 'perks' to see options.",
                perk_id
            )];
        }
        let (_, prereq, tier) = entry.unwrap();

        if tier_locked(*tier, self.player.bloodline) {
            return vec![format!(
                "Tier {} perks require Bloodline {}+.",
                tier,
                if *tier == 2 { 5 } else { 10 }
            )];
        }
        if let Some(req) = prereq {
            if !self.player.selected_perks.contains(&req.to_string()) {
                return vec![format!("Requires {} first.", req)];
            }
        }

        self.player.selected_perks.push(perk_id.to_string());
        self.player.perk_points -= 1;

        // Update combo threshold
        if perk_id == "SwiftStrikes" || perk_id == "ComboMaster" || perk_id == "QIPResonance" {
            self.combo_reset_threshold += 1;
        }

        vec![format!("  \u{2713} {} acquired!", perk_id)]
    }

    fn cmd_shop(&self) -> Vec<String> {
        let mut lines = vec![
            "\u{2550}\u{2550}\u{2550} Shop \u{2014} Weapons \u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}".to_string(),
        ];
        for w in weapon_catalog().iter().take(8) {
            let price = weapon_price(&w);
            let afford = if self.player.gold >= price {
                ""
            } else {
                " (can't afford)"
            };
            lines.push(format!(
                "  {} \u{2014} +{} atk, {:?} | {} gp | buy {}{}",
                w.name, w.attack_bonus, w.rarity, price, w.id, afford
            ));
        }
        lines.push(format!("  Gold: {} gp", self.player.gold));
        lines
    }

    fn cmd_buy(&mut self, item_id: &str) -> Vec<String> {
        let catalog = weapon_catalog();
        let weapon = catalog.iter().find(|w| w.id == item_id).cloned();
        match weapon {
            None => vec![format!(
                "  Item '{}' not found. Type 'shop' to browse.",
                item_id
            )],
            Some(w) => {
                let price = weapon_price(&w);
                if self.player.gold < price {
                    return vec![format!(
                        "  Need {} gp, have {} gp.",
                        price, self.player.gold
                    )];
                }
                self.player.gold -= price;
                let name = w.name.clone();
                self.player.loot_bag.push(w);
                vec![format!(
                    "  Purchased {}! Type 'equip {}' to equip it.",
                    name, item_id
                )]
            }
        }
    }

    fn cmd_sell(&mut self, _slot: &str) -> Vec<String> {
        if let Some(w) = self.player.weapon.take() {
            let value = weapon_price(&w) / 4;
            self.player.gold += value;
            vec![format!("  Sold {} for {} gp.", w.name, value)]
        } else {
            vec!["  Nothing to sell in weapon slot.".to_string()]
        }
    }

    fn cmd_equip(&mut self, item_id: &str) -> Vec<String> {
        let pos = self.player.loot_bag.iter().position(|w| w.id == item_id);
        match pos {
            None => vec![format!(
                "  '{}' not in loot bag. Check 'inventory'.",
                item_id
            )],
            Some(i) => {
                let weapon = self.player.loot_bag.remove(i);
                let name = weapon.name.clone();
                self.player.weapon = Some(weapon);
                vec![format!("  Equipped {}!", name)]
            }
        }
    }

    // ─── Helpers ────────────────────────────────────────────────────────────

    fn handle_enemy_defeat(&mut self, mut lines: Vec<String>) -> Vec<String> {
        let (xp, gold, name) = if let Some(e) = self.current_enemy.take() {
            (e.reward_xp, e.reward_gold, e.name.clone())
        } else {
            (0, 0, "Enemy".to_string())
        };

        self.is_in_combat = false;
        self.announced_attack = None;
        self.combo_depth = 0;

        // Gold with Scavenger bonus
        // TODO: call ib4-progression to apply perk bonuses
        let gold_mult = if self.player.has_perk("TreasureHunter") {
            1.5
        } else if self.player.has_perk("Scavenger") {
            1.2
        } else {
            1.0
        };
        let gold_gained = (gold as f32 * gold_mult) as u32;
        self.player.gold += gold_gained;

        // XP with InfinitySeeker bonus
        let xp_mult = if self.player.has_perk("InfinitySeeker") {
            1.5
        } else {
            1.0
        };
        let xp_gained = (xp as f32 * xp_mult) as u64;
        self.player.xp += xp_gained;
        let levelled = check_level_up(&mut self.player);

        lines.push(format!("  \u{2694}  {} defeated!", name));
        lines.push(format!("  +{} XP | +{} gp", xp_gained, gold_gained));
        for l in levelled {
            lines.push(l);
        }

        // Loot drop
        let drop_roll: f32 = self.rng.gen();
        if drop_roll < 0.15 {
            let catalog = weapon_catalog();
            let idx = self.rng.gen_range(0..catalog.len());
            let drop = catalog[idx].clone();
            lines.push(format!(
                "  \u{1f48e} Loot drop: {} [{:?}]! Type 'equip {}' to equip.",
                drop.name, drop.rarity, drop.id
            ));
            self.player.loot_bag.push(drop);
        }

        if self.arena_queue.is_empty() {
            lines.push(
                "  You have conquered this arena. The bloodline endures.".to_string(),
            );
        } else {
            lines.push(format!(
                "  Next: {}. Type 'explore' to continue.",
                self.arena_queue.front().map(|s| s.as_str()).unwrap_or("?")
            ));
        }
        lines
    }

    fn handle_player_death(&mut self, mut lines: Vec<String>) -> Vec<String> {
        lines.push("  \u{1f480} You have fallen.".to_string());
        lines.extend(self.do_rebirth(true));
        lines
    }

    fn handle_forced_rebirth(&mut self) -> Vec<String> {
        self.do_rebirth(false)
    }

    fn do_rebirth(&mut self, died: bool) -> Vec<String> {
        self.player.bloodline += 1;
        let perk_gained = self.player.bloodline <= 20;
        if perk_gained {
            self.player.perk_points += 1;
        }

        // Reset economy
        self.player.gold = 0;
        self.player.weapon = None;
        self.player.shield = None;
        self.player.loot_bag.clear();
        self.player.qip_scar_stacks = 0;
        self.current_enemy = None;
        self.is_in_combat = false;
        self.combo_depth = 0;

        // Restore HP/mana
        self.player.health = self.player.max_health;
        self.player.mana = self.player.max_mana;

        // Check magic unlocks
        // TODO: call ib4-progression::bloodline for magic unlock table
        let mut new_magic = Vec::new();
        for (bl, magic) in &[
            (3, MagicType::Lightning),
            (6, MagicType::Ice),
            (10, MagicType::Dark),
            (15, MagicType::Light),
        ] {
            if self.player.bloodline >= *bl && !self.player.magic_unlocks.contains(magic) {
                self.player.magic_unlocks.push(magic.clone());
                new_magic.push(format!("{}", magic));
            }
        }

        // Rebuild arena queue for new bloodline
        self.arena_queue = vec!["LightTitan", "HeavyTitan", "DarkKnight", "CorruptedGalath"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        let mut lines = vec![
            format!(
                "  \u{2550}\u{2550}\u{2550} BLOODLINE {} \u{2550}\u{2550}\u{2550}",
                self.player.bloodline_label()
            ),
            if died {
                "  The QIP preserves your essence. You are reborn.".to_string()
            } else {
                "  Three QIP Scars consumed your soul. Reborn.".to_string()
            },
            format!(
                "  Enemies grow stronger ({:.2}\u{d7} HP).",
                1.0 + self.player.bloodline as f32 * 0.15
            ),
        ];
        if perk_gained {
            lines.push(format!(
                "  +1 Perk Point! ({})",
                self.player.perk_points
            ));
        }
        for m in &new_magic {
            lines.push(format!("  \u{2728} New magic unlocked: {}", m));
        }
        lines
    }

    fn announce_next_attack(&mut self, lines: &mut Vec<String>) {
        if !self.is_in_combat {
            return;
        }
        let enemy = match self.current_enemy.as_ref() {
            Some(e) => e.clone(),
            None => return,
        };
        if enemy.is_stunned {
            if let Some(e) = self.current_enemy.as_mut() {
                e.tick_stun();
            }
            lines.push("  The enemy is stunned and cannot act!".to_string());
            return;
        }
        // TODO: call ib4-ai::titan::TitanAI::decide for smarter AI decisions
        let dir = self.random_dir();
        lines.push(narrative::fmt_attack_announce(&enemy.name, &dir, enemy.phase));
        self.announced_attack = Some(dir);
    }

    fn random_dir(&mut self) -> AttackDir {
        match self.rng.gen_range(0..3u32) {
            0 => AttackDir::Overhead,
            1 => AttackDir::Left,
            _ => AttackDir::Right,
        }
    }

    fn make_enemy_inline(&self, id: &str, hp_scale: f32) -> EnemyInstance {
        let (name, hp, atk, bl_req, xp, gold, drop, titan_type) = match id {
            "LightTitan" => (
                "Light Titan",
                150.0,
                20.0,
                0,
                50u64,
                75u32,
                0.15f32,
                ib4_core::types::TitanType::Warrior,
            ),
            "HeavyTitan" => (
                "Heavy Crusher",
                300.0,
                35.0,
                0,
                80,
                120,
                0.12,
                ib4_core::types::TitanType::Heavy,
            ),
            "DarkKnight" => (
                "Dark Knight",
                200.0,
                28.0,
                0,
                65,
                100,
                0.13,
                ib4_core::types::TitanType::Warrior,
            ),
            "MageTitan" => (
                "Arcane Titan",
                120.0,
                45.0,
                0,
                90,
                130,
                0.10,
                ib4_core::types::TitanType::Mage,
            ),
            "GiantTitan" => (
                "War Giant",
                500.0,
                50.0,
                1,
                150,
                200,
                0.08,
                ib4_core::types::TitanType::Heavy,
            ),
            "BloodSlave" => (
                "Blood Slave",
                250.0,
                32.0,
                1,
                100,
                150,
                0.11,
                ib4_core::types::TitanType::Warrior,
            ),
            "KuroShino" => (
                "Kuro Shino",
                180.0,
                40.0,
                2,
                120,
                175,
                0.09,
                ib4_core::types::TitanType::Warrior,
            ),
            "DeathlessSoldier" => (
                "Deathless Soldier",
                400.0,
                45.0,
                3,
                200,
                280,
                0.07,
                ib4_core::types::TitanType::Warrior,
            ),
            "ElementalTitan" => (
                "Elemental Titan",
                350.0,
                60.0,
                3,
                220,
                300,
                0.07,
                ib4_core::types::TitanType::Mage,
            ),
            "ShadowTitan" => (
                "Shadow Titan",
                280.0,
                55.0,
                4,
                250,
                350,
                0.06,
                ib4_core::types::TitanType::Warrior,
            ),
            "TwinBladeTitan" => (
                "Twin Blade",
                320.0,
                42.0,
                5,
                280,
                380,
                0.06,
                ib4_core::types::TitanType::Warrior,
            ),
            "CrystalGolem" => (
                "Crystal Golem",
                600.0,
                65.0,
                6,
                350,
                450,
                0.05,
                ib4_core::types::TitanType::Heavy,
            ),
            "QuantumSoldier" => (
                "Quantum Soldier",
                450.0,
                70.0,
                7,
                400,
                500,
                0.05,
                ib4_core::types::TitanType::Warrior,
            ),
            "Kuero" => (
                "Kuero",
                800.0,
                80.0,
                10,
                600,
                800,
                0.04,
                ib4_core::types::TitanType::Warrior,
            ),
            _ => (
                "Corrupted Galath",
                2000.0,
                120.0,
                20,
                2000,
                5000,
                0.03,
                ib4_core::types::TitanType::GodKing,
            ),
        };
        let scaled_hp = hp * hp_scale;
        EnemyInstance {
            id: id.to_string(),
            name: name.to_string(),
            titan_type,
            base_hp: scaled_hp,
            current_hp: scaled_hp,
            base_attack_damage: atk,
            attack_damage: atk,
            phase: 1,
            bloodline_required: bl_req,
            reward_xp: xp,
            reward_gold: gold,
            drop_chance: drop,
            pending_attack: None,
            is_stunned: false,
            stun_turns_remaining: 0,
            shield_active: false,
            perfect_parries_received: 0,
        }
    }
}

/// Derive a purchase price from weapon rarity and attack bonus.
fn weapon_price(w: &ib4_core::equipment::Weapon) -> u32 {
    let base: u32 = match w.rarity {
        Rarity::Common => 50,
        Rarity::Uncommon => 150,
        Rarity::Rare => 400,
        Rarity::Epic => 1000,
        Rarity::Legendary => 3000,
        Rarity::Infinity => 10000,
    };
    base + w.attack_bonus as u32 * 5
}

fn combo_multiplier(depth: u32) -> f32 {
    match depth {
        0 | 1 => 1.0,
        2 => 1.5,
        3 => 2.0,
        _ => 3.0,
    }
}

fn xp_to_next(player: &PlayerState) -> u64 {
    if player.level >= 45 {
        return 0;
    }
    let threshold = (100.0 * ((player.level + 1) as f64).powf(1.5)).round() as u64;
    threshold.saturating_sub(player.xp)
}

fn check_level_up(player: &mut PlayerState) -> Vec<String> {
    // TODO: call ib4-progression::xp::check_level_up
    let mut lines = Vec::new();
    while player.level < 45 {
        let threshold = (100.0 * ((player.level + 1) as f64).powf(1.5)).round() as u64;
        if player.xp >= threshold {
            player.level += 1;
            player.stat_points += 2;
            player.recalculate_stats();
            lines.push(format!(
                "  \u{2b06}  LEVEL UP! Now Level {}. +2 stat points.",
                player.level
            ));
        } else {
            break;
        }
    }
    lines
}

fn tier_locked(tier: u8, bloodline: i32) -> bool {
    match tier {
        2 => bloodline < 5,
        3 => bloodline < 10,
        _ => false,
    }
}
