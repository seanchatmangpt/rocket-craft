use std::path::PathBuf;
use tempfile::TempDir;
use std::fs;

/// Manages a temporary file system environment for testing.
///
/// This structure provides utilities for creating and reading files within
/// a temporary directory, ensuring that tests are isolated and don't
/// pollute the host file system.
pub struct TestEnvironment {
    /// The temporary directory used for the test.
    pub temp_dir: TempDir,
    /// The original working directory before the test started.
    pub original_dir: PathBuf,
}

impl TestEnvironment {
    /// Creates a new `TestEnvironment` with a fresh temporary directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the temporary directory or current working
    /// directory cannot be accessed.
    pub fn new() -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let original_dir = std::env::current_dir()?;
        
        // We don't change directory here, because it's global and can affect other tests.
        // Instead, the SUT should be able to accept a base path.
        
        Ok(Self {
            temp_dir,
            original_dir,
        })
    }

    /// Returns the absolute path to the temporary directory.
    pub fn path(&self) -> PathBuf {
        self.temp_dir.path().to_path_buf()
    }

    /// Creates a file with the given content at the specified relative path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file or its parent directories cannot be created.
    pub fn create_file(&self, relative_path: &str, content: &str) -> anyhow::Result<()> {
        let path = self.path().join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    /// Reads the content of a file at the specified relative path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    pub fn read_file(&self, relative_path: &str) -> anyhow::Result<String> {
        let path = self.path().join(relative_path);
        Ok(fs::read_to_string(path)?)
    }

    /// Checks if a file exists at the specified relative path.
    pub fn exists(&self, relative_path: &str) -> bool {
        self.path().join(relative_path).exists()
    }
}
