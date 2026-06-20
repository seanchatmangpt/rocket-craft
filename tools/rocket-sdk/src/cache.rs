//! Content-addressed build cache for the `rocket` CLI.
//!
//! This module provides an incremental build cache that lets `rocket build` /
//! `rocket test` skip work when nothing relevant has changed since the last run.
//!
//! Design goals (per the QoL mission):
//!   * **std-only hashing** — no dependency on `sha2`/`blake3`. We use a fast,
//!     deterministic FNV-1a hash computed over file contents. This is *not* a
//!     cryptographic hash; it is a change-detection hash, which is all a build
//!     cache needs.
//!   * **pure, testable functions** — the hashing, key derivation and freshness
//!     logic are all pure functions that can be unit-tested without running any
//!     real build.
//!   * **binary-asset safety** — large files and known UE4 binary assets
//!     (`.uasset`, `.umap`) are never read into memory. They are fingerprinted by
//!     `(mtime, size)` metadata only, honoring the repo rule that `versions/**`
//!     binaries must never be diffed/merged/hashed wholesale.
//!
//! The cache is persisted as `.rocket-cache.json` via `serde`.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::hash::Hasher;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Default cap on file size we are willing to read into memory for content
/// hashing. Files larger than this are fingerprinted by metadata only.
pub const DEFAULT_MAX_HASH_BYTES: u64 = 8 * 1024 * 1024; // 8 MiB

/// File extensions that are treated as opaque binary assets and are *never*
/// content-hashed (only metadata-fingerprinted), per the monorepo's
/// binary-asset rule for `versions/**`.
pub const BINARY_ASSET_EXTENSIONS: &[&str] = &[
    "uasset", "umap", "fbx", "pak", "ubulk", "uexp", "png", "jpg", "jpeg", "tga",
    "wav", "ogg", "mp4", "bin", "so", "dll", "dylib", "a", "lib", "exe", "pdb",
];

/// Cache file name written to the working directory.
pub const CACHE_FILE_NAME: &str = ".rocket-cache.json";

// ---------------------------------------------------------------------------
// Pure hashing
// ---------------------------------------------------------------------------

/// FNV-1a 64-bit hasher implementing [`std::hash::Hasher`].
///
/// We implement this explicitly (rather than using `DefaultHasher`) so the hash
/// is *stable across Rust versions and platforms* — `DefaultHasher`'s algorithm
/// is explicitly unspecified and may change, which would silently invalidate
/// every cache entry on a toolchain bump.
#[derive(Debug, Clone)]
pub struct Fnv1aHasher {
    state: u64,
}

impl Fnv1aHasher {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    pub fn new() -> Self {
        Self { state: Self::OFFSET_BASIS }
    }
}

impl Default for Fnv1aHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher for Fnv1aHasher {
    fn finish(&self) -> u64 {
        self.state
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut hash = self.state;
        for &b in bytes {
            hash ^= u64::from(b);
            hash = hash.wrapping_mul(Self::PRIME);
        }
        self.state = hash;
    }
}

/// Pure function: hash an arbitrary byte slice with FNV-1a, returned as a
/// lowercase hex string. Deterministic across runs/platforms.
pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut h = Fnv1aHasher::new();
    h.write(bytes);
    format!("{:016x}", h.finish())
}

/// Pure function: combine an ordered list of component strings into a single
/// stable hash. Used to derive a [`CacheKey`] from fingerprints + a command.
/// A length-prefixed separator avoids ambiguity (`"ab"+"c"` != `"a"+"bc"`).
pub fn hash_components<I, S>(components: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut h = Fnv1aHasher::new();
    for c in components {
        let s = c.as_ref();
        h.write(&(s.len() as u64).to_le_bytes());
        h.write(s.as_bytes());
        h.write(b"\x1f"); // unit separator
    }
    format!("{:016x}", h.finish())
}

/// Returns true if `path` should be treated as an opaque binary asset
/// (metadata-only fingerprint, never content-hashed).
pub fn is_binary_asset(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => {
            let ext = ext.to_ascii_lowercase();
            BINARY_ASSET_EXTENSIONS.contains(&ext.as_str())
        }
        None => false,
    }
}

