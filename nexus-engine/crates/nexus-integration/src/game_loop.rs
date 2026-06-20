pub use nexus_types::{AttackDir, Gold, Hp, ParryOutcome, Xp};
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatType {
    Attack,
    Defense,
    Magic,
    Health,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameCommand {
    Attack(AttackDir),
    Parry(Option<AttackDir>),
    Dodge,
    UseSpecial,
    OpenShop,
    BuyItem { item_index: usize },
    AllocateStat(StatType),
    SelectPerk(u8),
    LookAround,
    Surrender,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    pub turn: u32,
    pub kind: EventKind,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventKind {
    CombatHit { damage: f32, new_enemy_hp: f32 },
    CombatMiss,
    CombatParry(ParryOutcome),
    PlayerTookDamage { damage: f32, new_player_hp: f32 },
    EnemyDefeated { xp_gained: u64, gold_gained: u32 },
    Rebirth { new_bloodline: u32 },
    ItemPurchased { name: String, cost: u32 },
    StatAllocated(StatType),
    PerkSelected(String),
    LevelUp { new_level: u32 },
    SpecialActivated { ability: String },
    Info(String),
    MatchEnded,
}

// ──────────────────────────────────────────────────────────────────────────────
// Player state
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: u64,
    pub name: String,
    pub level: u32,
    pub xp: Xp,
    pub bloodline: u32,
    pub gold: Gold,
    pub hp: Hp,
    pub max_hp: Hp,
    pub attack: u32,
    pub defense: u32,
    pub magic: u32,
    pub stat_points: u32,
    pub perk_points: u32,
    pub selected_perks: Vec<String>,
    pub combo_depth: u32,
    pub qip_scar_stacks: u32,
    pub trans_am_gauge: f32,
    pub suit_id: String,
}

impl PlayerState {
    pub fn new(id: u64, name: impl Into<String>) -> Self {
        PlayerState {
            id,
            name: name.into(),
            level: 1,
            xp: Xp::new(0),
            bloodline: 0,
            gold: Gold::new(500),
            hp: Hp::new(100.0),
            max_hp: Hp::new(100.0),
            attack: 20,
            defense: 10,
            magic: 0,
            stat_points: 0,
            perk_points: 0,
            selected_perks: vec![],
            combo_depth: 0,
            qip_scar_stacks: 0,
            trans_am_gauge: 0.0,
            suit_id: "RX-78-2".to_string(),
        }
    }

    pub fn is_alive(&self) -> bool {
        !self.hp.is_dead()
    }

    pub fn take_damage(&mut self, dmg: f32) {
        self.hp = Hp::new((self.hp.value() - dmg.max(0.0)).max(0.0));
    }

    pub fn heal(&mut self, amount: f32) {
        self.hp = Hp::new((self.hp.value() + amount).min(self.max_hp.value()));
    }

    pub fn combo_multiplier(&self) -> f32 {
        match self.combo_depth {
            0 | 1 => 1.0,
            2 => 1.5,
            3 => 2.0,
            _ => 3.0,
        }
    }

    pub fn gain_xp(&mut self, xp: u64) -> bool {
        self.xp = self.xp + Xp::new(xp);
        let threshold = 100 * (self.level as u64).pow(2);
        if self.xp.value() >= threshold && self.level < 50 {
            self.level += 1;
            self.stat_points += 3;
            true
        } else {
            false
        }
    }

    pub fn spend_gold(&mut self, amount: u32) -> bool {
        if self.gold.value() >= amount {
            self.gold = self.gold - Gold::new(amount);
            true
        } else {
            false
        }
    }

    pub fn rebirth(&mut self) {
        let saved_xp = self.xp;
        let saved_level = self.level;
        let saved_bloodline = self.bloodline;
        let saved_perks = self.selected_perks.clone();
        *self = PlayerState::new(self.id, self.name.clone());
        self.xp = saved_xp;
        self.level = saved_level;
        self.bloodline = saved_bloodline + 1;
        self.selected_perks = saved_perks;
        if self.bloodline <= 20 {
            self.perk_points += 1;
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Enemy state
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyState {
    pub id: String,
    pub name: String,
    pub hp: f32,
    pub max_hp: f32,
    pub attack: f32,
    pub phase: u8,
    pub announced_dir: Option<AttackDir>,
    pub shield_active: bool,
    pub shield_parries_received: u32,
    pub is_godking: bool,
}

impl EnemyState {
    pub fn new(id: impl Into<String>, name: impl Into<String>, hp: f32, attack: f32) -> Self {
        let id = id.into();
        let is_godking = id == "CorruptedGalath";
        EnemyState {
            id,
            name: name.into(),
            hp,
            max_hp: hp,
            attack,
            phase: 1,
            announced_dir: None,
            shield_active: is_godking,
            shield_parries_received: 0,
            is_godking,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0.0
    }
    pub fn hp_pct(&self) -> f32 {
        if self.max_hp > 0.0 {
            self.hp / self.max_hp
        } else {
            0.0
        }
    }

    /// Apply damage; returns true if phase changed.
    pub fn take_damage(&mut self, dmg: f32) -> bool {
        if self.shield_active {
            return false;
        }
        let prev = self.phase;
        self.hp = (self.hp - dmg.max(0.0)).max(0.0);
        let new_phase = if self.hp_pct() > 0.6 {
            1
        } else if self.hp_pct() > 0.3 {
            2
        } else {
            3
        };
        if new_phase > self.phase {
            self.phase = new_phase;
        }
        self.phase > prev
    }

    /// Register a perfect parry; returns true if GodKing shield breaks.
    pub fn receive_perfect_parry(&mut self) -> bool {
        if self.shield_active && self.is_godking {
            self.shield_parries_received += 1;
            if self.shield_parries_received >= 3 {
                self.shield_active = false;
                return true;
            }
        }
        false
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Shop
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopItem {
    pub name: String,
    pub cost: u32,
    pub attack_bonus: u32,
    pub defense_bonus: u32,
}

pub fn default_shop() -> Vec<ShopItem> {
    vec![
        ShopItem {
            name: "Beam Saber".into(),
            cost: 100,
            attack_bonus: 15,
            defense_bonus: 0,
        },
        ShopItem {
            name: "Shield Frame".into(),
            cost: 80,
            attack_bonus: 0,
            defense_bonus: 10,
        },
        ShopItem {
            name: "Pilot Helm".into(),
            cost: 120,
            attack_bonus: 5,
            defense_bonus: 5,
        },
        ShopItem {
            name: "Bazooka".into(),
            cost: 200,
            attack_bonus: 25,
            defense_bonus: 0,
        },
    ]
}

// ──────────────────────────────────────────────────────────────────────────────
// Game session — the central orchestrator
// ──────────────────────────────────────────────────────────────────────────────

pub struct GameSession {
    pub player: PlayerState,
    pub current_enemy: Option<EnemyState>,
    pub shop: Vec<ShopItem>,
    pub turn: u32,
    pub events: Vec<GameEvent>,
    pub is_in_combat: bool,
    rng: SmallRng,
}

impl GameSession {
    pub fn new(player_id: u64, player_name: &str, seed: u64) -> Self {
        GameSession {
            player: PlayerState::new(player_id, player_name),
            current_enemy: None,
            shop: default_shop(),
            turn: 0,
            events: vec![],
            is_in_combat: false,
            rng: SmallRng::seed_from_u64(seed),
        }
    }

    /// Spawn the final boss directly (for integration tests).
    pub fn spawn_godking(&mut self) {
        let mut boss = EnemyState::new("CorruptedGalath", "Corrupted Galath", 5_000.0, 80.0);
        boss.announced_dir = Some(AttackDir::Overhead);
        self.current_enemy = Some(boss);
        self.is_in_combat = true;
    }

    /// Dispatch one player command; returns the events produced this turn.
    pub fn dispatch(&mut self, cmd: GameCommand) -> Vec<GameEvent> {
        self.turn += 1;
        let mut out: Vec<GameEvent> = vec![];

        match cmd {
            GameCommand::LookAround => {
                if !self.is_in_combat {
                    let enemy = self.spawn_random_enemy();
                    let msg = format!("A {} appears!", enemy.name);
                    self.current_enemy = Some(enemy);
                    self.is_in_combat = true;
                    out.push(self.ev(EventKind::Info(msg)));
                }
            }

            GameCommand::Attack(dir) => {
                if !self.is_in_combat || self.current_enemy.is_none() {
                    // Spawn on first attack
                    let enemy = self.spawn_random_enemy();
                    let msg = format!("A {} attacks!", enemy.name);
                    self.current_enemy = Some(enemy);
                    self.is_in_combat = true;
                    out.push(self.ev(EventKind::Info(msg)));
                }

                let _ = dir; // direction selects combo branch in full impl

                if let Some(enemy) = &mut self.current_enemy {
                    if enemy.shield_active {
                        out.push(self.ev(EventKind::CombatMiss));
                    } else {
                        let mult = self.player.combo_multiplier();
                        let dmg = ((self.player.attack as f32 * mult) - 2.0).max(1.0);
                        self.player.combo_depth = (self.player.combo_depth + 1).min(5);
                        if self.player.combo_depth >= 4 {
                            self.player.trans_am_gauge =
                                (self.player.trans_am_gauge + 0.25).min(1.0);
                        }
                        let phase_changed = enemy.take_damage(dmg);
                        let new_hp = enemy.hp;
                        let new_phase = enemy.phase;
                        let _ = enemy;
                        out.push(self.ev(EventKind::CombatHit {
                            damage: dmg,
                            new_enemy_hp: new_hp,
                        }));
                        if phase_changed {
                            out.push(self.ev(EventKind::SpecialActivated {
                                ability: format!("Phase {}", new_phase),
                            }));
                        }
                    }
                }

                // Check enemy defeated
                if self
                    .current_enemy
                    .as_ref()
                    .map(|e| !e.is_alive())
                    .unwrap_or(false)
                {
                    let xp = self
                        .current_enemy
                        .as_ref()
                        .map(|e| 50 + e.max_hp as u64 / 10)
                        .unwrap_or(50);
                    let gold = 25 + self.rng.random_range(0..50u32);
                    let leveled = self.player.gain_xp(xp);
                    self.player.gold = self.player.gold + Gold::new(gold);
                    self.player.combo_depth = 0;
                    self.current_enemy = None;
                    self.is_in_combat = false;
                    out.push(self.ev(EventKind::EnemyDefeated {
                        xp_gained: xp,
                        gold_gained: gold,
                    }));
                    if leveled {
                        out.push(self.ev(EventKind::LevelUp {
                            new_level: self.player.level,
                        }));
                    }
                } else if self.is_in_combat {
                    // Enemy counter-attack
                    if let Some(ev) = self.enemy_counter() {
                        out.push(ev);
                    }
                }
            }

            GameCommand::Parry(dir) => {
                self.player.combo_depth = 0;
                if let Some(enemy) = &mut self.current_enemy {
                    let announced = enemy.announced_dir;
                    let outcome = match (announced, dir) {
                        (Some(a), Some(d)) if a == d => ParryOutcome::Perfect,
                        (Some(_), None) | (Some(_), Some(_)) => ParryOutcome::Normal,
                        _ => ParryOutcome::Miss,
                    };

                    let shield_broken = if outcome == ParryOutcome::Perfect {
                        enemy.receive_perfect_parry()
                    } else {
                        false
                    };

                    // Extract what we need before dropping the borrow
                    let qip_eligible = enemy.is_godking && enemy.phase == 2 && !enemy.shield_active;
                    let dmg = match outcome {
                        ParryOutcome::Perfect => 0.0,
                        ParryOutcome::Normal => enemy.attack * 0.1,
                        ParryOutcome::Miss => enemy.attack,
                    };
                    let _ = enemy;

                    // QIP scar in GodKing Phase 2
                    if qip_eligible {
                        self.player.qip_scar_stacks += 1;
                        if self.player.qip_scar_stacks >= 3 {
                            self.player.rebirth();
                            let bl = self.player.bloodline;
                            out.push(self.ev(EventKind::Rebirth { new_bloodline: bl }));
                        }
                    }
                    if dmg > 0.0 {
                        self.player.take_damage(dmg);
                    }

                    out.push(self.ev(EventKind::CombatParry(outcome)));
                    if shield_broken {
                        out.push(self.ev(EventKind::Info("GodKing shield shattered!".into())));
                    }
                }

                if !self.player.is_alive() {
                    self.player.rebirth();
                    out.push(self.ev(EventKind::Rebirth {
                        new_bloodline: self.player.bloodline,
                    }));
                }
            }

            GameCommand::Dodge => {
                self.player.combo_depth = 0;
                out.push(self.ev(EventKind::Info("Dodged!".into())));
            }

            GameCommand::UseSpecial => {
                if self.player.trans_am_gauge >= 1.0 {
                    self.player.trans_am_gauge = 0.0;
                    self.player.attack = (self.player.attack as f32 * 3.0) as u32;
                    out.push(self.ev(EventKind::SpecialActivated {
                        ability: "Trans-Am".into(),
                    }));
                } else {
                    out.push(self.ev(EventKind::CombatMiss));
                }
            }

            GameCommand::OpenShop => {
                out.push(self.ev(EventKind::Info(format!(
                    "Shop open. Gold: {}",
                    self.player.gold.value()
                ))));
            }

            GameCommand::BuyItem { item_index } => {
                if item_index < self.shop.len() {
                    let item = self.shop[item_index].clone();
                    if self.player.spend_gold(item.cost) {
                        self.player.attack += item.attack_bonus;
                        self.player.defense += item.defense_bonus;
                        out.push(self.ev(EventKind::ItemPurchased {
                            name: item.name,
                            cost: item.cost,
                        }));
                    }
                }
            }

            GameCommand::AllocateStat(stat) => {
                if self.player.stat_points > 0 {
                    self.player.stat_points -= 1;
                    match stat {
                        StatType::Attack => self.player.attack += 5,
                        StatType::Defense => self.player.defense += 5,
                        StatType::Magic => self.player.magic += 5,
                        StatType::Health => {
                            self.player.max_hp = self.player.max_hp + Hp::new(10.0);
                            self.player.hp = Hp::new(
                                (self.player.hp.value() + 10.0).min(self.player.max_hp.value()),
                            );
                        }
                    }
                    out.push(self.ev(EventKind::StatAllocated(stat)));
                }
            }

            GameCommand::SelectPerk(perk_id) => {
                if self.player.perk_points > 0 {
                    let name = format!("Perk#{}", perk_id);
                    self.player.perk_points -= 1;
                    self.player.selected_perks.push(name.clone());
                    out.push(self.ev(EventKind::PerkSelected(name)));
                }
            }

            GameCommand::Surrender => {
                self.current_enemy = None;
                self.is_in_combat = false;
                out.push(self.ev(EventKind::MatchEnded));
            }
        }

        self.events.extend(out.clone());
        out
    }

    // ── private helpers ──────────────────────────────────────────────────────

    fn ev(&self, kind: EventKind) -> GameEvent {
        let message = format!("{:?}", kind);
        GameEvent {
            turn: self.turn,
            kind,
            message,
        }
    }

    fn spawn_random_enemy(&mut self) -> EnemyState {
        const ROSTER: &[(&str, f32, f32)] = &[
            ("StoneTitan", 150.0, 20.0),
            ("WarlordTitan", 200.0, 25.0),
            ("GoldTitan", 300.0, 35.0),
            ("QuantumTitan", 400.0, 45.0),
        ];
        let (name, hp, atk) = ROSTER[self.rng.random_range(0..ROSTER.len())];
        let mut e = EnemyState::new(name, name, hp, atk);
        let dirs = [AttackDir::Overhead, AttackDir::Left, AttackDir::Right];
        e.announced_dir = Some(dirs[self.rng.random_range(0..3)]);
        e
    }

    fn enemy_counter(&mut self) -> Option<GameEvent> {
        let (atk, _phase, _new_dir) = {
            let e = self.current_enemy.as_mut()?;
            let dirs = [AttackDir::Overhead, AttackDir::Left, AttackDir::Right];
            let dir = dirs[self.rng.random_range(0..3)];
            e.announced_dir = Some(dir);
            let phase_mult = 1.0 + (e.phase as f32 - 1.0) * 0.25;
            (e.attack * phase_mult, e.phase, dir)
        };
        self.player.take_damage(atk);
        let new_hp = self.player.hp.value();

        if !self.player.is_alive() {
            self.player.rebirth();
            return Some(self.ev(EventKind::Rebirth {
                new_bloodline: self.player.bloodline,
            }));
        }
        Some(self.ev(EventKind::PlayerTookDamage {
            damage: atk,
            new_player_hp: new_hp,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_types::{Gold, Hp, Xp};

    // ── PlayerState ───────────────────────────────────────────────────────────

    #[test]
    fn player_new_starts_alive_with_positive_hp() {
        let p = PlayerState::new(1, "Amuro");
        assert!(p.is_alive());
        assert!(p.hp.value() > 0.0);
    }

    #[test]
    fn take_damage_reduces_hp() {
        let mut p = PlayerState::new(1, "Amuro");
        let before = p.hp.value();
        p.take_damage(20.0);
        assert!(p.hp.value() < before);
    }

    #[test]
    fn take_damage_clamps_to_zero() {
        let mut p = PlayerState::new(1, "Amuro");
        p.take_damage(99999.0);
        assert_eq!(p.hp.value(), 0.0);
        assert!(!p.is_alive());
    }

    #[test]
    fn heal_increases_hp_up_to_max() {
        let mut p = PlayerState::new(1, "Amuro");
        p.take_damage(30.0);
        let after_damage = p.hp.value();
        p.heal(10.0);
        assert!(p.hp.value() > after_damage);
    }

    #[test]
    fn heal_does_not_exceed_max_hp() {
        let mut p = PlayerState::new(1, "Amuro");
        p.heal(99999.0);
        assert_eq!(p.hp.value(), p.max_hp.value());
    }

    #[test]
    fn combo_multiplier_is_1_at_zero_depth() {
        let p = PlayerState::new(1, "Amuro");
        assert_eq!(p.combo_depth, 0);
        assert!((p.combo_multiplier() - 1.0).abs() < 0.001);
    }

    #[test]
    fn combo_multiplier_scales_with_depth() {
        let mut p = PlayerState::new(1, "Amuro");
        p.combo_depth = 2;
        assert!((p.combo_multiplier() - 1.5).abs() < 0.001);
        p.combo_depth = 3;
        assert!((p.combo_multiplier() - 2.0).abs() < 0.001);
        p.combo_depth = 5;
        assert!((p.combo_multiplier() - 3.0).abs() < 0.001);
    }

    #[test]
    fn gain_xp_triggers_level_up_at_threshold() {
        let mut p = PlayerState::new(1, "Amuro");
        // threshold at level 1 = 100 * 1^2 = 100
        let leveled = p.gain_xp(100);
        assert!(leveled);
        assert_eq!(p.level, 2);
    }

    #[test]
    fn gain_xp_no_level_up_below_threshold() {
        let mut p = PlayerState::new(1, "Amuro");
        let leveled = p.gain_xp(50);
        assert!(!leveled);
        assert_eq!(p.level, 1);
    }

    #[test]
    fn spend_gold_succeeds_when_sufficient() {
        let mut p = PlayerState::new(1, "Amuro");
        let initial = p.gold.value();
        let ok = p.spend_gold(10);
        assert!(ok);
        assert_eq!(p.gold.value(), initial - 10);
    }

    #[test]
    fn spend_gold_fails_when_insufficient() {
        let mut p = PlayerState::new(1, "Amuro");
        let ok = p.spend_gold(p.gold.value() + 1);
        assert!(!ok);
    }

    #[test]
    fn rebirth_increments_bloodline_and_preserves_xp() {
        let mut p = PlayerState::new(1, "Amuro");
        p.gain_xp(50);
        let xp_before = p.xp;
        p.rebirth();
        assert_eq!(p.bloodline, 1);
        assert_eq!(p.xp, xp_before);
    }

    #[test]
    fn rebirth_gives_perk_point_for_first_20_bloodlines() {
        let mut p = PlayerState::new(1, "Amuro");
        p.rebirth();
        assert_eq!(p.perk_points, 1);
    }

    // ── EnemyState ────────────────────────────────────────────────────────────

    #[test]
    fn enemy_new_is_alive_and_phase_1() {
        let e = EnemyState::new("LightTitan", "Light Titan", 300.0, 20.0);
        assert!(e.is_alive());
        assert_eq!(e.phase, 1);
        assert_eq!(e.hp_pct(), 1.0);
    }

    #[test]
    fn enemy_take_damage_reduces_hp() {
        let mut e = EnemyState::new("LightTitan", "Light Titan", 300.0, 20.0);
        e.take_damage(100.0);
        assert!(e.hp < 300.0);
    }

    #[test]
    fn godking_starts_with_shield_active() {
        let e = EnemyState::new("CorruptedGalath", "Galath", 1000.0, 50.0);
        assert!(e.shield_active);
        assert!(e.is_godking);
    }

    #[test]
    fn godking_shield_blocks_damage() {
        let mut e = EnemyState::new("CorruptedGalath", "Galath", 1000.0, 50.0);
        let phase_changed = e.take_damage(500.0);
        assert!(!phase_changed, "shield should absorb damage without phase change");
        assert_eq!(e.hp, 1000.0, "HP should be unchanged while shield is active");
    }

    #[test]
    fn receive_perfect_parry_weakens_godking_shield() {
        let mut e = EnemyState::new("CorruptedGalath", "Galath", 1000.0, 50.0);
        // Each receive_perfect_parry increments shield_parries_received
        e.receive_perfect_parry();
        assert_eq!(e.shield_parries_received, 1);
    }

    #[test]
    fn hp_pct_is_zero_when_max_hp_is_zero() {
        let mut e = EnemyState::new("test", "test", 100.0, 10.0);
        e.max_hp = 0.0;
        assert_eq!(e.hp_pct(), 0.0);
    }

    // ── GameCommand / EventKind serde ────────────────────────────────────────

    #[test]
    fn game_command_serializes() {
        let cmd = GameCommand::Attack(AttackDir::Overhead);
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("Attack") || json.contains("attack") || json.contains("Overhead"));
    }

    #[test]
    fn stat_type_variants_are_distinct() {
        assert_ne!(
            serde_json::to_string(&StatType::Attack).unwrap(),
            serde_json::to_string(&StatType::Defense).unwrap()
        );
    }
}
