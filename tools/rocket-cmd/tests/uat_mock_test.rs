use anyhow::Result;
use un_test_utils::{MockUnrealCommandExecutor, UnrealCommandExecutor, UnrealEnvMock};

#[test]
fn test_uat_execution_mock() -> Result<()> {
    // 1. Setup the mock environment
    let env = UnrealEnvMock::new()?;
    env.setup_env();

    // 2. Setup the mock executor
    let mut mock_executor = MockUnrealCommandExecutor::new();

    // Define the expected call to UAT
    mock_executor
        .expect_exec()
        .withf(|command, args| {
            command.ends_with("AutomationTool")
                && args.iter().any(|a| a == "-cook")
                && args.iter().any(|a| a == "-build")
        })
        .times(1)
        .returning(|_, _| Ok("Build Successful".to_string()));

    // 3. Simulate what run_build would do
    let uat_path = env.uat_path();
    let args = vec![
        "BuildCookRun".to_string(),
        "-project=MyProject.uproject".to_string(),
        "-cook".to_string(),
        "-build".to_string(),
    ];

    let result = mock_executor.exec(uat_path.to_str().unwrap(), &args)?;

    // 4. Verify
    assert_eq!(result, "Build Successful");

    Ok(())
}
