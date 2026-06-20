use ib4_core::types::{AttackDir, MagicType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    // Combat
    Attack(AttackDir),
    Parry,
    PerfectParry(AttackDir),
    Dodge,
    Magic(MagicType),
    // Navigation
    Look,
    Explore,
    // Character
    Status,
    Inventory,
    AllocStat(String),
    Perks,
    SelectPerk(String),
    // Economy
    Shop,
    Buy(String),
    Sell(String),
    Equip(String),
    // Meta
    Save,
    Help,
    Quit,
}

impl Command {
    /// Parse a raw line. Case-insensitive. Returns Err with hint text.
    pub fn parse(line: &str) -> Result<Command, String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Type 'help' to see commands.".to_string());
        }

        match parts[0].to_lowercase().as_str() {
            "attack" | "a" => {
                let dir = parts
                    .get(1)
                    .map(|s| parse_dir(s))
                    .transpose()?
                    .unwrap_or(AttackDir::Right);
                Ok(Command::Attack(dir))
            }
            "parry" | "p" => {
                if let Some(dir_str) = parts.get(1) {
                    if *dir_str == "parry" {
                        return Err("Did you mean 'perfect parry <direction>'?".to_string());
                    }
                    Ok(Command::PerfectParry(parse_dir(dir_str)?))
                } else {
                    Ok(Command::Parry)
                }
            }
            "perfect" => {
                // "perfect parry <dir>"
                if parts.get(1).map(|s| *s == "parry").unwrap_or(false) {
                    if let Some(dir_str) = parts.get(2) {
                        Ok(Command::PerfectParry(parse_dir(dir_str)?))
                    } else {
                        Err("Usage: perfect parry <overhead|left|right>".to_string())
                    }
                } else {
                    Err("Did you mean 'perfect parry <direction>'?".to_string())
                }
            }
            "dodge" | "d" => Ok(Command::Dodge),
            "magic" | "cast" | "m" => {
                let magic = parts
                    .get(1)
                    .map(|s| parse_magic(s))
                    .transpose()?
                    .ok_or_else(|| "Usage: magic <fire|lightning|ice|dark|light>".to_string())?;
                Ok(Command::Magic(magic))
            }
            "look" | "l" => Ok(Command::Look),
            "explore" | "e" => Ok(Command::Explore),
            "status" | "s" | "stat" => Ok(Command::Status),
            "inventory" | "inv" | "i" => Ok(Command::Inventory),
            "alloc" | "allocate" => {
                let stat = parts
                    .get(1)
                    .ok_or("Usage: alloc <health|attack|defense|magic>")?
                    .to_string();
                Ok(Command::AllocStat(stat))
            }
            "perks" => Ok(Command::Perks),
            "perk" | "select" => {
                // "select perk <id>" or "perk <id>"
                let id = if parts.get(1).map(|s| *s == "perk").unwrap_or(false) {
                    parts.get(2)
                } else {
                    parts.get(1)
                }
                .ok_or("Usage: perk <PerkID>")?
                .to_string();
                Ok(Command::SelectPerk(id))
            }
            "shop" => Ok(Command::Shop),
            "buy" => Ok(Command::Buy(
                parts.get(1).ok_or("Usage: buy <item_id>")?.to_string(),
            )),
            "sell" => Ok(Command::Sell(
                parts.get(1).ok_or("Usage: sell <slot>")?.to_string(),
            )),
            "equip" => Ok(Command::Equip(
                parts.get(1).ok_or("Usage: equip <item_id>")?.to_string(),
            )),
            "save" => Ok(Command::Save),
            "help" | "?" => Ok(Command::Help),
            "quit" | "exit" | "q" => Ok(Command::Quit),
            other => Err(format!(
                "Unknown command: '{}'. Type 'help' for commands.",
                other
            )),
        }
    }
}

fn parse_dir(s: &str) -> Result<AttackDir, String> {
    match s.to_lowercase().as_str() {
        "overhead" | "up" | "o" => Ok(AttackDir::Overhead),
        "left" | "l" => Ok(AttackDir::Left),
        "right" | "r" => Ok(AttackDir::Right),
        _ => Err(format!(
            "Unknown direction: '{}'. Use: overhead, left, right",
            s
        )),
    }
}

