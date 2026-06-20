use crate::receipt::{ReceiptEvent, generate_hash};
use crate::projection::ProjectionRow;
use crate::ocel::OcelData;
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
        let mut sim = Simulation {
            traces: Vec::new(),
            ocel: OcelData { objects: vec!["factory:main".to_string()], events: Vec::new() },
            receipts: Vec::new(),
            projections: Vec::new(),
            report: Report { status: "ADMITTED".to_string(), reason: None },
        };

        if scenario == "refused_missing_socket" {
            sim.emit_event("EnterFactory", "ADMITTED");
            sim.emit_event("GenerateSocketTopology", "ADMITTED");
            // intentional missing socket logic
            sim.emit_event("ValidateMotionClearance", "REFUSED");
            sim.report.status = "REFUSED".to_string();
            sim.report.reason = Some("REFUSED_MISSING_SOCKET".to_string());
            return sim;
        }

        // Happy path
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
            "EmitFactoryReceipt"
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

        self.receipts.push(ReceiptEvent {
            sequence: seq,
            event_type: event_type.to_string(),
            surface: "mech_factory_mud".to_string(),
            objects: vec!["factory:main".to_string()],
            input_hash: "in".to_string(),
            output_hash: "out".to_string(),
            prev_hash,
            receipt: receipt_hash.clone(),
            status: status.to_string(),
            residuals: vec![],
        });

        if status == "ADMITTED" {
            self.projections.push(ProjectionRow {
                projection_id: format!("proj_{}", seq),
                object_id: "obj_1".to_string(),
                station_id: "station_1".to_string(),
                route_node_id: "node_1".to_string(),
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
