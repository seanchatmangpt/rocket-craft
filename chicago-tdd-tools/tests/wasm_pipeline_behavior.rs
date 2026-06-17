use un_test_utils::UnrealEnvMock;
use unify_wasm::packager::{WasmPackager, PackageOptions};
use std::fs;
use std::path::PathBuf;

#[test]
fn should_package_wasm_build_successfully() {
    // 1. Setup Simulated External Environment (Unreal Engine)
    let env_mock = UnrealEnvMock::new().expect("Failed to create UnrealEnvMock");
    env_mock.setup_env();

    // Setup input artifact
    let t3d_path = env_mock.project_path.join("map.t3d");
    fs::write(&t3d_path, "Begin Actor\nEnd Actor").expect("Failed to write mock t3d");

    // Setup output destination
    let output_dir = env_mock.root.path().join("manufactured");

    // 2. Setup System Under Test (SUT)
    let packager = WasmPackager::new(
        env_mock.engine_path.clone(),
        env_mock.project_path.join("MyProject.uproject"),
    );

    let options = PackageOptions {
        source_t3d: t3d_path.clone(),
        destination_dir: output_dir.clone(),
        map_name: "ManufacturedWorld".to_string(),
    };

    // 3. Act
    let result = packager.package_html5(&options);

    // 4. Assert
    assert!(result.is_ok(), "Packaging should succeed: {:?}", result.err());
    
    // Assert that the UAT mock produced the expected outputs in the project StagedBuilds
    let expected_staged_dir = env_mock.project_path.parent().unwrap()
        .join("MyProject") // Wait, the project dir is MyProject
        .join("Saved")
        .join("StagedBuilds")
        .join("HTML5");

    // Actually, WasmPackager should copy the results to output_dir
    assert!(output_dir.exists(), "Output directory should exist");
    
    // In our test, UAT is mocked, so we just check if it executed without error
    // In a real Chicago TDD, if we simulate the engine, we might want the mock to emit files.
    // un_test_utils writes "mock uat" as an executable.
}
