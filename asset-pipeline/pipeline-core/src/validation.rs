use std::collections::HashMap;
use std::io::Read as IoRead;
use std::path::PathBuf;

use tracing::warn;

use crate::config::PipelineSection;
use crate::error::PipelineError;
use crate::types::{DiscoveredAsset, Format, ValidatedAsset};

// ── ValidationConfig ──────────────────────────────────────────────────────────

/// Configuration parameters used by the `Validator`.
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Maximum allowed file size in bytes.  Files exceeding this are skipped.
    pub max_size_bytes: u64,
}

impl ValidationConfig {
    /// Build a `ValidationConfig` from the loaded pipeline configuration.
    pub fn from_pipeline_config(cfg: &PipelineSection) -> Self {
        Self {
            max_size_bytes: cfg.max_file_mb * 1024 * 1024,
        }
    }

    /// Convenience constructor: specify the limit in megabytes.
    pub fn with_max_mb(mb: u64) -> Self {
        Self {
            max_size_bytes: mb * 1024 * 1024,
        }
    }
}

// ── Validator ─────────────────────────────────────────────────────────────────

/// Stateful asset validator.
///
/// Holds a set of BLAKE3 content hashes already seen so that it can detect and
/// reject duplicate assets within a single pipeline run.  Pass a new `Validator`
/// for each run if you want a fresh dedup table.
pub struct Validator {
    config: ValidationConfig,
    /// Maps BLAKE3 content hash → path of the first asset seen with that hash.
    seen_hashes: HashMap<[u8; 32], PathBuf>,
}

impl Validator {
    /// Create a new `Validator` with the given configuration.
    pub fn new(config: ValidationConfig) -> Self {
        Self {
            config,
            seen_hashes: HashMap::new(),
        }
    }

    /// Validate a single discovered asset.
    ///
    /// Checks (in order):
    /// 1. Size limit — rejects files larger than `config.max_size_bytes`.
    /// 2. Duplicate hash — rejects files whose content matches a previously seen asset.
    /// 3. Magic bytes — does a quick sanity-check on the first bytes of the file to
    ///    confirm it is what its extension claims.
    ///
    /// # Returns
    /// - `Ok(ValidatedAsset)` — all checks passed; the hash is recorded.
    /// - `Err((PipelineError, DiscoveredAsset))` — a check failed; the caller retains
    ///   ownership of the original `DiscoveredAsset` for error-reporting purposes.
    pub fn validate(
        &mut self,
        asset: DiscoveredAsset,
    ) -> Result<ValidatedAsset, (PipelineError, DiscoveredAsset)> {
        // ── 1. Size check ──────────────────────────────────────────────────────
        if asset.file_size_bytes > self.config.max_size_bytes {
            let size_mb = asset.file_size_bytes as f64 / (1024.0 * 1024.0);
            let limit_mb = self.config.max_size_bytes / (1024 * 1024);
            return Err((
                PipelineError::FileTooLarge {
                    path: asset.path.clone(),
                    size_mb,
                    limit_mb,
                },
                asset,
            ));
        }

        // ── 2. Duplicate hash check ────────────────────────────────────────────
        if let Some(existing) = self.seen_hashes.get(&asset.hash) {
            return Err((
                PipelineError::Duplicate {
                    path: asset.path.clone(),
                    existing: existing.clone(),
                },
                asset,
            ));
        }

        // ── 3. Magic byte validation ───────────────────────────────────────────
        if let Err(err) = check_magic_bytes(&asset) {
            return Err((err, asset));
        }

        // ── 4. Record & advance state ──────────────────────────────────────────
        self.seen_hashes.insert(asset.hash, asset.path.clone());
        Ok(asset.into_validated())
    }

    /// Number of unique assets that have passed validation in this session.
    pub fn validated_count(&self) -> usize {
        self.seen_hashes.len()
    }

    /// Number of content hashes tracked (equals `validated_count` unless you
    /// manually manipulate the map or reset the validator).
    pub fn seen_hashes_count(&self) -> usize {
        self.seen_hashes.len()
    }
}

// ── Magic-byte validation ─────────────────────────────────────────────────────

