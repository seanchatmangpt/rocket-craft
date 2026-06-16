//! GoldenFile — snapshot / golden-file comparison for CLI output or serialised
//! data structures.  On first run the file is created; subsequent runs compare.

use std::path::PathBuf;

/// Wraps a path to a golden (snapshot) file and provides comparison helpers.
pub struct GoldenFile {
    path: PathBuf,
}

impl GoldenFile {
    /// Create a new `GoldenFile` referencing `path` (the file need not exist yet).
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Compare `content` against the stored golden file.
    ///
    /// * If the file does not yet exist it is **created** with `content` and
    ///   the assertion passes (first-run bootstrap).
    /// * If the file exists and its content differs from `content` the test
    ///   panics with a clear diff message.
    pub fn assert_matches(&self, content: &str) {
        if !self.path.exists() {
            self.update(content).unwrap_or_else(|e| {
                panic!("GoldenFile: could not write {:?}: {e}", self.path)
            });
            return;
        }
        let stored = self.read().unwrap_or_else(|e| {
            panic!("GoldenFile: could not read {:?}: {e}", self.path)
        });
        assert_eq!(
            stored, content,
            "GoldenFile mismatch for {:?}\n--- expected (stored) ---\n{}\n--- actual ---\n{}",
            self.path, stored, content
        );
    }

    /// Overwrite the golden file with `content`.
    pub fn update(&self, content: &str) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.path, content)
    }

    /// Return `true` if the golden file exists on disk.
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Read the golden file contents as a `String`.
    pub fn read(&self) -> std::io::Result<String> {
        std::fs::read_to_string(&self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("unify-test-golden");
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    #[test]
    fn assert_matches_creates_file_on_first_run() {
        let path = tmp_path("first_run.txt");
        // Make sure it doesn't exist beforehand
        let _ = std::fs::remove_file(&path);

        let gf = GoldenFile::new(&path);
        assert!(!gf.exists(), "file should not exist yet");

        gf.assert_matches("hello golden");
        assert!(gf.exists(), "file should have been created");
        assert_eq!(gf.read().unwrap(), "hello golden");

        // cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn assert_matches_passes_when_content_identical() {
        let path = tmp_path("identical.txt");
        let gf = GoldenFile::new(&path);
        gf.update("same content").unwrap();
        gf.assert_matches("same content"); // should not panic
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    #[should_panic(expected = "GoldenFile mismatch")]
    fn assert_matches_panics_on_mismatch() {
        let path = tmp_path("mismatch.txt");
        let gf = GoldenFile::new(&path);
        gf.update("original").unwrap();
        gf.assert_matches("different");
        let _ = std::fs::remove_file(&path);
    }
}
