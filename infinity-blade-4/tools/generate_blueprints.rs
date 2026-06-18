//! Infinity Blade 4 — Blueprint T3D Generator
//!
//! Standalone Rust program that uses `blueprint_core` to build all IB4 visual
//! scripting graphs and prints their UE4 T3D to stdout.
//!
//! # How to use
//! Copy this file into a binary crate that depends on `blueprint-rs/blueprint-core`:
//!
//! ```toml
//! # Cargo.toml
//! [package]
//! name   = "ib4-gen"
//! edition = "2021"
//!
//! [[bin]]
//! name = "generate_blueprints"
//! path = "src/generate_blueprints.rs"
//!
//! [dependencies]
//! blueprint_core = { path = "../../blueprint-rs/blueprint-core" }
//! ```
//!
//! Then run:
//! ```sh
//! cargo run --bin generate_blueprints > output.t3d
//! ```

use blueprint_core::ast::{BpNode, Pin};
use blueprint_core::builder::{BlueprintBuilder, VarType};
use blueprint_core::types::PinType;

// ---------------------------------------------------------------------------
// Helper: emit a titled separator to stdout between blueprints
// ---------------------------------------------------------------------------
fn section(title: &str) {
    tracing::info!("\n; ===== {} =====\n", title);
}

// ---------------------------------------------------------------------------
// BP_CombatChain
//
// Nodes:
//   1. K2Node_Event "BeginPlay"          → InitCombatComponent
//   2. K2Node_CallFunction InitCombatComponent
//   3. K2Node_Event "OnAttackInput"      (custom, EAttackDirection param)
//   4. K2Node_SwitchEnum                 → Overhead / Left / Right
//   5. K2Node_CallFunction PlayAttackMontage (Overhead)
//   6. K2Node_CallFunction PlayAttackMontage (Left)
//   7. K2Node_CallFunction PlayAttackMontage (Right)
//   8. K2Node_Event "OnParryInput"
//   9. K2Node_IfThenElse IsInParryWindow
//  10. K2Node_CallFunction ExecutePerfectParry
//  11. K2Node_CallFunction ExecuteNormalParry
//  12. K2Node_Event "OnEnemyHit"
//  13. K2Node_CallFunction IncrementCombo
//  14. K2Node_CallFunction ApplyDamage
// ---------------------------------------------------------------------------
fn build_combat_chain() -> String {
    let mut builder = BlueprintBuilder::new("BP_CombatChain", "Actor");

    // Variables
    builder.add_variable_mut("ComboCount", VarType::Int, Some("0".into()));
    builder.add_variable_mut("IsInParryWindow", VarType::Bool, Some("False".into()));
    builder.add_variable_mut("CurrentAttackDirection", VarType::String, Some("Overhead".into()));

    // --- BeginPlay → InitCombatComponent ---
    let begin_play = builder.begin_play_node();
    // add_call_function is not a builder method; we compose nodes manually via
    // EventBodyBuilder indirection. For the consuming-style equivalent we use
    // begin_play(...) with a closure. Here we use the mutable style and call
    // raw BpNode helpers exposed through the public AST types.

    // We record handles so we can wire pins afterward.
    let init_combat = {
        let pos = blueprint_core::types::NodePos::new(300, 0);
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_InitCombat")
            .at(pos.x, pos.y)
            .with_property("FunctionReference", "(MemberName=\"InitCombatComponent\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"));
        builder.blueprint_push_node_pub(node)
    };
    builder.exec_connect(&begin_play, &init_combat);

    // --- OnAttackInput (custom event) → SwitchEnum ---
    let on_attack = builder.custom_event_node("OnAttackInput");
    // SwitchEnum node
    let switch_attack = {
        let pos = blueprint_core::types::NodePos::new(300, 300);
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_SwitchEnum", "K2Node_SwitchEnum_AttackDir")
            .at(pos.x, pos.y)
            .with_property("Enum", "/Game/IB4/Enums/EAttackDirection.EAttackDirection")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("Overhead"))
            .with_pin(Pin::exec_output("Left"))
            .with_pin(Pin::exec_output("Right"))
            .with_pin(Pin::data_input("Selection", PinType::byte()));
        builder.blueprint_push_node_pub(node)
    };
    builder.exec_connect(&on_attack, &switch_attack);

    // Three attack montage branches
    let montage_overhead = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_MontageOverhead")
            .at(600, 200)
            .with_property("FunctionReference", "(MemberName=\"PlayAttackMontage\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"))
            .with_pin(Pin::data_input("Direction", PinType::byte()).with_default("Overhead"));
        builder.blueprint_push_node_pub(node)
    };
    let montage_left = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_MontageLeft")
            .at(600, 400)
            .with_property("FunctionReference", "(MemberName=\"PlayAttackMontage\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"))
            .with_pin(Pin::data_input("Direction", PinType::byte()).with_default("Left"));
        builder.blueprint_push_node_pub(node)
    };
    let montage_right = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_MontageRight")
            .at(600, 600)
            .with_property("FunctionReference", "(MemberName=\"PlayAttackMontage\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"))
            .with_pin(Pin::data_input("Direction", PinType::byte()).with_default("Right"));
        builder.blueprint_push_node_pub(node)
    };
    builder.connect(&switch_attack, "Overhead", &montage_overhead, "execute");
    builder.connect(&switch_attack, "Left",     &montage_left,     "execute");
    builder.connect(&switch_attack, "Right",    &montage_right,    "execute");

    // --- OnParryInput → Branch → ExecutePerfectParry / ExecuteNormalParry ---
    let on_parry = builder.custom_event_node("OnParryInput");
    let parry_branch = builder.branch_node();
    builder.exec_connect(&on_parry, &parry_branch);

    let perfect_parry = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_PerfectParry")
            .at(900, 750)
            .with_property("FunctionReference", "(MemberName=\"ExecutePerfectParry\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"));
        builder.blueprint_push_node_pub(node)
    };
    let normal_parry = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_NormalParry")
            .at(900, 900)
            .with_property("FunctionReference", "(MemberName=\"ExecuteNormalParry\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"));
        builder.blueprint_push_node_pub(node)
    };
    builder.connect(&parry_branch, "then", &perfect_parry, "execute");
    builder.connect(&parry_branch, "else", &normal_parry,  "execute");

    // --- OnEnemyHit → IncrementCombo → ApplyDamage ---
    let on_hit = builder.custom_event_node("OnEnemyHit");
    let increment_combo = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_IncrementCombo")
            .at(300, 1050)
            .with_property("FunctionReference", "(MemberName=\"IncrementCombo\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"));
        builder.blueprint_push_node_pub(node)
    };
    let apply_damage = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_ApplyDamage")
            .at(600, 1050)
            .with_property("FunctionReference", "(MemberName=\"ApplyDamage\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"))
            .with_pin(Pin::data_input("DamageAmount", PinType::float()).with_default("50.0"));
        builder.blueprint_push_node_pub(node)
    };
    builder.exec_connect(&on_hit,        &increment_combo);
    builder.exec_connect(&increment_combo, &apply_damage);

    builder.to_t3d()
}