// ---------------------------------------------------------------------------
// FileFingerprint / Fingerprinter
// ---------------------------------------------------------------------------

/// A fingerprint of a single file: enough to detect changes cheaply.
///
/// * `mtime` / `size` are read from metadata (cheap).
/// * `hash` is a content hash for text/code files, or `None` for binary/large
///   files that we deliberately do not read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileFingerprint {
    pub path: PathBuf,
    /// Modification time as whole seconds since the UNIX epoch.
    pub mtime: u64,
    pub size: u64,
    /// Content hash, or `None` when content hashing was skipped (binary/oversize).
    pub hash: Option<String>,
}

impl FileFingerprint {
    /// Stable string form used when deriving cache keys / diffing.
    pub fn signature(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.path.display(),
            self.mtime,
            self.size,
            self.hash.as_deref().unwrap_or("-")
        )
    }
}

/// Computes [`FileFingerprint`]s for files, deciding whether to content-hash
/// based on extension and size caps.
#[derive(Debug, Clone)]
pub struct Fingerprinter {
    pub max_hash_bytes: u64,
}

impl Default for Fingerprinter {
    fn default() -> Self {
        Self { max_hash_bytes: DEFAULT_MAX_HASH_BYTES }
    }
}

impl Fingerprinter {
    pub fn new(max_hash_bytes: u64) -> Self {
        Self { max_hash_bytes }
    }

    /// Whether this file's contents should be hashed (vs. metadata-only).
    pub fn should_content_hash(&self, path: &Path, size: u64) -> bool {
        !is_binary_asset(path) && size <= self.max_hash_bytes
    }

    /// Fingerprint a single file by reading its metadata and (conditionally)
    /// its contents. Returns an IO error if the file cannot be stat'd/read.
    pub fn fingerprint(&self, path: &Path) -> io::Result<FileFingerprint> {
        let meta = std::fs::metadata(path)?;
        let size = meta.len();
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let hash = if self.should_content_hash(path, size) {
            Some(self.hash_file(path)?)
        } else {
            None
        };

        Ok(FileFingerprint { path: path.to_path_buf(), mtime, size, hash })
    }

    /// Stream-hash a file's contents without loading it all at once.
    fn hash_file(&self, path: &Path) -> io::Result<String> {
        let f = std::fs::File::open(path)?;
        let mut reader = io::BufReader::new(f);
        let mut hasher = Fnv1aHasher::new();
        let mut buf = [0u8; 64 * 1024];
        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hasher.write(&buf[..n]);
        }
        Ok(format!("{:016x}", hasher.finish()))
    }
}

/// Now as whole seconds since the UNIX epoch (helper for timestamps).
pub fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// CacheKey
// ---------------------------------------------------------------------------

/// A content-addressed key for a build/test invocation: derived from the set of
/// input fingerprints plus the command string. Two invocations with identical
/// inputs and command produce the same key.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheKey(pub String);

impl CacheKey {
    /// Derive a key from a command string and a set of input fingerprints.
    /// Fingerprints are sorted by path so ordering of the input slice does not
    /// affect the key (set semantics).
    pub fn derive(command: &str, inputs: &[FileFingerprint]) -> Self {
        let mut sigs: Vec<String> = inputs.iter().map(|f| f.signature()).collect();
        sigs.sort();
        let mut components: Vec<String> = Vec::with_capacity(sigs.len() + 2);
        components.push(format!("cmd:{command}"));
        components.extend(sigs);
        CacheKey(hash_components(&components))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ---------------------------------------------------------------------------
// BuildCache
// ---------------------------------------------------------------------------

/// A recorded outcome for a cache key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Whether the recorded run succeeded.
    pub success: bool,
    /// When the entry was recorded (unix seconds).
    pub recorded_at: u64,
    /// The command that produced it (for diagnostics / `--explain`).
    pub command: String,
}

/// Persistent content-addressed build cache mapping [`CacheKey`] -> outcome.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildCache {
    /// Schema version, so future format changes can invalidate cleanly.
    #[serde(default = "default_version")]
    pub version: u32,
    /// Map of key string -> entry. `BTreeMap` keeps the JSON output stable.
    #[serde(default)]
    pub entries: BTreeMap<String, CacheEntry>,
}

