use std::path::PathBuf;
use std::process::Command;
use anyhow::Result;
use std::fs;

pub struct PackageOptions {
    pub source_t3d: PathBuf,
    pub destination_dir: PathBuf,
    pub map_name: String,
}

pub struct WasmPackager {
    engine_path: PathBuf,
    project_uproject: PathBuf,
}

impl WasmPackager {
    pub fn new(engine_path: PathBuf, project_uproject: PathBuf) -> Self {
        Self {
            engine_path,
            project_uproject,
        }
    }

    fn get_editor_path(&self) -> PathBuf {
        if cfg!(windows) {
            self.engine_path.join("Engine/Binaries/Win64/UE4Editor.exe")
        } else if cfg!(target_os = "macos") {
            self.engine_path.join("Engine/Binaries/Mac/UE4Editor")
        } else {
            self.engine_path.join("Engine/Binaries/Linux/UE4Editor")
        }
    }

    fn get_uat_path(&self) -> PathBuf {
        if cfg!(windows) {
            let path_bat = self.engine_path.join("Engine/Build/BatchFiles/RunUAT.bat");
            if path_bat.exists() {
                path_bat
            } else {
                self.engine_path.join("Engine/Binaries/DotNET/AutomationTool.exe")
            }
        } else {
            let path_sh = self.engine_path.join("Engine/Build/BatchFiles/RunUAT.sh");
            if path_sh.exists() {
                path_sh
            } else {
                self.engine_path.join("Engine/Binaries/DotNET/AutomationTool")
            }
        }
    }

    fn get_project_name(&self) -> String {
        self.project_uproject
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("UE4Game")
            .to_string()
    }

    pub fn package_html5(&self, options: &PackageOptions) -> Result<()> {
        // Step 1: Import Assets
        let editor_path = self.get_editor_path();

        let mut editor_cmd = Command::new(&editor_path);
        editor_cmd.arg(self.project_uproject.display().to_string())
            .arg("-run=ImportAssets")
            .arg(format!("-source={}", options.source_t3d.display()))
            .arg(format!("-dest=Game/Content/Maps/{}", options.map_name))
            .arg("-NoUI")
            .arg("-stdout")
            .arg("-AllowCommandletRendering");
            
        let status = editor_cmd.status()
            .map_err(|e| anyhow::anyhow!("Failed to run asset import command: {}", e))?;
        if !status.success() {
            return Err(anyhow::anyhow!("Asset import command failed with exit code: {:?}", status.code()));
        }

        // Step 2: Build and Package
        let uat_path = self.get_uat_path();
        
        let mut uat_cmd = Command::new(&uat_path);

        uat_cmd.arg("BuildCookRun")
            .arg(format!("-project={}", self.project_uproject.display()))
            .arg("-noP4")
            .arg("-platform=HTML5")
            .arg("-clientconfig=Shipping")
            .arg("-cook")
            .arg("-stage")
            .arg("-package")
            .arg(format!("-map={}", options.map_name))
            .arg("-pak")
            .arg("-prereqs")
            .arg("-nodebuginfo")
            .arg("-targetplatform=HTML5")
            .arg("-utf8output")
            .arg("-es3")
            .arg("-webgl2");

        let status = uat_cmd.status()
            .map_err(|e| anyhow::anyhow!("Failed to run UAT packaging command: {}", e))?;
        if !status.success() {
            return Err(anyhow::anyhow!("UAT packaging command failed with exit code: {:?}", status.code()));
        }

        // Create the destination dir
        fs::create_dir_all(&options.destination_dir)?;

        // Output would be in Saved/StagedBuilds/HTML5
        let project_dir = self.project_uproject.parent().unwrap();
        let staging_dir = project_dir.join("Saved").join("StagedBuilds").join("HTML5");

        if !staging_dir.exists() {
            return Err(anyhow::anyhow!(
                "HTML5 Staging directory not found: {}",
                staging_dir.display()
            ));
        }

        let mut copied_count = 0;
        // Copy files
        for entry in fs::read_dir(&staging_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let dest_file = options.destination_dir.join(path.file_name().unwrap());
                fs::copy(&path, &dest_file)?;
                copied_count += 1;
            }
        }

