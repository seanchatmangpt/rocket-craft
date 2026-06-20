# SOC2 Security Tooling — Rocket Craft Monorepo

This document describes the secret-scanning and SBOM generation capabilities
added to `rocket-sdk` in support of SOC2 Trust Service Criteria.  It covers:

- Which SOC2 controls are addressed
- How to wire the new modules into `lib.rs` and `main.rs`
- Every new `rocket security` CLI sub-command with flags and examples
- Any new Cargo dependencies (none required)

---

## SOC2 Controls Addressed

| Control | ID | Module | Mechanism |
|---|---|---|---|
| Logical and physical access controls | CC6.7 | `secret_scan` | Prevents hard-coded credentials from reaching the repository; blocks CI when Critical/High secrets are detected |
| Risk management — third-party | CC9.1 | `sbom` | Generates a complete inventory of all software dependencies (Rust + npm); enables supply-chain diff on every PR |
| Change management | CC8.1 | `sbom diff` | Detects dependency additions, removals, and version changes between two SBOM snapshots |
| Incident response evidence | CC7.3 | Both | JSON/SPDX output acts as timestamped audit evidence for SOC2 auditor requests |
| Vendor management | CC9.2 | `sbom` | SPDX 2.3 output can be submitted to customers and auditors as a standard artifact |

---

## Module Registration — `lib.rs`

Add the following two lines to `tools/rocket-sdk/src/lib.rs` (in any order,
alongside the existing `pub mod` declarations):

```rust
pub mod secret_scan;
pub mod sbom;
```

No new `[dependencies]` are required.  Both modules rely only on crates
already present in `tools/Cargo.lock`:

| Crate | Used for |
|---|---|
| `serde` + `serde_json` | JSON serialisation of `ScanReport`, `Sbom`, SPDX output |
| `chrono` | `DateTime<Utc>` timestamp in `Sbom` |
| `walkdir` | Directory traversal in `SecretScanner::scan_dir` |
| `anyhow` | Error propagation |
| `thiserror` | (Not used directly; available if you extend `SdkError`) |
| `blake3` | (Available; not used in initial release — reserved for file hashing) |
| `tempfile` | Test fixtures only (already a `dev-dependency`) |

---

## `main.rs` — Wiring the `security` Subcommand

### 1. Add imports

```rust
use rocket_sdk::secret_scan::{SecretScanner, Severity, check_ci_gate};
use rocket_sdk::sbom::Sbom;
```

### 2. Extend the `Commands` enum

```rust
/// SOC2 secret scanning and Software Bill of Materials tools
Security {
    #[command(subcommand)]
    security_cmd: SecuritySubcommands,
},
```

### 3. Add the `SecuritySubcommands` enum

```rust
#[derive(Subcommand)]
enum SecuritySubcommands {
    /// Scan source files for hard-coded secrets (SOC2 CC6.7)
    Scan {
        /// Root directory to scan (default: current working directory)
        #[arg(short, long, default_value = ".")]
        path: String,

        /// Emit findings as JSON to stdout instead of human-readable text
        #[arg(long)]
        json: bool,

        /// Exit non-zero if any finding matches these comma-separated severities
        /// Accepted values: critical, high, medium, low, info
        /// Example: --fail-on critical,high
        #[arg(long, default_value = "critical,high")]
        fail_on: String,

        /// Path to a .secretsignore file (default: <path>/.secretsignore)
        #[arg(long)]
        ignore_file: Option<String>,
    },

    /// Generate a Software Bill of Materials for this workspace (SOC2 CC9.1)
    Sbom {
        /// Workspace to generate the SBOM for
        /// Options: rust (reads Cargo.lock), npm (reads pwa-staff/package-lock.json), all
        #[arg(long, default_value = "all")]
        workspace: String,

        /// Output format
        /// Options: spdx (SPDX 2.3 JSON), csv, markdown
        #[arg(long, default_value = "spdx")]
        format: String,

        /// Write output to this file instead of stdout
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Compare two SBOM snapshots and report supply-chain changes
    SbomDiff {
        /// Path to the old SBOM JSON file (previously exported via `rocket security sbom`)
        #[arg(long)]
        old: String,

        /// Path to the new SBOM JSON file
        #[arg(long)]
        new: String,

        /// Emit result as JSON instead of Markdown
        #[arg(long)]
        json: bool,
    },

    /// Export the current workspace SBOM to a file for archival / auditor delivery
    SbomExport {
        /// Directory to write the SBOM files into (default: security-artifacts/)
        #[arg(long, default_value = "security-artifacts")]
        dir: String,

        /// Workspaces to include: rust, npm, all
        #[arg(long, default_value = "all")]
        workspace: String,
    },
}
```

