use std::collections::BTreeMap;
use std::fs;
use sha2::{Sha256, Digest};

fn calculate_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

fn main() {
    let file_path = "/Users/sac/rocket-craft/pwa-staff/test-results/tps-dflss-receipt.json";
    println!("Reading file: {}", file_path);
    let content = fs::read_to_string(file_path).expect("Failed to read JSON file");

    // 1. Parse using serde_json::Value (which preserves order because of the Cargo.toml feature)
    let mut value_ordered: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse as Value");
    let original_signature = value_ordered
        .as_object()
        .and_then(|obj| obj.get("signature"))
        .and_then(|sig| sig.as_str())
        .map(|s| s.to_string());

    println!("Original signature in file: {:?}", original_signature);

    let original_sig_str = match original_signature {
        Some(ref s) => s.as_str(),
        None => {
            println!("Error: 'signature' key not found in JSON.");
            return;
        }
    };

    // Remove signature from ordered value
    if let Some(obj) = value_ordered.as_object_mut() {
        obj.remove("signature");
    }

    let serialized_ordered = serde_json::to_string_pretty(&value_ordered).expect("Failed to serialize ordered");
    
    // Check ordered with and without newline
    let hash_ordered_no_nl = calculate_sha256(serialized_ordered.as_bytes());
    let hash_ordered_nl = calculate_sha256(format!("{}\n", serialized_ordered).as_bytes());
    let hash_ordered_crlf = calculate_sha256(format!("{}\r\n", serialized_ordered).as_bytes());

    println!("\n--- Case 1: Preserved Key Order (preserve_order) ---");
    println!("Hash (no newline):   {}", hash_ordered_no_nl);
    println!("Hash (with LF):      {}", hash_ordered_nl);
    println!("Hash (with CRLF):    {}", hash_ordered_crlf);
    
    if hash_ordered_no_nl == original_sig_str {
        println!("==> MATCH FOUND: Preserved Key Order, NO trailing newline!");
    } else if hash_ordered_nl == original_sig_str {
        println!("==> MATCH FOUND: Preserved Key Order, WITH LF trailing newline!");
    } else if hash_ordered_crlf == original_sig_str {
        println!("==> MATCH FOUND: Preserved Key Order, WITH CRLF trailing newline!");
    } else {
        println!("==> NO MATCH for Preserved Key Order.");
    }

    // 2. Parse using BTreeMap to force alphabetical sorting of keys
    let mut value_sorted: BTreeMap<String, serde_json::Value> = serde_json::from_str(&content).expect("Failed to parse as BTreeMap");
    value_sorted.remove("signature");

    let serialized_sorted = serde_json::to_string_pretty(&value_sorted).expect("Failed to serialize sorted");

    // Check sorted with and without newline
    let hash_sorted_no_nl = calculate_sha256(serialized_sorted.as_bytes());
    let hash_sorted_nl = calculate_sha256(format!("{}\n", serialized_sorted).as_bytes());
    let hash_sorted_crlf = calculate_sha256(format!("{}\r\n", serialized_sorted).as_bytes());

    println!("\n--- Case 2: Sorted Key Order (alphabetical / no preserve_order) ---");
    println!("Hash (no newline):   {}", hash_sorted_no_nl);
    println!("Hash (with LF):      {}", hash_sorted_nl);
    println!("Hash (with CRLF):    {}", hash_sorted_crlf);

    if hash_sorted_no_nl == original_sig_str {
        println!("==> MATCH FOUND: Alphabetical Key Order, NO trailing newline!");
    } else if hash_sorted_nl == original_sig_str {
        println!("==> MATCH FOUND: Alphabetical Key Order, WITH LF trailing newline!");
    } else if hash_sorted_crlf == original_sig_str {
        println!("==> MATCH FOUND: Alphabetical Key Order, WITH CRLF trailing newline!");
    } else {
        println!("==> NO MATCH for Alphabetical Key Order.");
    }
}
