use ib4_core::types::{AttackDir, MagicType};

#[derive(Debug, Clone)]
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
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        if parts.is_empty() {
            return Err("Type 'help' to see commands.".to_string());
        }

        match parts[0].to_lowercase().as_str() {
            "attack" | "a" => {
                let dir = parts.get(1).map(|s| parse_dir(s)).transpose()?
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
                let magic = parts.get(1).map(|s| parse_magic(s)).transpose()?
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