### 4. Handle the new commands in the `match` block

```rust
Commands::Security { security_cmd } => match security_cmd {
    SecuritySubcommands::Scan { path, json, fail_on, ignore_file } => {
        run_security_scan(&path, json, &fail_on, ignore_file.as_deref())
    }
    SecuritySubcommands::Sbom { workspace, format, output } => {
        run_security_sbom(&workspace, &format, output.as_deref())
    }
    SecuritySubcommands::SbomDiff { old, new, json } => {
        run_security_sbom_diff(&old, &new, json)
    }
    SecuritySubcommands::SbomExport { dir, workspace } => {
        run_security_sbom_export(&dir, &workspace)
    }
},
```

### 5. Implementation functions

```rust
fn run_security_scan(
    path: &str,
    json: bool,
    fail_on: &str,
    ignore_file: Option<&str>,
) -> Result<()> {
    use rocket_sdk::secret_scan::{SecretScanner, Severity, check_ci_gate};

    let root = std::path::Path::new(path);
    let default_ignore = root.join(".secretsignore");

    let scanner = if let Some(ig) = ignore_file {
        SecretScanner::new()
            .with_ignore_file(std::path::Path::new(ig))?
    } else {
        SecretScanner::new()
            .with_ignore_file(&default_ignore)?
    };

    let report = scanner.scan_dir(root)?;

    if json {
        println!("{}", report.to_json());
    } else {
        println!("{}", report.summary());
        for f in &report.findings {
            println!(
                "[{}] {}:{} — {} — {}",
                f.severity,
                f.file.display(),
                f.line_number,
                f.pattern_name,
                f.redacted_match,
            );
        }
    }

    // Parse --fail-on severities
    let block_severities: Vec<Severity> = fail_on
        .split(',')
        .filter_map(|s| match s.trim().to_lowercase().as_str() {
            "critical" => Some(Severity::Critical),
            "high"     => Some(Severity::High),
            "medium"   => Some(Severity::Medium),
            "low"      => Some(Severity::Low),
            "info"     => Some(Severity::Info),
            _          => None,
        })
        .collect();

    match check_ci_gate(&report, &block_severities) {
        Ok(()) => Ok(()),
        Err(blocking) => {
            eprintln!("\nCI gate FAILED: {} blocking finding(s)", blocking.len());
            std::process::exit(1);
        }
    }
}

fn run_security_sbom(workspace: &str, format: &str, output: Option<&str>) -> Result<()> {
    use rocket_sdk::sbom::Sbom;

    let mut all_packages = Vec::new();

    if workspace == "rust" || workspace == "all" {
        if let Ok(sbom) = Sbom::from_cargo_lock(std::path::Path::new("tools/Cargo.lock")) {
            all_packages.extend(sbom.packages);
        }
    }

    if workspace == "npm" || workspace == "all" {
        if let Ok(sbom) = Sbom::from_npm_lock(
            std::path::Path::new("pwa-staff/package-lock.json")
        ) {
            all_packages.extend(sbom.packages);
        }
    }

    let combined = Sbom {
        document_name: format!("rocket-craft-{}", workspace),
        spdx_version: "SPDX-2.3",
        created: chrono::Utc::now(),
        creator: "rocket-cmd/security sbom".to_string(),
        packages: all_packages,
    };

    let rendered = match format {
        "csv"      => combined.to_csv(),
        "markdown" => combined.to_markdown_table(),
        _          => combined.to_spdx_json(), // default: spdx
    };

    match output {
        Some(path) => std::fs::write(path, &rendered)?,
        None       => println!("{}", rendered),
    }
    Ok(())
}

fn run_security_sbom_diff(old_path: &str, new_path: &str, json: bool) -> Result<()> {
    use rocket_sdk::sbom::{Sbom, SbomDiff};

    let old_json = std::fs::read_to_string(old_path)?;
    let new_json = std::fs::read_to_string(new_path)?;

    // Re-hydrate from serialised Sbom JSON
    let old: Sbom = serde_json::from_str(&old_json)?;
    let new: Sbom = serde_json::from_str(&new_json)?;
    let diff = Sbom::diff(&old, &new);

    if json {
        println!("{}", serde_json::to_string_pretty(&diff)?);
    } else {
        println!("{}", diff.to_markdown());
    }
    Ok(())
}

fn run_security_sbom_export(dir: &str, workspace: &str) -> Result<()> {
    use rocket_sdk::sbom::Sbom;

    std::fs::create_dir_all(dir)?;
    let ts = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");

    if workspace == "rust" || workspace == "all" {
        if let Ok(sbom) = Sbom::from_cargo_lock(std::path::Path::new("tools/Cargo.lock")) {
            let path = format!("{}/sbom-rust-{}.spdx.json", dir, ts);
            std::fs::write(&path, sbom.to_spdx_json())?;
            println!("Exported Rust SBOM to {}", path);
        }
    }

    if workspace == "npm" || workspace == "all" {
        if let Ok(sbom) = Sbom::from_npm_lock(
            std::path::Path::new("pwa-staff/package-lock.json")
        ) {
            let path = format!("{}/sbom-npm-{}.spdx.json", dir, ts);
            std::fs::write(&path, sbom.to_spdx_json())?;
            println!("Exported npm SBOM to {}", path);
        }
    }

    Ok(())
}
```