        if copied_count == 0 {
            return Err(anyhow::anyhow!(
                "No packaged files found in HTML5 staging directory: {}",
                staging_dir.display()
            ));
        }

        // Verify the expected HTML5 files were copied
        let project_name = self.get_project_name();
        let expected_files = [
            format!("{}-HTML5-Shipping.html", project_name),
            format!("{}-HTML5-Shipping.js", project_name),
            format!("{}-HTML5-Shipping.wasm", project_name),
            format!("{}-HTML5-Shipping.data", project_name),
        ];
        for file_name in &expected_files {
            let file_path = options.destination_dir.join(file_name);
            if !file_path.exists() {
                return Err(anyhow::anyhow!(
                    "Expected packaged HTML5 file not found in destination: {}",
                    file_path.display()
                ));
            }
        }

        Ok(())
    }

    pub fn package_windows(&self, options: &PackageOptions) -> Result<()> {
        // Step 1: Import Assets
        let editor_path = self.get_editor_path();

        let mut editor_cmd = Command::new(&editor_path);
        editor_cmd.arg(self.project_uproject.display().to_string())
            .arg("-run=ImportAssets")
            .arg(format!("-source={}", options.source_t3d.display()))
            .arg(format!("-dest=Game/Content/Maps/{}", options.map_name))
            .arg("-NoUI")
            .arg("-stdout")
            .arg("-AllowCommandletRendering");
            
        let status = editor_cmd.status()
            .map_err(|e| anyhow::anyhow!("Failed to run asset import command: {}", e))?;
        if !status.success() {
            return Err(anyhow::anyhow!("Asset import command failed with exit code: {:?}", status.code()));
        }

        // Step 2: Build and Package for Win64
        let uat_path = self.get_uat_path();

        let mut uat_cmd = Command::new(&uat_path);
        uat_cmd.arg("BuildCookRun")
            .arg(format!("-project={}", self.project_uproject.display()))
            .arg("-noP4")
            .arg("-platform=Win64")
            .arg("-clientconfig=Shipping")
            .arg("-cook")
            .arg("-stage")
            .arg("-package")
            .arg(format!("-map={}", options.map_name))
            .arg("-pak")
            .arg("-prereqs")
            .arg("-nodebuginfo")
            .arg("-targetplatform=Win64")
            .arg("-utf8output");

        let status = uat_cmd.status()
            .map_err(|e| anyhow::anyhow!("Failed to run UAT packaging command: {}", e))?;
        if !status.success() {
            return Err(anyhow::anyhow!("UAT packaging command failed with exit code: {:?}", status.code()));
        }

        // Create the destination dir under destination_dir/win64
        let win64_dest_dir = options.destination_dir.join("win64");
        fs::create_dir_all(&win64_dest_dir)?;

        // Output would be in Saved/StagedBuilds/Win64
        let project_dir = self.project_uproject.parent().unwrap();
        let staging_dir = project_dir.join("Saved").join("StagedBuilds").join("Win64");

        if !staging_dir.exists() {
            return Err(anyhow::anyhow!(
                "Win64 Staging directory not found: {}",
                staging_dir.display()
            ));
        }

        let mut copied_count = 0;
        // Copy files
        for entry in fs::read_dir(&staging_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let dest_file = win64_dest_dir.join(path.file_name().unwrap());
                fs::copy(&path, &dest_file)?;
                copied_count += 1;
            }
        }

        if copied_count == 0 {
            return Err(anyhow::anyhow!(
                "No packaged files found in Win64 staging directory: {}",
                staging_dir.display()
            ));
        }

        // Verify Windows executable
        let project_name = self.get_project_name();
        let exe_path = win64_dest_dir.join(format!("{}-Windows-Shipping.exe", project_name));
        if !exe_path.exists() {
            // UE4 sometimes uses the project name differently for the executable
            // but for Win64 Shipping it's usually ProjectName-Windows-Shipping.exe or just ProjectName.exe
            // Actually, let's just check if ANY .exe exists if the specific one doesn't
            if !win64_dest_dir.read_dir()?.any(|e| e.map(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("exe")).unwrap_or(false)) {
                return Err(anyhow::anyhow!(
                    "Expected Win64 executable not found in: {}",
                    win64_dest_dir.display()
                ));
            }
        }

        Ok(())
    }

    pub fn package_linux(&self, options: &PackageOptions) -> Result<()> {
        // Step 1: Import Assets
        let editor_path = self.get_editor_path();

        let mut editor_cmd = Command::new(&editor_path);
        editor_cmd.arg(self.project_uproject.display().to_string())
            .arg("-run=ImportAssets")
            .arg(format!("-source={}", options.source_t3d.display()))
            .arg(format!("-dest=Game/Content/Maps/{}", options.map_name))
            .arg("-NoUI")
            .arg("-stdout")
            .arg("-AllowCommandletRendering");
            
        let status = editor_cmd.status()
            .map_err(|e| anyhow::anyhow!("Failed to run asset import command: {}", e))?;
        if !status.success() {
            return Err(anyhow::anyhow!("Asset import command failed with exit code: {:?}", status.code()));
        }

        // Step 2: Build and Package for Linux
        let uat_path = self.get_uat_path();

        let mut uat_cmd = Command::new(&uat_path);
        uat_cmd.arg("BuildCookRun")
            .arg(format!("-project={}", self.project_uproject.display()))
            .arg("-noP4")
            .arg("-platform=Linux")
            .arg("-clientconfig=Shipping")
            .arg("-cook")
            .arg("-stage")
            .arg("-package")
            .arg(format!("-map={}", options.map_name))
            .arg("-pak")
            .arg("-prereqs")
            .arg("-nodebuginfo")
            .arg("-targetplatform=Linux")
            .arg("-utf8output");

        let status = uat_cmd.status()
            .map_err(|e| anyhow::anyhow!("Failed to run UAT packaging command: {}", e))?;
        if !status.success() {
            return Err(anyhow::anyhow!("UAT packaging command failed with exit code: {:?}", status.code()));
        }

        // Create the destination dir under destination_dir/linux
        let linux_dest_dir = options.destination_dir.join("linux");
        fs::create_dir_all(&linux_dest_dir)?;

        // Output would be in Saved/StagedBuilds/Linux
        let project_dir = self.project_uproject.parent().unwrap();
        let staging_dir = project_dir.join("Saved").join("StagedBuilds").join("Linux");

        if !staging_dir.exists() {
            return Err(anyhow::anyhow!(
                "Linux Staging directory not found: {}",
                staging_dir.display()
            ));
        }

        let mut copied_count = 0;
        // Copy files
        for entry in fs::read_dir(&staging_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let dest_file = linux_dest_dir.join(path.file_name().unwrap());
                fs::copy(&path, &dest_file)?;
                copied_count += 1;
            }
        }

        if copied_count == 0 {
            return Err(anyhow::anyhow!(
                "No packaged files found in Linux staging directory: {}",
                staging_dir.display()
            ));
        }

        // Verify Linux binary
        let project_name = self.get_project_name();
        let sh_path = linux_dest_dir.join(format!("{}-Linux-Shipping.sh", project_name));
        if !sh_path.exists() {
            // Also check for just ProjectName.sh or no extension binary
            if !linux_dest_dir.read_dir()?.any(|e| e.map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                name.contains(project_name.as_str()) || name.ends_with(".sh")
            }).unwrap_or(false)) {
                return Err(anyhow::anyhow!(
                    "Expected Linux script/binary not found in: {}",
                    linux_dest_dir.display()
                ));
            }
        }

        Ok(())
    }
}
