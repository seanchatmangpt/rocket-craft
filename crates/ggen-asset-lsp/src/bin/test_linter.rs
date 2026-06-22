#[path = "../analyzers/mod.rs"]
pub mod analyzers;

#[path = "../diagnostics.rs"]
mod diagnostics;
use diagnostics::run_diagnostics;
use lsp_types_max::NumberOrString;
use std::fs;
use std::path::PathBuf;

fn main() {
    let doc_path = PathBuf::from("../../generated/mech_assets/reference_fabric_001/usd/SM_Head.usda");
    let content = fs::read_to_string(&doc_path).unwrap();
    let diags = run_diagnostics(&doc_path, &content);
    
    if diags.is_empty() {
        println!("PASSED: No diagnostics found!");
    } else {
        for d in diags {
            println!("{:?}: {}", d.code, d.message);
        }
    }
}
