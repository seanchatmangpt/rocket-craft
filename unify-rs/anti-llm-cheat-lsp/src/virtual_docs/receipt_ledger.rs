use serde_json::Value;
use std::fs;

pub fn generate_ledger_markdown(receipts_dir: &str) -> String {
    let mut out = String::new();
    out.push_str("# Receipt Ledger\n\n");
    out.push_str("| Receipt Path | Digest Algorithm | Digest | Boundary | Checkpoint | Raw Command | Status |\n");
    out.push_str("| --- | --- | --- | --- | --- | --- | --- |\n");

    if let Ok(entries) = fs::read_dir(receipts_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(val) = serde_json::from_str::<Value>(&content) {
                        let digest = val
                            .get("digest")
                            .and_then(|d| d.as_str())
                            .unwrap_or("unknown");
                        let alg = val
                            .get("digest_algorithm")
                            .and_then(|d| d.as_str())
                            .unwrap_or("unknown");
                        let boundary = val
                            .get("boundary")
                            .and_then(|d| d.as_str())
                            .unwrap_or("unknown");
                        let checkpoint = val
                            .get("checkpoint")
                            .and_then(|d| d.as_str())
                            .unwrap_or("unknown");
                        let cmd = val
                            .get("raw_command")
                            .and_then(|d| d.as_str())
                            .unwrap_or("unknown");
                        let status = val
                            .get("status")
                            .and_then(|d| d.as_str())
                            .unwrap_or("ADMITTED");

                        out.push_str(&format!(
                            "| {} | {} | {} | {} | {} | {} | {} |\n",
                            path.file_name().unwrap().to_string_lossy(),
                            alg,
                            digest,
                            boundary,
                            checkpoint,
                            cmd,
                            status
                        ));
                    }
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn write_receipt(dir: &std::path::Path, filename: &str, json: &str) {
        let mut f = fs::File::create(dir.join(filename)).unwrap();
        f.write_all(json.as_bytes()).unwrap();
    }

    #[test]
    fn empty_dir_returns_header_only() {
        let tmp = tempfile::tempdir().unwrap();
        let md = generate_ledger_markdown(tmp.path().to_str().unwrap());
        assert!(md.starts_with("# Receipt Ledger"));
        // table header row present
        assert!(md.contains("Digest Algorithm"));
        // no data rows
        let rows: Vec<_> = md.lines().filter(|l| l.starts_with("| ") && !l.contains("---") && !l.contains("Receipt Path")).collect();
        assert!(rows.is_empty());
    }

    #[test]
    fn nonexistent_dir_returns_header_only() {
        let md = generate_ledger_markdown("/tmp/nonexistent_ledger_test_xyz_abc");
        assert!(md.starts_with("# Receipt Ledger"));
    }

    #[test]
    fn valid_receipt_json_appears_in_table() {
        let tmp = tempfile::tempdir().unwrap();
        write_receipt(tmp.path(), "r1.json", r#"{
            "digest": "abc123", "digest_algorithm": "BLAKE3",
            "boundary": "stage1", "checkpoint": "cp1",
            "raw_command": "cargo test", "status": "PASS"
        }"#);
        let md = generate_ledger_markdown(tmp.path().to_str().unwrap());
        assert!(md.contains("abc123"));
        assert!(md.contains("BLAKE3"));
        assert!(md.contains("PASS"));
        assert!(md.contains("r1.json"));
    }

    #[test]
    fn missing_fields_fall_back_to_unknown() {
        let tmp = tempfile::tempdir().unwrap();
        write_receipt(tmp.path(), "r2.json", r#"{}"#);
        let md = generate_ledger_markdown(tmp.path().to_str().unwrap());
        // each missing field → "unknown"
        let unknowns: usize = md.matches("unknown").count();
        assert!(unknowns >= 5, "expected at least 5 'unknown' placeholders, got {unknowns}");
    }

    #[test]
    fn non_json_files_are_ignored() {
        let tmp = tempfile::tempdir().unwrap();
        let mut f = fs::File::create(tmp.path().join("notes.txt")).unwrap();
        f.write_all(b"not json").unwrap();
        let md = generate_ledger_markdown(tmp.path().to_str().unwrap());
        let rows: Vec<_> = md.lines().filter(|l| l.starts_with("| ") && !l.contains("---") && !l.contains("Receipt Path")).collect();
        assert!(rows.is_empty());
    }
}
