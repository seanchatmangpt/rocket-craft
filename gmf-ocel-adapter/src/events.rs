use serde::{Deserialize, Serialize};

use nexus_tps::{JidokaHalt, MechTpsReceipt, PartSlot, PartStateVector};

use crate::ocel::{OcelEvent, OcelObjectRef};

/// Every GMF physical event maps to exactly one activity name in OCEL.
/// The name is the canonical cross-layer identifier — the MUD command,
/// the UE4 event, and the wasm4pm process mining query all use this string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GmfEventKind {
    // ── Part manufacturing ──────────────────────────────────────────────────
    PartGenerated,
    PartValidated,
    PartAdmitted,
    PartRefused,
    PartAttached,
    PartSalvaged,
    PartRemanufactured,

    // ── Socket operations ───────────────────────────────────────────────────
    SocketMatched,
    SocketMismatch,
    SocketStressWarning,
    SocketMateForced,

    // ── Assembly ────────────────────────────────────────────────────────────
    AssemblyStarted,
    AssemblyAdmitted,
    AssemblyRefused,
    AssemblyReceipted,

    // ── Jidoka / Andon (Toyota Production System) ──────────────────────────
    JidokaHaltEmitted,
    AndonOpened,
    AndonClosed,
    ReworkRequested,

    // ── Receipt chain ───────────────────────────────────────────────────────
    ReceiptIssued,
    ReceiptVerified,
    ReceiptRefused,

    // ── Heat / thermal (MechWarrior-inspired) ──────────────────────────────
    ThermalLoadObserved,
    ThermalOverload,
    ThermalCoolingApplied,
    WeaponGroupFired,
    HeatSafetyOverridden,

    // ── Location-based damage (BattleTech-inspired) ─────────────────────────
    DamageObserved,
    PartDestroyed,
    JointDegraded,
    ArmorPenetrated,

    // ── Repair / refit lifecycle (BATTLETECH-inspired) ─────────────────────
    MaintenanceRequested,
    DiagnosisCompleted,
    RepairQuoteIssued,
    RepairCompleted,
    RefitCompleted,
    ReturnToServiceAdmitted,

    // ── Pilot / cockpit (Titanfall / Steel Battalion inspired) ─────────────
    PilotEntered,
    PilotExited,
    PilotEjected,
    CockpitSealed,
    PowerOnSequenceStarted,
    JointsUnlocked,
    ShutdownSequenceStarted,

    // ── Battery / field support (Titanfall rodeo-inspired) ─────────────────
    BatteryExtracted,
    BatteryInserted,
    FieldRepairApplied,
    MechRecovered,

    // ── Zone / circuit breaker health ──────────────────────────────────────
    ZoneCircuitOpened,
    ZoneCircuitClosed,
    ZoneCircuitHalfOpen,
    ZoneHealthDegraded,

    // ── Infrastructure (Into the Breach-inspired) ──────────────────────────
    EdenGridHealthChanged,
    InfrastructureAttacked,
    InfrastructureRepaired,
    CivilizationHealthUpdated,

    // ── Mission (MechWarrior 5 mercenary company-inspired) ─────────────────
    MissionContractAccepted,
    MissionCompleted,
    LanceDeployed,
    LanceRecalled,

    // ── RL / autonomic ─────────────────────────────────────────────────────
    MapeKCycleExecuted,
    RlActionSelected,
    WeibullRulUpdated,
}