// ---------------------------------------------------------------------------
// BP_EquipmentPickup
// ---------------------------------------------------------------------------
fn build_equipment_pickup() -> String {
    let mut builder = BlueprintBuilder::new("BP_EquipmentPickup", "Actor");

    builder.add_variable_mut("ItemData", VarType::String, None);
    builder.add_variable_mut("bIsPickedUp", VarType::Bool, Some("False".into()));

    // BeginOverlap → IsPlayerCharacter branch → EquipItem → PlayPickupSound → DestroyActor
    let on_overlap = builder.custom_event_node("BeginOverlap");
    let player_check = builder.branch_node();
    builder.exec_connect(&on_overlap, &player_check);

    let equip_item = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_EquipItem")
            .at(600, 0)
            .with_property("FunctionReference", "(MemberName=\"EquipItem\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"));
        builder.blueprint_push_node_pub(node)
    };
    let play_sound = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_PlayPickupSound")
            .at(900, 0)
            .with_property("FunctionReference", "(MemberName=\"PlayPickupSound\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"));
        builder.blueprint_push_node_pub(node)
    };
    let destroy_actor = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_DestroyActor")
            .at(1200, 0)
            .with_property("FunctionReference", "(MemberParent=Class'/Script/Engine.Actor',MemberName=\"K2_DestroyActor\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"));
        builder.blueprint_push_node_pub(node)
    };
    builder.connect(&player_check, "then", &equip_item,    "execute");
    builder.exec_connect(&equip_item,    &play_sound);
    builder.exec_connect(&play_sound,    &destroy_actor);

    // OnEquipPressed → OpenEquipmentMenu
    let on_equip_pressed = builder.custom_event_node("OnEquipPressed");
    let open_menu = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_OpenEquipmentMenu")
            .at(300, 300)
            .with_property("FunctionReference", "(MemberName=\"OpenEquipmentMenu\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"));
        builder.blueprint_push_node_pub(node)
    };
    builder.exec_connect(&on_equip_pressed, &open_menu);

    builder.to_t3d()
}

