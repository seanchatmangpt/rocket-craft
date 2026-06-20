use blake3;
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CrateMeta {
    pub id: String,
    pub name: String,
    pub version: String,
    pub ontology_count: usize,
    pub query_count: usize,
    pub blake3_hash: String,
    pub category: String,
}

fn get_home_dir() -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| anyhow::anyhow!("Could not find HOME or USERPROFILE environment variables"))?;
    Ok(PathBuf::from(home))
}

fn calculate_file_hashes(path: &Path) -> anyhow::Result<(String, String)> {
    let mut file = fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0; 8192];

    loop {
        let count = io::Read::read(&mut file, &mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    let blake_hex = hasher.finalize().to_hex().to_string();
    Ok((blake_hex.clone(), blake_hex))
}

pub fn run_copy(manifest_path: &str) -> anyhow::Result<()> {
    println!(
        "{}",
        "===================================================="
            .cyan()
            .bold()
    );
    println!(
        "{}",
        "      Consolidate Ontology Catalogue (Rust)        "
            .cyan()
            .bold()
    );
    println!(
        "{}",
        "===================================================="
            .cyan()
            .bold()
    );

    let home = get_home_dir()?;
    let target_base = home.join("ggen/ontology_catalogue");
    println!("Target catalogue base: {}", target_base.display());

    let manifest = Path::new(manifest_path);
    if !manifest.exists() {
        anyhow::bail!("Manifest file '{}' not found.", manifest_path);
    }

    let file = fs::File::open(manifest)?;
    let reader = io::BufReader::new(file);

    let mut copied = 0;
    let mut skipped = 0;
    let mut errors = 0;

    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let src_path = Path::new(trimmed);
        if !src_path.exists() {
            skipped += 1;
            continue;
        }

        let rel_path = if src_path.starts_with(&home) {
            src_path.strip_prefix(&home)?
        } else if src_path.starts_with("/") {
            src_path.strip_prefix("/")?
        } else {
            src_path
        };

        let dest_path = target_base.join(rel_path);

        if let Some(parent) = dest_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!(
                    "[WARN] Failed to create directories for '{}': {}",
                    dest_path.display(),
                    e
                );
                errors += 1;
                continue;
            }
        }

        match fs::copy(src_path, &dest_path) {
            Ok(_) => {
                copied += 1;
            }
            Err(e) => {
                eprintln!(
                    "[WARN] Failed to copy '{}' to '{}': {}",
                    src_path.display(),
                    dest_path.display(),
                    e
                );
                errors += 1;
            }
        }
    }

    println!(
        "{}",
        "----------------------------------------------------"
            .cyan()
            .bold()
    );
    println!(
        "{} Catalogue consolidation complete.",
        "[SUCCESS]".green().bold()
    );
    println!(
        "Summary: Copied={}, Skipped={}, Errors={}",
        copied, skipped, errors
    );
    println!(
        "{}",
        "===================================================="
            .cyan()
            .bold()
    );

    Ok(())
}

