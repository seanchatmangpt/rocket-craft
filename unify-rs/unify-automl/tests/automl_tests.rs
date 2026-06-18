use std::fs;
use unify_automl::balancer::optimize_balance;
use unify_automl::cli::dispatch_dev_command;
use unify_automl::discovery::scan_directory;

#[test]
fn test_auto_binding_unconfigured_component() {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_dir = std::env::temp_dir().join(format!("unify_auto_bind_{}", ts));
    fs::create_dir_all(&temp_dir).unwrap();

    let comp_file = temp_dir.join("MyNewUnconfiguredComponent.rs");
    fs::write(
        &comp_file,
        "// @UnifyAutoBind: MyNewUnconfiguredComponent\n",
    )
    .unwrap();

    let registry = scan_directory(&temp_dir).expect("failed to scan directory");
    let found_comp = registry
        .components
        .iter()
        .find(|c| c.name == "MyNewUnconfiguredComponent");
    assert!(
        found_comp.is_some(),
        "MyNewUnconfiguredComponent not found in registry"
    );

    // Simulate auto-binding/wiring
    let mut wired_components = Vec::new();
    for comp in registry.components {
        if comp.binding_tag.contains("@UnifyAutoBind") {
            wired_components.push(comp.name.clone());
        }
    }
    assert!(wired_components.contains(&"MyNewUnconfiguredComponent".to_string()));

    fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn test_game_balance_optimizer() {
    let res = optimize_balance(4, 0.5, 5).expect("failed to run game balance optimizer");
    let total_points = res.allocation.health
        + res.allocation.attack
        + res.allocation.defense
        + res.allocation.magic;
    assert_eq!(total_points, 4, "Total points allocated should equal 4");
    assert!(
        res.player_win_rate >= 0.0 && res.player_win_rate <= 1.0,
        "Win rate should be between 0.0 and 1.0"
    );
    assert!(res.avg_turns >= 0.0, "Average turns should be non-negative");
}

#[test]
fn test_developer_cli_scaffolding_and_lifecycle() {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_dir = std::env::temp_dir().join(format!("unify_dev_env_{}", ts));

    // Execute dev init on a temporary path
    let init_args = vec!["init".to_string(), temp_dir.to_string_lossy().into_owned()];
    let output = dispatch_dev_command(&init_args).expect("failed to execute dev init");
    assert!(output.success, "dev init should succeed");

    // Verify the temporary path is populated with dev_config.json and test_component.rs
    assert!(
        temp_dir.join("dev_config.json").exists(),
        "dev_config.json does not exist"
    );
    assert!(
        temp_dir.join("test_component.rs").exists(),
        "test_component.rs does not exist"
    );

    // Verify the auto-binding test component is discoverable from the scaffolded path
    let registry = scan_directory(&temp_dir).expect("failed to scan scaffolded path");
    let found = registry
        .components
        .iter()
        .any(|c| c.name == "TempComponent");
    assert!(
        found,
        "TempComponent should be discoverable from the scaffolded path"
    );

    fs::remove_dir_all(&temp_dir).ok();
}