/// Read the first `n` bytes of the file at `path`.
///
/// Returns `Ok(bytes_read)` with the filled prefix; `Ok(buf)` may be shorter
/// than `n` if the file is smaller.  Returns an `Err` only on genuine I/O
/// failures.
fn read_prefix(path: &std::path::Path, n: usize) -> std::io::Result<Vec<u8>> {
    let mut f = std::fs::File::open(path)?;
    let mut buf = vec![0u8; n];
    let bytes_read = f.read(&mut buf)?;
    buf.truncate(bytes_read);
    Ok(buf)
}

/// Return an `UnsupportedFormat` error for the asset with a descriptive note
/// appended to the extension field.
fn magic_error(asset: &DiscoveredAsset, note: &str) -> PipelineError {
    PipelineError::UnsupportedFormat {
        path: asset.path.clone(),
        ext: format!("{}: {}", asset.source_format.extension(), note),
    }
}

/// Perform format-specific magic-byte checks.
///
/// Returns `Ok(())` when the file looks valid (or when we can't be sure and
/// choose to let it pass with a warning).  Returns an error only when we are
/// confident the file is not what it claims to be.
fn check_magic_bytes(asset: &DiscoveredAsset) -> Result<(), PipelineError> {
    let path = &asset.path;

    match asset.source_format {
        // ── GLB ───────────────────────────────────────────────────────────────
        // The GLB container spec mandates the 4-byte magic `glTF` (0x46_54_6C_67).
        Format::Glb => {
            let buf = read_prefix(path, 4).map_err(PipelineError::StagingFailed)?;
            if buf.len() < 4 {
                return Err(magic_error(asset, "file too short to contain GLB header"));
            }
            if &buf[..4] != b"glTF" {
                return Err(magic_error(asset, "invalid magic bytes (expected 'glTF')"));
            }
        }

        // ── FBX Binary ────────────────────────────────────────────────────────
        // Binary FBX files begin with the 23-byte signature:
        // "Kaydara FBX Binary  \x00\x1a\x00"
        // We only check the human-readable prefix.
        Format::Fbx => {
            let buf = read_prefix(path, 23).map_err(|e| {
                warn!(path = %path.display(), error = %e, "could not read FBX header; allowing file to pass");
                // Treat I/O errors on FBX as best-effort — return a sentinel we
                // check below.
                e
            });
            match buf {
                Err(_) => {
                    // Best-effort: let the file pass if we cannot read it.
                    warn!(path = %path.display(), "FBX magic-byte check skipped due to I/O error");
                }
                Ok(bytes) => {
                    if bytes.len() >= 20 {
                        // Binary FBX: starts with "Kaydara FBX Binary  "
                        if &bytes[..20] != b"Kaydara FBX Binary  " {
                            // Could be ASCII FBX — those begin with semicolons / text.
                            // We permit ASCII FBX by checking for printable first byte.
                            if bytes[0].is_ascii() && !bytes[0].is_ascii_control() {
                                // Looks like ASCII — warn and allow.
                                warn!(
                                    path = %path.display(),
                                    "FBX file does not have binary magic; treating as ASCII FBX and allowing"
                                );
                            } else {
                                return Err(magic_error(
                                    asset,
                                    "invalid magic bytes (expected Kaydara FBX Binary or ASCII FBX)",
                                ));
                            }
                        }
                    } else {
                        // File too short for a complete binary header; warn and allow.
                        warn!(
                            path = %path.display(),
                            "FBX file too short to verify magic bytes; allowing"
                        );
                    }
                }
            }
        }

        // ── STL ───────────────────────────────────────────────────────────────
        // Binary STL: 80-byte header then a 4-byte little-endian triangle count.
        // ASCII STL: begins with "solid".
        // We check that either the file starts with "solid" (ASCII) or does NOT
        // start with "solid" (binary).  Both are valid; only a completely empty
        // or unreadable file is rejected.
        Format::Stl => {
            let buf = read_prefix(path, 5).map_err(|e| {
                warn!(path = %path.display(), error = %e, "could not read STL header; allowing");
                e
            });
            match buf {
                Err(_) => {
                    warn!(path = %path.display(), "STL magic-byte check skipped due to I/O error");
                }
                Ok(bytes) => {
                    if bytes.is_empty() {
                        return Err(magic_error(asset, "file is empty"));
                    }
                    // Both ASCII ("solid...") and binary STL are accepted; just verify
                    // the file has some content.
                }
            }
        }

        // ── OBJ ───────────────────────────────────────────────────────────────
        // OBJ is plain ASCII.  We verify the first non-whitespace byte is
        // printable ASCII (0x20–0x7E).
        Format::Obj => {
            let buf = read_prefix(path, 64).map_err(PipelineError::StagingFailed)?;
            if buf.is_empty() {
                return Err(magic_error(asset, "file is empty"));
            }
            let first_significant = buf.iter().copied().find(|b| !b.is_ascii_whitespace());
            match first_significant {
                None => {
                    return Err(magic_error(asset, "file contains only whitespace"));
                }
                Some(b) if !b.is_ascii() || b.is_ascii_control() => {
                    return Err(magic_error(
                        asset,
                        "first non-whitespace byte is not printable ASCII (binary data?)",
                    ));
                }
                Some(_) => {} // valid
            }
        }

        // ── glTF JSON ─────────────────────────────────────────────────────────
        // glTF is a JSON file; its first non-whitespace character must be `{`.
        Format::Gltf => {
            let buf = read_prefix(path, 64).map_err(PipelineError::StagingFailed)?;
            if buf.is_empty() {
                return Err(magic_error(asset, "file is empty"));
            }
            let first_significant = buf.iter().copied().find(|b| !b.is_ascii_whitespace());
            match first_significant {
                None => {
                    return Err(magic_error(asset, "file contains only whitespace"));
                }
                Some(b'{') => {} // valid JSON object opening
                Some(other) => {
                    return Err(magic_error(
                        asset,
                        &format!(
                            "expected '{{' as first non-whitespace character, found '{}' (0x{:02X})",
                            other as char, other
                        ),
                    ));
                }
            }
        }

        // ── COLLADA (DAE) ─────────────────────────────────────────────────────
        // DAE is XML; must start with `<?xml` or `<COLLADA`.
        Format::Dae => {
            let buf = read_prefix(path, 64).map_err(PipelineError::StagingFailed)?;
            if buf.is_empty() {
                return Err(magic_error(asset, "file is empty"));
            }
            // Skip any BOM or whitespace before the XML declaration.
            let trimmed: &[u8] = buf
                .iter()
                .position(|b| !b.is_ascii_whitespace())
                .map(|i| &buf[i..])
                // Handle UTF-8 BOM (0xEF 0xBB 0xBF)
                .unwrap_or(&buf);

            let trimmed = if trimmed.starts_with(b"\xEF\xBB\xBF") {
                &trimmed[3..]
            } else {
                trimmed
            };

            let looks_like_xml = trimmed.starts_with(b"<?xml")
                || trimmed.starts_with(b"<?XML")
                || trimmed.starts_with(b"<COLLADA")
                || trimmed.starts_with(b"<collada");

            if !looks_like_xml {
                return Err(magic_error(
                    asset,
                    "does not appear to be XML (expected '<?xml' or '<COLLADA')",
                ));
            }
        }
    }

    Ok(())
}

