//! SOC2 CC9.1 — Software Bill of Materials (SBOM) generation.
//!
//! Parses `Cargo.lock` and `package-lock.json` files into a unified package
//! inventory and exports SPDX 2.3 JSON, CSV, and Markdown.  A `diff` function
//! detects supply-chain changes between two snapshots.
//!
//! # Usage
//! ```rust,no_run
//! use rocket_sdk::sbom::Sbom;
//! let sbom = Sbom::from_cargo_lock(std::path::Path::new("Cargo.lock")).unwrap();
//! println!("{}", sbom.to_markdown_table());
//! ```

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// SbomPackage
// ---------------------------------------------------------------------------

/// A single package / dependency in the bill of materials.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SbomPackage {
    /// Package name (crate name or npm package name).
    pub name: String,
    /// Semantic version string.
    pub version: String,
    /// SPDX license expression, if available.
    pub license: Option<String>,
    /// Source registry or URL.
    pub source: String,
    /// Content hash (checksum from Cargo.lock, or integrity hash from package-lock.json).
    pub checksum: Option<String>,
    /// True if this is a direct dependency (first-level).
    pub is_direct: bool,
}

// ---------------------------------------------------------------------------
// Sbom
// ---------------------------------------------------------------------------

/// A complete Software Bill of Materials document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sbom {
    /// Human-readable name for this SBOM document.
    pub document_name: String,
    /// SPDX version string, always "SPDX-2.3".
    pub spdx_version: &'static str,
    /// Timestamp when the SBOM was generated.
    pub created: DateTime<Utc>,
    /// Tool/organization that created this SBOM.
    pub creator: String,
    /// All packages found.
    pub packages: Vec<SbomPackage>,
}

impl Sbom {
    // -----------------------------------------------------------------------
    // Cargo.lock parser (hand-rolled — no `toml` dep)
    // -----------------------------------------------------------------------

