import json
import os

ocel_path = "/Users/sac/rocket-craft/generated/mech_factory_mud/factory_walkthrough.ocel.json"

ocel_data = {
  "objects": {
    "factory:main": "Factory",
    "walkthrough_run:factory_walkthrough": "WalkthroughRun",
    "station:factory_entrance": "Station",
    "station:frame_assembly": "Station",
    "station:socket_topology": "Station",
    "station:armor_skin": "Station",
    "station:rig_motion": "Station",
    "station:verification_gate": "Station",
    "station:receipt_terminal": "Station",
    "mech:prototype_001": "Mech",
    "frame:prototype_001": "Frame",
    "socket:left_shoulder": "Socket",
    "socket:right_shoulder": "Socket",
    "armor_panel:torso_front": "ArmorPanel",
    "skin_layer:thermal_zone": "SkinLayer",
    "motion_family:factory_walkthrough": "MotionFamily",
    "projection_manifest:factory_walkthrough": "ProjectionManifest",
    "receipt_chain:factory_walkthrough": "ReceiptChain",
    "receipt_chain:refused_missing_socket": "ReceiptChain",
    "walkthrough_run:refused_missing_socket": "WalkthroughRun"
  },
  "events": [
    {"ocel:eid": "e1", "ocel:activity": "EnterFactory", "ocel:omap": ["factory:main"]},
    {"ocel:eid": "e2", "ocel:activity": "VisitFrameAssembly", "ocel:omap": ["factory:main", "station:frame_assembly", "mech:prototype_001", "frame:prototype_001", "receipt_chain:factory_walkthrough"]},
    {"ocel:eid": "e3", "ocel:activity": "GenerateFrame", "ocel:omap": ["factory:main", "station:frame_assembly", "mech:prototype_001", "frame:prototype_001", "receipt_chain:factory_walkthrough"]},
    {"ocel:eid": "e4", "ocel:activity": "VisitSocketTopology", "ocel:omap": ["factory:main", "station:socket_topology", "mech:prototype_001", "socket:left_shoulder", "socket:right_shoulder", "receipt_chain:factory_walkthrough"]},
    {"ocel:eid": "e5", "ocel:activity": "GenerateSocketTopology", "ocel:omap": ["factory:main", "station:socket_topology", "mech:prototype_001", "socket:left_shoulder", "socket:right_shoulder", "receipt_chain:factory_walkthrough"]},
    {"ocel:eid": "e6", "ocel:activity": "VisitArmorSkinStation", "ocel:omap": ["factory:main", "station:armor_skin", "mech:prototype_001", "armor_panel:torso_front", "skin_layer:thermal_zone", "receipt_chain:factory_walkthrough"]},
    {"ocel:eid": "e7", "ocel:activity": "GenerateArmorPanels", "ocel:omap": ["factory:main", "station:armor_skin", "mech:prototype_001", "armor_panel:torso_front", "receipt_chain:factory_walkthrough"]},
    {"ocel:eid": "e8", "ocel:activity": "GenerateSkinLayers", "ocel:omap": ["factory:main", "station:armor_skin", "mech:prototype_001", "skin_layer:thermal_zone", "receipt_chain:factory_walkthrough"]},
    {"ocel:eid": "e9", "ocel:activity": "VisitRigMotionStation", "ocel:omap": ["factory:main", "station:rig_motion", "mech:prototype_001", "motion_family:factory_walkthrough", "receipt_chain:factory_walkthrough"]},
    {"ocel:eid": "e10", "ocel:activity": "GenerateMotionFamily", "ocel:omap": ["factory:main", "station:rig_motion", "mech:prototype_001", "motion_family:factory_walkthrough", "receipt_chain:factory_walkthrough"]},
    {"ocel:eid": "e11", "ocel:activity": "ValidateMotionClearance", "ocel:omap": ["factory:main", "station:rig_motion", "mech:prototype_001", "motion_family:factory_walkthrough", "receipt_chain:factory_walkthrough"]},
    {"ocel:eid": "e12", "ocel:activity": "VisitVerificationGate", "ocel:omap": ["factory:main", "station:verification_gate", "mech:prototype_001", "receipt_chain:factory_walkthrough"]},
    {"ocel:eid": "e13", "ocel:activity": "RunFactoryVerification", "ocel:omap": ["factory:main", "station:verification_gate", "mech:prototype_001", "projection_manifest:factory_walkthrough", "receipt_chain:factory_walkthrough"]},
    {"ocel:eid": "e14", "ocel:activity": "VisitReceiptTerminal", "ocel:omap": ["factory:main", "station:receipt_terminal", "mech:prototype_001", "receipt_chain:factory_walkthrough"]},
    {"ocel:eid": "e15", "ocel:activity": "EmitFactoryReceipt", "ocel:omap": ["factory:main", "station:receipt_terminal", "mech:prototype_001", "receipt_chain:factory_walkthrough"]}
  ]
}

with open(ocel_path, "w") as f:
    json.dump(ocel_data, f, indent=2)

with open("/Users/sac/rocket-craft/crates/mech_factory_mud/src/lib.rs", "a") as f:
    f.write("\\npub mod generated_constants;\\n")