// ---------------------------------------------------------------------------
// BP_BloodlineLevelUp
// ---------------------------------------------------------------------------
fn build_bloodline_level_up() -> String {
    let mut builder = BlueprintBuilder::new("BP_BloodlineLevelUp", "Actor");

    builder.add_variable_mut("CurrentXP",   VarType::Float, Some("0.0".into()));
    builder.add_variable_mut("XPThreshold", VarType::Float, Some("1000.0".into()));
    builder.add_variable_mut("BloodlineLevel", VarType::Int, Some("1".into()));

    // OnXPGained (float XP) → AddXP → XPThreshold check → LevelUpBloodline → ShowLevelUpUI → GrantBloodlinePerk
    let on_xp = builder.custom_event_node("OnXPGained");
    let add_xp = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_AddXP")
            .at(300, 0)
            .with_property("FunctionReference", "(MemberName=\"AddXP\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"))
            .with_pin(Pin::data_input("XP", PinType::float()));
        builder.blueprint_push_node_pub(node)
    };
    let xp_check = builder.branch_node();
    builder.exec_connect(&on_xp,  &add_xp);
    builder.exec_connect(&add_xp, &xp_check);

    let level_up = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_LevelUpBloodline")
            .at(900, 0)
            .with_property("FunctionReference", "(MemberName=\"LevelUpBloodline\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"));
        builder.blueprint_push_node_pub(node)
    };
    let show_ui = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_ShowLevelUpUI")
            .at(1200, 0)
            .with_property("FunctionReference", "(MemberName=\"ShowLevelUpUI\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"));
        builder.blueprint_push_node_pub(node)
    };
    let grant_perk = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_GrantBloodlinePerk")
            .at(1500, 0)
            .with_property("FunctionReference", "(MemberName=\"GrantBloodlinePerk\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"));
        builder.blueprint_push_node_pub(node)
    };
    builder.connect(&xp_check, "then", &level_up, "execute");
    builder.exec_connect(&level_up,  &show_ui);
    builder.exec_connect(&show_ui,   &grant_perk);

    // OnDeath → SaveBloodlineProgress → TriggerRebirth
    let on_death = builder.custom_event_node("OnDeath");
    let save_progress = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_SaveBloodlineProgress")
            .at(300, 450)
            .with_property("FunctionReference", "(MemberName=\"SaveBloodlineProgress\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"));
        builder.blueprint_push_node_pub(node)
    };
    let trigger_rebirth = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_TriggerRebirth")
            .at(600, 450)
            .with_property("FunctionReference", "(MemberName=\"TriggerRebirth\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"));
        builder.blueprint_push_node_pub(node)
    };
    builder.exec_connect(&on_death,      &save_progress);
    builder.exec_connect(&save_progress, &trigger_rebirth);

    builder.to_t3d()
}

