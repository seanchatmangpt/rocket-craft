use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    // Dynamically retrieve home directory to avoid hardcoded /Users/ paths
    let home = env::var("HOME").expect("HOME environment variable is not set");
    let manifest_path = format!("{}/cargo-cicd/Cargo.toml", home);

    // Delegate execution to the patched cargo-cicd workspace binary
    let status = Command::new("cargo")
        .arg("run")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("--bin")
        .arg("cargo-cicd")
        .arg("--")
        .args(&args)
        .status()
        .expect("Failed to execute cargo-cicd wrapper");

    std::process::exit(status.code().unwrap_or(1));
}
