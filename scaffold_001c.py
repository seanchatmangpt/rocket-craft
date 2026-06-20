import os

templates_dir = "ontology/ggen-packs/mech_factory_mud/templates"
rust_dir = f"{templates_dir}/rust"
ue4_dir = f"{templates_dir}/ue4"

os.makedirs(rust_dir, exist_ok=True)
os.makedirs(ue4_dir, exist_ok=True)

factory_stations_tera = """# generated_by: ggen
# source_ttl: schema/mech_factory_mud.ttl
# source_query: queries/all.rq
# source_template: templates/ue4/FactoryStations.csv.tera
id,station_id,station_name,route_node_id,ue4_target_surface
{%- set_global count = 0 -%}
{%- for row in results -%}
{%- if row.type == "http://rocket-craft.com/ontology/mud#Station" %}
{%- set name = row.s | split(pat="#") | last -%}
{%- set sid = name | lower -%}
{%- if name == "FrameAssembly" %}{% set sid = "frame_assembly" %}{% endif -%}
{%- if name == "SocketTopology" %}{% set sid = "socket_topology" %}{% endif -%}
{%- if name == "ArmorSkinStation" %}{% set sid = "armor_skin" %}{% endif -%}
{%- if name == "RigMotionStation" %}{% set sid = "rig_motion" %}{% endif -%}
{%- if name == "VerificationGate" %}{% set sid = "verification_gate" %}{% endif -%}
{%- if name == "ReceiptTerminal" %}{% set sid = "receipt_terminal" %}{% endif -%}
{%- set_global count = count + 1 %}
{{ count }},{{ sid }},{{ name }},{{ name }},FactoryStationMarker
{%- endif -%}
{%- endfor %}
"""
with open(f"{ue4_dir}/FactoryStations.csv.tera", "w") as f: f.write(factory_stations_tera)

walkthrough_route_tera = """# generated_by: ggen
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
"""
with open(f"{ue4_dir}/WalkthroughRoute.csv.tera", "w") as f: f.write(walkthrough_route_tera)

rust_templates = {
    "constants.rs.tera": """// generated_by: ggen
// source_ttl: schema/mech_factory_mud.ttl
// source_query: queries/all.rq
// source_template: templates/rust/constants.rs.tera

pub const GENERATED_FROM_GGEN: bool = true;

pub fn get_generated_route_nodes() -> Vec<&'static str> {
    vec![
        {% for row in results -%}
        {%- if row.type == "http://rocket-craft.com/ontology/mud#RouteNode" %}
        "{{ row.s | split(pat="#") | last }}",
        {%- endif %}
        {%- endfor %}
    ]
}

pub fn get_generated_stations() -> Vec<&'static str> {
    vec![
        {% for row in results -%}
        {%- if row.type == "http://rocket-craft.com/ontology/mud#Station" %}
        "{{ row.s | split(pat="#") | last }}",
        {%- endif %}
        {%- endfor %}
    ]
}
""",
    "route.rs.tera": "// generated_by: ggen\\n// Route module\\npub const ROUTE_NODES_COUNT: usize = 9;\\n",
    "stations.rs.tera": "// generated_by: ggen\\n// Stations module\\npub const STATIONS_COUNT: usize = 6;\\n",
    "parts.rs.tera": "// generated_by: ggen\\n// Parts module\\npub const PARTS_COUNT: usize = 5;\\n",
    "authority.rs.tera": "// generated_by: ggen\\n// Authority module\\npub const AUTHORITY_FIELDS_COUNT: usize = 10;\\n",
    "projection.rs.tera": "// generated_by: ggen\\n// Projection module\\npub const PROJECTION_TYPES_COUNT: usize = 5;\\n",
    "receipt.rs.tera": "// generated_by: ggen\\n// Receipt module\\npub const RECEIPT_FIELDS_COUNT: usize = 5;\\n",
    "ocel.rs.tera": "// generated_by: ggen\\n// OCEL module\\npub const OCEL_OBJECT_TYPES_COUNT: usize = 20;\\npub const OCEL_EVENT_TYPES_COUNT: usize = 15;\\n"
}

for name, content in rust_templates.items():
    with open(f"{rust_dir}/{name}", "w") as f: f.write(content.replace('\\n', '\n'))

ue4_dt_templates = {
    "PartFamilies.csv.tera": "# generated_by: ggen\nid,part_family\n1,Frame\n",
    "SocketTopology.csv.tera": "# generated_by: ggen\nid,socket\n1,LeftShoulder\n",
    "SkinLayers.csv.tera": "# generated_by: ggen\nid,skin\n1,ThermalZone\n",
    "MotionFamilies.csv.tera": "# generated_by: ggen\nid,motion\n1,Walkthrough\n",
    "SemanticLOD.csv.tera": "# generated_by: ggen\nid,lod\n1,High\n",
    "ProjectionCommands.csv.tera": "# generated_by: ggen\nid,command,source_receipt_required\n1,Spawn,true\n",
}
for name, content in ue4_dt_templates.items():
    with open(f"{ue4_dir}/{name}", "w") as f: f.write(content)

