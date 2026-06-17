use std::path::PathBuf;
use tempfile::TempDir;
use std::fs;
use mockall::automock;

/// A trait for executing commands related to Unreal Engine.
///
/// This abstraction allows for mocking command execution in unit tests.
#[automock]
pub trait UnrealCommandExecutor {
    /// Executes a command with the given arguments.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to start or exits with a non-zero status.
    fn exec(&self, command: &str, args: &[String]) -> anyhow::Result<String>;
}

/// A mock environment that simulates an Unreal Engine project structure.
///
/// This is used in unit tests to provide a realistic file system layout for
/// engine and project directories without requiring a full engine installation.
pub struct UnrealEnvMock {
    /// The root temporary directory containing the mock environment.
    pub root: TempDir,
    /// The simulated path to the engine installation.
    pub engine_path: PathBuf,
    /// The simulated path to the project directory.
    pub project_path: PathBuf,
}

impl UnrealEnvMock {
    /// Creates a new `UnrealEnvMock` with a basic engine and project structure.
    ///
    /// # Errors
    ///
    /// Returns an error if the temporary directory or file structure cannot be created.
    pub fn new() -> anyhow::Result<Self> {
        let root = TempDir::new()?;
        let root_path = root.path();

        let engine_path = root_path.join("Engine");
        fs::create_dir_all(engine_path.join("Binaries/Win64"))?;
        fs::create_dir_all(engine_path.join("Binaries/DotNET"))?;
        
        // Mock Editor, UBT, and UAT
        #[cfg(windows)]
        {
            fs::write(engine_path.join("Binaries/DotNET/UnrealBuildTool.exe"), "@echo off\nexit /b 0")?;
            fs::write(engine_path.join("Binaries/DotNET/AutomationTool.exe"), "@echo off\nexit /b 0")?;
            fs::write(engine_path.join("Binaries/Win64/UE4Editor.exe"), "@echo off\nexit /b 0")?;
        }
        #[cfg(not(windows))]
        {
            let ubt_path = engine_path.join("Binaries/DotNET/UnrealBuildTool");
            let uat_path = engine_path.join("Binaries/DotNET/AutomationTool");
            
            let mac_editor_dir = engine_path.join("Binaries/Mac");
            let linux_editor_dir = engine_path.join("Binaries/Linux");
            fs::create_dir_all(&mac_editor_dir)?;
            fs::create_dir_all(&linux_editor_dir)?;
            
            let mac_editor_path = mac_editor_dir.join("UE4Editor");
            let linux_editor_path = linux_editor_dir.join("UE4Editor");

            let script_content = "#!/bin/sh\nexit 0\n";
            fs::write(&ubt_path, script_content)?;
            fs::write(&uat_path, script_content)?;
            fs::write(&mac_editor_path, script_content)?;
            fs::write(&linux_editor_path, script_content)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                for path in [&ubt_path, &uat_path, &mac_editor_path, &linux_editor_path] {
                    let mut perms = fs::metadata(path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(path, perms)?;
                }
            }
        }

        let project_path = root_path.join("MyProject");
        fs::create_dir_all(project_path.join("Source"))?;
        fs::create_dir_all(project_path.join("Config"))?;
        fs::write(project_path.join("MyProject.uproject"), "{}")?;

        Ok(Self {
            root,
            engine_path,
            project_path,
        })
    }

    /// Creates a mock plugin within the project.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin directory or file cannot be created.
    pub fn create_plugin(&self, name: &str) -> anyhow::Result<PathBuf> {
        let plugin_path = self.project_path.join("Plugins").join(name);
        fs::create_dir_all(plugin_path.join("Source"))?;
        fs::write(plugin_path.join(format!("{}.uplugin", name)), "{}")?;
        Ok(plugin_path)
    }

    /// Overwrites the .uproject file with the specified content.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn write_uproject(&self, content: &str) -> anyhow::Result<()> {
        let name = self.project_path.file_name().unwrap().to_str().unwrap();
        fs::write(self.project_path.join(format!("{}.uproject", name)), content)?;
        Ok(())
    }

    /// Returns the path to the mock Unreal Build Tool (UBT).
    pub fn ubt_path(&self) -> PathBuf {
        let mut path = self.engine_path.join("Binaries/DotNET/UnrealBuildTool");
        if cfg!(windows) {
            path.set_extension("exe");
        }
        path
    }

    /// Returns the path to the mock Unreal Automation Tool (UAT).
    pub fn uat_path(&self) -> PathBuf {
        let mut path = self.engine_path.join("Binaries/DotNET/AutomationTool");
        if cfg!(windows) {
            path.set_extension("exe");
        }
        path
    }

    /// Sets environment variables pointing to the mock engine and project paths.
    pub fn setup_env(&self) {
        unsafe {
            std::env::set_var("UNREAL_ENGINE_PATH", &self.engine_path);
            std::env::set_var("PROJECT_PATH", &self.project_path);
        }
    }
}