    /// Parse a `Cargo.lock` file (TOML v3 format) into an `Sbom`.
    ///
    /// Uses a line-by-line state machine rather than a TOML library so that no
    /// additional dependency is required.
    pub fn from_cargo_lock(path: &Path) -> Result<Sbom> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("reading Cargo.lock at {}", path.display()))?;

        let packages = parse_cargo_lock(&content);
        let document_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Cargo.lock".to_string());

        Ok(Sbom {
            document_name,
            spdx_version: "SPDX-2.3",
            created: Utc::now(),
            creator: "rocket-sdk/sbom".to_string(),
            packages,
        })
    }

    // -----------------------------------------------------------------------
    // package-lock.json parser
    // -----------------------------------------------------------------------

    /// Parse an npm `package-lock.json` (v2 or v3) file into an `Sbom`.
    pub fn from_npm_lock(path: &Path) -> Result<Sbom> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("reading package-lock.json at {}", path.display()))?;
        let v: Value = serde_json::from_str(&content)
            .with_context(|| "parsing package-lock.json as JSON")?;

        let packages = parse_npm_lock(&v);
        let document_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "package-lock.json".to_string());

        Ok(Sbom {
            document_name,
            spdx_version: "SPDX-2.3",
            created: Utc::now(),
            creator: "rocket-sdk/sbom".to_string(),
            packages,
        })
    }

    // -----------------------------------------------------------------------
    // Exports
    // -----------------------------------------------------------------------

    /// Emit SPDX 2.3 JSON.
    pub fn to_spdx_json(&self) -> String {
        let packages: Vec<Value> = self
            .packages
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let mut obj = json!({
                    "SPDXID": format!("SPDXRef-Package-{}", i + 1),
                    "name": p.name,
                    "versionInfo": p.version,
                    "downloadLocation": p.source,
                    "filesAnalyzed": false,
                });
                if let Some(lic) = &p.license {
                    obj["licenseConcluded"] = Value::String(lic.clone());
                    obj["licenseDeclared"] = Value::String(lic.clone());
                } else {
                    obj["licenseConcluded"] = Value::String("NOASSERTION".to_string());
                    obj["licenseDeclared"] = Value::String("NOASSERTION".to_string());
                }
                if let Some(ck) = &p.checksum {
                    obj["checksums"] = json!([{
                        "algorithm": "SHA256",
                        "checksumValue": ck
                    }]);
                }
                obj
            })
            .collect();

        let doc = json!({
            "spdxVersion": self.spdx_version,
            "dataLicense": "CC0-1.0",
            "SPDXID": "SPDXRef-DOCUMENT",
            "name": self.document_name,
            "documentNamespace": format!("https://rocket-craft.example/sbom/{}", uuid_lite(&self.document_name, &self.created)),
            "creationInfo": {
                "created": self.created.to_rfc3339(),
                "creators": [format!("Tool: {}", self.creator)]
            },
            "packages": packages
        });

        serde_json::to_string_pretty(&doc).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
    }

    /// Emit a CSV with header row.
    pub fn to_csv(&self) -> String {
        let mut rows = Vec::new();
        rows.push("name,version,license,source,checksum,is_direct".to_string());
        for p in &self.packages {
            rows.push(format!(
                "{},{},{},{},{},{}",
                csv_escape(&p.name),
                csv_escape(&p.version),
                p.license.as_deref().map(csv_escape).unwrap_or_default(),
                csv_escape(&p.source),
                p.checksum.as_deref().map(csv_escape).unwrap_or_default(),
                p.is_direct,
            ));
        }
        rows.join("\n")
    }

    /// Emit a GitHub-flavoured Markdown table.
    pub fn to_markdown_table(&self) -> String {
        let mut lines = Vec::new();
        lines.push("| Name | Version | License | Source | Direct |".to_string());
        lines.push("|------|---------|---------|--------|--------|".to_string());
        for p in &self.packages {
            lines.push(format!(
                "| {} | {} | {} | {} | {} |",
                md_escape(&p.name),
                md_escape(&p.version),
                p.license.as_deref().map(md_escape).unwrap_or_else(|| "NOASSERTION".to_string()),
                md_escape(&p.source),
                if p.is_direct { "yes" } else { "no" },
            ));
        }
        lines.join("\n")
    }

    // -----------------------------------------------------------------------
    // Diff
    // -----------------------------------------------------------------------

    /// Compare two SBOM snapshots and return the differences.
    pub fn diff(old: &Sbom, new: &Sbom) -> SbomDiff {
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut version_changes = Vec::new();

        // Build maps keyed by name
        let old_map: std::collections::HashMap<&str, &SbomPackage> =
            old.packages.iter().map(|p| (p.name.as_str(), p)).collect();
        let new_map: std::collections::HashMap<&str, &SbomPackage> =
            new.packages.iter().map(|p| (p.name.as_str(), p)).collect();

        for (name, new_pkg) in &new_map {
            match old_map.get(name) {
                None => added.push((*new_pkg).clone()),
                Some(old_pkg) => {
                    if old_pkg.version != new_pkg.version {
                        version_changes.push(((*old_pkg).clone(), (*new_pkg).clone()));
                    }
                }
            }
        }

        for (name, old_pkg) in &old_map {
            if !new_map.contains_key(name) {
                removed.push((*old_pkg).clone());
            }
        }

        // Sort for deterministic output
        added.sort_by(|a, b| a.name.cmp(&b.name));
        removed.sort_by(|a, b| a.name.cmp(&b.name));
        version_changes.sort_by(|a, b| a.0.name.cmp(&b.0.name));

        SbomDiff { added, removed, version_changes }
    }
}

// ---------------------------------------------------------------------------
// SbomDiff
// ---------------------------------------------------------------------------

/// Supply-chain change report between two SBOM snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomDiff {
    /// Packages present in `new` but not in `old`.
    pub added: Vec<SbomPackage>,
    /// Packages present in `old` but not in `new`.
    pub removed: Vec<SbomPackage>,
    /// Packages where the version changed between `old` and `new`.
    /// Each tuple is `(old_package, new_package)`.
    pub version_changes: Vec<(SbomPackage, SbomPackage)>,
}

impl SbomDiff {
    /// True if any packages were removed.
    pub fn has_removals(&self) -> bool {
        !self.removed.is_empty()
    }

    /// True if any packages were added.
    pub fn has_additions(&self) -> bool {
        !self.added.is_empty()
    }

    /// True if no changes of any kind were detected.
    pub fn is_unchanged(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.version_changes.is_empty()
    }

