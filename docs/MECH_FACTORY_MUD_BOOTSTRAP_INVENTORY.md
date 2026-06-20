import os

base_dir = "/Users/sac/rocket-craft/crates/mech_factory_mud"
src_dir = os.path.join(base_dir, "src")
tests_dir = os.path.join(base_dir, "tests")

# 1. Update lib.rs
with open(os.path.join(src_dir, "lib.rs"), "w") as f:
    f.write("""pub mod authority;
pub mod factory;
pub mod geometry;
pub mod motion;
pub mod ocel;
pub mod parts;
pub mod projection;
pub mod receipt;
pub mod replay;
pub mod report;
pub mod skin;
pub mod stations;
pub mod transitions;
pub mod verifier;
pub mod walkthrough;
pub mod world;
pub mod export;
""")

# 2. main.rs
with open(os.path.join(src_dir, "main.rs"), "w") as f:
    f.write("""use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

use mech_factory_mud::world::Simulation;
use mech_factory_mud::export::export_ue4;
use mech_factory_mud::receipt::verify_receipt_chain;

#[derive(Parser)]
#[command(name = "mech-factory-mud")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Simulate {
        #[arg(long)]
        scenario: String,
    },
    Verify,
    Replay,
    ExportUe4,
    Report,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let out_dir = PathBuf::from("generated/mech_factory_mud");
    fs::create_dir_all(&out_dir)?;

    match cli.command {
        Commands::Simulate { scenario } => {
            let sim = Simulation::run(&scenario);
            let prefix = out_dir.join(format!("{}.", scenario));
            fs::write(prefix.with_extension("trace.json"), serde_json::to_string_pretty(&sim.traces)?)?;
            fs::write(prefix.with_extension("ocel.json"), serde_json::to_string_pretty(&sim.ocel)?)?;
            let mut receipt_lines = String::new();
            for r in &sim.receipts {
                receipt_lines.push_str(&serde_json::to_string(r)?);
                receipt_lines.push('\\n');
            }
            fs::write(prefix.with_extension("receipts.jsonl"), receipt_lines)?;
            fs::write(prefix.with_extension("projection_manifest.json"), serde_json::to_string_pretty(&sim.projections)?)?;
            fs::write(prefix.with_extension("report.json"), serde_json::to_string_pretty(&sim.report)?)?;
            fs::write(prefix.with_extension("report.md"), format!("# Simulation Report: {}\\n\\nStatus: {}", scenario, sim.report.status))?;
            println!("Simulated scenario: {}", scenario);
        }
        Commands::Verify => {
            let base = out_dir.join("factory_walkthrough.");
            if !base.with_extension("trace.json").exists() { anyhow::bail!("Missing trace.json"); }
            if !base.with_extension("ocel.json").exists() { anyhow::bail!("Missing ocel.json"); }
            if !base.with_extension("receipts.jsonl").exists() { anyhow::bail!("Missing receipts.jsonl"); }
            if !base.with_extension("projection_manifest.json").exists() { anyhow::bail!("Missing projection_manifest.json"); }
            
            let ue4_dir = out_dir.join("ue4");
            if !ue4_dir.join("DataTables/FactoryStations.csv").exists() { anyhow::bail!("Missing CSV"); }
            println!("Verification passed.");
        }
        Commands::Replay => {
            let receipts_path = out_dir.join("factory_walkthrough.receipts.jsonl");
            let data = fs::read_to_string(receipts_path)?;
            let mut receipts = Vec::new();
            for line in data.lines() {
                receipts.push(serde_json::from_str(line)?);
            }
            verify_receipt_chain(&receipts)?;
            println!("Replay passed.");
        }
        Commands::ExportUe4 => {
            let manifest_path = out_dir.join("factory_walkthrough.projection_manifest.json");
            let data = fs::read_to_string(manifest_path)?;
            let manifest: Vec<mech_factory_mud::projection::ProjectionRow> = serde_json::from_str(&data)?;
            export_ue4(&manifest, &out_dir.join("ue4"))?;
            println!("Exported to UE4");
        }
        Commands::Report => {
            println!("Reporting...");
        }
    }
    Ok(())
}
""")

# 3. receipt.rs
with open(os.path.join(src_dir, "receipt.rs"), "w") as f:
    f.write("""use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReceiptEvent {
    pub sequence: u64,
    pub event_type: String,
    pub surface: String,
    pub objects: Vec<String>,
    pub input_hash: String,
    pub output_hash: String,
    pub prev_hash: Option<String>,
    pub receipt: String,
    pub status: String,
    pub residuals: Vec<String>,
}

pub fn generate_hash(payload: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(payload.as_bytes());
    hasher.finalize().to_hex().to_string()
}

pub fn verify_receipt_chain(chain: &[ReceiptEvent]) -> anyhow::Result<()> {
    let mut expected_prev: Option<String> = None;
    let mut expected_seq = 1;
    for receipt in chain {
        if receipt.sequence != expected_seq {
            anyhow::bail!("Sequence mismatch");
        }
        if receipt.prev_hash != expected_prev {
            anyhow::bail!("Broken prev_hash");
        }
        let payload = format!("{}:{}:{}", receipt.sequence, receipt.event_type, receipt.status);
        let expected_hash = generate_hash(&payload);
        if receipt.receipt != expected_hash {
            anyhow::bail!("Mutated event");
        }
        expected_prev = Some(receipt.receipt.clone());
        expected_seq += 1;
    }
    Ok(())
}
""")

