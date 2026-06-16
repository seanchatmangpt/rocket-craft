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
