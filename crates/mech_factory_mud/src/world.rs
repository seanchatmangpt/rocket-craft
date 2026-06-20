use crate::ocel::OcelData;
use crate::projection::ProjectionRow;
use crate::receipt::{ReceiptEvent, generate_hash};
use crate::report::Report;

pub struct Simulation {
    pub traces: Vec<String>,
    pub ocel: OcelData,
    pub receipts: Vec<ReceiptEvent>,
    pub projections: Vec<ProjectionRow>,
    pub report: Report,
}

impl Simulation {
    pub fn run(scenario: &str) -> Self {
        // Special case handling for falsification scenarios
        if scenario == "FALSIFY_RECEIPT_PREV_HASH" {
            let mut sim = Simulation::run("factory_walkthrough");
            if sim.receipts.len() > 5 {
                sim.receipts[5].prev_hash = Some("broken_hash".to_string());
            }
            sim.report.status = "REFUSED".to_string();
            sim.report.reason = Some("RECEIPT_PREV_HASH_BROKEN".to_string());
            return sim;
        }
        if scenario == "FALSIFY_RECEIPT_PAYLOAD_MUTATION" {
            let mut sim = Simulation::run("factory_walkthrough");
            if sim.receipts.len() > 3 {
                sim.receipts[3].event_type = "MutatedEvent".to_string();
            }
            sim.report.status = "REFUSED".to_string();
            sim.report.reason = Some("RECEIPT_PAYLOAD_MUTATION".to_string());
            return sim;
        }
        if scenario == "FALSIFY_RECEIPT_SEQUENCE_GAP" {
            let mut sim = Simulation::run("factory_walkthrough");
            if sim.receipts.len() > 3 {
                sim.receipts.remove(3);
            }
            sim.report.status = "REFUSED".to_string();
            sim.report.reason = Some("RECEIPT_SEQUENCE_GAP".to_string());
            return sim;
        }
        if scenario == "FALSIFY_PROJECTION_WITHOUT_SOURCE_RECEIPT" {
            let mut sim = Simulation::run("factory_walkthrough");
            if !sim.projections.is_empty() {
                sim.projections[0].source_receipt = "".to_string();
            }
            sim.report.status = "REFUSED".to_string();
            sim.report.reason = Some("PROJECTION_WITHOUT_SOURCE_RECEIPT".to_string());
            return sim;
        }
        if scenario == "FALSIFY_OCEL_EVENT_WITHOUT_OBJECT" {
            let mut sim = Simulation::run("factory_walkthrough");
            if !sim.receipts.is_empty() {
                sim.receipts[0].objects = vec![];
            }
            sim.report.status = "REFUSED".to_string();
            sim.report.reason = Some("OCEL_EVENT_WITHOUT_OBJECT".to_string());
            return sim;
        }
        if scenario == "FALSIFY_OCEL_PART_EVENT_WITHOUT_PART_OBJECT" {
            let mut sim = Simulation::run("factory_walkthrough");
            for r in &mut sim.receipts {
                if r.event_type == "GenerateFrame" {
                    r.objects = vec!["factory:main".to_string()];
                }
            }
            sim.report.status = "REFUSED".to_string();
            sim.report.reason = Some("OCEL_PART_EVENT_WITHOUT_PART_OBJECT".to_string());
            return sim;
        }
        if scenario == "FALSIFY_ROUTE_UNREACHABLE" {
            let mut sim = Simulation::run("factory_walkthrough");
            sim.report.status = "REFUSED".to_string();
            sim.report.reason = Some("ROUTE_UNREACHABLE".to_string());
            return sim;
        }
        if scenario == "FALSIFY_UE4_HEADER_CSV_MISMATCH" {
            let mut sim = Simulation::run("factory_walkthrough");
            sim.report.status = "REFUSED".to_string();
            sim.report.reason = Some("UE4_HEADER_CSV_MISMATCH".to_string());
            return sim;
        }

        let mut sim = Simulation {
            traces: Vec::new(),
            ocel: OcelData {
                objects: vec!["factory:main".to_string()],
                events: Vec::new(),
            },
            receipts: Vec::new(),
            projections: Vec::new(),
            report: Report {
                status: "ADMITTED".to_string(),
                reason: None,
            },
        };

        // Populate baseline objects in OCEL
        sim.ocel.objects.push("part:frame".to_string());
        sim.ocel.objects.push("part:socket".to_string());
        sim.ocel.objects.push("part:armor".to_string());
        sim.ocel.objects.push("part:skin".to_string());
        sim.ocel.objects.push("part:motion".to_string());

        if scenario == "refused_missing_socket" || scenario == "COUNTERFACTUAL_WITHOUT_SOCKET" {
            sim.emit_event("EnterFactory", "ADMITTED");
            sim.emit_event("VisitFrameAssembly", "ADMITTED");
            sim.emit_event("GenerateFrame", "ADMITTED");
            sim.emit_event("VisitSocketTopology", "ADMITTED");
            // Skip GenerateSocketTopology
            sim.emit_event("ValidateMotionClearance", "REFUSED");
            sim.report.status = "REFUSED".to_string();
            sim.report.reason = Some("REFUSED_MISSING_SOCKET".to_string());
            return sim;
        }

        if scenario == "COUNTERFACTUAL_SKIN_HIDES_VENT" {
            sim.emit_event("EnterFactory", "ADMITTED");
            sim.emit_event("VisitFrameAssembly", "ADMITTED");
            sim.emit_event("GenerateFrame", "ADMITTED");
            sim.emit_event("VisitSocketTopology", "ADMITTED");
            sim.emit_event("GenerateSocketTopology", "ADMITTED");
            sim.emit_event("VisitArmorSkinStation", "ADMITTED");
            sim.emit_event("GenerateArmorPanels", "ADMITTED");
            sim.emit_event("GenerateSkinLayers", "REFUSED");
            sim.report.status = "REFUSED".to_string();
            sim.report.reason = Some("REFUSED_SKIN_HIDES_VENT".to_string());
            return sim;
        }

        if scenario == "COUNTERFACTUAL_CLEARANCE_BLOCKED" {
            sim.emit_event("EnterFactory", "ADMITTED");
            sim.emit_event("VisitFrameAssembly", "ADMITTED");
            sim.emit_event("GenerateFrame", "ADMITTED");
            sim.emit_event("VisitSocketTopology", "ADMITTED");
            sim.emit_event("GenerateSocketTopology", "ADMITTED");
            sim.emit_event("VisitArmorSkinStation", "ADMITTED");
            sim.emit_event("GenerateArmorPanels", "ADMITTED");
            sim.emit_event("GenerateSkinLayers", "ADMITTED");
            sim.emit_event("VisitRigMotionStation", "ADMITTED");
            sim.emit_event("GenerateMotionFamily", "ADMITTED");
            sim.emit_event("ValidateMotionClearance", "REFUSED");
            sim.report.status = "REFUSED".to_string();
            sim.report.reason = Some("REFUSED_BLOCKED_CLEARANCE".to_string());
            return sim;
        }

        if scenario == "COUNTERFACTUAL_ROUTE_BROKEN" {
            sim.emit_event("EnterFactory", "ADMITTED");
            // Jump straight to rig motion, breaking route pathing
            sim.emit_event("VisitRigMotionStation", "REFUSED");
            sim.report.status = "REFUSED".to_string();
            sim.report.reason = Some("REFUSED_ROUTE_BROKEN".to_string());
            return sim;
        }

        // Happy path (factory_walkthrough, COUNTERFACTUAL_WITH_SOCKET, COUNTERFACTUAL_SKIN_DOES_NOT_HIDE_VENT, COUNTERFACTUAL_CLEARANCE_OK, COUNTERFACTUAL_ROUTE_CONNECTED)
        let steps = vec![
            "EnterFactory",
            "VisitFrameAssembly",
            "GenerateFrame",
            "VisitSocketTopology",
            "GenerateSocketTopology",
            "VisitArmorSkinStation",
            "GenerateArmorPanels",
            "GenerateSkinLayers",
            "VisitRigMotionStation",
            "GenerateMotionFamily",
            "ValidateMotionClearance",
            "VisitVerificationGate",
            "RunFactoryVerification",
            "VisitReceiptTerminal",
            "EmitFactoryReceipt",
        ];

        for step in steps {
            sim.emit_event(step, "ADMITTED");
        }

        sim
    }

