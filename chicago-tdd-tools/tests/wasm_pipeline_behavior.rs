use un_test_utils::UnrealEnvMock;
use unify_wasm::packager::{WasmPackager, PackageOptions};
use std::fs;

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
        env_mock.root.path().to_path_buf(),
        env_mock.project_path.join("MyProject.uproject"),
    );

    let options = PackageOptions {
        source_t3d: t3d_path.clone(),
        destination_dir: output_dir.clone(),
        map_name: "ManufacturedWorld".to_string(),
    };

    let staging_dir = env_mock.project_path.parent().unwrap()
        .join("MyProject")
        .join("Saved")
        .join("StagedBuilds")
        .join("HTML5");
        
    fs::create_dir_all(&staging_dir).expect("Failed to create staging dir");
    fs::write(staging_dir.join("MyProject-HTML5-Shipping.html"), "mock html").unwrap();
    fs::write(staging_dir.join("MyProject-HTML5-Shipping.js"), "mock js").unwrap();
    fs::write(staging_dir.join("MyProject-HTML5-Shipping.wasm"), "mock wasm").unwrap();
    fs::write(staging_dir.join("MyProject-HTML5-Shipping.data"), "mock data").unwrap();

    // 3. Act
    let result = packager.package_html5(&options);

    // 4. Assert
    assert!(result.is_ok(), "Packaging should succeed: {:?}", result.err());
    
    // Actually, WasmPackager should copy the results to output_dir
    assert!(output_dir.exists(), "Output directory should exist");
    assert!(output_dir.join("MyProject-HTML5-Shipping.wasm").exists(), "Wasm file should be packaged");
    
    // In our test, UAT is mocked, so we just check if it executed without error
    // In a real Chicago TDD, if we simulate the engine, we might want the mock to emit files.
    // un_test_utils writes "mock uat" as an executable.
}

#[test]
fn should_pass_es3_and_webgl2_flags_to_uat() {
    // 1. Setup Simulated External Environment (Unreal Engine)
    let env_mock = UnrealEnvMock::new().expect("Failed to create UnrealEnvMock");
    env_mock.setup_env();

    // Setup input artifact
    let t3d_path = env_mock.project_path.join("map.t3d");
    fs::write(&t3d_path, "Begin Actor\nEnd Actor").expect("Failed to write mock t3d");

    // Setup output destination
    let output_dir = env_mock.root.path().join("manufactured");
    let log_path = env_mock.root.path().join("uat_args.log");

    // Overwrite the mock UAT to record its arguments
    let uat_path = env_mock.uat_path();
    #[cfg(windows)]
    {
        fs::write(&uat_path, format!("@echo %* > \"{}\"\nexit /b 0", log_path.display())).unwrap();
    }
    #[cfg(not(windows))]
    {
        let script_content = format!(
            "#!/bin/sh\necho \"$@\" > \"{}\"\nexit 0\n",
            log_path.display()
        );
        fs::write(&uat_path, script_content).unwrap();
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&uat_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&uat_path, perms).unwrap();
        }
    }

    // 2. Setup System Under Test (SUT)
    let packager = WasmPackager::new(
        env_mock.root.path().to_path_buf(),
        env_mock.project_path.join("MyProject.uproject"),
    );

    let options = PackageOptions {
        source_t3d: t3d_path.clone(),
        destination_dir: output_dir.clone(),
        map_name: "ManufacturedWorld".to_string(),
    };

    // Setup staging dir so packaging doesn't fail on copying
    let staging_dir = env_mock.project_path.parent().unwrap()
        .join("MyProject")
        .join("Saved")
        .join("StagedBuilds")
        .join("HTML5");
        
    fs::create_dir_all(&staging_dir).expect("Failed to create staging dir");
    fs::write(staging_dir.join("MyProject-HTML5-Shipping.html"), "mock html").unwrap();
    fs::write(staging_dir.join("MyProject-HTML5-Shipping.js"), "mock js").unwrap();
    fs::write(staging_dir.join("MyProject-HTML5-Shipping.wasm"), "mock wasm").unwrap();
    fs::write(staging_dir.join("MyProject-HTML5-Shipping.data"), "mock data").unwrap();

    // 3. Act
    let result = packager.package_html5(&options);

    // 4. Assert
    assert!(result.is_ok(), "Packaging should succeed: {:?}", result.err());
    
    // Check the recorded arguments
    let recorded_args = fs::read_to_string(&log_path).expect("Failed to read uat_args.log");
    
    assert!(recorded_args.contains("-es3"), "UAT should be called with -es3 flag");
    assert!(recorded_args.contains("-webgl2"), "UAT should be called with -webgl2 flag");
    assert!(recorded_args.contains("-platform=HTML5"), "UAT should be called with -platform=HTML5");
}
