#[path = "../analyzers/mod.rs"]
pub mod analyzers;
#[path = "../diagnostics.rs"]
mod diagnostics;
use diagnostics::run_diagnostics;
use lsp_types_max::NumberOrString;
use std::fs;
use std::path::PathBuf;

fn main() {
    let usd_dir = PathBuf::from(
        "../../generated/mech_assets/reference_fabric_001/usd",
    );
    let mut total = 0usize;
    let mut binding_errors = 0usize;
    for entry in walkdir::WalkDir::new(&usd_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) != Some("usda") {
            continue;
        }
        let content = match fs::read_to_string(p) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let diags = run_diagnostics(&p.to_path_buf(), &content);
        for d in diags {
            let code = d
                .code
                .clone()
                .map(|c| match c {
                    NumberOrString::Number(n) => n.to_string(),
                    NumberOrString::String(s) => s,
                })
                .unwrap_or_default();
            if code == "missing-material-binding" || code == "invalid-material-binding" {
                binding_errors += 1;
                println!(
                    "[{}] {}:{} {}",
                    code,
                    p.display(),
                    d.range.start.line,
                    d.message
                );
            }
        }
        total += 1;
    }
    println!("SCANNED {} usda files", total);
    println!("BINDING_ERRORS {}", binding_errors);
    if binding_errors == 0 {
        println!("PASSED: zero missing/invalid material-binding");
    } else {
        println!("FAILED");
        std::process::exit(1);
    }
}