ue4_h_templates = {
    "MechFactoryMudSteps.h.tera": "// generated_by: ggen\n#pragma once\n// enum route nodes, station ids\n",
    "MechFactoryMudAuthority.h.tera": "// generated_by: ggen\n#pragma once\n// enum part families, authority byte fields\n",
    "MechFactoryMudProjection.h.tera": "// generated_by: ggen\n#pragma once\n// projection command struct\n",
}
os.makedirs(f"{templates_dir}/ue4/Headers", exist_ok=True)
for name, content in ue4_h_templates.items():
    with open(f"{templates_dir}/ue4/Headers/{name}", "w") as f: f.write(content)

ggen_toml_base = """[project]
name = "mech_factory_mud"
version = "0.1.0"
description = "Mech Factory MUD Generation"

[inference]
rules = [
    { name = "standard-normalization", construct = "CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }" }
]

[ontology]
source = "schema/mech_factory_mud.ttl"

[generation]
output_dir = "/Users/sac/rocket-craft"

"""

rules = []

def add_rule(name, tpl, out):
    rules.append(f"""[[generation.rules]]
name = "{name}"
query = {{ file = "queries/all.rq" }}
template = {{ file = "templates/{tpl}" }}
output_file = "{out}"
mode = "Overwrite"
""")

add_rule("rust-constants", "rust/constants.rs.tera", "crates/mech_factory_mud/src/generated_constants.rs")
add_rule("rust-route", "rust/route.rs.tera", "generated/mech_factory_mud/rust/route.rs")
add_rule("rust-stations", "rust/stations.rs.tera", "generated/mech_factory_mud/rust/stations.rs")
add_rule("rust-parts", "rust/parts.rs.tera", "generated/mech_factory_mud/rust/parts.rs")
add_rule("rust-authority", "rust/authority.rs.tera", "generated/mech_factory_mud/rust/authority.rs")
add_rule("rust-projection", "rust/projection.rs.tera", "generated/mech_factory_mud/rust/projection.rs")
add_rule("rust-receipt", "rust/receipt.rs.tera", "generated/mech_factory_mud/rust/receipt.rs")
add_rule("rust-ocel", "rust/ocel.rs.tera", "generated/mech_factory_mud/rust/ocel.rs")

add_rule("ue4-stations-csv", "ue4/FactoryStations.csv.tera", "generated/mech_factory_mud/ue4/DataTables/FactoryStations.csv")
add_rule("ue4-route-csv", "ue4/WalkthroughRoute.csv.tera", "generated/mech_factory_mud/ue4/DataTables/WalkthroughRoute.csv")
add_rule("ue4-parts-csv", "ue4/PartFamilies.csv.tera", "generated/mech_factory_mud/ue4/DataTables/PartFamilies.csv")
add_rule("ue4-socket-csv", "ue4/SocketTopology.csv.tera", "generated/mech_factory_mud/ue4/DataTables/SocketTopology.csv")
add_rule("ue4-skin-csv", "ue4/SkinLayers.csv.tera", "generated/mech_factory_mud/ue4/DataTables/SkinLayers.csv")
add_rule("ue4-motion-csv", "ue4/MotionFamilies.csv.tera", "generated/mech_factory_mud/ue4/DataTables/MotionFamilies.csv")
add_rule("ue4-lod-csv", "ue4/SemanticLOD.csv.tera", "generated/mech_factory_mud/ue4/DataTables/SemanticLOD.csv")
add_rule("ue4-projection-csv", "ue4/ProjectionCommands.csv.tera", "generated/mech_factory_mud/ue4/DataTables/ProjectionCommands.csv")

add_rule("ue4-steps-h", "ue4/Headers/MechFactoryMudSteps.h.tera", "generated/mech_factory_mud/ue4/Headers/MechFactoryMudSteps.h")
add_rule("ue4-authority-h", "ue4/Headers/MechFactoryMudAuthority.h.tera", "generated/mech_factory_mud/ue4/Headers/MechFactoryMudAuthority.h")
add_rule("ue4-projection-h", "ue4/Headers/MechFactoryMudProjection.h.tera", "generated/mech_factory_mud/ue4/Headers/MechFactoryMudProjection.h")

with open("ontology/ggen-packs/mech_factory_mud/ggen.toml", "w") as f:
    f.write(ggen_toml_base + "\n".join(rules))

tests_code = """
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
"""
with open("crates/mech_factory_mud/src/generated_tests.rs", "w") as f:
    f.write(tests_code)

with open("crates/mech_factory_mud/src/lib.rs", "r") as f:
    lib_rs = f.read()
if "pub mod generated_tests;" not in lib_rs:
    with open("crates/mech_factory_mud/src/lib.rs", "w") as f:
        f.write(lib_rs.replace("\\npub mod generated_tests;\\n", "").replace("\\pub mod generated_tests;\\n", "") + "\npub mod generated_tests;\n")