---

## CLI Reference — `rocket security`

### `rocket security scan`

Scan the repository for hard-coded secrets.  Implements SOC2 CC6.7.

```
USAGE:
    rocket security scan [OPTIONS]

OPTIONS:
    -p, --path <PATH>             Root directory to scan [default: .]
        --json                    Emit findings as JSON
        --fail-on <SEVERITIES>    Comma-separated severities that trigger a non-zero exit
                                  [default: critical,high]
                                  Values: critical, high, medium, low, info
        --ignore-file <FILE>      Path to a .secretsignore file
                                  [default: <path>/.secretsignore]

EXAMPLES:
    # Scan the whole repo, block on critical or high secrets
    rocket security scan

    # Scan only the pwa-staff directory
    rocket security scan --path pwa-staff/

    # Output findings as JSON (useful for piping to jq or SIEM)
    rocket security scan --json

    # Block only on critical (allow high findings through — use with care)
    rocket security scan --fail-on critical

    # Provide a custom ignore file
    rocket security scan --ignore-file .ci/secretsignore

EXIT CODES:
    0   No blocking findings
    1   One or more findings with severity in --fail-on
```

#### `.secretsignore` file format

Lines beginning with `#` are comments.  Each non-empty line is a path suffix
or exact path to exclude from scanning.  Trailing `*` is supported as a simple
prefix glob.

```
# Ignore the local Supabase configuration (safe keys only)
supabase/.branches

# Ignore test fixtures that contain intentionally planted secrets
tools/rocket-sdk/tests/fixtures/

# Ignore everything under a vendor directory
vendor/*
```

### `rocket security sbom`

Generate a Software Bill of Materials.  Implements SOC2 CC9.1.

