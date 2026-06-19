use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6d];
const DEFAULT_MIN_SIZE: u64 = 1024 * 1024; // 1 MB — stubs are always < 1 MB

#[derive(Debug)]
pub struct WasmInfo {
    pub path: PathBuf,
    pub size_bytes: u64,
}

/// Verify `path` has valid WASM magic bytes and meets the minimum size floor.
pub fn validate_wasm_artifact(path: &Path, min_size_bytes: Option<u64>) -> Result<WasmInfo> {
    let min = min_size_bytes.unwrap_or(DEFAULT_MIN_SIZE);

    let meta = std::fs::metadata(path)
        .map_err(|e| anyhow::anyhow!("cannot stat {}: {}", path.display(), e))?;

    let size = meta.len();
    if size < min {
        bail!(
            "{} is only {} bytes — looks like a stub (expected ≥ {} bytes for a real WASM build)",
            path.display(),
            size,
            min
        );
    }

    let mut header = [0u8; 4];
    let mut f = std::fs::File::open(path)
        .map_err(|e| anyhow::anyhow!("cannot open {}: {}", path.display(), e))?;
    use std::io::Read;
    f.read_exact(&mut header)
        .map_err(|e| anyhow::anyhow!("cannot read header of {}: {}", path.display(), e))?;

    if header != WASM_MAGIC {
        bail!(
            "{} has invalid WASM magic: {:02x} {:02x} {:02x} {:02x} (expected 00 61 73 6d)",
            path.display(),
            header[0],
            header[1],
            header[2],
            header[3]
        );
    }

    Ok(WasmInfo {
        path: path.to_owned(),
        size_bytes: size,
    })
}

/// Walk `dir` and return the first `.wasm` file that passes magic + size validation.
pub fn find_wasm_in_dir(dir: &Path, min_size_bytes: Option<u64>) -> Result<WasmInfo> {
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path().extension().and_then(|s| s.to_str()) == Some("wasm") {
            if let Ok(info) = validate_wasm_artifact(entry.path(), min_size_bytes) {
                return Ok(info);
            }
        }
    }
    bail!(
        "no valid WASM artifact found in {} (min size {} bytes)",
        dir.display(),
        min_size_bytes.unwrap_or(DEFAULT_MIN_SIZE)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_file(dir: &TempDir, name: &str, data: &[u8]) -> PathBuf {
        let path = dir.path().join(name);
        std::fs::write(&path, data).unwrap();
        path
    }

    #[test]
    fn valid_wasm_passes() {
        let dir = TempDir::new().unwrap();
        // Real WASM magic + padded to exceed 1 MB min
        let mut data = vec![0x00u8, 0x61, 0x73, 0x6d];
        data.extend(vec![0u8; 1024 * 1024 + 1]);
        let path = write_file(&dir, "test.wasm", &data);
        let info = validate_wasm_artifact(&path, None).unwrap();
        assert!(info.size_bytes > 1024 * 1024);
    }

    #[test]
    fn wrong_magic_rejected() {
        let dir = TempDir::new().unwrap();
        let mut data = vec![0xDEu8, 0xAD, 0xBE, 0xEF];
        data.extend(vec![0u8; 1024 * 1024 + 1]);
        let path = write_file(&dir, "bad.wasm", &data);
        let err = validate_wasm_artifact(&path, None).unwrap_err();
        assert!(err.to_string().contains("invalid WASM magic"));
    }

    #[test]
    fn stub_size_rejected() {
        let dir = TempDir::new().unwrap();
        // Valid magic but only 8 bytes — stub
        let data = vec![0x00u8, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
        let path = write_file(&dir, "stub.wasm", &data);
        let err = validate_wasm_artifact(&path, None).unwrap_err();
        assert!(err.to_string().contains("looks like a stub"));
    }

    #[test]
    fn custom_min_size_respected() {
        let dir = TempDir::new().unwrap();
        let mut data = vec![0x00u8, 0x61, 0x73, 0x6d];
        data.extend(vec![0u8; 500]);
        let path = write_file(&dir, "small.wasm", &data);
        // Allow tiny WASM with explicit min_size = 100 bytes
        let info = validate_wasm_artifact(&path, Some(100)).unwrap();
        assert_eq!(info.path, path);
    }

    #[test]
    fn find_wasm_in_nested_dir() {
        let dir = TempDir::new().unwrap();
        let sub = dir.path().join("HTML5");
        std::fs::create_dir_all(&sub).unwrap();
        let mut data = vec![0x00u8, 0x61, 0x73, 0x6d];
        data.extend(vec![0u8; 1024 * 1024 + 1]);
        std::fs::write(sub.join("Game.wasm"), &data).unwrap();
        let info = find_wasm_in_dir(dir.path(), None).unwrap();
        assert!(info.path.ends_with("Game.wasm"));
    }

    #[test]
    fn find_wasm_empty_dir_errors() {
        let dir = TempDir::new().unwrap();
        let err = find_wasm_in_dir(dir.path(), None).unwrap_err();
        assert!(err.to_string().contains("no valid WASM artifact"));
    }
}