fn default_version() -> u32 {
    1
}

impl BuildCache {
    pub fn new() -> Self {
        Self { version: default_version(), entries: BTreeMap::new() }
    }

    /// Load a cache from a JSON file. A missing file yields an empty cache; a
    /// corrupt file is treated as empty (cache is advisory, never fatal).
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_else(|_| Self::new()),
            Err(_) => Self::new(),
        }
    }

    /// Convenience: load `.rocket-cache.json` from `dir`.
    pub fn load_from_dir(dir: &Path) -> Self {
        Self::load(&dir.join(CACHE_FILE_NAME))
    }

    /// Persist the cache to a JSON file (pretty-printed for diff-friendliness).
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        std::fs::write(path, json)
    }

    /// Convenience: save to `.rocket-cache.json` in `dir`.
    pub fn save_to_dir(&self, dir: &Path) -> io::Result<()> {
        self.save(&dir.join(CACHE_FILE_NAME))
    }

    /// Is there a recorded *successful* entry for this key? A fresh key means
    /// the inputs+command are unchanged since a previous successful run, so the
    /// work can be skipped.
    pub fn is_fresh(&self, key: &CacheKey) -> bool {
        self.entries.get(key.as_str()).map(|e| e.success).unwrap_or(false)
    }

    /// Look up the recorded entry for a key, if any.
    pub fn get(&self, key: &CacheKey) -> Option<&CacheEntry> {
        self.entries.get(key.as_str())
    }

    /// Record an outcome for a key, overwriting any prior entry.
    pub fn record(&mut self, key: &CacheKey, command: &str, success: bool) {
        self.entries.insert(
            key.0.clone(),
            CacheEntry { success, recorded_at: now_unix_secs(), command: command.to_string() },
        );
    }

    /// Remove a single key from the cache.
    pub fn invalidate(&mut self, key: &CacheKey) {
        self.entries.remove(key.as_str());
    }

    /// Drop every entry.
    pub fn invalidate_all(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn fp(path: &str, mtime: u64, size: u64, hash: Option<&str>) -> FileFingerprint {
        FileFingerprint {
            path: PathBuf::from(path),
            mtime,
            size,
            hash: hash.map(|s| s.to_string()),
        }
    }

    #[test]
    fn hash_is_deterministic() {
        let a = hash_bytes(b"hello world");
        let b = hash_bytes(b"hello world");
        assert_eq!(a, b);
        assert_eq!(a.len(), 16); // 64-bit hex
    }

    #[test]
    fn hash_differs_on_different_input() {
        assert_ne!(hash_bytes(b"hello"), hash_bytes(b"hellp"));
        assert_ne!(hash_bytes(b""), hash_bytes(b"\0"));
    }

    #[test]
    fn fnv1a_matches_known_vector() {
        // FNV-1a 64-bit of empty input is the offset basis.
        assert_eq!(hash_bytes(b""), format!("{:016x}", Fnv1aHasher::OFFSET_BASIS));
    }

    #[test]
    fn hash_components_is_order_sensitive_and_unambiguous() {
        let ab = hash_components(["a", "b"]);
        let ba = hash_components(["b", "a"]);
        assert_ne!(ab, ba);
        // length-prefixing prevents "ab"+"c" colliding with "a"+"bc"
        assert_ne!(hash_components(["ab", "c"]), hash_components(["a", "bc"]));
    }

    #[test]
    fn binary_asset_detection() {
        assert!(is_binary_asset(Path::new("Foo.uasset")));
        assert!(is_binary_asset(Path::new("Level.umap")));
        assert!(is_binary_asset(Path::new("a/b/Mesh.FBX"))); // case-insensitive
        assert!(!is_binary_asset(Path::new("main.rs")));
        assert!(!is_binary_asset(Path::new("README")));
    }

    #[test]
    fn fingerprinter_skips_binary_and_oversize() {
        let fpr = Fingerprinter::new(100);
        assert!(!fpr.should_content_hash(Path::new("Level.uasset"), 10));
        assert!(!fpr.should_content_hash(Path::new("big.rs"), 101));
        assert!(fpr.should_content_hash(Path::new("small.rs"), 100));
    }

    #[test]
    fn fingerprint_real_file_has_hash_for_text() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("a.rs");
        fs::write(&p, b"fn main() {}").unwrap();

        let fpr = Fingerprinter::default();
        let f = fpr.fingerprint(&p).unwrap();
        assert_eq!(f.size, 12);
        assert!(f.hash.is_some());
        assert_eq!(f.hash.as_deref().unwrap(), hash_bytes(b"fn main() {}"));
    }

    #[test]
    fn fingerprint_binary_file_has_no_hash() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("Level.uasset");
        fs::write(&p, b"\x00\x01\x02binary").unwrap();

        let fpr = Fingerprinter::default();
        let f = fpr.fingerprint(&p).unwrap();
        assert!(f.hash.is_none(), "binary assets must not be content-hashed");
        assert!(f.size > 0);
    }

    #[test]
    fn cache_key_is_set_order_independent() {
        let cmd = "cargo test -p rocket-sdk";
        let a = fp("a.rs", 1, 10, Some("aa"));
        let b = fp("b.rs", 2, 20, Some("bb"));
        let k1 = CacheKey::derive(cmd, &[a.clone(), b.clone()]);
        let k2 = CacheKey::derive(cmd, &[b, a]);
        assert_eq!(k1, k2, "input ordering must not affect key");
    }

    #[test]
    fn cache_key_changes_with_inputs_and_command() {
        let a = fp("a.rs", 1, 10, Some("aa"));
        let a2 = fp("a.rs", 1, 10, Some("ab")); // content changed
        let base = CacheKey::derive("cmd", &[a.clone()]);
        assert_ne!(base, CacheKey::derive("cmd", &[a2]));
        assert_ne!(base, CacheKey::derive("other-cmd", &[a]));
    }

    #[test]
    fn cache_freshness_record_and_invalidate() {
        let mut cache = BuildCache::new();
        let key = CacheKey::derive("cmd", &[fp("a.rs", 1, 1, Some("h"))]);

        assert!(!cache.is_fresh(&key));
        cache.record(&key, "cmd", true);
        assert!(cache.is_fresh(&key));

        cache.invalidate(&key);
        assert!(!cache.is_fresh(&key));
    }

    #[test]
    fn failed_runs_are_not_fresh() {
        let mut cache = BuildCache::new();
        let key = CacheKey::derive("cmd", &[fp("a.rs", 1, 1, Some("h"))]);
        cache.record(&key, "cmd", false);
        assert!(!cache.is_fresh(&key), "a failed run must not count as fresh");
        assert!(cache.get(&key).is_some());
    }

    #[test]
    fn cache_round_trips_through_json() {
        let dir = tempfile::tempdir().unwrap();
        let mut cache = BuildCache::new();
        let key = CacheKey::derive("cmd", &[fp("a.rs", 1, 1, Some("h"))]);
        cache.record(&key, "cmd", true);
        cache.save_to_dir(dir.path()).unwrap();

        let loaded = BuildCache::load_from_dir(dir.path());
        assert!(loaded.is_fresh(&key));
        assert_eq!(loaded.len(), 1);
    }

    #[test]
    fn corrupt_or_missing_cache_loads_empty() {
        let dir = tempfile::tempdir().unwrap();
        // missing
        assert!(BuildCache::load_from_dir(dir.path()).is_empty());
        // corrupt
        fs::write(dir.path().join(CACHE_FILE_NAME), b"{not json").unwrap();
        assert!(BuildCache::load_from_dir(dir.path()).is_empty());
    }

    #[test]
    fn invalidate_all_clears() {
        let mut cache = BuildCache::new();
        cache.record(&CacheKey("x".into()), "c", true);
        cache.record(&CacheKey("y".into()), "c", true);
        assert_eq!(cache.len(), 2);
        cache.invalidate_all();
        assert!(cache.is_empty());
    }
}