// ---------------------------------------------------------------------------
// BP_TitanBossFight
// ---------------------------------------------------------------------------
fn build_titan_boss_fight() -> String {
    let mut builder = BlueprintBuilder::new("BP_TitanBossFight", "Actor");

    builder.add_variable_mut("CurrentPhase",    VarType::Int,   Some("1".into()));
    builder.add_variable_mut("HealthPercent",   VarType::Float, Some("1.0".into()));
    builder.add_variable_mut("ShieldBreakReady",VarType::Bool,  Some("False".into()));

    // BeginPlay → EnterPhase1
    let begin_play = builder.begin_play_node();
    let enter_p1 = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_EnterPhase1")
            .at(300, 0)
            .with_property("FunctionReference", "(MemberName=\"EnterPhase1\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"));
        builder.blueprint_push_node_pub(node)
    };
    builder.exec_connect(&begin_play, &enter_p1);

    // OnHealthChanged → SwitchFloat → EnterPhase2 / EnterPhase3
    let on_health = builder.custom_event_node("OnHealthChanged");
    let switch_float = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_SwitchFloat", "K2Node_SwitchFloat_PhaseGate")
            .at(300, 300)
            .with_property("PinNames", "(\"Phase2Threshold\",\"Phase3Threshold\")")
            .with_property("PinValues", "(0.600000,0.300000)")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::data_input("Selection", PinType::float()))
            .with_pin(Pin::exec_output("Phase2Threshold"))
            .with_pin(Pin::exec_output("Phase3Threshold"))
            .with_pin(Pin::exec_output("Default"));
        builder.blueprint_push_node_pub(node)
    };
    builder.exec_connect(&on_health, &switch_float);

    let enter_p2 = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_EnterPhase2")
            .at(600, 150)
            .with_property("FunctionReference", "(MemberName=\"EnterPhase2\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"));
        builder.blueprint_push_node_pub(node)
    };
    let enter_p3 = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_EnterPhase3")
            .at(600, 450)
            .with_property("FunctionReference", "(MemberName=\"EnterPhase3\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"));
        builder.blueprint_push_node_pub(node)
    };
    builder.connect(&switch_float, "Phase2Threshold", &enter_p2, "execute");
    builder.connect(&switch_float, "Phase3Threshold", &enter_p3, "execute");

    // OnPhase2Enter → EnableShieldBreak → SpawnMinions
    let on_p2_enter = builder.custom_event_node("OnPhase2Enter");
    let enable_shield = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_EnableShieldBreak")
            .at(300, 750)
            .with_property("FunctionReference", "(MemberName=\"EnableShieldBreak\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"));
        builder.blueprint_push_node_pub(node)
    };
    let spawn_minions = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_SpawnMinions")
            .at(600, 750)
            .with_property("FunctionReference", "(MemberName=\"SpawnMinions\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"))
            .with_pin(Pin::data_input("Count", PinType::int()).with_default("3"));
        builder.blueprint_push_node_pub(node)
    };
    builder.exec_connect(&on_p2_enter,   &enable_shield);
    builder.exec_connect(&enable_shield, &spawn_minions);

    // OnPhase3Enter (custom) → CameraShake + PlayBossRoarSound
    let on_p3_enter = builder.custom_event_node("OnPhase3Enter");
    let boss_roar = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_PlayBossRoar")
            .at(300, 1050)
            .with_property("FunctionReference", "(MemberName=\"PlayBossRoarSound\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"));
        builder.blueprint_push_node_pub(node)
    };
    builder.exec_connect(&on_p3_enter, &boss_roar);

    builder.to_t3d()
}

