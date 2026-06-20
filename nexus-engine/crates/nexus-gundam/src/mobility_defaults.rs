// Mobility Default implementations — lore-grounded from UC Gundam official tech manuals.
// Physical quantities annotated with QUDT 2.1 units (https://qudt.org/).
// Source of truth: nexus-engine/.specify/schema/gundam_nexus.ttl
// ggen seed: nexus-engine/ggen.toml rule "generate-mobility-specs"

use crate::mech_primitives::{Mobility, Walking, Flight, Hover, Aquatic, Space, AABB};

impl Default for Mobility {
    /// Canonical baseline: RX-78-2 Gundam (UC 0079) bipedal locomotion spec.
    /// mass: 60,000 kg (QUDT unit:KiloGM) — dry weight per MG 1/100 Ver.3.0 tech manual
    /// max_speed: 45.83 m/s = 165 km/h (QUDT unit:M-PER-SEC)
    fn default() -> Self {
        Self {
            id: "rx-78-2-mobility-baseline".to_string(),
            mass: 60_000.0,
            occupancy: AABB::default(),
            clearance: AABB::default(),
            load_capacity: 55_000.0,
            max_speed: 45.83,
        }
    }
}

impl Default for Walking {
    /// RX-78-2 Gundam (UC 0079): bipedal, 2 legs, 165 km/h (45.83 m/s) ground speed.
    /// Source: Bandai MG 1/100 RX-78-2 Gundam Ver.3.0 instruction manual.
    fn default() -> Self {
        Self {
            physical: Mobility::default(),
            leg_count: 2,
        }
    }
}

impl Default for Flight {
    /// Wing Gundam (XXXG-01W, AC 195): 7,150 kg, Mach 1+ (≥340 m/s), 14.4m wing span.
    /// Source: Gundam Wing MG 1/100 Wing Gundam instruction manual + Endless Waltz specs.
    fn default() -> Self {
        Self {
            physical: Mobility {
                id: "wing-gundam-flight-baseline".to_string(),
                mass: 7_150.0,
                occupancy: AABB::default(),
                clearance: AABB::default(),
                load_capacity: 4_000.0,
                max_speed: 340.0,
            },
            wing_span: 14.4,
        }
    }
}

impl Default for Hover {
    /// The O (PMX-003, UC 0087): 58,800 kg, ~150 km/h (41.67 m/s), 1.2m ground clearance.
    /// Source: Z Gundam setting notes / Zeta-era design documentation.
    fn default() -> Self {
        Self {
            physical: Mobility {
                id: "the-o-hover-baseline".to_string(),
                mass: 58_800.0,
                occupancy: AABB::default(),
                clearance: AABB::default(),
                load_capacity: 50_000.0,
                max_speed: 41.67,
            },
            ground_clearance: 1.2,
        }
    }
}

impl Default for Aquatic {
    /// Gogg (MSM-03, UC 0079): 76,400 kg, 45 knots (23.15 m/s), rated to 100m depth.
    /// Source: Bandai HG Gogg spec sheet / 0079 setting notes.
    fn default() -> Self {
        Self {
            physical: Mobility {
                id: "gogg-aquatic-baseline".to_string(),
                mass: 76_400.0,
                occupancy: AABB::default(),
                clearance: AABB::default(),
                load_capacity: 60_000.0,
                max_speed: 23.15,
            },
            depth_rating: 100.0,
        }
    }
}

impl Default for Space {
    /// Gelgoog (MS-14A, UC 0079): 73,800 kg, 890 kN thrust across 4 main verniers.
    /// Boost acceleration: 890,000 N / 73,800 kg ≈ 12.1 m/s².
    /// Source: Bandai MG 1/100 Gelgoog instruction manual.
    fn default() -> Self {
        Self {
            physical: Mobility {
                id: "gelgoog-space-baseline".to_string(),
                mass: 73_800.0,
                occupancy: AABB::default(),
                clearance: AABB::default(),
                load_capacity: 65_000.0,
                max_speed: 12.1,
            },
            thruster_count: 4,
        }
    }
}
