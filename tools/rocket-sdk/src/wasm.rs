use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6d];
const DEFAULT_MIN_SIZE: u64 = 1024 * 1024; // 1 MB — stubs are always < 1 MB

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
            path.display(), size, min
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
            path.display(), header[0], header[1], header[2], header[3]
        );
    }

    Ok(WasmInfo { path: path.to_owned(), size_bytes: size })
}

/// Walk `dir` and return the first `.wasm` file that passes magic + size validation.
pub fn find_wasm_in_dir(dir: &Path, min_size_bytes: Option<u64>) -> Result<WasmInfo> {
    for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
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