// ---------------------------------------------------------------------------
// BP_MagicCasting
// ---------------------------------------------------------------------------
fn build_magic_casting() -> String {
    let mut builder = BlueprintBuilder::new("BP_MagicCasting", "Actor");

    builder.add_variable_mut("MagicPoints",    VarType::Float, Some("100.0".into()));
    builder.add_variable_mut("MaxMagicPoints", VarType::Float, Some("100.0".into()));
    builder.add_variable_mut("CurrentMagicType", VarType::String, Some("Fire".into()));

    // OnMagicInput (EMagicType) → HasEnoughMagic branch → SwitchEnum → Fire/Lightning/Ice
    let on_magic = builder.custom_event_node("OnMagicInput");
    let magic_check = builder.branch_node();
    builder.exec_connect(&on_magic, &magic_check);

    let switch_magic = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_SwitchEnum", "K2Node_SwitchEnum_MagicType")
            .at(600, 0)
            .with_property("Enum", "/Game/IB4/Enums/EMagicType.EMagicType")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("Fire"))
            .with_pin(Pin::exec_output("Lightning"))
            .with_pin(Pin::exec_output("Ice"))
            .with_pin(Pin::data_input("Selection", PinType::byte()));
        builder.blueprint_push_node_pub(node)
    };
    builder.connect(&magic_check, "then", &switch_magic, "execute");

    // Fire spell
    let cast_fire = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_CastFireSpell")
            .at(900, -150)
            .with_property("FunctionReference", "(MemberName=\"CastFireSpell\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"))
            .with_pin(Pin::data_input("Damage", PinType::float()).with_default("75.0"));
        builder.blueprint_push_node_pub(node)
    };
    // Lightning spell
    let cast_lightning = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_CastLightningSpell")
            .at(900, 150)
            .with_property("FunctionReference", "(MemberName=\"CastLightningSpell\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"))
            .with_pin(Pin::data_input("Damage", PinType::float()).with_default("60.0"));
        builder.blueprint_push_node_pub(node)
    };
    // Ice spell
    let cast_ice = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_CastIceSpell")
            .at(900, 450)
            .with_property("FunctionReference", "(MemberName=\"CastIceSpell\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"))
            .with_pin(Pin::data_input("Damage", PinType::float()).with_default("55.0"));
        builder.blueprint_push_node_pub(node)
    };
    builder.connect(&switch_magic, "Fire",      &cast_fire,      "execute");
    builder.connect(&switch_magic, "Lightning", &cast_lightning, "execute");
    builder.connect(&switch_magic, "Ice",       &cast_ice,       "execute");

    // OnMagicHit → ApplyStatusEffect (branch on magic type)
    let on_magic_hit = builder.custom_event_node("OnMagicHit");
    let status_branch = builder.branch_node();
    builder.exec_connect(&on_magic_hit, &status_branch);

    let apply_burn = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_ApplyBurn")
            .at(600, 750)
            .with_property("FunctionReference", "(MemberName=\"ApplyBurnEffect\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"))
            .with_pin(Pin::data_input("Duration", PinType::float()).with_default("3.0"));
        builder.blueprint_push_node_pub(node)
    };
    let apply_freeze = {
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", "K2Node_CallFunction_ApplyFreeze")
            .at(600, 900)
            .with_property("FunctionReference", "(MemberName=\"ApplyFreezeEffect\")")
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"))
            .with_pin(Pin::data_input("Duration", PinType::float()).with_default("2.0"));
        builder.blueprint_push_node_pub(node)
    };
    builder.connect(&status_branch, "then", &apply_burn,   "execute");
    builder.connect(&status_branch, "else", &apply_freeze, "execute");

    builder.to_t3d()
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------
fn main() {
    section("BP_CombatChain");
    tracing::info!("{}", build_combat_chain());

    section("BP_EquipmentPickup");
    tracing::info!("{}", build_equipment_pickup());

    section("BP_BloodlineLevelUp");
    tracing::info!("{}", build_bloodline_level_up());

    section("BP_TitanBossFight");
    tracing::info!("{}", build_titan_boss_fight());

    section("BP_MagicCasting");
    tracing::info!("{}", build_magic_casting());
}