pub fn run_index() -> anyhow::Result<()> {
    println!(
        "{}",
        "===================================================="
            .cyan()
            .bold()
    );
    println!(
        "{}",
        "    Index O-Crates (Registry Builder - Rust)        "
            .cyan()
            .bold()
    );
    println!(
        "{}",
        "===================================================="
            .cyan()
            .bold()
    );

    let home = get_home_dir()?;
    let catalogue_root = home.join("ggen/ontology_catalogue");
    let marketplace_root = home.join("ggen/marketplace");
    let index_json = marketplace_root.join("registry/index.json");

    println!(
        "Indexing O-Crates in catalogue: {}",
        catalogue_root.display()
    );

    if !catalogue_root.exists() {
        anyhow::bail!(
            "Catalogue root directory does not exist: {}",
            catalogue_root.display()
        );
    }

    let registry_dir = marketplace_root.join("registry");
    fs::create_dir_all(&registry_dir)?;

    let mut registry: Vec<serde_json::Value> = if index_json.exists() {
        match fs::read_to_string(&index_json) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(e) => {
                eprintln!("[WARN] Failed to read existing index.json: {}", e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    let mut crate_count = 0;

    // Traverse catalogue root for direct child directories
    for entry_result in fs::read_dir(&catalogue_root)? {
        let entry = entry_result?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let crate_id = match path.file_name().and_then(|s| s.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let mut ttl_count = 0;
        let mut rq_count = 0;
        let mut total_bytes = 0;
        let mut all_files = Vec::new();

        for file_entry in WalkDir::new(&path) {
            let file_entry = match file_entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            if file_entry.file_type().is_file() {
                let file_path = file_entry.path();
                let file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("");

                let mut matches = false;
                if file_name.ends_with(".ttl") {
                    ttl_count += 1;
                    matches = true;
                }
                if file_name.ends_with(".rq") {
                    rq_count += 1;
                    matches = true;
                }

                if matches {
                    if let Ok(metadata) = file_path.metadata() {
                        total_bytes += metadata.len();
                    }
                    all_files.push(file_path.to_path_buf());
                }
            }
        }

        // Sort paths deterministically to match python behaviour
        all_files.sort();

        let mut crate_blake_hasher = blake3::Hasher::new();

        for f_path in &all_files {
            match calculate_file_hashes(f_path) {
                Ok((blake_hex, _)) => {
                    crate_blake_hasher.update(blake_hex.as_bytes());
                }
                Err(e) => {
                    eprintln!(
                        "[WARN] Failed to compute hash for '{}': {}",
                        f_path.display(),
                        e
                    );
                }
            }
        }

        let blake3_hash_hex = crate_blake_hasher.finalize().to_hex().to_string();

        let crate_meta = serde_json::json!({
            "id": crate_id,
            "name": crate_id,
            "version": "1.0.0",
            "ontologyCount": ttl_count,
            "queryCount": rq_count,
            "totalBytes": total_bytes,
            "blake3Hash": blake3_hash_hex,
            "category": "ontology-crate"
        });

        // Update registry: remove existing with same ID
        if let Some(pos) = registry
            .iter()
            .position(|item| item.get("id").and_then(|v| v.as_str()) == Some(crate_id.as_str()))
        {
            registry.remove(pos);
        }
        registry.push(crate_meta);
        crate_count += 1;

        println!(
            "{} Indexed Crate: {} (Ontologies: {}, Queries: {})",
            "[INFO]".blue().bold(),
            crate_id.bold(),
            ttl_count,
            rq_count
        );
    }

    let serialized = serde_json::to_string_pretty(&registry)?;
    fs::write(&index_json, serialized)?;

    println!(
        "{}",
        "----------------------------------------------------"
            .cyan()
            .bold()
    );
    println!(
        "{} Indexed {} O-Crates successfully into registry.",
        "[SUCCESS]".green().bold(),
        crate_count
    );
    println!("Registry location: {}", index_json.display());
    println!(
        "{}",
        "===================================================="
            .cyan()
            .bold()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    // ── CrateMeta serde ───────────────────────────────────────────────────────

    #[test]
    fn crate_meta_serializes_to_camel_case_json() {
        let meta = CrateMeta {
            id: "my-crate-1.0.0".into(),
            name: "my-crate".into(),
            version: "1.0.0".into(),
            ontology_count: 3,
            query_count: 7,
            blake3_hash: "abc123".into(),
            category: "rdf".into(),
        };
        let json = serde_json::to_string(&meta).unwrap();
        assert!(json.contains("\"ontologyCount\""));
        assert!(json.contains("\"queryCount\""));
        assert!(json.contains("\"blake3Hash\""));
        assert!(!json.contains("ontology_count")); // snake_case must not appear
    }

    #[test]
    fn crate_meta_deserializes_from_camel_case_json() {
        let json = r#"{"id":"x","name":"x","version":"0.1.0","ontologyCount":2,"queryCount":5,"blake3Hash":"ff","category":"gen"}"#;
        let meta: CrateMeta = serde_json::from_str(json).unwrap();
        assert_eq!(meta.ontology_count, 2);
        assert_eq!(meta.query_count, 5);
        assert_eq!(meta.blake3_hash, "ff");
    }

    #[test]
    fn crate_meta_clone_is_independent() {
        let a = CrateMeta {
            id: "a".into(), name: "a".into(), version: "1".into(),
            ontology_count: 0, query_count: 0, blake3_hash: "h".into(), category: "c".into(),
        };
        let mut b = a.clone();
        b.name = "b".into();
        assert_eq!(a.name, "a");
    }

    // ── calculate_file_hashes ─────────────────────────────────────────────────

    #[test]
    fn calculate_file_hashes_returns_64_hex_chars() {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(b"hello rocket").unwrap();
        let (h1, h2) = calculate_file_hashes(tmp.path()).unwrap();
        assert_eq!(h1.len(), 64);
        assert_eq!(h1, h2); // both outputs are the same hash
        assert!(h1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn calculate_file_hashes_same_content_same_hash() {
        let mut a = NamedTempFile::new().unwrap();
        let mut b = NamedTempFile::new().unwrap();
        a.write_all(b"data").unwrap();
        b.write_all(b"data").unwrap();
        let (ha, _) = calculate_file_hashes(a.path()).unwrap();
        let (hb, _) = calculate_file_hashes(b.path()).unwrap();
        assert_eq!(ha, hb);
    }

    #[test]
    fn calculate_file_hashes_different_content_different_hash() {
        let mut a = NamedTempFile::new().unwrap();
        let mut b = NamedTempFile::new().unwrap();
        a.write_all(b"foo").unwrap();
        b.write_all(b"bar").unwrap();
        let (ha, _) = calculate_file_hashes(a.path()).unwrap();
        let (hb, _) = calculate_file_hashes(b.path()).unwrap();
        assert_ne!(ha, hb);
    }

    #[test]
    fn calculate_file_hashes_missing_file_returns_err() {
        assert!(calculate_file_hashes(std::path::Path::new("/nonexistent/file.ttl")).is_err());
    }
}