impl GmfEventKind {
    pub fn activity_name(&self) -> &'static str {
        match self {
            Self::PartGenerated => "part.generated",
            Self::PartValidated => "part.validated",
            Self::PartAdmitted => "part.admitted",
            Self::PartRefused => "part.refused",
            Self::PartAttached => "part.attached",
            Self::PartSalvaged => "part.salvaged",
            Self::PartRemanufactured => "part.remanufactured",
            Self::SocketMatched => "socket.matched",
            Self::SocketMismatch => "socket.mismatch",
            Self::SocketStressWarning => "socket.stress_warning",
            Self::SocketMateForced => "socket.mate_forced",
            Self::AssemblyStarted => "assembly.started",
            Self::AssemblyAdmitted => "assembly.admitted",
            Self::AssemblyRefused => "assembly.refused",
            Self::AssemblyReceipted => "assembly.receipted",
            Self::JidokaHaltEmitted => "jidoka.halt",
            Self::AndonOpened => "andon.opened",
            Self::AndonClosed => "andon.closed",
            Self::ReworkRequested => "rework.requested",
            Self::ReceiptIssued => "receipt.issued",
            Self::ReceiptVerified => "receipt.verified",
            Self::ReceiptRefused => "receipt.refused",
            Self::ThermalLoadObserved => "thermal.load_observed",
            Self::ThermalOverload => "thermal.overload",
            Self::ThermalCoolingApplied => "thermal.cooling_applied",
            Self::WeaponGroupFired => "weapon.group_fired",
            Self::HeatSafetyOverridden => "heat_safety.overridden",
            Self::DamageObserved => "damage.observed",
            Self::PartDestroyed => "part.destroyed",
            Self::JointDegraded => "joint.degraded",
            Self::ArmorPenetrated => "armor.penetrated",
            Self::MaintenanceRequested => "maintenance.requested",
            Self::DiagnosisCompleted => "diagnosis.completed",
            Self::RepairQuoteIssued => "repair.quote_issued",
            Self::RepairCompleted => "repair.completed",
            Self::RefitCompleted => "refit.completed",
            Self::ReturnToServiceAdmitted => "return_to_service.admitted",
            Self::PilotEntered => "pilot.entered",
            Self::PilotExited => "pilot.exited",
            Self::PilotEjected => "pilot.ejected",
            Self::CockpitSealed => "cockpit.sealed",
            Self::PowerOnSequenceStarted => "power_on.started",
            Self::JointsUnlocked => "joints.unlocked",
            Self::ShutdownSequenceStarted => "shutdown.started",
            Self::BatteryExtracted => "battery.extracted",
            Self::BatteryInserted => "battery.inserted",
            Self::FieldRepairApplied => "field_repair.applied",
            Self::MechRecovered => "mech.recovered",
            Self::ZoneCircuitOpened => "zone.circuit_opened",
            Self::ZoneCircuitClosed => "zone.circuit_closed",
            Self::ZoneCircuitHalfOpen => "zone.circuit_half_open",
            Self::ZoneHealthDegraded => "zone.health_degraded",
            Self::EdenGridHealthChanged => "eden.grid_health_changed",
            Self::InfrastructureAttacked => "infrastructure.attacked",
            Self::InfrastructureRepaired => "infrastructure.repaired",
            Self::CivilizationHealthUpdated => "civilization.health_updated",
            Self::MissionContractAccepted => "mission.contract_accepted",
            Self::MissionCompleted => "mission.completed",
            Self::LanceDeployed => "lance.deployed",
            Self::LanceRecalled => "lance.recalled",
            Self::MapeKCycleExecuted => "mape_k.cycle_executed",
            Self::RlActionSelected => "rl.action_selected",
            Self::WeibullRulUpdated => "weibull.rul_updated",
        }
    }
}

/// A GmfEvent is the bridge type: a physical event with typed kind and
/// object refs, ready to be serialized into OCEL 2.0 JSON for wasm4pm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GmfEvent {
    pub id: String,
    pub kind: GmfEventKind,
    pub timestamp_ms: u64,
    pub object_refs: Vec<OcelObjectRef>,
    pub attributes: serde_json::Map<String, serde_json::Value>,
}

impl GmfEvent {
    pub fn new(id: impl Into<String>, kind: GmfEventKind, timestamp_ms: u64) -> Self {
        Self {
            id: id.into(),
            kind,
            timestamp_ms,
            object_refs: Vec::new(),
            attributes: serde_json::Map::new(),
        }
    }

    pub fn with_object(mut self, object_id: impl Into<String>, qualifier: impl Into<String>) -> Self {
        self.object_refs.push(OcelObjectRef {
            object_id: object_id.into(),
            qualifier: qualifier.into(),
        });
        self
    }

    pub fn with_attr(mut self, key: impl Into<String>, val: impl Into<serde_json::Value>) -> Self {
        self.attributes.insert(key.into(), val.into());
        self
    }

    pub fn into_ocel_event(self) -> OcelEvent {
        OcelEvent {
            id: self.id,
            activity: self.kind.activity_name().to_string(),
            timestamp_ms: self.timestamp_ms,
            object_refs: self.object_refs,
            attributes: self.attributes,
        }
    }
}

// ── Conversion from nexus-tps domain types ──────────────────────────────────

/// Convert a JidokaHalt into the appropriate GmfEvent for OCEL emission.
pub fn jidoka_halt_to_event(halt: &JidokaHalt, event_id: impl Into<String>, ts: u64) -> GmfEvent {
    let halt_code = match halt {
        JidokaHalt::SocketMismatch { .. } => "socket_mismatch",
        JidokaHalt::CollisionVolumeIntersects { .. } => "collision_volume_intersects",
        JidokaHalt::MassExceedsFrameCapacity { .. } => "mass_exceeds_frame_capacity",
        JidokaHalt::MotionBoundsViolated { .. } => "motion_bounds_violated",
    };
    GmfEvent::new(event_id, GmfEventKind::JidokaHaltEmitted, ts)
        .with_attr("halt_code", halt_code)
        .with_attr("halt_description", halt.to_string())
}

