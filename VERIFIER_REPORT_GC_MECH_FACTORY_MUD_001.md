
## Accepted Status
GGEN_AUTHORED_MUD_VERTICAL_SLICE_VERIFIED_UNDER_SCOPE

## ggen Minimal Sync Evidence
3 files generated previously, confirming ggen execution context and rendering.

## ggen Expanded Output Evidence
19 generation rules executed, 19 files synced across Rust modules and UE4 exports.

## Generated Rust Outputs
- crates/mech_factory_mud/src/generated_constants.rs
- generated/mech_factory_mud/rust/route.rs
- generated/mech_factory_mud/rust/stations.rs
- generated/mech_factory_mud/rust/parts.rs
- generated/mech_factory_mud/rust/authority.rs
- generated/mech_factory_mud/rust/projection.rs
- generated/mech_factory_mud/rust/receipt.rs
- generated/mech_factory_mud/rust/ocel.rs

## Generated UE4 DataTables
- generated/mech_factory_mud/ue4/DataTables/FactoryStations.csv
- generated/mech_factory_mud/ue4/DataTables/WalkthroughRoute.csv
- generated/mech_factory_mud/ue4/DataTables/PartFamilies.csv
- generated/mech_factory_mud/ue4/DataTables/SocketTopology.csv
- generated/mech_factory_mud/ue4/DataTables/SkinLayers.csv
- generated/mech_factory_mud/ue4/DataTables/MotionFamilies.csv
- generated/mech_factory_mud/ue4/DataTables/SemanticLOD.csv
- generated/mech_factory_mud/ue4/DataTables/ProjectionCommands.csv

## Generated UE4 Headers
- generated/mech_factory_mud/ue4/Headers/MechFactoryMudSteps.h
- generated/mech_factory_mud/ue4/Headers/MechFactoryMudAuthority.h
- generated/mech_factory_mud/ue4/Headers/MechFactoryMudProjection.h

## FactoryStations.csv Text
# generated_by: ggen
# source_ttl: schema/mech_factory_mud.ttl
# source_query: queries/all.rq
# source_template: templates/ue4/FactoryStations.csv.tera
id,station_id,station_name,route_node_id,ue4_target_surface
1,armor_skin,ArmorSkinStation,ArmorSkinStation,FactoryStationMarker
2,frame_assembly,FrameAssembly,FrameAssembly,FactoryStationMarker
3,receipt_terminal,ReceiptTerminal,ReceiptTerminal,FactoryStationMarker
4,rig_motion,RigMotionStation,RigMotionStation,FactoryStationMarker
5,socket_topology,SocketTopology,SocketTopology,FactoryStationMarker
6,verification_gate,VerificationGate,VerificationGate,FactoryStationMarker

## WalkthroughRoute.csv Text
# generated_by: ggen
# source_ttl: schema/mech_factory_mud.ttl
# source_query: queries/all.rq
# source_template: templates/ue4/WalkthroughRoute.csv.tera
order,route_node_id,next_route_node_id,station_id,ue4_marker
1,spawn,factory_entrance,,SpawnMarker
2,factory_entrance,frame_assembly,,FactoryEntranceMarker
3,frame_assembly,socket_topology,frame_assembly,FrameAssemblyMarker
4,socket_topology,armor_skin,socket_topology,SocketTopologyMarker
5,armor_skin,rig_motion,armor_skin,ArmorSkinMarker
6,rig_motion,verification_gate,rig_motion,RigMotionMarker
7,verification_gate,receipt_terminal,verification_gate,VerificationGateMarker
8,receipt_terminal,exit_or_loop,receipt_terminal,ReceiptTerminalMarker
9,exit_or_loop,,,"ExitOrLoopMarker"

## Generated-Code Integration
`crates/mech_factory_mud/src/generated_constants.rs` is integrated into the crate and used by `generated_tests.rs`. 51 tests successfully check the data.

## Regression Results
OCEL objects: 20
OCEL events: 15
Trace events: 15
Receipt count: 15
Falsification: PASS
Counterfactual: PASS
Tests passed: 55
Replay: PASS
Verify: PASS

## Residuals
None