fn parse_magic(s: &str) -> Result<MagicType, String> {
    match s.to_lowercase().as_str() {
        "fire" | "f" => Ok(MagicType::Fire),
        "lightning" | "thunder" | "l" => Ok(MagicType::Lightning),
        "ice" | "frost" | "i" => Ok(MagicType::Ice),
        "dark" | "shadow" | "d" => Ok(MagicType::Dark),
        "light" | "heal" => Ok(MagicType::Light),
        _ => Err(format!(
            "Unknown magic: '{}'. Use: fire, lightning, ice, dark, light",
            s
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Result<Command, String> {
        Command::parse(s)
    }

    // ── empty / whitespace ────────────────────────────────────────────────────

    #[test]
    fn empty_input_returns_help_hint() {
        assert!(parse("").is_err());
        assert!(parse("   ").is_err());
    }

    // ── Attack ────────────────────────────────────────────────────────────────

    #[test]
    fn attack_no_dir_defaults_to_right() {
        assert_eq!(parse("attack"), Ok(Command::Attack(AttackDir::Right)));
        assert_eq!(parse("a"), Ok(Command::Attack(AttackDir::Right)));
    }

    #[test]
    fn attack_with_directions() {
        assert_eq!(parse("attack overhead"), Ok(Command::Attack(AttackDir::Overhead)));
        assert_eq!(parse("attack left"), Ok(Command::Attack(AttackDir::Left)));
        assert_eq!(parse("attack right"), Ok(Command::Attack(AttackDir::Right)));
    }

    #[test]
    fn attack_direction_aliases() {
        assert_eq!(parse("attack up"), Ok(Command::Attack(AttackDir::Overhead)));
        assert_eq!(parse("attack o"), Ok(Command::Attack(AttackDir::Overhead)));
        assert_eq!(parse("attack l"), Ok(Command::Attack(AttackDir::Left)));
        assert_eq!(parse("attack r"), Ok(Command::Attack(AttackDir::Right)));
    }

    #[test]
    fn attack_unknown_direction_returns_error() {
        assert!(parse("attack diagonal").is_err());
    }

    // ── Parry / PerfectParry ──────────────────────────────────────────────────

    #[test]
    fn parry_no_dir_is_normal_parry() {
        assert_eq!(parse("parry"), Ok(Command::Parry));
        assert_eq!(parse("p"), Ok(Command::Parry));
    }

    #[test]
    fn parry_with_dir_is_perfect_parry() {
        assert_eq!(parse("parry overhead"), Ok(Command::PerfectParry(AttackDir::Overhead)));
        assert_eq!(parse("parry left"), Ok(Command::PerfectParry(AttackDir::Left)));
    }

    #[test]
    fn perfect_parry_three_word_form() {
        assert_eq!(parse("perfect parry right"), Ok(Command::PerfectParry(AttackDir::Right)));
    }

    #[test]
    fn perfect_parry_missing_dir_returns_error() {
        assert!(parse("perfect parry").is_err());
    }

    // ── Dodge ────────────────────────────────────────────────────────────────

    #[test]
    fn dodge_and_alias() {
        assert_eq!(parse("dodge"), Ok(Command::Dodge));
        assert_eq!(parse("d"), Ok(Command::Dodge));
    }

    // ── Magic ─────────────────────────────────────────────────────────────────

    #[test]
    fn magic_all_types() {
        assert_eq!(parse("magic fire"), Ok(Command::Magic(MagicType::Fire)));
        assert_eq!(parse("magic lightning"), Ok(Command::Magic(MagicType::Lightning)));
        assert_eq!(parse("magic ice"), Ok(Command::Magic(MagicType::Ice)));
        assert_eq!(parse("magic dark"), Ok(Command::Magic(MagicType::Dark)));
        assert_eq!(parse("magic light"), Ok(Command::Magic(MagicType::Light)));
    }

    #[test]
    fn magic_aliases() {
        assert_eq!(parse("cast fire"), Ok(Command::Magic(MagicType::Fire)));
        assert_eq!(parse("magic heal"), Ok(Command::Magic(MagicType::Light)));
        assert_eq!(parse("magic thunder"), Ok(Command::Magic(MagicType::Lightning)));
        assert_eq!(parse("magic frost"), Ok(Command::Magic(MagicType::Ice)));
        assert_eq!(parse("magic shadow"), Ok(Command::Magic(MagicType::Dark)));
    }

    #[test]
    fn magic_no_type_returns_error() {
        assert!(parse("magic").is_err());
    }

    // ── Navigation / Status ───────────────────────────────────────────────────

    #[test]
    fn look_explore_and_aliases() {
        assert_eq!(parse("look"), Ok(Command::Look));
        assert_eq!(parse("l"), Ok(Command::Look));
        assert_eq!(parse("explore"), Ok(Command::Explore));
        assert_eq!(parse("e"), Ok(Command::Explore));
    }

    #[test]
    fn status_and_aliases() {
        assert_eq!(parse("status"), Ok(Command::Status));
        assert_eq!(parse("s"), Ok(Command::Status));
        assert_eq!(parse("stat"), Ok(Command::Status));
    }

    #[test]
    fn inventory_and_aliases() {
        assert_eq!(parse("inventory"), Ok(Command::Inventory));
        assert_eq!(parse("inv"), Ok(Command::Inventory));
        assert_eq!(parse("i"), Ok(Command::Inventory));
    }

    // ── Alloc / Perks ─────────────────────────────────────────────────────────

    #[test]
    fn alloc_stat_stores_stat_name() {
        assert_eq!(parse("alloc health"), Ok(Command::AllocStat("health".into())));
        assert_eq!(parse("allocate attack"), Ok(Command::AllocStat("attack".into())));
    }

    #[test]
    fn alloc_no_stat_returns_error() {
        assert!(parse("alloc").is_err());
    }

    #[test]
    fn perks_command() {
        assert_eq!(parse("perks"), Ok(Command::Perks));
    }

    #[test]
    fn select_perk_stores_id() {
        assert_eq!(parse("perk AncestralBlood"), Ok(Command::SelectPerk("AncestralBlood".into())));
        assert_eq!(parse("select perk TitanBane"), Ok(Command::SelectPerk("TitanBane".into())));
    }

    // ── Economy ──────────────────────────────────────────────────────────────

    #[test]
    fn shop_buy_sell_equip() {
        assert_eq!(parse("shop"), Ok(Command::Shop));
        assert_eq!(parse("buy beam_saber"), Ok(Command::Buy("beam_saber".into())));
        assert_eq!(parse("sell slot_1"), Ok(Command::Sell("slot_1".into())));
        assert_eq!(parse("equip iron_shield"), Ok(Command::Equip("iron_shield".into())));
    }

    #[test]
    fn buy_no_arg_returns_error() {
        assert!(parse("buy").is_err());
    }

    // ── Meta ──────────────────────────────────────────────────────────────────

    #[test]
    fn save_help_quit_and_aliases() {
        assert_eq!(parse("save"), Ok(Command::Save));
        assert_eq!(parse("help"), Ok(Command::Help));
        assert_eq!(parse("?"), Ok(Command::Help));
        assert_eq!(parse("quit"), Ok(Command::Quit));
        assert_eq!(parse("exit"), Ok(Command::Quit));
        assert_eq!(parse("q"), Ok(Command::Quit));
    }

    // ── Case insensitivity ────────────────────────────────────────────────────

    #[test]
    fn commands_are_case_insensitive() {
        assert_eq!(parse("ATTACK OVERHEAD"), Ok(Command::Attack(AttackDir::Overhead)));
        assert_eq!(parse("MAGIC FIRE"), Ok(Command::Magic(MagicType::Fire)));
        assert_eq!(parse("QUIT"), Ok(Command::Quit));
    }

    // ── Unknown command ───────────────────────────────────────────────────────

    #[test]
    fn unknown_command_returns_error_with_hint() {
        let e = parse("fireball").unwrap_err();
        assert!(e.contains("fireball"));
        assert!(e.contains("help"));
    }
}
