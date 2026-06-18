use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, BufRead};
use walkdir::WalkDir;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest as ShaDigest};
use blake3;
use colored::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CrateMeta {
    pub id: String,
    pub name: String,
    pub version: String,
    pub ontology_count: usize,
    pub query_count: usize,
    pub sha256_hash: String,
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
    let mut sha_hasher = Sha256::new();
    let mut blake_hasher = blake3::Hasher::new();
    let mut buffer = [0; 8192];

    loop {
        let count = io::Read::read(&mut file, &mut buffer)?;
        if count == 0 {
            break;
        }
        sha_hasher.update(&buffer[..count]);
        blake_hasher.update(&buffer[..count]);
    }

    let sha_hex = format!("{:x}", sha_hasher.finalize());
    let blake_hex = blake_hasher.finalize().to_hex().to_string();
    Ok((sha_hex, blake_hex))
}

pub fn run_copy(manifest_path: &str) -> anyhow::Result<()> {
    println!("{}", "====================================================".cyan().bold());
    println!("{}", "      Consolidate Ontology Catalogue (Rust)        ".cyan().bold());
    println!("{}", "====================================================".cyan().bold());

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
                eprintln!("[WARN] Failed to create directories for '{}': {}", dest_path.display(), e);
                errors += 1;
                continue;
            }
        }

        match fs::copy(src_path, &dest_path) {
            Ok(_) => {
                copied += 1;
            }
            Err(e) => {
                eprintln!("[WARN] Failed to copy '{}' to '{}': {}", src_path.display(), dest_path.display(), e);
                errors += 1;
            }
        }
    }

    println!("{}", "----------------------------------------------------".cyan().bold());
    println!("{} Catalogue consolidation complete.", "[SUCCESS]".green().bold());
    println!("Summary: Copied={}, Skipped={}, Errors={}", copied, skipped, errors);
    println!("{}", "====================================================".cyan().bold());

    Ok(())
}

pub fn run_index() -> anyhow::Result<()> {
    println!("{}", "====================================================".cyan().bold());
    println!("{}", "    Index O-Crates (Registry Builder - Rust)        ".cyan().bold());
    println!("{}", "====================================================".cyan().bold());

    let home = get_home_dir()?;
    let catalogue_root = home.join("ggen/ontology_catalogue");
    let marketplace_root = home.join("ggen/marketplace");
    let index_json = marketplace_root.join("registry/index.json");

    println!("Indexing O-Crates in catalogue: {}", catalogue_root.display());

    if !catalogue_root.exists() {
        anyhow::bail!("Catalogue root directory does not exist: {}", catalogue_root.display());
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

        let mut crate_sha_hasher = Sha256::new();
        let mut crate_blake_hasher = blake3::Hasher::new();

        for f_path in &all_files {
            match calculate_file_hashes(f_path) {
                Ok((sha_hex, blake_hex)) => {
                    crate_sha_hasher.update(sha_hex.as_bytes());
                    crate_blake_hasher.update(blake_hex.as_bytes());
                }
                Err(e) => {
                    eprintln!("[WARN] Failed to compute hash for '{}': {}", f_path.display(), e);
                }
            }
        }

        let sha256_hash_hex = format!("{:x}", crate_sha_hasher.finalize());
        let blake3_hash_hex = crate_blake_hasher.finalize().to_hex().to_string();

        let crate_meta = serde_json::json!({
            "id": crate_id,
            "name": crate_id,
            "version": "1.0.0",
            "ontologyCount": ttl_count,
            "queryCount": rq_count,
            "totalBytes": total_bytes,
            "sha256Hash": sha256_hash_hex,
            "blake3Hash": blake3_hash_hex,
            "category": "ontology-crate"
        });

        // Update registry: remove existing with same ID
        if let Some(pos) = registry.iter().position(|item| item.get("id").and_then(|v| v.as_str()) == Some(crate_id.as_str())) {
            registry.remove(pos);
        }
        registry.push(crate_meta);
        crate_count += 1;

        println!("{} Indexed Crate: {} (Ontologies: {}, Queries: {})", "[INFO]".blue().bold(), crate_id.bold(), ttl_count, rq_count);
    }

    let serialized = serde_json::to_string_pretty(&registry)?;
    fs::write(&index_json, serialized)?;

    println!("{}", "----------------------------------------------------".cyan().bold());
    println!("{} Indexed {} O-Crates successfully into registry.", "[SUCCESS]".green().bold(), crate_count);
    println!("Registry location: {}", index_json.display());
    println!("{}", "====================================================".cyan().bold());

    Ok(())
}