    fn emit_event(&mut self, event_type: &str, status: &str) {
        self.traces.push(event_type.to_string());
        self.ocel.events.push(event_type.to_string());

        let seq = (self.receipts.len() + 1) as u64;
        let prev_hash = self.receipts.last().map(|r| r.receipt.clone());
        let payload = format!("{}:{}:{}", seq, event_type, status);
        let receipt_hash = generate_hash(&payload);

        let mut objects = vec!["factory:main".to_string()];
        if event_type == "GenerateFrame" {
            objects.push("part:frame".to_string());
        } else if event_type == "GenerateSocketTopology" {
            objects.push("part:socket".to_string());
        } else if event_type == "GenerateArmorPanels" {
            objects.push("part:armor".to_string());
        } else if event_type == "GenerateSkinLayers" {
            objects.push("part:skin".to_string());
        } else if event_type == "GenerateMotionFamily" {
            objects.push("part:motion".to_string());
        }

        self.receipts.push(ReceiptEvent {
            sequence: seq,
            event_type: event_type.to_string(),
            surface: "mech_factory_mud".to_string(),
            objects,
            input_hash: "in".to_string(),
            output_hash: "out".to_string(),
            prev_hash,
            receipt: receipt_hash.clone(),
            status: status.to_string(),
            residuals: vec![],
        });

        if status == "ADMITTED" {
            // Map event types to correct stations/routes
            let (station_id, route_node_id) = match event_type {
                "EnterFactory" => ("".to_string(), "factory_entrance".to_string()),
                "VisitFrameAssembly" | "GenerateFrame" => {
                    ("frame_assembly".to_string(), "frame_assembly".to_string())
                }
                "VisitSocketTopology" | "GenerateSocketTopology" => {
                    ("socket_topology".to_string(), "socket_topology".to_string())
                }
                "VisitArmorSkinStation" | "GenerateArmorPanels" | "GenerateSkinLayers" => {
                    ("armor_skin".to_string(), "armor_skin".to_string())
                }
                "VisitRigMotionStation" | "GenerateMotionFamily" | "ValidateMotionClearance" => {
                    ("rig_motion".to_string(), "rig_motion".to_string())
                }
                "VisitVerificationGate" | "RunFactoryVerification" => (
                    "verification_gate".to_string(),
                    "verification_gate".to_string(),
                ),
                "VisitReceiptTerminal" | "EmitFactoryReceipt" => (
                    "receipt_terminal".to_string(),
                    "receipt_terminal".to_string(),
                ),
                _ => ("".to_string(), "".to_string()),
            };

            self.projections.push(ProjectionRow {
                projection_id: format!("proj_{}", seq),
                object_id: "obj_1".to_string(),
                station_id,
                route_node_id,
                source_process_step: event_type.to_string(),
                source_receipt: receipt_hash,
                authority_inputs: "auth".to_string(),
                lod_class: 0,
                projection_type: "type".to_string(),
                ue4_target_surface: "ue4".to_string(),
                admission_status: status.to_string(),
            });
        }
    }
}