    /// Render a Markdown summary of the diff for PR descriptions or audit logs.
    pub fn to_markdown(&self) -> String {
        let mut lines = Vec::new();
        lines.push("## SBOM Diff".to_string());

        if self.is_unchanged() {
            lines.push("No dependency changes detected.".to_string());
            return lines.join("\n");
        }

        if !self.added.is_empty() {
            lines.push(format!("\n### Added ({} packages)", self.added.len()));
            lines.push("| Name | Version |".to_string());
            lines.push("|------|---------|".to_string());
            for p in &self.added {
                lines.push(format!("| {} | {} |", md_escape(&p.name), md_escape(&p.version)));
            }
        }

        if !self.removed.is_empty() {
            lines.push(format!("\n### Removed ({} packages)", self.removed.len()));
            lines.push("| Name | Version |".to_string());
            lines.push("|------|---------|".to_string());
            for p in &self.removed {
                lines.push(format!("| {} | {} |", md_escape(&p.name), md_escape(&p.version)));
            }
        }

        if !self.version_changes.is_empty() {
            lines.push(format!("\n### Version Changes ({} packages)", self.version_changes.len()));
            lines.push("| Name | Old Version | New Version |".to_string());
            lines.push("|------|-------------|-------------|".to_string());
            for (old, new) in &self.version_changes {
                lines.push(format!(
                    "| {} | {} | {} |",
                    md_escape(&old.name),
                    md_escape(&old.version),
                    md_escape(&new.version)
                ));
            }
        }

        lines.join("\n")
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Escape a CSV field value (wrap in double quotes if it contains commas,
/// quotes, or newlines).
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Escape pipe characters in Markdown table cells.
fn md_escape(s: &str) -> String {
    s.replace('|', r"\|")
}

/// Generate a stable, deterministic pseudo-UUID from a document name + timestamp.
/// This avoids needing the `uuid` crate.
fn uuid_lite(name: &str, ts: &DateTime<Utc>) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    name.hash(&mut h);
    ts.timestamp().hash(&mut h);
    let n = h.finish();
    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        (n >> 32) as u32,
        (n >> 16) as u16 & 0xFFFF,
        n as u16 & 0x0FFF,
        ((n >> 48) as u16 & 0x3FFF) | 0x8000,
        n & 0x0000_FFFF_FFFF,
    )
}

// ---------------------------------------------------------------------------
// Cargo.lock parser — state machine
// ---------------------------------------------------------------------------

/// State machine state for parsing `[[package]]` blocks.
#[derive(Default)]
struct CargoLockPkg {
    name: Option<String>,
    version: Option<String>,
    source: Option<String>,
    checksum: Option<String>,
}

/// Parse the content of a `Cargo.lock` file (v3 TOML) and return a list of
/// `SbomPackage` values.  Uses only `str` methods — no `toml` crate.
fn parse_cargo_lock(content: &str) -> Vec<SbomPackage> {
    let mut packages = Vec::new();
    let mut current: Option<CargoLockPkg> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[[package]]" {
            // Flush previous block
            if let Some(pkg) = current.take() {
                if let (Some(name), Some(version)) = (pkg.name, pkg.version) {
                    packages.push(SbomPackage {
                        name,
                        version,
                        license: None,
                        source: pkg.source.unwrap_or_else(|| "local".to_string()),
                        checksum: pkg.checksum,
                        is_direct: false, // Cargo.lock doesn't distinguish direct vs transitive
                    });
                }
            }
            current = Some(CargoLockPkg::default());
            continue;
        }

        if let Some(ref mut pkg) = current {
            if let Some(val) = extract_toml_string(trimmed, "name") {
                pkg.name = Some(val);
            } else if let Some(val) = extract_toml_string(trimmed, "version") {
                pkg.version = Some(val);
            } else if let Some(val) = extract_toml_string(trimmed, "source") {
                pkg.source = Some(val);
            } else if let Some(val) = extract_toml_string(trimmed, "checksum") {
                pkg.checksum = Some(val);
            }
        }
    }

    // Flush last block
    if let Some(pkg) = current {
        if let (Some(name), Some(version)) = (pkg.name, pkg.version) {
            packages.push(SbomPackage {
                name,
                version,
                license: None,
                source: pkg.source.unwrap_or_else(|| "local".to_string()),
                checksum: pkg.checksum,
                is_direct: false,
            });
        }
    }

    packages
}

