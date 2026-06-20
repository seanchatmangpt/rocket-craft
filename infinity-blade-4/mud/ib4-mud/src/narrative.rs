use ib4_core::types::{AttackDir, MagicType};

pub fn fmt_hp_bar(current: f32, max: f32, width: u32) -> String {
    let pct = (current / max).clamp(0.0, 1.0);
    let filled = (pct * width as f32).round() as u32;
    let bar: String = (0..width)
        .map(|i| if i < filled { '\u{2588}' } else { '\u{2591}' })
        .collect();
    format!("[{}] {:.0}/{:.0}", bar, current, max)
}

pub fn fmt_attack_announce(enemy_name: &str, dir: &AttackDir, phase: u8) -> String {
    let dir_str = match dir {
        AttackDir::Overhead => "OVERHEAD",
        AttackDir::Left => "LEFT",
        AttackDir::Right => "RIGHT",
    };
    match phase {
        1 => format!(
            "  \u{2694}  {} telegraphs a {} strike! [Parry opportunity]",
            enemy_name, dir_str
        ),
        2 => format!(
            "  \u{1f525} ENRAGED {} lunges {} with terrifying speed!",
            enemy_name, dir_str
        ),
        3 => format!(
            "  \u{26a1} {} attacks {} \u{2014} but something feels wrong...",
            enemy_name, dir_str
        ),
        _ => format!("  \u{2694}  {} attacks {}!", enemy_name, dir_str),
    }
}

pub fn fmt_parry_result(perfect: bool, dir: &AttackDir) -> String {
    if perfect {
        format!(
            "  \u{2728} PERFECT PARRY! You anticipated the {} strike \u{2014} time stutters!",
            dir
        )
    } else {
        format!("  \u{1f6e1}  Normal parry. You deflect the {} blow.", dir)
    }
}

pub fn fmt_damage_dealt(damage: f32, is_crit: bool, combo_depth: u32) -> String {
    let combo_label = match combo_depth {
        0 | 1 => String::new(),
        2 => " [COMBO x2!]".to_string(),
        3 => " [COMBO x3!]".to_string(),
        n => format!(" [COMBO x{}!]", n),
    };
    if is_crit {
        format!(
            "  \u{1f4a5} CRITICAL HIT! {:.0} damage!{}",
            damage, combo_label
        )
    } else {
        format!("  \u{2694}  {:.0} damage.{}", damage, combo_label)
    }
}

pub fn fmt_magic_use(magic: &MagicType, damage: f32, heal: f32) -> String {
    match magic {
        MagicType::Fire => format!(
            "  \u{1f525} Deathless fire erupts! {:.0} damage + Burn (3 turns)",
            damage
        ),
        MagicType::Lightning => format!(
            "  \u{26a1} Lightning bolt! {:.0} damage + Stun (1 turn)",
            damage
        ),
        MagicType::Ice => format!(
            "  \u{2744}  Ice lance! {:.0} damage + Freeze (2 turns)",
            damage
        ),
        MagicType::Dark => format!(
            "  \u{1f311} Dark void! {:.0} damage (ignores 50% defense) + Dark curse",
            damage
        ),
        MagicType::Light => format!(
            "  \u{2728} QIP light restores {:.0} HP. Status effects cleared.",
            heal
        ),
    }
}

pub fn fmt_enemy_hp(name: &str, hp: f32, max_hp: f32, phase: u8) -> String {
    let phase_label = match phase {
        1 => "",
        2 => " [ENRAGED]",
        3 => " [FRACTURE]",
        _ => "",
    };
    format!(
        "  \u{25b8} {}{}: {:.0}/{:.0} HP",
        name, phase_label, hp, max_hp
    )
}

