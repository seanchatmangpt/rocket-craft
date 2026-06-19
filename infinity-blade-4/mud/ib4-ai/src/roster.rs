use ib4_core::{enemy::EnemyInstance, types::TitanType};

pub struct EnemyDef {
    pub id: &'static str,
    pub name: &'static str,
    pub titan_type: TitanType,
    pub base_hp: f32,
    pub attack_damage: f32,
    pub bloodline_required: i32,
    pub reward_xp: u64,
    pub reward_gold: u32,
    pub drop_chance: f32,
}

static ENEMY_TABLE: &[EnemyDef] = &[
    EnemyDef {
        id: "LightTitan",
        name: "Light Titan",
        titan_type: TitanType::Warrior,
        base_hp: 150.0,
        attack_damage: 20.0,
        bloodline_required: 0,
        reward_xp: 50,
        reward_gold: 75,
        drop_chance: 0.15,
    },
    EnemyDef {
        id: "HeavyTitan",
        name: "Heavy Titan",
        titan_type: TitanType::Heavy,
        base_hp: 300.0,
        attack_damage: 35.0,
        bloodline_required: 0,
        reward_xp: 80,
        reward_gold: 120,
        drop_chance: 0.12,
    },
    EnemyDef {
        id: "DarkKnight",
        name: "Dark Knight",
        titan_type: TitanType::Warrior,
        base_hp: 200.0,
        attack_damage: 28.0,
        bloodline_required: 0,
        reward_xp: 65,
        reward_gold: 100,
        drop_chance: 0.13,
    },
    EnemyDef {
        id: "MageTitan",
        name: "Mage Titan",
        titan_type: TitanType::Mage,
        base_hp: 120.0,
        attack_damage: 45.0,
        bloodline_required: 0,
        reward_xp: 90,
        reward_gold: 130,
        drop_chance: 0.10,
    },
    EnemyDef {
        id: "GiantTitan",
        name: "Giant Titan",
        titan_type: TitanType::Heavy,
        base_hp: 500.0,
        attack_damage: 50.0,
        bloodline_required: 1,
        reward_xp: 150,
        reward_gold: 200,
        drop_chance: 0.08,
    },
    EnemyDef {
        id: "BloodSlave",
        name: "Blood Slave",
        titan_type: TitanType::Warrior,
        base_hp: 250.0,
        attack_damage: 32.0,
        bloodline_required: 1,
        reward_xp: 100,
        reward_gold: 150,
        drop_chance: 0.11,
    },
    EnemyDef {
        id: "KuroShino",
        name: "Kuro Shino",
        titan_type: TitanType::Warrior,
        base_hp: 180.0,
        attack_damage: 40.0,
        bloodline_required: 2,
        reward_xp: 120,
        reward_gold: 175,
        drop_chance: 0.09,
    },
    EnemyDef {
        id: "DeathlessSoldier",
        name: "Deathless Soldier",
        titan_type: TitanType::Warrior,
        base_hp: 400.0,
        attack_damage: 45.0,
        bloodline_required: 3,
        reward_xp: 200,
        reward_gold: 280,
        drop_chance: 0.07,
    },
    EnemyDef {
        id: "ElementalTitan",
        name: "Elemental Titan",
        titan_type: TitanType::Mage,
        base_hp: 350.0,
        attack_damage: 60.0,
        bloodline_required: 3,
        reward_xp: 220,
        reward_gold: 300,
        drop_chance: 0.07,
    },
    EnemyDef {
        id: "ShadowTitan",
        name: "Shadow Titan",
        titan_type: TitanType::Warrior,
        base_hp: 280.0,
        attack_damage: 55.0,
        bloodline_required: 4,
        reward_xp: 250,
        reward_gold: 350,
        drop_chance: 0.06,
    },
    EnemyDef {
        id: "TwinBladeTitan",
        name: "Twin Blade Titan",
        titan_type: TitanType::Warrior,
        base_hp: 320.0,
        attack_damage: 42.0,
        bloodline_required: 5,
        reward_xp: 280,
        reward_gold: 380,
        drop_chance: 0.06,
    },
    EnemyDef {
        id: "CrystalGolem",
        name: "Crystal Golem",
        titan_type: TitanType::Heavy,
        base_hp: 600.0,
        attack_damage: 65.0,
        bloodline_required: 6,
        reward_xp: 350,
        reward_gold: 450,
        drop_chance: 0.05,
    },
    EnemyDef {
        id: "QuantumSoldier",
        name: "Quantum Soldier",
        titan_type: TitanType::Warrior,
        base_hp: 450.0,
        attack_damage: 70.0,
        bloodline_required: 7,
        reward_xp: 400,
        reward_gold: 500,
        drop_chance: 0.05,
    },
    EnemyDef {
        id: "Kuero",
        name: "Kuero",
        titan_type: TitanType::Warrior,
        base_hp: 800.0,
        attack_damage: 80.0,
        bloodline_required: 10,
        reward_xp: 600,
        reward_gold: 800,
        drop_chance: 0.04,
    },
    EnemyDef {
        id: "CorruptedGalath",
        name: "Corrupted Galath",
        titan_type: TitanType::GodKing,
        base_hp: 2000.0,
        attack_damage: 120.0,
        bloodline_required: 20,
        reward_xp: 2000,
        reward_gold: 5000,
        drop_chance: 0.03,
    },
];

pub fn all_enemies() -> &'static [EnemyDef] {
    ENEMY_TABLE
}

pub fn enemy_by_id(id: &str) -> Option<&'static EnemyDef> {
    ENEMY_TABLE.iter().find(|e| e.id == id)
}

/// Spawn an EnemyInstance from the table, scaling HP by bloodline.
pub fn spawn_enemy(id: &str, bloodline: i32) -> Option<EnemyInstance> {
    let def = enemy_by_id(id)?;
    let hp_scale = 1.0 + bloodline.max(0) as f32 * 0.15;
    let scaled_hp = def.base_hp * hp_scale;
    Some(EnemyInstance {
        id: def.id.to_string(),
        name: def.name.to_string(),
        titan_type: def.titan_type.clone(),
        base_hp: scaled_hp,
        current_hp: scaled_hp,
        base_attack_damage: def.attack_damage,
        attack_damage: def.attack_damage,
        phase: 1,
        bloodline_required: def.bloodline_required,
        reward_xp: def.reward_xp,
        reward_gold: def.reward_gold,
        drop_chance: def.drop_chance,
        pending_attack: None,
        is_stunned: false,
        stun_turns_remaining: 0,
        shield_active: false,
        perfect_parries_received: 0,
    })
}

/// Returns ordered enemy IDs for an arena run at the given bloodline.
/// 3 regular enemies (scaled to bloodline) + CorruptedGalath.
pub fn arena_sequence(bloodline: i32) -> Vec<&'static str> {
    let mut available: Vec<&EnemyDef> = ENEMY_TABLE
        .iter()
        .filter(|e| e.bloodline_required <= bloodline && e.id != "CorruptedGalath")
        .collect();
    available.sort_by_key(|b| std::cmp::Reverse(b.bloodline_required));
    let mut seq: Vec<&'static str> = available.iter().take(3).map(|e| e.id).collect();
    seq.push("CorruptedGalath");
    seq
}