/// Convert a successful part generation into an OCEL event.
pub fn part_generated_event(
    psv: &PartStateVector,
    part_object_id: impl Into<String>,
    zone_object_id: impl Into<String>,
    event_id: impl Into<String>,
    ts: u64,
) -> GmfEvent {
    GmfEvent::new(event_id, GmfEventKind::PartGenerated, ts)
        .with_object(part_object_id, "generated_part")
        .with_object(zone_object_id, "zone")
        .with_attr("part_slot", format!("{:?}", psv.part_slot))
        .with_attr("civilization_id", psv.civilization_id as i64)
        .with_attr("frame_id", psv.frame_id as i64)
        .with_attr("armor_profile", psv.armor_profile as f64)
        .with_attr("joint_profile", psv.joint_profile as f64)
        .with_attr("mass_profile", psv.mass_profile as f64)
        .with_attr("weapon_profile", psv.weapon_profile as f64)
        .with_attr("motion_profile", psv.motion_profile as f64)
}

/// Convert a MechTpsReceipt (full mech assembly) into an OCEL receipt.issued event.
pub fn assembly_receipt_event(
    receipt: &MechTpsReceipt,
    assembly_object_id: impl Into<String>,
    event_id: impl Into<String>,
    ts: u64,
) -> GmfEvent {
    GmfEvent::new(event_id, GmfEventKind::ReceiptIssued, ts)
        .with_object(assembly_object_id, "admitted_assembly")
        .with_attr("receipt_hash", receipt.receipt_hash.clone())
        .with_attr("lineage_hash", receipt.lineage_hash.clone())
        .with_attr("final_decision", receipt.final_decision.clone())
        .with_attr("component_count", receipt.component_count as i64)
        .with_attr("gates_passed", receipt.passed_gates.len() as i64)
}

/// Emit a thermal overload event (MechWarrior heat → wasm4pm HealthStatus).
pub fn thermal_overload_event(
    part_object_id: impl Into<String>,
    zone_object_id: impl Into<String>,
    thermal_load: f32,
    event_id: impl Into<String>,
    ts: u64,
) -> GmfEvent {
    GmfEvent::new(event_id, GmfEventKind::ThermalOverload, ts)
        .with_object(part_object_id, "overheated_part")
        .with_object(zone_object_id, "zone")
        .with_attr("thermal_load", thermal_load as f64)
        .with_attr("threshold_exceeded", true)
}

/// Emit a damage observed event (BattleTech location-based damage → OCEL).
pub fn damage_observed_event(
    part_slot: PartSlot,
    part_object_id: impl Into<String>,
    attacker_object_id: impl Into<String>,
    damage_amount: f32,
    event_id: impl Into<String>,
    ts: u64,
) -> GmfEvent {
    GmfEvent::new(event_id, GmfEventKind::DamageObserved, ts)
        .with_object(part_object_id, "damaged_part")
        .with_object(attacker_object_id, "attacker")
        .with_attr("part_slot", format!("{:?}", part_slot))
        .with_attr("damage_amount", damage_amount as f64)
}

/// Emit a pilot-entered event (Titanfall pilot/Titan bond).
pub fn pilot_entered_event(
    pilot_object_id: impl Into<String>,
    cockpit_object_id: impl Into<String>,
    mech_object_id: impl Into<String>,
    event_id: impl Into<String>,
    ts: u64,
) -> GmfEvent {
    GmfEvent::new(event_id, GmfEventKind::PilotEntered, ts)
        .with_object(pilot_object_id, "pilot")
        .with_object(cockpit_object_id, "cockpit")
        .with_object(mech_object_id, "mech")
}

/// Emit a battery inserted event (Titanfall rodeo / field support).
pub fn battery_inserted_event(
    battery_object_id: impl Into<String>,
    mech_object_id: impl Into<String>,
    field_agent_object_id: impl Into<String>,
    shield_restore: f32,
    event_id: impl Into<String>,
    ts: u64,
) -> GmfEvent {
    GmfEvent::new(event_id, GmfEventKind::BatteryInserted, ts)
        .with_object(battery_object_id, "battery")
        .with_object(mech_object_id, "mech")
        .with_object(field_agent_object_id, "field_agent")
        .with_attr("shield_restore", shield_restore as f64)
}