/// Extract a TOML string value for `key = "value"` from a trimmed line.
///
/// Returns `Some(value)` without surrounding quotes; `None` if the line does
/// not match.
fn extract_toml_string(line: &str, key: &str) -> Option<String> {
    // Pattern: `key = "value"` or `key = 'value'`
    let prefix = format!("{} = ", key);
    if !line.starts_with(&prefix) {
        return None;
    }
    let rest = &line[prefix.len()..];
    if (rest.starts_with('"') && rest.ends_with('"'))
        || (rest.starts_with('\'') && rest.ends_with('\''))
    {
        Some(rest[1..rest.len() - 1].to_string())
    } else {
        // Bare value (shouldn't appear for these keys but handle gracefully)
        Some(rest.to_string())
    }
}

// ---------------------------------------------------------------------------
// package-lock.json parser
// ---------------------------------------------------------------------------

/// Parse an npm `package-lock.json` value into a list of `SbomPackage`s.
///
/// Supports lockfile v2 (`dependencies` map) and v3 (`packages` map).
fn parse_npm_lock(v: &Value) -> Vec<SbomPackage> {
    let mut packages = Vec::new();

    // Lockfile v3: `packages` object where keys are paths like "node_modules/foo"
    if let Some(pkgs) = v.get("packages").and_then(|p| p.as_object()) {
        for (path_key, pkg_val) in pkgs {
            // Skip the root package entry (empty key or "")
            if path_key.is_empty() {
                continue;
            }
            let name = extract_npm_package_name(path_key, pkg_val);
            let version = pkg_val
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0")
                .to_string();
            let license = pkg_val
                .get("license")
                .and_then(|l| l.as_str())
                .map(str::to_string);
            let resolved = pkg_val
                .get("resolved")
                .and_then(|r| r.as_str())
                .unwrap_or("https://registry.npmjs.org/")
                .to_string();
            let integrity = pkg_val
                .get("integrity")
                .and_then(|i| i.as_str())
                .map(str::to_string);
            let is_dev = pkg_val
                .get("dev")
                .and_then(|d| d.as_bool())
                .unwrap_or(false);

            packages.push(SbomPackage {
                name,
                version,
                license,
                source: resolved,
                checksum: integrity,
                is_direct: !is_dev,
            });
        }
        return packages;
    }

    // Lockfile v2: `dependencies` object where keys are package names
    if let Some(deps) = v.get("dependencies").and_then(|d| d.as_object()) {
        for (name, pkg_val) in deps {
            let version = pkg_val
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0")
                .to_string();
            let resolved = pkg_val
                .get("resolved")
                .and_then(|r| r.as_str())
                .unwrap_or("https://registry.npmjs.org/")
                .to_string();
            let integrity = pkg_val
                .get("integrity")
                .and_then(|i| i.as_str())
                .map(str::to_string);

            packages.push(SbomPackage {
                name: name.clone(),
                version,
                license: None,
                source: resolved,
                checksum: integrity,
                is_direct: true,
            });
        }
    }

    packages
}