pub fn help_text() -> String {
    r#"
═══════════════════════════════════════════════
  INFINITY BLADE IV — COMMAND REFERENCE
═══════════════════════════════════════════════
  COMBAT:
    attack <overhead|left|right>   Attack in direction (a o/l/r)
    parry [overhead|left|right]    Parry incoming attack
    perfect parry <dir>            Perfect parry (match exact direction)
    dodge                          Dodge roll (d)
    magic <fire|lightning|ice|     Cast magic spell (m <type>)
          dark|light>

  NAVIGATION:
    look / explore                 View arena and enemy

  CHARACTER:
    status                         Show stats and HP (s)
    inventory                      Show equipment (i)
    alloc <stat>                   Spend stat point (health/attack/defense/magic)
    perks                          Show perk tree
    perk <PerkID>                  Select a perk

  ECONOMY:
    shop                           Browse equipment shop
    buy <item_id>                  Purchase equipment
    sell <slot>                    Sell equipped item
    equip <item_id>                Equip from loot bag

  META:
    save                           Save game
    help                           This menu
    quit                           Exit
═══════════════════════════════════════════════"#
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── fmt_hp_bar ────────────────────────────────────────────────────────────

    #[test]
    fn hp_bar_full_health_is_all_filled() {
        let bar = fmt_hp_bar(100.0, 100.0, 10);
        // Should start with [██████████] (10 filled)
        assert!(bar.starts_with('['));
        assert!(bar.contains("██████████"));
        assert!(bar.contains("100/100"));
    }

    #[test]
    fn hp_bar_zero_health_is_all_empty() {
        let bar = fmt_hp_bar(0.0, 100.0, 10);
        assert!(bar.contains("░░░░░░░░░░"));
        assert!(bar.contains("0/100"));
    }

    #[test]
    fn hp_bar_half_health_has_five_filled_five_empty() {
        let bar = fmt_hp_bar(50.0, 100.0, 10);
        let inner: String = bar.chars()
            .skip(1) // skip '['
            .take(10)
            .collect();
        let filled = inner.chars().filter(|&c| c == '█').count();
        let empty = inner.chars().filter(|&c| c == '░').count();
        assert_eq!(filled, 5);
        assert_eq!(empty, 5);
    }

    #[test]
    fn hp_bar_overheal_clamps_to_full() {
        // current > max should not panic, clamps to 1.0
        let bar = fmt_hp_bar(150.0, 100.0, 10);
        assert!(bar.contains("██████████"));
    }

    // ── fmt_attack_announce ───────────────────────────────────────────────────

    #[test]
    fn attack_announce_phase1_contains_direction_and_enemy() {
        let s = fmt_attack_announce("GodKing", &AttackDir::Overhead, 1);
        assert!(s.contains("GodKing"));
        assert!(s.contains("OVERHEAD"));
        assert!(s.contains("Parry opportunity"));
    }

    #[test]
    fn attack_announce_phase2_contains_enraged() {
        let s = fmt_attack_announce("Titan", &AttackDir::Left, 2);
        assert!(s.contains("ENRAGED"));
        assert!(s.contains("LEFT"));
    }

    #[test]
    fn attack_announce_phase3_contains_wrong() {
        let s = fmt_attack_announce("Titan", &AttackDir::Right, 3);
        assert!(s.contains("RIGHT"));
        assert!(s.contains("wrong"));
    }

    #[test]
    fn attack_announce_other_phase_uses_default() {
        let s = fmt_attack_announce("Mook", &AttackDir::Left, 5);
        assert!(s.contains("Mook"));
        assert!(s.contains("LEFT"));
    }

    // ── fmt_parry_result ──────────────────────────────────────────────────────

    #[test]
    fn parry_result_perfect_contains_perfect_and_dir() {
        let s = fmt_parry_result(true, &AttackDir::Overhead);
        assert!(s.contains("PERFECT PARRY"));
        assert!(s.contains("Overhead"));
    }

    #[test]
    fn parry_result_normal_contains_deflect_and_dir() {
        let s = fmt_parry_result(false, &AttackDir::Left);
        assert!(s.contains("Normal parry"));
        assert!(s.contains("Left"));
    }

    // ── fmt_damage_dealt ──────────────────────────────────────────────────────

    #[test]
    fn damage_dealt_no_combo_no_crit() {
        let s = fmt_damage_dealt(42.0, false, 1);
        assert!(s.contains("42"));
        assert!(!s.contains("COMBO"));
        assert!(!s.contains("CRITICAL"));
    }

    #[test]
    fn damage_dealt_crit_contains_critical_hit() {
        let s = fmt_damage_dealt(99.0, true, 1);
        assert!(s.contains("CRITICAL HIT"));
        assert!(s.contains("99"));
    }

    #[test]
    fn damage_dealt_combo_2_shows_combo_x2() {
        let s = fmt_damage_dealt(30.0, false, 2);
        assert!(s.contains("COMBO x2"));
    }

    #[test]
    fn damage_dealt_combo_3_shows_combo_x3() {
        let s = fmt_damage_dealt(30.0, false, 3);
        assert!(s.contains("COMBO x3"));
    }

    #[test]
    fn damage_dealt_combo_4_shows_dynamic_count() {
        let s = fmt_damage_dealt(30.0, false, 4);
        assert!(s.contains("COMBO x4"));
    }

    // ── fmt_magic_use ─────────────────────────────────────────────────────────

    #[test]
    fn magic_fire_mentions_fire_and_burn() {
        let s = fmt_magic_use(&MagicType::Fire, 60.0, 0.0);
        assert!(s.contains("60"));
        assert!(s.contains("Burn"));
    }

    #[test]
    fn magic_lightning_mentions_stun() {
        let s = fmt_magic_use(&MagicType::Lightning, 75.0, 0.0);
        assert!(s.contains("Stun"));
    }

    #[test]
    fn magic_ice_mentions_freeze() {
        let s = fmt_magic_use(&MagicType::Ice, 55.0, 0.0);
        assert!(s.contains("Freeze"));
    }

    #[test]
    fn magic_dark_mentions_defense_ignore() {
        let s = fmt_magic_use(&MagicType::Dark, 80.0, 0.0);
        assert!(s.contains("50% defense"));
    }

    #[test]
    fn magic_light_uses_heal_amount() {
        let s = fmt_magic_use(&MagicType::Light, 0.0, 35.0);
        assert!(s.contains("35"));
        assert!(s.contains("HP"));
    }

    // ── fmt_enemy_hp ──────────────────────────────────────────────────────────

    #[test]
    fn enemy_hp_phase1_no_label() {
        let s = fmt_enemy_hp("Titan", 200.0, 500.0, 1);
        assert!(s.contains("Titan"));
        assert!(s.contains("200/500"));
        assert!(!s.contains("ENRAGED"));
    }

    #[test]
    fn enemy_hp_phase2_shows_enraged() {
        let s = fmt_enemy_hp("GodKing", 150.0, 1000.0, 2);
        assert!(s.contains("ENRAGED"));
    }

    #[test]
    fn enemy_hp_phase3_shows_fracture() {
        let s = fmt_enemy_hp("GodKing", 50.0, 1000.0, 3);
        assert!(s.contains("FRACTURE"));
    }

    // ── help_text ─────────────────────────────────────────────────────────────

    #[test]
    fn help_text_is_non_empty_and_contains_commands() {
        let h = help_text();
        assert!(h.contains("attack"));
        assert!(h.contains("magic"));
        assert!(h.contains("save"));
        assert!(h.contains("quit"));
    }
}