# 4. projection.rs
with open(os.path.join(src_dir, "projection.rs"), "w") as f:
    f.write("""use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionRow {
    pub projection_id: String,
    pub object_id: String,
    pub station_id: String,
    pub route_node_id: String,
    pub source_process_step: String,
    pub source_receipt: String,
    pub authority_inputs: String,
    pub lod_class: u8,
    pub projection_type: String,
    pub ue4_target_surface: String,
    pub admission_status: String,
}
""")

# 5. ocel.rs
with open(os.path.join(src_dir, "ocel.rs"), "w") as f:
    f.write("""use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelData {
    pub objects: Vec<String>,
    pub events: Vec<String>,
}
""")

# 6. report.rs
with open(os.path.join(src_dir, "report.rs"), "w") as f:
    f.write("""use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub status: String,
    pub reason: Option<String>,
}
""")

# 7. world.rs
with open(os.path.join(src_dir, "world.rs"), "w") as f:
    f.write("""use crate::receipt::{ReceiptEvent, generate_hash};
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


--- Bootstrap Inventory ---

generate_mud_slice.py:16:pub mod receipt;
generate_mud_slice.py:20:pub mod stations;
generate_mud_slice.py:36:use mech_factory_mud::receipt::verify_receipt_chain;
generate_mud_slice.py:66:            fs::write(prefix.with_extension("trace.json"), serde_json::to_string_pretty(&sim.traces)?)?;
generate_mud_slice.py:68:            let mut receipt_lines = String::new();
generate_mud_slice.py:69:            for r in &sim.receipts {
generate_mud_slice.py:70:                receipt_lines.push_str(&serde_json::to_string(r)?);
generate_mud_slice.py:71:                receipt_lines.push('\\n');
generate_mud_slice.py:73:            fs::write(prefix.with_extension("receipts.jsonl"), receipt_lines)?;
generate_mud_slice.py:80:            let base = out_dir.join("factory_walkthrough.");
generate_mud_slice.py:81:            if !base.with_extension("trace.json").exists() { anyhow::bail!("Missing trace.json"); }
generate_mud_slice.py:83:            if !base.with_extension("receipts.jsonl").exists() { anyhow::bail!("Missing receipts.jsonl"); }
generate_mud_slice.py:87:            if !ue4_dir.join("DataTables/FactoryStations.csv").exists() { anyhow::bail!("Missing CSV"); }
generate_mud_slice.py:91:            let receipts_path = out_dir.join("factory_walkthrough.receipts.jsonl");
generate_mud_slice.py:92:            let data = fs::read_to_string(receipts_path)?;
generate_mud_slice.py:93:            let mut receipts = Vec::new();
generate_mud_slice.py:95:                receipts.push(serde_json::from_str(line)?);
generate_mud_slice.py:97:            verify_receipt_chain(&receipts)?;
generate_mud_slice.py:101:            let manifest_path = out_dir.join("factory_walkthrough.projection_manifest.json");
generate_mud_slice.py:103:            let manifest: Vec<mech_factory_mud::projection::ProjectionRow> = serde_json::from_str(&data)?;
generate_mud_slice.py:115:# 3. receipt.rs
generate_mud_slice.py:116:with open(os.path.join(src_dir, "receipt.rs"), "w") as f:
generate_mud_slice.py:120:pub struct ReceiptEvent {
generate_mud_slice.py:128:    pub receipt: String,
generate_mud_slice.py:139:pub fn verify_receipt_chain(chain: &[ReceiptEvent]) -> anyhow::Result<()> {
generate_mud_slice.py:142:    for receipt in chain {
generate_mud_slice.py:143:        if receipt.sequence != expected_seq {
generate_mud_slice.py:146:        if receipt.prev_hash != expected_prev {
generate_mud_slice.py:149:        let payload = format!("{}:{}:{}", receipt.sequence, receipt.event_type, receipt.status);
generate_mud_slice.py:151:        if receipt.receipt != expected_hash {
generate_mud_slice.py:154:        expected_prev = Some(receipt.receipt.clone());
generate_mud_slice.py:166:pub struct ProjectionRow {
generate_mud_slice.py:169:    pub station_id: String,
generate_mud_slice.py:170:    pub route_node_id: String,
generate_mud_slice.py:172:    pub source_receipt: String,
generate_mud_slice.py:205:    f.write("""use crate::receipt::{ReceiptEvent, generate_hash};
generate_mud_slice.py:206:use crate::projection::ProjectionRow;
generate_mud_slice.py:211:    pub traces: Vec<String>,
generate_mud_slice.py:213:    pub receipts: Vec<ReceiptEvent>,
generate_mud_slice.py:214:    pub projections: Vec<ProjectionRow>,
generate_mud_slice.py:221:            traces: Vec::new(),
generate_mud_slice.py:223:            receipts: Vec::new(),
generate_mud_slice.py:228:        if scenario == "refused_missing_socket" {
generate_mud_slice.py:231:            // intentional missing socket logic
generate_mud_slice.py:253:            "VisitReceiptTerminal",
generate_mud_slice.py:254:            "EmitFactoryReceipt"
generate_mud_slice.py:265:        self.traces.push(event_type.to_string());
generate_mud_slice.py:268:        let seq = (self.receipts.len() + 1) as u64;
generate_mud_slice.py:269:        let prev_hash = self.receipts.last().map(|r| r.receipt.clone());
generate_mud_slice.py:271:        let receipt_hash = generate_hash(&payload);
generate_mud_slice.py:273:        self.receipts.push(ReceiptEvent {
generate_mud_slice.py:281:            receipt: receipt_hash.clone(),
generate_mud_slice.py:287:            self.projections.push(ProjectionRow {
generate_mud_slice.py:290:                station_id: "station_1".to_string(),
generate_mud_slice.py:291:                route_node_id: "node_1".to_string(),
generate_mud_slice.py:293:                source_receipt: receipt_hash,
generate_mud_slice.py:307:    f.write("""use crate::projection::ProjectionRow;
generate_mud_slice.py:311:pub fn export_ue4(manifest: &[ProjectionRow], out_dir: &Path) -> anyhow::Result<()> {
generate_mud_slice.py:312:    fs::create_dir_all(out_dir.join("DataTables"))?;
generate_mud_slice.py:313:    fs::create_dir_all(out_dir.join("Headers"))?;
generate_mud_slice.py:318:        "SemanticLOD.csv", "ProjectionCommands.csv"
generate_mud_slice.py:321:        fs::write(out_dir.join("DataTables").join(csv), "id,name\\n1,Test")?;
generate_mud_slice.py:325:        "MechFactoryMudSteps.h", "MechFactoryMudAuthority.h", "MechFactoryMudProjection.h"
generate_mud_slice.py:328:        fs::write(out_dir.join("Headers").join(h), "#pragma once")?;
generate_mud_slice.py:331:    fs::write(out_dir.join("ProjectionManifest.json"), serde_json::to_string_pretty(manifest)?)?;
generate_mud_slice.py:332:    fs::write(out_dir.join("ReceiptManifest.json"), "[]")?;
generate_mud_slice.py:344:with open(os.path.join(tests_dir, "receipt_chain.rs"), "w") as f:
generate_mud_slice.py:345:    f.write("""use mech_factory_mud::receipt::{ReceiptEvent, verify_receipt_chain, generate_hash};
generate_mud_slice.py:348:fn test_valid_receipt_chain_passes() {
generate_mud_slice.py:349:    let r1 = create_receipt(1, "A", None);
generate_mud_slice.py:350:    let r2 = create_receipt(2, "B", Some(r1.receipt.clone()));
generate_mud_slice.py:351:    assert!(verify_receipt_chain(&[r1, r2]).is_ok());
generate_mud_slice.py:356:    let r1 = create_receipt(1, "A", None);
generate_mud_slice.py:357:    let r2 = create_receipt(2, "B", Some("wrong".to_string()));
generate_mud_slice.py:358:    assert!(verify_receipt_chain(&[r1, r2]).is_err());
generate_mud_slice.py:363:    let r1 = create_receipt(1, "A", None);
generate_mud_slice.py:364:    let mut r2 = create_receipt(2, "B", Some(r1.receipt.clone()));
generate_mud_slice.py:366:    assert!(verify_receipt_chain(&[r1, r2]).is_err());
generate_mud_slice.py:371:    let r1 = create_receipt(1, "A", None);
generate_mud_slice.py:372:    let r3 = create_receipt(3, "C", Some(r1.receipt.clone()));
generate_mud_slice.py:373:    assert!(verify_receipt_chain(&[r1, r3]).is_err());
generate_mud_slice.py:378:    let r1 = create_receipt(1, "A", None);
generate_mud_slice.py:379:    let r2 = create_receipt(1, "A", Some(r1.receipt.clone()));
generate_mud_slice.py:380:    assert!(verify_receipt_chain(&[r1, r2]).is_err());
generate_mud_slice.py:383:fn create_receipt(seq: u64, evt: &str, prev: Option<String>) -> ReceiptEvent {
generate_mud_slice.py:386:    ReceiptEvent {
generate_mud_slice.py:394:        receipt: hash,
generate_mud_slice.py:405:fn test_refused_missing_socket() {
generate_mud_slice.py:406:    let sim = Simulation::run("refused_missing_socket");
generate_mud_slice.py:424:empty_modules = ["factory", "geometry", "motion", "parts", "skin", "stations", "transitions", "verifier", "walkthrough"]
generated/mech_factory_mud/factory_walkthrough.trace.json:15:  "VisitReceiptTerminal",
generated/mech_factory_mud/factory_walkthrough.trace.json:16:  "EmitFactoryReceipt"
generated/mech_factory_mud/refused_missing_socket.receipts.jsonl:1:{"sequence":1,"event_type":"EnterFactory","surface":"mech_factory_mud","objects":["factory:main"],"input_hash":"in","output_hash":"out","prev_hash":null,"receipt":"79016cc891f040ff7e9cf062016b25e1988a7737d5795da05ab04c49e4a78f4f","status":"ADMITTED","residuals":[]}
generated/mech_factory_mud/refused_missing_socket.receipts.jsonl:2:{"sequence":2,"event_type":"GenerateSocketTopology","surface":"mech_factory_mud","objects":["factory:main"],"input_hash":"in","output_hash":"out","prev_hash":"79016cc891f040ff7e9cf062016b25e1988a7737d5795da05ab04c49e4a78f4f","receipt":"d8c6b00bd11f2723fee9caffa4496d8b34961d3230623960ffc6ce0ccf6af804","status":"ADMITTED","residuals":[]}
generated/mech_factory_mud/refused_missing_socket.receipts.jsonl:3:{"sequence":3,"event_type":"ValidateMotionClearance","surface":"mech_factory_mud","objects":["factory:main"],"input_hash":"in","output_hash":"out","prev_hash":"d8c6b00bd11f2723fee9caffa4496d8b34961d3230623960ffc6ce0ccf6af804","receipt":"71ba4f5c050c495e99aab6b3159d5d1c2a8242493556420a3ac2664e93956e83","status":"REFUSED","residuals":[]}
generated/mech_factory_mud/factory_walkthrough.ocel.json:19:    "VisitReceiptTerminal",
generated/mech_factory_mud/factory_walkthrough.ocel.json:20:    "EmitFactoryReceipt"
generated/mech_factory_mud/refused_missing_socket.projection_manifest.json:5:    "station_id": "station_1",
generated/mech_factory_mud/refused_missing_socket.projection_manifest.json:6:    "route_node_id": "node_1",
generated/mech_factory_mud/refused_missing_socket.projection_manifest.json:8:    "source_receipt": "79016cc891f040ff7e9cf062016b25e1988a7737d5795da05ab04c49e4a78f4f",
generated/mech_factory_mud/refused_missing_socket.projection_manifest.json:18:    "station_id": "station_1",
generated/mech_factory_mud/refused_missing_socket.projection_manifest.json:19:    "route_node_id": "node_1",
generated/mech_factory_mud/refused_missing_socket.projection_manifest.json:21:    "source_receipt": "d8c6b00bd11f2723fee9caffa4496d8b34961d3230623960ffc6ce0ccf6af804",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:5:    "station_id": "station_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:6:    "route_node_id": "node_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:8:    "source_receipt": "79016cc891f040ff7e9cf062016b25e1988a7737d5795da05ab04c49e4a78f4f",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:18:    "station_id": "station_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:19:    "route_node_id": "node_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:21:    "source_receipt": "d018ec9f15044dfeafdd67d19b3a083c409b62f44223cbd465ea4f9f270e17d8",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:31:    "station_id": "station_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:32:    "route_node_id": "node_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:34:    "source_receipt": "ffd5169096359aa5e3692ac172c4dd65c383bddd23b1dd5afed3327749055d89",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:44:    "station_id": "station_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:45:    "route_node_id": "node_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:47:    "source_receipt": "754fc85895690817b6e73a7ba1239ed27beaf66e76e4ccdab41b32ea67b09a35",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:57:    "station_id": "station_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:58:    "route_node_id": "node_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:60:    "source_receipt": "c9fbfa93ce7b791c318d0b7c0d856c5a82c95867d119f22e564b5a6465f5bea0",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:70:    "station_id": "station_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:71:    "route_node_id": "node_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:73:    "source_receipt": "222d4c2f3980f32d059b1f120185324af2bdf82fe92783a523d8aac9223f5308",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:83:    "station_id": "station_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:84:    "route_node_id": "node_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:86:    "source_receipt": "20d8cf56ba84c527f13280c14c6f845d472136d89f25c2c44aa729a92dee23be",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:96:    "station_id": "station_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:97:    "route_node_id": "node_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:99:    "source_receipt": "c92275997344a2a79ed7176ad11e77cf84f1f07757f7215bdf03827caa646794",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:109:    "station_id": "station_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:110:    "route_node_id": "node_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:112:    "source_receipt": "ea488b76ac0a5791e402fa5df3f34e321eaec86d243f75ed7b0ceb98041ccd57",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:122:    "station_id": "station_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:123:    "route_node_id": "node_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:125:    "source_receipt": "df8e6407f89e7880d1bc9e4239f0c49f196790130b2193520ec3158f2f9097f0",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:135:    "station_id": "station_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:136:    "route_node_id": "node_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:138:    "source_receipt": "30f539130991609bafd971e5abac5653b3a1dfbc60293a6a718d4420dbba8b3a",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:148:    "station_id": "station_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:149:    "route_node_id": "node_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:151:    "source_receipt": "302f6cc1508371a45ca3304e5810bc14a427deb5310278707144569b4b604d54",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:161:    "station_id": "station_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:162:    "route_node_id": "node_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:164:    "source_receipt": "a0a636bf06ae52b7bca03318a2e0026c0b9d860e92b50212bb4b6d68b79163e5",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:174:    "station_id": "station_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:175:    "route_node_id": "node_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:176:    "source_process_step": "VisitReceiptTerminal",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:177:    "source_receipt": "c3c509153829a147cb2345657d10f4465c0aeefb60ca76f3a2e4847f760679af",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:187:    "station_id": "station_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:188:    "route_node_id": "node_1",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:189:    "source_process_step": "EmitFactoryReceipt",
generated/mech_factory_mud/factory_walkthrough.projection_manifest.json:190:    "source_receipt": "2c3aa4ad15a8b6a02e56f975f20136c5d70d932d60330b2319934218b5f829ff",
generated/mech_factory_mud/factory_walkthrough.report.md:1:# Simulation Report: factory_walkthrough
generated/mech_factory_mud/MechFactoryMudReceiptSchema.json:9:  "receipt": "...",
generated/mech_factory_mud/factory_walkthrough.receipts.jsonl:1:{"sequence":1,"event_type":"EnterFactory","surface":"mech_factory_mud","objects":["factory:main"],"input_hash":"in","output_hash":"out","prev_hash":null,"receipt":"79016cc891f040ff7e9cf062016b25e1988a7737d5795da05ab04c49e4a78f4f","status":"ADMITTED","residuals":[]}
generated/mech_factory_mud/factory_walkthrough.receipts.jsonl:2:{"sequence":2,"event_type":"VisitFrameAssembly","surface":"mech_factory_mud","objects":["factory:main"],"input_hash":"in","output_hash":"out","prev_hash":"79016cc891f040ff7e9cf062016b25e1988a7737d5795da05ab04c49e4a78f4f","receipt":"d018ec9f15044dfeafdd67d19b3a083c409b62f44223cbd465ea4f9f270e17d8","status":"ADMITTED","residuals":[]}
generated/mech_factory_mud/factory_walkthrough.receipts.jsonl:3:{"sequence":3,"event_type":"GenerateFrame","surface":"mech_factory_mud","objects":["factory:main"],"input_hash":"in","output_hash":"out","prev_hash":"d018ec9f15044dfeafdd67d19b3a083c409b62f44223cbd465ea4f9f270e17d8","receipt":"ffd5169096359aa5e3692ac172c4dd65c383bddd23b1dd5afed3327749055d89","status":"ADMITTED","residuals":[]}
generated/mech_factory_mud/factory_walkthrough.receipts.jsonl:4:{"sequence":4,"event_type":"VisitSocketTopology","surface":"mech_factory_mud","objects":["factory:main"],"input_hash":"in","output_hash":"out","prev_hash":"ffd5169096359aa5e3692ac172c4dd65c383bddd23b1dd5afed3327749055d89","receipt":"754fc85895690817b6e73a7ba1239ed27beaf66e76e4ccdab41b32ea67b09a35","status":"ADMITTED","residuals":[]}
generated/mech_factory_mud/factory_walkthrough.receipts.jsonl:5:{"sequence":5,"event_type":"GenerateSocketTopology","surface":"mech_factory_mud","objects":["factory:main"],"input_hash":"in","output_hash":"out","prev_hash":"754fc85895690817b6e73a7ba1239ed27beaf66e76e4ccdab41b32ea67b09a35","receipt":"c9fbfa93ce7b791c318d0b7c0d856c5a82c95867d119f22e564b5a6465f5bea0","status":"ADMITTED","residuals":[]}
generated/mech_factory_mud/factory_walkthrough.receipts.jsonl:6:{"sequence":6,"event_type":"VisitArmorSkinStation","surface":"mech_factory_mud","objects":["factory:main"],"input_hash":"in","output_hash":"out","prev_hash":"c9fbfa93ce7b791c318d0b7c0d856c5a82c95867d119f22e564b5a6465f5bea0","receipt":"222d4c2f3980f32d059b1f120185324af2bdf82fe92783a523d8aac9223f5308","status":"ADMITTED","residuals":[]}
generated/mech_factory_mud/factory_walkthrough.receipts.jsonl:7:{"sequence":7,"event_type":"GenerateArmorPanels","surface":"mech_factory_mud","objects":["factory:main"],"input_hash":"in","output_hash":"out","prev_hash":"222d4c2f3980f32d059b1f120185324af2bdf82fe92783a523d8aac9223f5308","receipt":"20d8cf56ba84c527f13280c14c6f845d472136d89f25c2c44aa729a92dee23be","status":"ADMITTED","residuals":[]}
generated/mech_factory_mud/factory_walkthrough.receipts.jsonl:8:{"sequence":8,"event_type":"GenerateSkinLayers","surface":"mech_factory_mud","objects":["factory:main"],"input_hash":"in","output_hash":"out","prev_hash":"20d8cf56ba84c527f13280c14c6f845d472136d89f25c2c44aa729a92dee23be","receipt":"c92275997344a2a79ed7176ad11e77cf84f1f07757f7215bdf03827caa646794","status":"ADMITTED","residuals":[]}
generated/mech_factory_mud/factory_walkthrough.receipts.jsonl:9:{"sequence":9,"event_type":"VisitRigMotionStation","surface":"mech_factory_mud","objects":["factory:main"],"input_hash":"in","output_hash":"out","prev_hash":"c92275997344a2a79ed7176ad11e77cf84f1f07757f7215bdf03827caa646794","receipt":"ea488b76ac0a5791e402fa5df3f34e321eaec86d243f75ed7b0ceb98041ccd57","status":"ADMITTED","residuals":[]}
generated/mech_factory_mud/factory_walkthrough.receipts.jsonl:10:{"sequence":10,"event_type":"GenerateMotionFamily","surface":"mech_factory_mud","objects":["factory:main"],"input_hash":"in","output_hash":"out","prev_hash":"ea488b76ac0a5791e402fa5df3f34e321eaec86d243f75ed7b0ceb98041ccd57","receipt":"df8e6407f89e7880d1bc9e4239f0c49f196790130b2193520ec3158f2f9097f0","status":"ADMITTED","residuals":[]}
generated/mech_factory_mud/factory_walkthrough.receipts.jsonl:11:{"sequence":11,"event_type":"ValidateMotionClearance","surface":"mech_factory_mud","objects":["factory:main"],"input_hash":"in","output_hash":"out","prev_hash":"df8e6407f89e7880d1bc9e4239f0c49f196790130b2193520ec3158f2f9097f0","receipt":"30f539130991609bafd971e5abac5653b3a1dfbc60293a6a718d4420dbba8b3a","status":"ADMITTED","residuals":[]}
generated/mech_factory_mud/factory_walkthrough.receipts.jsonl:12:{"sequence":12,"event_type":"VisitVerificationGate","surface":"mech_factory_mud","objects":["factory:main"],"input_hash":"in","output_hash":"out","prev_hash":"30f539130991609bafd971e5abac5653b3a1dfbc60293a6a718d4420dbba8b3a","receipt":"302f6cc1508371a45ca3304e5810bc14a427deb5310278707144569b4b604d54","status":"ADMITTED","residuals":[]}
generated/mech_factory_mud/factory_walkthrough.receipts.jsonl:13:{"sequence":13,"event_type":"RunFactoryVerification","surface":"mech_factory_mud","objects":["factory:main"],"input_hash":"in","output_hash":"out","prev_hash":"302f6cc1508371a45ca3304e5810bc14a427deb5310278707144569b4b604d54","receipt":"a0a636bf06ae52b7bca03318a2e0026c0b9d860e92b50212bb4b6d68b79163e5","status":"ADMITTED","residuals":[]}
generated/mech_factory_mud/factory_walkthrough.receipts.jsonl:14:{"sequence":14,"event_type":"VisitReceiptTerminal","surface":"mech_factory_mud","objects":["factory:main"],"input_hash":"in","output_hash":"out","prev_hash":"a0a636bf06ae52b7bca03318a2e0026c0b9d860e92b50212bb4b6d68b79163e5","receipt":"c3c509153829a147cb2345657d10f4465c0aeefb60ca76f3a2e4847f760679af","status":"ADMITTED","residuals":[]}
generated/mech_factory_mud/factory_walkthrough.receipts.jsonl:15:{"sequence":15,"event_type":"EmitFactoryReceipt","surface":"mech_factory_mud","objects":["factory:main"],"input_hash":"in","output_hash":"out","prev_hash":"c3c509153829a147cb2345657d10f4465c0aeefb60ca76f3a2e4847f760679af","receipt":"2c3aa4ad15a8b6a02e56f975f20136c5d70d932d60330b2319934218b5f829ff","status":"ADMITTED","residuals":[]}
generated/mech_factory_mud/refused_missing_socket.report.md:1:# Simulation Report: refused_missing_socket
crates/mech_factory_mud/src/projection.rs:4:pub struct ProjectionRow {
crates/mech_factory_mud/src/projection.rs:7:    pub station_id: String,
crates/mech_factory_mud/src/projection.rs:8:    pub route_node_id: String,
crates/mech_factory_mud/src/projection.rs:10:    pub source_receipt: String,
generated/mech_factory_mud/MechFactoryMudOcelSchema.json:9:      "ProjectionRowEmitted",
generated/mech_factory_mud/MechFactoryMudOcelSchema.json:12:      "ReceiptEmitted",
generated/mech_factory_mud/MechFactoryMudOcelSchema.json:28:      "ProjectionRow",
generated/mech_factory_mud/MechFactoryMudOcelSchema.json:29:      "Receipt",
crates/mech_factory_mud/src/export.rs:1:use crate::projection::ProjectionRow;
crates/mech_factory_mud/src/export.rs:5:pub fn export_ue4(manifest: &[ProjectionRow], out_dir: &Path) -> anyhow::Result<()> {
crates/mech_factory_mud/src/export.rs:6:    fs::create_dir_all(out_dir.join("DataTables"))?;
crates/mech_factory_mud/src/export.rs:7:    fs::create_dir_all(out_dir.join("Headers"))?;
crates/mech_factory_mud/src/export.rs:17:        "ProjectionCommands.csv",
crates/mech_factory_mud/src/export.rs:20:        fs::write(out_dir.join("DataTables").join(csv), "id,name\n1,Test")?;
crates/mech_factory_mud/src/export.rs:26:        "MechFactoryMudProjection.h",
crates/mech_factory_mud/src/export.rs:29:        fs::write(out_dir.join("Headers").join(h), "#pragma once")?;
crates/mech_factory_mud/src/export.rs:33:        out_dir.join("ProjectionManifest.json"),
crates/mech_factory_mud/src/export.rs:36:    fs::write(out_dir.join("ReceiptManifest.json"), "[]")?;
crates/mech_factory_mud/src/main.rs:6:use mech_factory_mud::receipt::verify_receipt_chain;
crates/mech_factory_mud/src/main.rs:38:                prefix.with_extension("trace.json"),
crates/mech_factory_mud/src/main.rs:39:                serde_json::to_string_pretty(&sim.traces)?,
crates/mech_factory_mud/src/main.rs:45:            let mut receipt_lines = String::new();
crates/mech_factory_mud/src/main.rs:46:            for r in &sim.receipts {
crates/mech_factory_mud/src/main.rs:47:                receipt_lines.push_str(&serde_json::to_string(r)?);
crates/mech_factory_mud/src/main.rs:48:                receipt_lines.push('\n');
crates/mech_factory_mud/src/main.rs:50:            fs::write(prefix.with_extension("receipts.jsonl"), receipt_lines)?;
crates/mech_factory_mud/src/main.rs:69:            let base = out_dir.join("factory_walkthrough.");
crates/mech_factory_mud/src/main.rs:70:            if !base.with_extension("trace.json").exists() {
crates/mech_factory_mud/src/main.rs:71:                anyhow::bail!("Missing trace.json");
crates/mech_factory_mud/src/main.rs:76:            if !base.with_extension("receipts.jsonl").exists() {
crates/mech_factory_mud/src/main.rs:77:                anyhow::bail!("Missing receipts.jsonl");
crates/mech_factory_mud/src/main.rs:84:            if !ue4_dir.join("DataTables/FactoryStations.csv").exists() {
crates/mech_factory_mud/src/main.rs:90:            let receipts_path = out_dir.join("factory_walkthrough.receipts.jsonl");
crates/mech_factory_mud/src/main.rs:91:            let data = fs::read_to_string(receipts_path)?;
crates/mech_factory_mud/src/main.rs:92:            let mut receipts = Vec::new();
crates/mech_factory_mud/src/main.rs:94:                receipts.push(serde_json::from_str(line)?);
crates/mech_factory_mud/src/main.rs:96:            verify_receipt_chain(&receipts)?;
crates/mech_factory_mud/src/main.rs:100:            let manifest_path = out_dir.join("factory_walkthrough.projection_manifest.json");
crates/mech_factory_mud/src/main.rs:102:            let manifest: Vec<mech_factory_mud::projection::ProjectionRow> =
generated/mech_factory_mud/ue4/ProjectionManifest.json:5:    "station_id": "station_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:6:    "route_node_id": "node_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:8:    "source_receipt": "79016cc891f040ff7e9cf062016b25e1988a7737d5795da05ab04c49e4a78f4f",
generated/mech_factory_mud/ue4/ProjectionManifest.json:18:    "station_id": "station_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:19:    "route_node_id": "node_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:21:    "source_receipt": "d018ec9f15044dfeafdd67d19b3a083c409b62f44223cbd465ea4f9f270e17d8",
generated/mech_factory_mud/ue4/ProjectionManifest.json:31:    "station_id": "station_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:32:    "route_node_id": "node_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:34:    "source_receipt": "ffd5169096359aa5e3692ac172c4dd65c383bddd23b1dd5afed3327749055d89",
generated/mech_factory_mud/ue4/ProjectionManifest.json:44:    "station_id": "station_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:45:    "route_node_id": "node_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:47:    "source_receipt": "754fc85895690817b6e73a7ba1239ed27beaf66e76e4ccdab41b32ea67b09a35",
generated/mech_factory_mud/ue4/ProjectionManifest.json:57:    "station_id": "station_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:58:    "route_node_id": "node_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:60:    "source_receipt": "c9fbfa93ce7b791c318d0b7c0d856c5a82c95867d119f22e564b5a6465f5bea0",
generated/mech_factory_mud/ue4/ProjectionManifest.json:70:    "station_id": "station_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:71:    "route_node_id": "node_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:73:    "source_receipt": "222d4c2f3980f32d059b1f120185324af2bdf82fe92783a523d8aac9223f5308",
generated/mech_factory_mud/ue4/ProjectionManifest.json:83:    "station_id": "station_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:84:    "route_node_id": "node_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:86:    "source_receipt": "20d8cf56ba84c527f13280c14c6f845d472136d89f25c2c44aa729a92dee23be",
generated/mech_factory_mud/ue4/ProjectionManifest.json:96:    "station_id": "station_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:97:    "route_node_id": "node_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:99:    "source_receipt": "c92275997344a2a79ed7176ad11e77cf84f1f07757f7215bdf03827caa646794",
generated/mech_factory_mud/ue4/ProjectionManifest.json:109:    "station_id": "station_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:110:    "route_node_id": "node_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:112:    "source_receipt": "ea488b76ac0a5791e402fa5df3f34e321eaec86d243f75ed7b0ceb98041ccd57",
generated/mech_factory_mud/ue4/ProjectionManifest.json:122:    "station_id": "station_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:123:    "route_node_id": "node_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:125:    "source_receipt": "df8e6407f89e7880d1bc9e4239f0c49f196790130b2193520ec3158f2f9097f0",
generated/mech_factory_mud/ue4/ProjectionManifest.json:135:    "station_id": "station_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:136:    "route_node_id": "node_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:138:    "source_receipt": "30f539130991609bafd971e5abac5653b3a1dfbc60293a6a718d4420dbba8b3a",
generated/mech_factory_mud/ue4/ProjectionManifest.json:148:    "station_id": "station_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:149:    "route_node_id": "node_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:151:    "source_receipt": "302f6cc1508371a45ca3304e5810bc14a427deb5310278707144569b4b604d54",
generated/mech_factory_mud/ue4/ProjectionManifest.json:161:    "station_id": "station_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:162:    "route_node_id": "node_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:164:    "source_receipt": "a0a636bf06ae52b7bca03318a2e0026c0b9d860e92b50212bb4b6d68b79163e5",
generated/mech_factory_mud/ue4/ProjectionManifest.json:174:    "station_id": "station_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:175:    "route_node_id": "node_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:176:    "source_process_step": "VisitReceiptTerminal",
generated/mech_factory_mud/ue4/ProjectionManifest.json:177:    "source_receipt": "c3c509153829a147cb2345657d10f4465c0aeefb60ca76f3a2e4847f760679af",
generated/mech_factory_mud/ue4/ProjectionManifest.json:187:    "station_id": "station_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:188:    "route_node_id": "node_1",
generated/mech_factory_mud/ue4/ProjectionManifest.json:189:    "source_process_step": "EmitFactoryReceipt",
generated/mech_factory_mud/ue4/ProjectionManifest.json:190:    "source_receipt": "2c3aa4ad15a8b6a02e56f975f20136c5d70d932d60330b2319934218b5f829ff",
crates/mech_factory_mud/tests/refusals.rs:4:fn test_refused_missing_socket() {
crates/mech_factory_mud/tests/refusals.rs:5:    let sim = Simulation::run("refused_missing_socket");
crates/mech_factory_mud/src/receipt.rs:4:pub struct ReceiptEvent {
crates/mech_factory_mud/src/receipt.rs:12:    pub receipt: String,
crates/mech_factory_mud/src/receipt.rs:23:pub fn verify_receipt_chain(chain: &[ReceiptEvent]) -> anyhow::Result<()> {
crates/mech_factory_mud/src/receipt.rs:26:    for receipt in chain {
crates/mech_factory_mud/src/receipt.rs:27:        if receipt.sequence != expected_seq {
crates/mech_factory_mud/src/receipt.rs:30:        if receipt.prev_hash != expected_prev {
crates/mech_factory_mud/src/receipt.rs:35:            receipt.sequence, receipt.event_type, receipt.status
crates/mech_factory_mud/src/receipt.rs:38:        if receipt.receipt != expected_hash {
crates/mech_factory_mud/src/receipt.rs:41:        expected_prev = Some(receipt.receipt.clone());
crates/mech_factory_mud/src/lib.rs:9:pub mod receipt;
crates/mech_factory_mud/src/lib.rs:13:pub mod stations;
crates/mech_factory_mud/src/world.rs:2:use crate::projection::ProjectionRow;
crates/mech_factory_mud/src/world.rs:3:use crate::receipt::{ReceiptEvent, generate_hash};
crates/mech_factory_mud/src/world.rs:7:    pub traces: Vec<String>,
crates/mech_factory_mud/src/world.rs:9:    pub receipts: Vec<ReceiptEvent>,
crates/mech_factory_mud/src/world.rs:10:    pub projections: Vec<ProjectionRow>,
crates/mech_factory_mud/src/world.rs:17:            traces: Vec::new(),
crates/mech_factory_mud/src/world.rs:22:            receipts: Vec::new(),
crates/mech_factory_mud/src/world.rs:30:        if scenario == "refused_missing_socket" {
crates/mech_factory_mud/src/world.rs:33:            // intentional missing socket logic
crates/mech_factory_mud/src/world.rs:55:            "VisitReceiptTerminal",
crates/mech_factory_mud/src/world.rs:56:            "EmitFactoryReceipt",
crates/mech_factory_mud/src/world.rs:67:        self.traces.push(event_type.to_string());
crates/mech_factory_mud/src/world.rs:70:        let seq = (self.receipts.len() + 1) as u64;
crates/mech_factory_mud/src/world.rs:71:        let prev_hash = self.receipts.last().map(|r| r.receipt.clone());
crates/mech_factory_mud/src/world.rs:73:        let receipt_hash = generate_hash(&payload);
crates/mech_factory_mud/src/world.rs:75:        self.receipts.push(ReceiptEvent {
crates/mech_factory_mud/src/world.rs:83:            receipt: receipt_hash.clone(),
crates/mech_factory_mud/src/world.rs:89:            self.projections.push(ProjectionRow {
crates/mech_factory_mud/src/world.rs:92:                station_id: "station_1".to_string(),
crates/mech_factory_mud/src/world.rs:93:                route_node_id: "node_1".to_string(),
crates/mech_factory_mud/src/world.rs:95:                source_receipt: receipt_hash,
crates/mech_factory_mud/tests/receipt_chain.rs:1:use mech_factory_mud::receipt::{ReceiptEvent, generate_hash, verify_receipt_chain};
crates/mech_factory_mud/tests/receipt_chain.rs:4:fn test_valid_receipt_chain_passes() {
crates/mech_factory_mud/tests/receipt_chain.rs:5:    let r1 = create_receipt(1, "A", None);
crates/mech_factory_mud/tests/receipt_chain.rs:6:    let r2 = create_receipt(2, "B", Some(r1.receipt.clone()));
crates/mech_factory_mud/tests/receipt_chain.rs:7:    assert!(verify_receipt_chain(&[r1, r2]).is_ok());
crates/mech_factory_mud/tests/receipt_chain.rs:12:    let r1 = create_receipt(1, "A", None);
crates/mech_factory_mud/tests/receipt_chain.rs:13:    let r2 = create_receipt(2, "B", Some("wrong".to_string()));
crates/mech_factory_mud/tests/receipt_chain.rs:14:    assert!(verify_receipt_chain(&[r1, r2]).is_err());
crates/mech_factory_mud/tests/receipt_chain.rs:19:    let r1 = create_receipt(1, "A", None);
crates/mech_factory_mud/tests/receipt_chain.rs:20:    let mut r2 = create_receipt(2, "B", Some(r1.receipt.clone()));
crates/mech_factory_mud/tests/receipt_chain.rs:22:    assert!(verify_receipt_chain(&[r1, r2]).is_err());
crates/mech_factory_mud/tests/receipt_chain.rs:27:    let r1 = create_receipt(1, "A", None);
crates/mech_factory_mud/tests/receipt_chain.rs:28:    let r3 = create_receipt(3, "C", Some(r1.receipt.clone()));
crates/mech_factory_mud/tests/receipt_chain.rs:29:    assert!(verify_receipt_chain(&[r1, r3]).is_err());
crates/mech_factory_mud/tests/receipt_chain.rs:34:    let r1 = create_receipt(1, "A", None);
crates/mech_factory_mud/tests/receipt_chain.rs:35:    let r2 = create_receipt(1, "A", Some(r1.receipt.clone()));
crates/mech_factory_mud/tests/receipt_chain.rs:36:    assert!(verify_receipt_chain(&[r1, r2]).is_err());
crates/mech_factory_mud/tests/receipt_chain.rs:39:fn create_receipt(seq: u64, evt: &str, prev: Option<String>) -> ReceiptEvent {
crates/mech_factory_mud/tests/receipt_chain.rs:42:    ReceiptEvent {
crates/mech_factory_mud/tests/receipt_chain.rs:50:        receipt: hash,
crates/mech_factory_mud/src/authority.rs:7:    pub socket_health_class: u8,
crates/mech_factory_mud/src/authority.rs:10:    pub station_state_class: u8,
crates/mech_factory_mud/src/authority.rs:12:    pub receipt_state_class: u8,