```
USAGE:
    rocket security sbom [OPTIONS]

OPTIONS:
        --workspace <WS>    Workspace to include: rust | npm | all [default: all]
        --format <FMT>      Output format: spdx | csv | markdown [default: spdx]
    -o, --output <FILE>     Write output to FILE instead of stdout

EXAMPLES:
    # Emit SPDX 2.3 JSON for all workspaces to stdout
    rocket security sbom

    # Emit Markdown table for the Rust workspace only
    rocket security sbom --workspace rust --format markdown

    # Save CSV to a file for spreadsheet review
    rocket security sbom --format csv --output security-artifacts/sbom.csv

    # Generate SPDX JSON for the npm workspace
    rocket security sbom --workspace npm --output sbom-npm.spdx.json
```

### `rocket security sbom diff`

Compare two SBOM snapshots and emit a supply-chain change report.

```
USAGE:
    rocket security sbom-diff --old <FILE> --new <FILE> [--json]

OPTIONS:
        --old <FILE>    Path to the previous SBOM JSON file
        --new <FILE>    Path to the current SBOM JSON file
        --json          Emit result as JSON instead of Markdown

EXAMPLES:
    # Compare yesterday's SBOM with today's
    rocket security sbom-diff \
        --old security-artifacts/sbom-rust-20260619T120000Z.spdx.json \
        --new security-artifacts/sbom-rust-20260620T120000Z.spdx.json

    # Emit JSON diff for machine parsing
    rocket security sbom-diff --old old.json --new new.json --json
```

### `rocket security sbom export`

Export SBOM files to a timestamped directory for archival or auditor delivery.

```
USAGE:
    rocket security sbom-export [OPTIONS]

OPTIONS:
        --dir <DIR>           Output directory [default: security-artifacts]
        --workspace <WS>      rust | npm | all [default: all]

EXAMPLES:
    # Export all SBOMs to security-artifacts/
    rocket security sbom-export

    # Export only the Rust SBOM to a custom directory
    rocket security sbom-export --workspace rust --dir /tmp/sbom-delivery
```

---

## Detected Secret Types

The following 22 patterns are included in the built-in pattern library:

| Pattern Name | Severity | Description |
|---|---|---|
| `aws-access-key` | CRITICAL | AWS access key ID (`AKIA` + 16 upper-alnum chars) |
| `aws-secret-access-key` | CRITICAL | AWS secret access key (40+ chars after `aws_secret`) |
| `github-pat` | CRITICAL | GitHub PAT (`ghp_`, `ghs_`, `gho_`, `ghr_`, `ghu_` + 36+ chars) |
| `supabase-service-role-key` | CRITICAL | Supabase service_role JWT (NOT the safe local anon key) |
| `stripe-secret-key` | CRITICAL | Stripe secret key (`sk_live_` or `sk_test_`) |
| `pem-rsa-private-key` | CRITICAL | RSA private key PEM header |
| `pem-ec-private-key` | CRITICAL | EC private key PEM header |
| `pem-private-key` | CRITICAL | PKCS#8 private key PEM header |
| `openssh-private-key` | CRITICAL | OpenSSH private key PEM header |
| `database-url-with-password` | CRITICAL | `postgres://user:realpass@host` (non-placeholder password) |
| `generic-jwt` | HIGH | Generic JWT (`eyJ...`) with header.payload.signature structure |
| `env-password` | HIGH | `PASSWORD=`, `PASS=`, `SECRET=`, `TOKEN=` with real value |
| `google-api-key` | HIGH | Google API key (`AIza` + 35 chars) |
| `slack-token` | HIGH | Slack API token (`xox[baprs]-`) |
| `sendgrid-key` | HIGH | SendGrid API key (`SG.` + 22+ chars) |
| `twilio-account-sid` | HIGH | Twilio Account SID (`AC` + 32 hex chars) |
| `twilio-auth-token` | HIGH | Twilio auth token adjacent to keyword `twilio` |
| `generic-api-key` | MEDIUM | `api_key` / `apikey` / `api-key` = long value |
| `gradle-keystore-password` | MEDIUM | `storePassword` or `keyPassword` with real value |
| `hardcoded-ip-credentials` | MEDIUM | `credentials@<x.x.x.x>` pattern |
| `stripe-publishable-key` | LOW | Stripe publishable key (`pk_live_` — worth tracking) |
| `generic-base64-credential` | INFO | Long base64 string assigned to credential-like variable |

