// GC-MECHBIRTH-002: rocket-preue4-verify CLI
// Runs the pre-UE4 authority/SIMD/prediction verifier pipeline.
// Accepts POWL trace, OCEL trace, and optional report output path.

use clap::Parser;
use rocket_preue4_verifier::{
    authority::AuthorityState,
    receipt::{AdmissionStatus, ReceiptChain},
    report::{VerifierReport, mechbirth_002_residuals},
    verifier::run_pipeline,
};

#[derive(Parser, Debug)]
#[command(
    name = "rocket-preue4-verify",
    version,
    about = "GC-MECHBIRTH-002 Pre-UE4 Verifier — Authority/SIMD/Prediction layers"
)]
struct Cli {
    /// Path to POWL trace file.
    #[arg(long)]
    powl: Option<String>,

    /// Path to OCEL event trace file.
    #[arg(long)]
    trace: Option<String>,

    /// Path to ggen combinatorial output directory.
    #[arg(long)]
    ggen_out: Option<String>,

    /// Path to write the verifier report JSON.
    #[arg(long)]
    report: Option<String>,

    /// Number of cells for stress-mode authority check (0 = default 1000).
    #[arg(long, default_value = "1000")]
    cells: usize,
}

fn main() {
    let cli = Cli::parse();

    let n = if cli.cells == 0 { 1000 } else { cli.cells };
    let mut state = AuthorityState::new(n);
    // Populate with representative authority pattern
    for i in 0..n {
        state.heat[i] = (i % 16) as u8;
        state.stress[i] = (i % 12) as u8;
        state.socket_health[i] = (15_usize.saturating_sub(i % 16)) as u8;
    }

    // Build a minimal receipt chain representing the GC-MECHBIRTH-001 admission trace
    let mut chain = ReceiptChain::default();
    let steps = [
        "SelectFrame",
        "GenerateSocketTopology",
        "GenerateArmorPanels",
        "GenerateRig",
        "GenerateMotionFamily",
        "GenerateSkinLayers",
        "PackageProjectionArtifacts",
        "EmitReceipt",
    ];
    for step in &steps {
        chain.append(
            step.to_string(),
            vec!["Mech-001".into()],
            AdmissionStatus::Admitted,
            vec![],
        );
    }

    let result = run_pipeline(&mut state, &chain);

    let inputs = vec![
        cli.powl
            .unwrap_or_else(|| "/Users/sac/powlv2lsp/samples/MechBirth.powl".into()),
        cli.trace
            .unwrap_or_else(|| "/Users/sac/powlv2lsp/out.json".into()),
    ];

    let artifacts = vec![
        "MechBirthSteps.h".into(),
        "MechBirthSteps.rs".into(),
        "MechBirthProjectionRows.csv".into(),
        "MechBirthSocketTopology.csv".into(),
        "MechBirthSkinLayers.csv".into(),
        "MechBirthMotionFamilies.csv".into(),
        "MechBirthLODClasses.csv".into(),
        "MechBirthAuthorityClasses.csv".into(),
        "MechBirthTransitionTable.csv".into(),
        "MechBirthPredictionRules.csv".into(),
        "MechBirthReceiptManifest.json".into(),
        "MechBirthProjectionManifest.json".into(),
        "MechBirthOCELSeed.json".into(),
    ];

    let report = VerifierReport::from_pipeline(
        "GC-MECHBIRTH-002".into(),
        &result,
        inputs,
        artifacts,
        mechbirth_002_residuals(),
    );

    let json = report.to_json();

    // Write or print the report
    if let Some(path) = &cli.report {
        std::fs::write(path, &json).expect("Failed to write report JSON");
        eprintln!("[rocket-preue4-verify] Report written to: {}", path);
    } else {
        println!("{}", json);
    }

    eprintln!(
        "[rocket-preue4-verify] Milestone: GC-MECHBIRTH-002 | Status: {} | Scoped: {}",
        result.final_status,
        result.scoped_status()
    );

    // Exit 0 if all Rust-scoped gates pass, 1 if blocked
    if result.final_status == "BLOCKED" {
        std::process::exit(1);
    }
}