// ── Batch helper ─────────────────────────────────────────────────────────────

/// Validate a collection of discovered assets in order using a fresh `Validator`.
///
/// * Skippable errors (`FileTooLarge`, `Duplicate`, `UnsupportedFormat`) are
///   collected and returned alongside the valid assets so callers can report them
///   without aborting the run.
/// * Non-skippable errors (e.g. `StagingFailed` wrapping an unexpected I/O
///   error) are also collected rather than panicking — the asset is dropped and
///   the error is surfaced in the skipped list so the caller can decide whether
///   to abort.
///
/// Returns `(validated, skipped_with_reasons)`.
pub fn validate_batch(
    assets: Vec<DiscoveredAsset>,
    config: ValidationConfig,
) -> (Vec<ValidatedAsset>, Vec<(PathBuf, PipelineError)>) {
    let mut validator = Validator::new(config);
    let mut validated = Vec::with_capacity(assets.len());
    let mut skipped: Vec<(PathBuf, PipelineError)> = Vec::new();

    for asset in assets {
        match validator.validate(asset) {
            Ok(v) => validated.push(v),
            Err((err, original)) => {
                skipped.push((original.path, err));
            }
        }
    }

    (validated, skipped)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Helper: build a `DiscoveredAsset` pointing at `path` with the given hash,
    /// format, and size.  The path does not need to exist unless a test exercises
    /// magic-byte reading.
    fn make_asset(
        path: impl Into<PathBuf>,
        hash: [u8; 32],
        format: Format,
        size: u64,
    ) -> DiscoveredAsset {
        DiscoveredAsset::new(path.into(), hash, format, size)
    }

    // ── Size checks ───────────────────────────────────────────────────────────

    #[test]
    fn size_check_rejects_oversized_file() {
        // Write a 5-byte tempfile but declare its logical size as 101 MB.
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(b"hello").unwrap();

        let limit_mb: u64 = 100;
        let config = ValidationConfig::with_max_mb(limit_mb);
        let mut v = Validator::new(config);

        // Exceed the limit by 1 byte over 100 MB.
        let oversized_bytes = (limit_mb * 1024 * 1024) + 1;
        let asset = make_asset(tmp.path(), [0u8; 32], Format::Obj, oversized_bytes);

        let result = v.validate(asset);
        assert!(result.is_err(), "expected Err for oversized file");
        let (err, _) = result.unwrap_err();
        assert!(
            matches!(err, PipelineError::FileTooLarge { .. }),
            "expected FileTooLarge, got: {err:?}"
        );
    }

    #[test]
    fn size_check_passes_at_limit() {
        // A file whose declared size is exactly max_size_bytes should pass the
        // size gate (the check is strictly greater-than).
        let mut tmp = NamedTempFile::new().unwrap();
        // Write an OBJ-like header so the magic-byte check also passes.
        tmp.write_all(b"v 0 0 0\n").unwrap();

        let limit_mb: u64 = 10;
        let max_bytes = limit_mb * 1024 * 1024;
        let config = ValidationConfig::with_max_mb(limit_mb);
        let mut v = Validator::new(config);

        let asset = make_asset(tmp.path(), [1u8; 32], Format::Obj, max_bytes);
        let result = v.validate(asset);
        assert!(
            result.is_ok(),
            "expected Ok for file at exactly the size limit, got: {result:?}"
        );
    }

    // ── Duplicate detection ───────────────────────────────────────────────────

    #[test]
    fn duplicate_hash_is_rejected() {
        let mut tmp1 = NamedTempFile::new().unwrap();
        tmp1.write_all(b"v 0 0 0\n").unwrap();

        let mut tmp2 = NamedTempFile::new().unwrap();
        tmp2.write_all(b"v 0 0 0\n").unwrap();

        let shared_hash = [42u8; 32];
        let config = ValidationConfig::with_max_mb(500);
        let mut v = Validator::new(config);

        let asset1 = make_asset(tmp1.path(), shared_hash, Format::Obj, 8);
        let asset2 = make_asset(tmp2.path(), shared_hash, Format::Obj, 8);

        // First asset passes.
        assert!(v.validate(asset1).is_ok());

        // Second asset with the same hash must be rejected.
        let result = v.validate(asset2);
        assert!(result.is_err(), "expected Err for duplicate hash");
        let (err, _) = result.unwrap_err();
        assert!(
            matches!(err, PipelineError::Duplicate { .. }),
            "expected Duplicate, got: {err:?}"
        );
    }

    #[test]
    fn unique_hashes_both_pass() {
        let mut tmp1 = NamedTempFile::new().unwrap();
        tmp1.write_all(b"v 0 0 0\n").unwrap();

        let mut tmp2 = NamedTempFile::new().unwrap();
        tmp2.write_all(b"v 1 1 1\n").unwrap();

        let config = ValidationConfig::with_max_mb(500);
        let mut v = Validator::new(config);

        let asset1 = make_asset(tmp1.path(), [1u8; 32], Format::Obj, 8);
        let asset2 = make_asset(tmp2.path(), [2u8; 32], Format::Obj, 8);

        assert!(v.validate(asset1).is_ok(), "first asset should pass");
        assert!(v.validate(asset2).is_ok(), "second asset with different hash should pass");
        assert_eq!(v.validated_count(), 2);
    }

    // ── Batch helper ──────────────────────────────────────────────────────────

    #[test]
    fn validate_batch_collects_skippable_errors() {
        let mut good_tmp = NamedTempFile::new().unwrap();
        good_tmp.write_all(b"v 0 0 0\n").unwrap();

        let mut dup_tmp = NamedTempFile::new().unwrap();
        dup_tmp.write_all(b"v 0 0 0\n").unwrap();

        let shared_hash = [99u8; 32];
        let good_asset = make_asset(good_tmp.path(), shared_hash, Format::Obj, 8);
        let dup_asset = make_asset(dup_tmp.path(), shared_hash, Format::Obj, 8);

        let (validated, skipped) =
            validate_batch(vec![good_asset, dup_asset], ValidationConfig::with_max_mb(500));

        assert_eq!(validated.len(), 1, "one asset should be valid");
        assert_eq!(skipped.len(), 1, "one asset should be skipped as duplicate");
        assert!(
            matches!(skipped[0].1, PipelineError::Duplicate { .. }),
            "skipped reason should be Duplicate"
        );
    }

    // ── Magic byte: GLB ───────────────────────────────────────────────────────

    #[test]
    fn glb_magic_byte_valid() {
        // Write a minimal GLB container (just the 4-byte magic is enough for
        // our check).
        let mut tmp = NamedTempFile::new().unwrap();
        // GLB magic: b"glTF" followed by version + length placeholders.
        tmp.write_all(b"glTF\x02\x00\x00\x00\x14\x00\x00\x00").unwrap();

        let config = ValidationConfig::with_max_mb(500);
        let mut v = Validator::new(config);

        let size = tmp.path().metadata().unwrap().len();
        let asset = make_asset(tmp.path(), [10u8; 32], Format::Glb, size);

        assert!(
            v.validate(asset).is_ok(),
            "GLB with valid magic should pass"
        );
    }

    #[test]
    fn glb_magic_byte_invalid() {
        let mut tmp = NamedTempFile::new().unwrap();
        // Write garbage bytes that are NOT the glTF magic.
        tmp.write_all(b"\x00\x01\x02\x03garbage here").unwrap();

        let config = ValidationConfig::with_max_mb(500);
        let mut v = Validator::new(config);

        let size = tmp.path().metadata().unwrap().len();
        let asset = make_asset(tmp.path(), [11u8; 32], Format::Glb, size);

        let result = v.validate(asset);
        assert!(result.is_err(), "GLB with invalid magic should fail");
        let (err, _) = result.unwrap_err();
        assert!(
            matches!(err, PipelineError::UnsupportedFormat { .. }),
            "expected UnsupportedFormat, got: {err:?}"
        );
    }

    // ── Magic byte: OBJ ───────────────────────────────────────────────────────

    #[test]
    fn obj_file_passes_magic_check() {
        let mut tmp = NamedTempFile::new().unwrap();
        // Write a minimal OBJ vertex line.
        tmp.write_all(b"v 0 0 0\n").unwrap();

        let config = ValidationConfig::with_max_mb(500);
        let mut v = Validator::new(config);

        let size = tmp.path().metadata().unwrap().len();
        let asset = make_asset(tmp.path(), [12u8; 32], Format::Obj, size);

        assert!(
            v.validate(asset).is_ok(),
            "OBJ with text content should pass magic check"
        );
    }

    // ── Empty file ────────────────────────────────────────────────────────────

    #[test]
    fn empty_file_fails_magic_check() {
        // An empty .obj file should fail the magic-byte check.
        let tmp = NamedTempFile::new().unwrap();
        // Do not write anything — file is empty.

        let config = ValidationConfig::with_max_mb(500);
        let mut v = Validator::new(config);

        let asset = make_asset(tmp.path(), [13u8; 32], Format::Obj, 0);

        let result = v.validate(asset);
        assert!(result.is_err(), "empty OBJ file should fail magic check");
        let (err, _) = result.unwrap_err();
        assert!(
            matches!(err, PipelineError::UnsupportedFormat { .. }),
            "expected UnsupportedFormat for empty file, got: {err:?}"
        );
    }

    // ── Count helpers ─────────────────────────────────────────────────────────

    #[test]
    fn validated_count_tracks_successes() {
        let config = ValidationConfig::with_max_mb(500);
        let mut v = Validator::new(config);
        assert_eq!(v.validated_count(), 0);
        assert_eq!(v.seen_hashes_count(), 0);

        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(b"v 1 2 3\n").unwrap();
        let size = tmp.path().metadata().unwrap().len();
        let asset = make_asset(tmp.path(), [20u8; 32], Format::Obj, size);
        v.validate(asset).unwrap();

        assert_eq!(v.validated_count(), 1);
        assert_eq!(v.seen_hashes_count(), 1);
    }
}