/// Extract the package name from a v3 lockfile path key like `node_modules/foo`
/// or `node_modules/@scope/pkg`.
fn extract_npm_package_name(path_key: &str, pkg_val: &Value) -> String {
    // If the `name` field is explicitly present in the package object, use it
    if let Some(name) = pkg_val.get("name").and_then(|n| n.as_str()) {
        return name.to_string();
    }
    // Strip leading "node_modules/" prefix(es) for nested deps
    let stripped = path_key
        .split("node_modules/")
        .last()
        .unwrap_or(path_key);
    stripped.to_string()
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Cargo.lock parsing helpers
    // -----------------------------------------------------------------------

    const MINIMAL_CARGO_LOCK: &str = r#"
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 3

[[package]]
name = "anyhow"
version = "1.0.82"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f538837af36e6f6a9be0faa67f9a314f8119e4e4b5867c6ab40ed60360142519"

[[package]]
name = "my-local-crate"
version = "0.1.0"

[[package]]
name = "serde"
version = "1.0.200"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aaabbbccc"
"#;

    #[test]
    fn test_parse_cargo_lock_package_count() {
        let pkgs = parse_cargo_lock(MINIMAL_CARGO_LOCK);
        assert_eq!(pkgs.len(), 3, "Should parse 3 packages");
    }

    #[test]
    fn test_parse_cargo_lock_names() {
        let pkgs = parse_cargo_lock(MINIMAL_CARGO_LOCK);
        let names: Vec<&str> = pkgs.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"anyhow"), "Should contain anyhow");
        assert!(names.contains(&"serde"), "Should contain serde");
        assert!(names.contains(&"my-local-crate"), "Should contain local crate");
    }

    #[test]
    fn test_parse_cargo_lock_versions() {
        let pkgs = parse_cargo_lock(MINIMAL_CARGO_LOCK);
        let anyhow = pkgs.iter().find(|p| p.name == "anyhow").unwrap();
        assert_eq!(anyhow.version, "1.0.82");
    }

    #[test]
    fn test_parse_cargo_lock_checksum() {
        let pkgs = parse_cargo_lock(MINIMAL_CARGO_LOCK);
        let anyhow = pkgs.iter().find(|p| p.name == "anyhow").unwrap();
        assert!(anyhow.checksum.is_some(), "anyhow should have a checksum");
        assert!(anyhow.checksum.as_deref().unwrap().starts_with("f538837a"));
    }

    #[test]
    fn test_parse_cargo_lock_local_crate_no_checksum() {
        let pkgs = parse_cargo_lock(MINIMAL_CARGO_LOCK);
        let local = pkgs.iter().find(|p| p.name == "my-local-crate").unwrap();
        assert!(local.checksum.is_none(), "Local crate should have no checksum");
        assert_eq!(local.source, "local", "Local crate source should be 'local'");
    }

    #[test]
    fn test_parse_cargo_lock_source_url() {
        let pkgs = parse_cargo_lock(MINIMAL_CARGO_LOCK);
        let anyhow = pkgs.iter().find(|p| p.name == "anyhow").unwrap();
        assert!(anyhow.source.contains("crates.io-index"), "Source should contain crates.io-index");
    }

    // -----------------------------------------------------------------------
    // Sbom::from_cargo_lock
    // -----------------------------------------------------------------------

    #[test]
    fn test_from_cargo_lock_creates_sbom() {
        use std::io::Write;
        let dir = tempfile::TempDir::new().unwrap();
        let lock_path = dir.path().join("Cargo.lock");
        let mut f = std::fs::File::create(&lock_path).unwrap();
        writeln!(f, "{}", MINIMAL_CARGO_LOCK).unwrap();

        let sbom = Sbom::from_cargo_lock(&lock_path).unwrap();
        assert_eq!(sbom.spdx_version, "SPDX-2.3");
        assert!(!sbom.packages.is_empty());
        assert_eq!(sbom.document_name, "Cargo.lock");
    }

    // -----------------------------------------------------------------------
    // package-lock.json parsing
    // -----------------------------------------------------------------------

    const MINIMAL_NPM_LOCK_V3: &str = r#"{
  "name": "pwa-staff",
  "version": "1.0.0",
  "lockfileVersion": 3,
  "packages": {
    "": {
      "name": "pwa-staff",
      "version": "1.0.0"
    },
    "node_modules/typescript": {
      "version": "5.4.5",
      "resolved": "https://registry.npmjs.org/typescript/-/typescript-5.4.5.tgz",
      "integrity": "sha512-vcI4UpRgg81oIRUFwR0j1Q3JEdC0F1M4T9XXSF7kd11L7okQi18t/7Kd8j7GFo5ZQvKFDXtB7PGaXwN6rCEw==",
      "license": "Apache-2.0"
    },
    "node_modules/vitest": {
      "version": "1.6.0",
      "resolved": "https://registry.npmjs.org/vitest/-/vitest-1.6.0.tgz",
      "integrity": "sha512-abc123def456",
      "license": "MIT",
      "dev": true
    }
  }
}"#;

    const MINIMAL_NPM_LOCK_V2: &str = r#"{
  "name": "pwa-staff",
  "version": "1.0.0",
  "lockfileVersion": 2,
  "dependencies": {
    "esbuild": {
      "version": "0.21.5",
      "resolved": "https://registry.npmjs.org/esbuild/-/esbuild-0.21.5.tgz",
      "integrity": "sha512-xyz=="
    }
  }
}"#;

    #[test]
    fn test_parse_npm_lock_v3_package_count() {
        let v: Value = serde_json::from_str(MINIMAL_NPM_LOCK_V3).unwrap();
        let pkgs = parse_npm_lock(&v);
        assert_eq!(pkgs.len(), 2, "Should parse 2 non-root packages");
    }

    #[test]
    fn test_parse_npm_lock_v3_names() {
        let v: Value = serde_json::from_str(MINIMAL_NPM_LOCK_V3).unwrap();
        let pkgs = parse_npm_lock(&v);
        let names: Vec<&str> = pkgs.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"typescript"), "Should contain typescript");
        assert!(names.contains(&"vitest"), "Should contain vitest");
    }

    #[test]
    fn test_parse_npm_lock_v3_license() {
        let v: Value = serde_json::from_str(MINIMAL_NPM_LOCK_V3).unwrap();
        let pkgs = parse_npm_lock(&v);
        let ts = pkgs.iter().find(|p| p.name == "typescript").unwrap();
        assert_eq!(ts.license.as_deref(), Some("Apache-2.0"));
    }

    #[test]
    fn test_parse_npm_lock_v3_dev_package() {
        let v: Value = serde_json::from_str(MINIMAL_NPM_LOCK_V3).unwrap();
        let pkgs = parse_npm_lock(&v);
        let vitest = pkgs.iter().find(|p| p.name == "vitest").unwrap();
        // dev=true means is_direct=false in our model
        assert!(!vitest.is_direct, "dev packages should not be marked direct");
    }

    #[test]
    fn test_parse_npm_lock_v2_fallback() {
        let v: Value = serde_json::from_str(MINIMAL_NPM_LOCK_V2).unwrap();
        let pkgs = parse_npm_lock(&v);
        assert_eq!(pkgs.len(), 1);
        assert_eq!(pkgs[0].name, "esbuild");
        assert_eq!(pkgs[0].version, "0.21.5");
    }

    // -----------------------------------------------------------------------
    // SPDX JSON export
    // -----------------------------------------------------------------------

    #[test]
    fn test_to_spdx_json_is_valid_json() {
        let sbom = minimal_sbom();
        let json_str = sbom.to_spdx_json();
        let v: Value = serde_json::from_str(&json_str).expect("SPDX output must be valid JSON");
        assert_eq!(v["spdxVersion"], "SPDX-2.3");
    }

    #[test]
    fn test_to_spdx_json_contains_package() {
        let sbom = minimal_sbom();
        let json_str = sbom.to_spdx_json();
        let v: Value = serde_json::from_str(&json_str).unwrap();
        let packages = v["packages"].as_array().unwrap();
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0]["name"], "anyhow");
    }

    #[test]
    fn test_to_spdx_json_has_creation_info() {
        let sbom = minimal_sbom();
        let json_str = sbom.to_spdx_json();
        let v: Value = serde_json::from_str(&json_str).unwrap();
        assert!(v["creationInfo"].is_object(), "Should have creationInfo block");
        assert!(v["creationInfo"]["created"].as_str().is_some(), "Should have created timestamp");
    }

    #[test]
    fn test_to_spdx_json_checksum_field() {
        let sbom = minimal_sbom();
        let json_str = sbom.to_spdx_json();
        let v: Value = serde_json::from_str(&json_str).unwrap();
        let pkgs = v["packages"].as_array().unwrap();
        let checksums = &pkgs[0]["checksums"];
        assert!(checksums.is_array(), "Should have checksums array");
        assert_eq!(checksums[0]["algorithm"], "SHA256");
    }

    // -----------------------------------------------------------------------
    // CSV export
    // -----------------------------------------------------------------------

    #[test]
    fn test_to_csv_has_header_row() {
        let sbom = minimal_sbom();
        let csv = sbom.to_csv();
        let first_line = csv.lines().next().unwrap();
        assert!(first_line.contains("name"), "First row must be header");
        assert!(first_line.contains("version"), "Header must contain version");
        assert!(first_line.contains("license"), "Header must contain license");
    }

    #[test]
    fn test_to_csv_contains_package_data() {
        let sbom = minimal_sbom();
        let csv = sbom.to_csv();
        assert!(csv.contains("anyhow"), "CSV must contain package name");
        assert!(csv.contains("1.0.82"), "CSV must contain version");
    }

    #[test]
    fn test_to_csv_escapes_commas() {
        let mut sbom = minimal_sbom();
        sbom.packages[0].source = "registry,with,commas".to_string();
        let csv = sbom.to_csv();
        assert!(csv.contains("\"registry,with,commas\""), "Commas in fields must be quoted");
    }

    // -----------------------------------------------------------------------
    // Markdown table export
    // -----------------------------------------------------------------------

    #[test]
    fn test_to_markdown_table_has_header() {
        let sbom = minimal_sbom();
        let md = sbom.to_markdown_table();
        assert!(md.contains("| Name |"), "Should have Name column header");
        assert!(md.contains("| Version |"), "Should have Version column header");
        assert!(md.contains("| License |"), "Should have License column header");
    }

    #[test]
    fn test_to_markdown_table_has_separator() {
        let sbom = minimal_sbom();
        let md = sbom.to_markdown_table();
        assert!(md.contains("|------|"), "Should have separator row");
    }

    #[test]
    fn test_to_markdown_table_has_data_row() {
        let sbom = minimal_sbom();
        let md = sbom.to_markdown_table();
        assert!(md.contains("anyhow"), "Should contain package name in data");
        assert!(md.contains("1.0.82"), "Should contain version in data");
    }

    // -----------------------------------------------------------------------
    // SbomDiff
    // -----------------------------------------------------------------------

    fn make_pkg(name: &str, version: &str) -> SbomPackage {
        SbomPackage {
            name: name.to_string(),
            version: version.to_string(),
            license: Some("MIT".to_string()),
            source: "https://registry.npmjs.org/".to_string(),
            checksum: None,
            is_direct: true,
        }
    }

    #[test]
    fn test_diff_detects_added_package() {
        let old = sbom_with_packages(vec![make_pkg("anyhow", "1.0.0")]);
        let new = sbom_with_packages(vec![
            make_pkg("anyhow", "1.0.0"),
            make_pkg("serde", "1.0.200"),
        ]);
        let diff = Sbom::diff(&old, &new);
        assert!(diff.has_additions(), "Should detect added package");
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].name, "serde");
    }

    #[test]
    fn test_diff_detects_removed_package() {
        let old = sbom_with_packages(vec![
            make_pkg("anyhow", "1.0.0"),
            make_pkg("serde", "1.0.200"),
        ]);
        let new = sbom_with_packages(vec![make_pkg("anyhow", "1.0.0")]);
        let diff = Sbom::diff(&old, &new);
        assert!(diff.has_removals(), "Should detect removed package");
        assert_eq!(diff.removed.len(), 1);
        assert_eq!(diff.removed[0].name, "serde");
    }

    #[test]
    fn test_diff_detects_version_change() {
        let old = sbom_with_packages(vec![make_pkg("anyhow", "1.0.0")]);
        let new = sbom_with_packages(vec![make_pkg("anyhow", "1.0.82")]);
        let diff = Sbom::diff(&old, &new);
        assert_eq!(diff.version_changes.len(), 1, "Should detect version change");
        assert_eq!(diff.version_changes[0].0.version, "1.0.0");
        assert_eq!(diff.version_changes[0].1.version, "1.0.82");
    }

    #[test]
    fn test_diff_no_changes() {
        let sbom = sbom_with_packages(vec![make_pkg("anyhow", "1.0.0")]);
        let diff = Sbom::diff(&sbom, &sbom);
        assert!(diff.is_unchanged(), "Identical SBOMs should produce no diff");
        assert!(!diff.has_additions());
        assert!(!diff.has_removals());
    }

    #[test]
    fn test_diff_markdown_no_changes() {
        let sbom = sbom_with_packages(vec![]);
        let diff = Sbom::diff(&sbom, &sbom);
        assert!(diff.to_markdown().contains("No dependency changes"), "Unchanged diff should say so");
    }

    #[test]
    fn test_diff_markdown_shows_added() {
        let old = sbom_with_packages(vec![]);
        let new = sbom_with_packages(vec![make_pkg("serde", "1.0.200")]);
        let diff = Sbom::diff(&old, &new);
        let md = diff.to_markdown();
        assert!(md.contains("Added"), "Markdown should mention Added");
        assert!(md.contains("serde"), "Markdown should name the added package");
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn minimal_sbom() -> Sbom {
        sbom_with_packages(vec![SbomPackage {
            name: "anyhow".to_string(),
            version: "1.0.82".to_string(),
            license: Some("MIT OR Apache-2.0".to_string()),
            source: "registry+https://github.com/rust-lang/crates.io-index".to_string(),
            checksum: Some("abc123def456".to_string()),
            is_direct: true,
        }])
    }

    fn sbom_with_packages(packages: Vec<SbomPackage>) -> Sbom {
        Sbom {
            document_name: "test-sbom".to_string(),
            spdx_version: "SPDX-2.3",
            created: Utc::now(),
            creator: "test".to_string(),
            packages,
        }
    }
}
