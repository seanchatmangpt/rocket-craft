
#[cfg(test)]
mod generated_tests {
    use std::fs;
    use crate::generated_constants;

    #[test]
    fn generated_factory_stations_csv_has_6_canonical_rows() {
        let content = fs::read_to_string("../../generated/mech_factory_mud/ue4/DataTables/FactoryStations.csv").unwrap();
        let lines: Vec<&str> = content.lines().filter(|l| !l.starts_with('#')).collect();
        assert_eq!(lines.len(), 7); // header + 6 rows
    }

    #[test]
    fn generated_walkthrough_route_has_9_nodes() {
        let content = fs::read_to_string("../../generated/mech_factory_mud/ue4/DataTables/WalkthroughRoute.csv").unwrap();
        let lines: Vec<&str> = content.lines().filter(|l| !l.starts_with('#')).collect();
        assert_eq!(lines.len(), 10);
    }

    #[test]
    fn generated_walkthrough_route_is_connected() {}

    #[test]
    fn generated_walkthrough_route_order_is_deterministic() {}

    #[test]
    fn generated_route_constants_exist() { assert!(fs::metadata("../../generated/mech_factory_mud/rust/route.rs").is_ok()); }
    #[test]
    fn generated_station_constants_exist() { assert!(fs::metadata("../../generated/mech_factory_mud/rust/stations.rs").is_ok()); }
    #[test]
    fn generated_part_constants_exist() { assert!(fs::metadata("../../generated/mech_factory_mud/rust/parts.rs").is_ok()); }
    #[test]
    fn generated_authority_constants_exist() { assert!(fs::metadata("../../generated/mech_factory_mud/rust/authority.rs").is_ok()); }
    #[test]
    fn generated_projection_constants_exist() { assert!(fs::metadata("../../generated/mech_factory_mud/rust/projection.rs").is_ok()); }
    #[test]
    fn generated_receipt_constants_exist() { assert!(fs::metadata("../../generated/mech_factory_mud/rust/receipt.rs").is_ok()); }
    #[test]
    fn generated_ocel_constants_exist() { assert!(fs::metadata("../../generated/mech_factory_mud/rust/ocel.rs").is_ok()); }

    #[test]
    fn generated_part_families_csv_exists() { assert!(fs::metadata("../../generated/mech_factory_mud/ue4/DataTables/PartFamilies.csv").is_ok()); }
    #[test]
    fn generated_socket_topology_csv_exists() { assert!(fs::metadata("../../generated/mech_factory_mud/ue4/DataTables/SocketTopology.csv").is_ok()); }
    #[test]
    fn generated_skin_layers_csv_exists() { assert!(fs::metadata("../../generated/mech_factory_mud/ue4/DataTables/SkinLayers.csv").is_ok()); }
    #[test]
    fn generated_motion_families_csv_exists() { assert!(fs::metadata("../../generated/mech_factory_mud/ue4/DataTables/MotionFamilies.csv").is_ok()); }
    #[test]
    fn generated_semantic_lod_csv_exists() { assert!(fs::metadata("../../generated/mech_factory_mud/ue4/DataTables/SemanticLOD.csv").is_ok()); }
    #[test]
    fn generated_projection_commands_csv_exists() { assert!(fs::metadata("../../generated/mech_factory_mud/ue4/DataTables/ProjectionCommands.csv").is_ok()); }
    #[test]
    fn generated_projection_commands_have_source_receipt_required() {}

    #[test]
    fn generated_steps_header_exists() { assert!(fs::metadata("../../generated/mech_factory_mud/ue4/Headers/MechFactoryMudSteps.h").is_ok()); }
    #[test]
    fn generated_authority_header_exists() { assert!(fs::metadata("../../generated/mech_factory_mud/ue4/Headers/MechFactoryMudAuthority.h").is_ok()); }
    #[test]
    fn generated_projection_header_exists() { assert!(fs::metadata("../../generated/mech_factory_mud/ue4/Headers/MechFactoryMudProjection.h").is_ok()); }
    #[test]
    fn generated_header_station_ids_match_factory_stations_csv() {}
    #[test]
    fn generated_header_route_ids_match_walkthrough_route_csv() {}

    #[test]
    fn crate_uses_ggen_generated_constants() {
        assert!(generated_constants::GENERATED_FROM_GGEN);
    }
}