/// Emit an infrastructure health changed event (Into the Breach civilization defense).
pub fn eden_grid_health_changed(
    node_object_id: impl Into<String>,
    health_level: u8,
    delta: i8,
    event_id: impl Into<String>,
    ts: u64,
) -> GmfEvent {
    GmfEvent::new(event_id, GmfEventKind::EdenGridHealthChanged, ts)
        .with_object(node_object_id, "grid_node")
        .with_attr("health_level", health_level as i64)
        .with_attr("delta", delta as i64)
        .with_attr("civilization_standing", if health_level > 0 { "standing" } else { "fallen" })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── GmfEventKind ─────────────────────────────────────────────────────────

    #[test]
    fn activity_names_nonempty_for_key_variants() {
        let kinds = [
            GmfEventKind::PartGenerated,
            GmfEventKind::AssemblyReceipted,
            GmfEventKind::JidokaHaltEmitted,
            GmfEventKind::ThermalOverload,
            GmfEventKind::PilotEntered,
            GmfEventKind::MapeKCycleExecuted,
        ];
        for k in &kinds {
            let name = k.activity_name();
            assert!(!name.is_empty(), "{:?}.activity_name() is empty", k);
            assert!(name.contains('.'), "activity name '{name}' should be namespaced");
        }
    }

    // ── GmfEvent builder ─────────────────────────────────────────────────────

    #[test]
    fn gmf_event_new_sets_fields() {
        let ev = GmfEvent::new("ev-001", GmfEventKind::PartGenerated, 5000);
        assert_eq!(ev.id, "ev-001");
        assert_eq!(ev.timestamp_ms, 5000);
        assert!(ev.object_refs.is_empty());
        assert!(ev.attributes.is_empty());
    }

    #[test]
    fn with_object_accumulates_refs() {
        let ev = GmfEvent::new("ev-1", GmfEventKind::SocketMatched, 0)
            .with_object("part-a", "source")
            .with_object("part-b", "target");
        assert_eq!(ev.object_refs.len(), 2);
        assert_eq!(ev.object_refs[0].qualifier, "source");
        assert_eq!(ev.object_refs[1].qualifier, "target");
    }

    #[test]
    fn with_attr_inserts_key_value() {
        let ev = GmfEvent::new("ev-2", GmfEventKind::ThermalOverload, 0)
            .with_attr("heat_level", 95_i64);
        assert!(ev.attributes.contains_key("heat_level"));
    }

    #[test]
    fn into_ocel_event_sets_activity_from_kind() {
        let ev = GmfEvent::new("ev-3", GmfEventKind::ReceiptIssued, 1234)
            .into_ocel_event();
        assert_eq!(ev.id, "ev-3");
        assert_eq!(ev.activity, "receipt.issued");
        assert_eq!(ev.timestamp_ms, 1234);
    }

    // ── factory functions ─────────────────────────────────────────────────────

    #[test]
    fn thermal_overload_event_links_part_and_zone() {
        let ev = thermal_overload_event("part-1", "zone-head", 90.0, "ev-th-1", 100);
        assert_eq!(ev.id, "ev-th-1");
        assert!(ev.attributes.contains_key("thermal_load"));
        assert_eq!(ev.object_refs.len(), 2);
    }

    #[test]
    fn damage_observed_event_sets_part_slot() {
        let ev = damage_observed_event(PartSlot::ArmL, "part-1", "attacker-1", 30.0, "ev-dmg-1", 200);
        assert_eq!(ev.id, "ev-dmg-1");
        assert!(ev.attributes.contains_key("part_slot"));
        assert!(ev.attributes.contains_key("damage_amount"));
    }

    #[test]
    fn pilot_entered_event_links_pilot_cockpit_mech() {
        let ev = pilot_entered_event("pilot-1", "cockpit-1", "mech-1", "ev-pe-1", 300);
        assert_eq!(ev.object_refs.len(), 3);
    }

    #[test]
    fn battery_inserted_event_links_three_objects() {
        let ev = battery_inserted_event("bat-1", "mech-1", "agent-1", 0.5, "ev-bi-1", 400);
        assert_eq!(ev.object_refs.len(), 3);
        assert!(ev.attributes.contains_key("shield_restore"));
    }

    #[test]
    fn eden_grid_health_changed_reflects_standing() {
        let ev = eden_grid_health_changed("node-1", 5, 1, "ev-eg-1", 500);
        let standing = ev.attributes.get("civilization_standing")
            .and_then(|v| v.as_str()).unwrap_or("");
        assert_eq!(standing, "standing");

        let fallen = eden_grid_health_changed("node-1", 0, -3, "ev-eg-2", 600);
        let s2 = fallen.attributes.get("civilization_standing")
            .and_then(|v| v.as_str()).unwrap_or("");
        assert_eq!(s2, "fallen");
    }
}