### Known-safe value: Supabase local anon key

The key `sb_publishable_ACJWlzQHlZjBrEguHvfOxg_3BJgxAaH` (documented in
`CLAUDE.md` as safe to commit) is explicitly allowlisted and will **never** be
flagged, regardless of which patterns are active.

---

## SBOM Package Sources

| Source | Lock file | Parser |
|---|---|---|
| Rust (crates.io) | `tools/Cargo.lock` | `Sbom::from_cargo_lock` — hand-rolled TOML v3 state machine |
| npm | `pwa-staff/package-lock.json` | `Sbom::from_npm_lock` — `serde_json`; supports lockfileVersion 2 and 3 |

Additional lock files (e.g., `nexus-engine/Cargo.lock`, `unify-rs/Cargo.lock`)
can be ingested by calling `Sbom::from_cargo_lock` with each path and merging the
`packages` vectors before exporting.

---

## SPDX 2.3 JSON Fields

The `to_spdx_json()` export includes the following SPDX 2.3 fields:

| SPDX Field | Value |
|---|---|
| `spdxVersion` | `"SPDX-2.3"` |
| `dataLicense` | `"CC0-1.0"` |
| `SPDXID` | `"SPDXRef-DOCUMENT"` |
| `name` | lock-file name (e.g. `"Cargo.lock"`) |
| `documentNamespace` | `"https://rocket-craft.example/sbom/<uuid>"` |
| `creationInfo.created` | RFC 3339 UTC timestamp |
| `creationInfo.creators` | `["Tool: rocket-sdk/sbom"]` |
| `packages[].SPDXID` | `"SPDXRef-Package-N"` |
| `packages[].name` | package name |
| `packages[].versionInfo` | version string |
| `packages[].downloadLocation` | source URL or `"local"` |
| `packages[].licenseConcluded` | SPDX license expression or `"NOASSERTION"` |
| `packages[].licenseDeclared` | same as `licenseConcluded` |
| `packages[].checksums` | `[{"algorithm": "SHA256", "checksumValue": "..."}]` |
| `packages[].filesAnalyzed` | `false` |

---

## CI Integration

Add the following to `.github/workflows/ci.yml` to gate every push:

```yaml
  secret-scan:
    name: SOC2 Secret Scan
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Build rocket-cmd
        run: cd tools && cargo build --release -p rocket-cmd
      - name: Run secret scan
        run: ./tools/target/release/rocket-cmd security scan --fail-on critical,high

  sbom-export:
    name: SOC2 SBOM Export
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Build rocket-cmd
        run: cd tools && cargo build --release -p rocket-cmd
      - name: Export SBOM
        run: ./tools/target/release/rocket-cmd security sbom-export --dir security-artifacts
      - uses: actions/upload-artifact@v4
        with:
          name: sbom-${{ github.sha }}
          path: security-artifacts/
          retention-days: 90
```

---

## File Locations

| File | Purpose |
|---|---|
| `tools/rocket-sdk/src/secret_scan.rs` | SOC2 CC6.7 secret detection library |
| `tools/rocket-sdk/src/sbom.rs` | SOC2 CC9.1 SBOM generation library |
| `SOC2_SECURITY.md` | This document — wiring guide and CLI reference |
| `security-artifacts/` | Generated SBOM output (add to `.gitignore` or archive via CI) |
| `.secretsignore` | Project-level ignore list for the secret scanner |
