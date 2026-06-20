use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use lsp_types_max::{
    Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range,
};
use serde_json::Value;

pub fn find_asset_root(doc_path: &Path) -> Option<PathBuf> {
    for ancestor in doc_path.ancestors() {
        if let Some(parent) = ancestor.parent() {
            if parent.file_name() == Some(OsStr::new("mech_assets")) {
                if let Some(grandparent) = parent.parent() {
                    if grandparent.file_name() == Some(OsStr::new("generated")) {
                        return Some(ancestor.to_path_buf());
                    }
                }
            }
        }
    }
    None
}

fn collect_materials_from_mtlx(asset_root: &Path) -> HashSet<String> {
    let mut materials = HashSet::new();
    for entry in walkdir::WalkDir::new(asset_root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path().extension().and_then(|s| s.to_str()) == Some("mtlx") {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                for line in content.lines() {
                    if line.contains("<surfacematerial") || line.contains("<material") {
                        if let Some(name_idx) = line.find("name=\"") {
                            let start = name_idx + "name=\"".len();
                            if let Some(end) = line[start..].find('"') {
                                let mat_name = &line[start..start + end];
                                materials.insert(mat_name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    materials
}

fn get_receipted_prims(asset_root: &Path) -> HashSet<String> {
    let mut receipted = HashSet::new();

    // 1. Check <asset_root>/receipts/*.json (except receipt.json itself)
    let receipts_dir = asset_root.join("receipts");
    if receipts_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&receipts_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if stem != "receipt" {
                            receipted.insert(stem.to_string());
                        }
                    }
                }
            }
        }
    }

    // Helper function to search a serde_json::Value recursively for string matches
    fn search_value(value: &Value, set: &mut HashSet<String>) {
        match value {
            Value::String(s) => {
                set.insert(s.clone());
            }
            Value::Array(arr) => {
                for v in arr {
                    search_value(v, set);
                }
            }
            Value::Object(map) => {
                for (k, v) in map {
                    set.insert(k.clone());
                    search_value(v, set);
                }
            }
            _ => {}
        }
    }

    // 2. Check <asset_root>/receipt.json and <asset_root>/receipts/receipt.json
    let receipt_paths = vec![
        asset_root.join("receipt.json"),
        receipts_dir.join("receipt.json"),
    ];
    for path in receipt_paths {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(val) = serde_json::from_str::<Value>(&content) {
                    search_value(&val, &mut receipted);
                }
            }
        }
    }

    receipted
}

struct MeshPrim {
    name: String,
    start_line: usize,
    has_payload: bool,
    has_material_binding: bool,
    material_name: Option<String>,
    material_binding_line: Option<usize>,
    brace_level: usize,
    started: bool,
}

pub fn run_diagnostics(doc_path: &Path, content: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let asset_root = match find_asset_root(doc_path) {
        Some(root) => root,
        None => return diagnostics,
    };

    let file_name = doc_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let is_usda = doc_path.extension().and_then(|s| s.to_str()) == Some("usda");
    let is_part_file = is_usda && !file_name.starts_with("ASSET_");

    if is_part_file {
        let file_name_lower = file_name.to_lowercase();
        let is_head_file = file_name_lower.contains("head");
        let is_torso_file = file_name_lower.contains("torso");
        let is_blade_left_file = file_name_lower.contains("blade") && file_name_lower.contains("left");
        let is_blade_right_file = file_name_lower.contains("blade") && file_name_lower.contains("right");
        let is_wing_left_file = file_name_lower.contains("wing") && file_name_lower.contains("left");
        let is_wing_right_file = file_name_lower.contains("wing") && file_name_lower.contains("right");

        // Helper to extract vectors for translation/scale
        let extract_vectors = |txt: &str| -> Vec<(f64, f64, f64)> {
            let mut vectors = Vec::new();
            for line in txt.lines() {
                if line.contains("translate") || line.contains("scale") {
                    if let Some(start) = line.find('(') {
                        if let Some(end) = line[start..].find(')') {
                            let parts: Vec<&str> = line[start + 1..start + end].split(',').collect();
                            if parts.len() == 3 {
                                let x = parts[0].trim().parse::<f64>().unwrap_or(0.0);
                                let y = parts[1].trim().parse::<f64>().unwrap_or(0.0);
                                let z = parts[2].trim().parse::<f64>().unwrap_or(0.0);
                                vectors.push((x, y, z));
                            }
                        }
                    }
                }
            }
            vectors
        };

        // Helper to extract extents numbers
        let extract_extents_numbers = |txt: &str| -> Option<Vec<f64>> {
            for line in txt.lines() {
                if line.contains("extents") {
                    if let Some(eq_idx) = line.find('=') {
                        let val_part = &line[eq_idx + 1..];
                        let mut numbers = Vec::new();
                        let mut current_num = String::new();
                        for c in val_part.chars() {
                            if c.is_numeric() || c == '.' || c == '-' || c == '+' || c == 'e' || c == 'E' {
                                current_num.push(c);
                            } else {
                                if !current_num.is_empty() {
                                    if let Ok(val) = current_num.parse::<f64>() {
                                        numbers.push(val);
                                    }
                                    current_num.clear();
                                }
                            }
                        }
                        if !current_num.is_empty() {
                            if let Ok(val) = current_num.parse::<f64>() {
                                numbers.push(val);
                            }
                        }
                        if !numbers.is_empty() {
                            return Some(numbers);
                        }
                    }
                }
            }
            None
        };

        // Helper to categorize component prims
        let get_component_category = |prim_name: &str| -> Option<&'static str> {
            let lower = prim_name.to_lowercase();
            if lower.contains("head") {
                Some("head")
            } else if lower.contains("torso") {
                Some("torso")
            } else if lower.contains("blade") {
                if lower.contains("left") || lower.contains("_l") || lower.ends_with("l") {
                    Some("blade_left")
                } else if lower.contains("right") || lower.contains("_r") || lower.ends_with("r") {
                    Some("blade_right")
                } else {
                    Some("blade")
                }
            } else if lower.contains("wing") {
                if lower.contains("left") || lower.contains("_l") || lower.ends_with("l") {
                    Some("wing_left")
                } else if lower.contains("right") || lower.contains("_r") || lower.ends_with("r") {
                    Some("wing_right")
                } else {
                    Some("wing")
                }
            } else if lower.contains("arm") || lower.contains("hand") {
                Some("arm")
            } else if lower.contains("leg") {
                Some("leg")
            } else if lower.contains("shoulder") {
                Some("shoulder")
            } else if lower.contains("backpack") || lower.contains("thruster") {
                Some("backpack")
            } else if lower.contains("weapon") || lower.contains("loadout") {
                Some("weapon")
            } else {
                None
            }
        };

        let usd_dir = asset_root.join("usd");
        let mut has_duplicate_geom = false;
        let mut lacks_mirror_proof = false;
        let mut master_extents = None;

        if usd_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&usd_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_file() {
                        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                        if name.ends_with(".usda") {
                            if name.starts_with("ASSET_") {
                                // Extract master bounds/extents
                                if let Ok(master_content) = std::fs::read_to_string(&path) {
                                    if let Some(ext) = extract_extents_numbers(&master_content) {
                                        master_extents = Some(ext);
                                    }
                                }
                            } else if name != file_name {
                                // Part file
                                if let Ok(other_content) = std::fs::read_to_string(&path) {
                                    // USD301 & USD306: check for identical content
                                    if other_content.trim() == content.trim() {
                                        has_duplicate_geom = true;
                                    }

                                    // USD305: check mirrored variants
                                    let is_counterpart = (is_blade_left_file && name.to_lowercase().contains("blade") && name.to_lowercase().contains("right"))
                                        || (is_blade_right_file && name.to_lowercase().contains("blade") && name.to_lowercase().contains("left"))
                                        || (is_wing_left_file && name.to_lowercase().contains("wing") && name.to_lowercase().contains("right"))
                                        || (is_wing_right_file && name.to_lowercase().contains("wing") && name.to_lowercase().contains("left"));

                                    if is_counterpart {
                                        if other_content.trim() == content.trim() {
                                            lacks_mirror_proof = true;
                                        } else {
                                            let current_vecs = extract_vectors(content);
                                            let other_vecs = extract_vectors(&other_content);
                                            if !current_vecs.is_empty() && current_vecs.len() == other_vecs.len() {
                                                for (v1, v2) in current_vecs.iter().zip(other_vecs.iter()) {
                                                    // identical translations/scaling instead of sign inversion on the X axis
                                                    if (v1.0 - v2.0).abs() < 1e-5 && v1.0.abs() > 1e-5 {
                                                        lacks_mirror_proof = true;
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let first_line_len = lines.first().map(|l| l.len()).unwrap_or(0) as u32;

        // USD301
        if has_duplicate_geom {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: first_line_len },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("USD301".to_string())),
                source: Some("ggen-asset-lsp".to_string()),
                message: "USD301 ERROR: duplicate USD geometry fingerprint".to_string(),
                ..Default::default()
            });
        }

        // USD302
        let contains_world = content.contains("/World") || content.contains("\"World\"");
        let defines_all_components = {
            let has_head = content.contains("SM_Head") || content.contains("\"Head\"");
            let has_torso = content.contains("SM_Torso") || content.contains("\"Torso\"");
            let has_blade = content.contains("SM_Blade") || content.contains("\"Blade\"");
            let has_wing = content.contains("SM_Wing") || content.contains("\"Wing\"");
            has_head && has_torso && has_blade && has_wing
        };
        if contains_world || defines_all_components {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: first_line_len },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("USD302".to_string())),
                source: Some("ggen-asset-lsp".to_string()),
                message: "USD302 ERROR: part file renders full assembly".to_string(),
                ..Default::default()
            });
        }

        // USD308, USD312
        // Look for references = @... in part files
        for (line_idx, line) in lines.iter().enumerate() {
            if line.contains("references = @") {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: line_idx as u32, character: 0 },
                        end: Position { line: line_idx as u32, character: line.len() as u32 },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("USD308".to_string())),
                    source: Some("ggen-asset-lsp".to_string()),
                    message: "USD308 ERROR: part file contains assembly-level children".to_string(),
                    ..Default::default()
                });
                
                if line.contains("ASSET_") || line.contains("ASSET_ReferenceFabric_001") {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: line_idx as u32, character: 0 },
                            end: Position { line: line_idx as u32, character: line.len() as u32 },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("USD312".to_string())),
                        source: Some("ggen-asset-lsp".to_string()),
                        message: "USD312 ERROR: part file references assembly root".to_string(),
                        ..Default::default()
                    });
                }
            }
        }

        // USD309 & USD311
        // Parse lines to detect socket declarations and check if they are Mesh or contain nested Mesh.
        let mut in_socket = false;
        let mut socket_brace_level = 0;
        let mut brace_count = 0;
        for (line_idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Track brace level
            for c in trimmed.chars() {
                if c == '{' {
                    brace_count += 1;
                } else if c == '}' {
                    brace_count -= 1;
                    if in_socket && brace_count < socket_brace_level {
                        in_socket = false;
                    }
                }
            }

            if trimmed.contains("def ") && (trimmed.contains("socket") || trimmed.contains("Socket")) {
                if trimmed.contains("Mesh") {
                    // USD309: socket declared as Mesh
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: line_idx as u32, character: 0 },
                            end: Position { line: line_idx as u32, character: line.len() as u32 },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("USD309".to_string())),
                        source: Some("ggen-asset-lsp".to_string()),
                        message: "USD309 ERROR: socket emitted as attached geometry instead of mount declaration".to_string(),
                        ..Default::default()
                    });
                } else if trimmed.contains("Xform") {
                    in_socket = true;
                    socket_brace_level = brace_count;
                }
            }

            if in_socket && trimmed.contains("def Mesh") {
                // USD311: socket contains mesh payload
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: line_idx as u32, character: 0 },
                        end: Position { line: line_idx as u32, character: line.len() as u32 },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("USD311".to_string())),
                    source: Some("ggen-asset-lsp".to_string()),
                    message: "USD311 ERROR: socket prim contains mesh payload".to_string(),
                    ..Default::default()
                });
            }
        }

        // USD303 & USD310
        // Look for foreign component prims on matching prim declaration lines
        for (line_idx, line) in lines.iter().enumerate() {
            if let Some(def_idx) = line.find("def ") {
                let after_def = line[def_idx + 4..].trim();
                let prim_name = if after_def.starts_with('"') {
                    after_def[1..].find('"').map(|q2| after_def[1..1 + q2].to_string())
                } else {
                    let mut parts = after_def.split_whitespace();
                    if parts.next().is_some() {
                        after_def.find('"').and_then(|q1| {
                            after_def[q1 + 1..].find('"').map(|q2| after_def[q1 + 1..q1 + 1 + q2].to_string())
                        })
                    } else {
                        None
                    }
                };

                if let Some(name) = prim_name {
                    if let Some(cat) = get_component_category(&name) {
                        let is_foreign = match cat {
                            "head" => !is_head_file,
                            "torso" => !is_torso_file,
                            "blade" => !(is_blade_left_file || is_blade_right_file),
                            "blade_left" => !is_blade_left_file,
                            "blade_right" => !is_blade_right_file,
                            "wing" => !(is_wing_left_file || is_wing_right_file),
                            "wing_left" => !is_wing_left_file,
                            "wing_right" => !is_wing_right_file,
                            "arm" => true,
                            "leg" => true,
                            "shoulder" => true,
                            "backpack" => true,
                            "weapon" => true,
                            _ => false,
                        };
                        if is_foreign {
                            diagnostics.push(Diagnostic {
                                range: Range {
                                    start: Position { line: line_idx as u32, character: 0 },
                                    end: Position { line: line_idx as u32, character: line.len() as u32 },
                                },
                                severity: Some(DiagnosticSeverity::ERROR),
                                code: Some(NumberOrString::String("USD303".to_string())),
                                source: Some("ggen-asset-lsp".to_string()),
                                message: "USD303 ERROR: part-local file contains foreign component prims".to_string(),
                                ..Default::default()
                            });

                            diagnostics.push(Diagnostic {
                                range: Range {
                                    start: Position { line: line_idx as u32, character: 0 },
                                    end: Position { line: line_idx as u32, character: line.len() as u32 },
                                },
                                severity: Some(DiagnosticSeverity::ERROR),
                                code: Some(NumberOrString::String("USD310".to_string())),
                                source: Some("ggen-asset-lsp".to_string()),
                                message: "USD310 ERROR: part-scope query returned nonlocal rows".to_string(),
                                ..Default::default()
                            });
                        }
                    }
                }
            }
        }

        // USD304
        // Check if expected part root is missing
        let mut expected_roots = Vec::new();
        if is_head_file {
            expected_roots.push("SM_Head");
            expected_roots.push("Head");
        } else if is_torso_file {
            expected_roots.push("SM_Torso");
            expected_roots.push("Torso");
        } else if is_blade_left_file {
            expected_roots.push("SM_Blade_Left");
            expected_roots.push("Blade_Left");
            expected_roots.push("SM_Blade_L");
            expected_roots.push("Blade_L");
        } else if is_blade_right_file {
            expected_roots.push("SM_Blade_Right");
            expected_roots.push("Blade_Right");
            expected_roots.push("SM_Blade_R");
            expected_roots.push("Blade_R");
        } else if is_wing_left_file {
            expected_roots.push("SM_WingArray_Left");
            expected_roots.push("WingArray_Left");
            expected_roots.push("SM_Wing_Left");
            expected_roots.push("Wing_Left");
        } else if is_wing_right_file {
            expected_roots.push("SM_WingArray_Right");
            expected_roots.push("WingArray_Right");
            expected_roots.push("SM_Wing_Right");
            expected_roots.push("Wing_Right");
        } else {
            let stem = doc_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if !stem.is_empty() {
                expected_roots.push(stem);
            }
        }

        let mut has_expected_root = false;
        // Parse all prim names in the content to check if they match any expected root
        for line in lines.iter() {
            if let Some(def_idx) = line.find("def ") {
                let after_def = line[def_idx + 4..].trim();
                let prim_name = if after_def.starts_with('"') {
                    after_def[1..].find('"').map(|q2| after_def[1..1 + q2].to_string())
                } else {
                    let mut parts = after_def.split_whitespace();
                    if parts.next().is_some() {
                        after_def.find('"').and_then(|q1| {
                            after_def[q1 + 1..].find('"').map(|q2| after_def[q1 + 1..q1 + 1 + q2].to_string())
                        })
                    } else {
                        None
                    }
                };
                if let Some(name) = prim_name {
                    if expected_roots.iter().any(|&r| r == name) {
                        has_expected_root = true;
                        break;
                    }
                }
            }
        }

        if !has_expected_root && !expected_roots.is_empty() {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: first_line_len },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("USD304".to_string())),
                source: Some("ggen-asset-lsp".to_string()),
                message: "USD304 ERROR: expected part root missing".to_string(),
                ..Default::default()
            });
        }

        // USD305
        if lacks_mirror_proof {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: first_line_len },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("USD305".to_string())),
                source: Some("ggen-asset-lsp".to_string()),
                message: "USD305 ERROR: mirrored part lacks mirror transform proof".to_string(),
                ..Default::default()
            });
        }

        // USD306
        if has_duplicate_geom {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: first_line_len },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("USD306".to_string())),
                source: Some("ggen-asset-lsp".to_string()),
                message: "USD306 ERROR: generated USD files share identical source template expansion".to_string(),
                ..Default::default()
            });
        }

        // USD307
        if let Some(p_ext) = extract_extents_numbers(content) {
            for &val in p_ext.iter() {
                if val.abs() > 160.0 {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: 0, character: 0 },
                            end: Position { line: 0, character: first_line_len },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("USD307".to_string())),
                        source: Some("ggen-asset-lsp".to_string()),
                        message: "USD307 ERROR: part bounding box exceeds declared component envelope".to_string(),
                        ..Default::default()
                    });
                    break;
                }
            }
        }
        if let Some(m_ext) = master_extents {
            if let Some(p_ext) = extract_extents_numbers(content) {
                if m_ext.len() == p_ext.len() {
                    let matches = m_ext.iter().zip(p_ext.iter()).all(|(m, p)| (m - p).abs() < 1e-5);
                    if matches {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: 0, character: 0 },
                                end: Position { line: 0, character: first_line_len },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String("USD307".to_string())),
                            source: Some("ggen-asset-lsp".to_string()),
                            message: "USD307 ERROR: part bounding box overlaps full-asset bounds".to_string(),
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }

    let defined_materials = collect_materials_from_mtlx(&asset_root);
    let receipted_prims = get_receipted_prims(&asset_root);

    let mut current_mesh: Option<MeshPrim> = None;
    let mut prim_declarations = Vec::new(); // Vec<(Name, LineIndex, is_mesh_or_xform)>

    for (line_idx, line) in lines.iter().enumerate() {
        // Look for prim declaration
        if let Some(def_idx) = line.find("def ") {
            let after_def = line[def_idx + 4..].trim();
            if after_def.starts_with('"') {
                if let Some(q2) = after_def[1..].find('"') {
                    let name = &after_def[1..1 + q2];
                    prim_declarations.push((name.to_string(), line_idx, true));
                }
            } else {
                let mut parts = after_def.split_whitespace();
                if let Some(prim_type) = parts.next() {
                    let is_physical = prim_type == "Xform" || prim_type == "Mesh";
                    if let Some(q1) = after_def.find('"') {
                        if let Some(q2) = after_def[q1 + 1..].find('"') {
                            let name = &after_def[q1 + 1..q1 + 1 + q2];
                            prim_declarations.push((name.to_string(), line_idx, is_physical));
                        }
                    }
                }
            }
        }

        // Look for def Mesh
        if let Some(start_idx) = line.find("def Mesh \"") {
            let name_start = start_idx + "def Mesh \"".len();
            if let Some(end_idx) = line[name_start..].find('"') {
                let name = &line[name_start..name_start + end_idx];
                let has_payload = line.contains("payload = @");
                let mut brace_level = 0;
                let mut started = false;
                for c in line.chars() {
                    if c == '{' {
                        brace_level += 1;
                        started = true;
                    }
                }
                current_mesh = Some(MeshPrim {
                    name: name.to_string(),
                    start_line: line_idx,
                    has_payload,
                    has_material_binding: false,
                    material_name: None,
                    material_binding_line: None,
                    brace_level,
                    started,
                });
            }
        } else if let Some(ref mut mesh) = current_mesh {
            // Inside Mesh prim block
            if line.contains("payload = @") {
                mesh.has_payload = true;
            }
            if line.contains("rel material:binding = </") {
                mesh.has_material_binding = true;
                if let Some(idx) = line.find("rel material:binding = </") {
                    let start = idx + "rel material:binding = </".len();
                    if let Some(end) = line[start..].find('>') {
                        let path = &line[start..start + end];
                        let mat_name = path.split('/').last().unwrap_or("").trim();
                        mesh.material_name = Some(mat_name.to_string());
                        mesh.material_binding_line = Some(line_idx);
                    }
                }
            }

            // Count braces
            for c in line.chars() {
                if c == '{' {
                    mesh.brace_level += 1;
                    mesh.started = true;
                } else if c == '}' {
                    if mesh.brace_level > 0 {
                        mesh.brace_level -= 1;
                    }
                }
            }

            if mesh.started && mesh.brace_level == 0 {
                // Closed mesh block. Check diagnostics.
                if !mesh.has_payload {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: mesh.start_line as u32, character: 0 },
                            end: Position { line: mesh.start_line as u32, character: lines[mesh.start_line].len() as u32 },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("missing-payload".to_string())),
                        source: Some("ggen-asset-lsp".to_string()),
                        message: format!("Missing payload reference: Mesh prim '{}' must contain 'payload = @...'", mesh.name),
                        ..Default::default()
                    });
                }

                if !mesh.has_material_binding {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: mesh.start_line as u32, character: 0 },
                            end: Position { line: mesh.start_line as u32, character: lines[mesh.start_line].len() as u32 },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("missing-material-binding".to_string())),
                        source: Some("ggen-asset-lsp".to_string()),
                        message: format!("Missing material binding: Mesh prim '{}' does not contain a 'rel material:binding' statement", mesh.name),
                        ..Default::default()
                    });
                } else if let Some(ref mat_name) = mesh.material_name {
                    if !defined_materials.contains(mat_name) {
                        let bind_line = mesh.material_binding_line.unwrap_or(mesh.start_line);
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: bind_line as u32, character: 0 },
                                end: Position { line: bind_line as u32, character: lines[bind_line].len() as u32 },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String("invalid-material-binding".to_string())),
                            source: Some("ggen-asset-lsp".to_string()),
                            message: format!("Invalid material binding: Material '{}' is not defined in any .mtlx file", mat_name),
                            ..Default::default()
                        });
                    }
                }

                current_mesh = None;
            }
        }
    }

    // Check if the last mesh block wasn't closed but EOF reached
    if let Some(mesh) = current_mesh {
        if !mesh.has_payload {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position { line: mesh.start_line as u32, character: 0 },
                    end: Position { line: mesh.start_line as u32, character: lines[mesh.start_line].len() as u32 },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("missing-payload".to_string())),
                source: Some("ggen-asset-lsp".to_string()),
                message: format!("Missing payload reference: Mesh prim '{}' must contain 'payload = @...'", mesh.name),
                ..Default::default()
            });
        }
        if !mesh.has_material_binding {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position { line: mesh.start_line as u32, character: 0 },
                    end: Position { line: mesh.start_line as u32, character: lines[mesh.start_line].len() as u32 },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("missing-material-binding".to_string())),
                source: Some("ggen-asset-lsp".to_string()),
                message: format!("Missing material binding: Mesh prim '{}' does not contain a 'rel material:binding' statement", mesh.name),
                ..Default::default()
            });
        } else if let Some(ref mat_name) = mesh.material_name {
            if !defined_materials.contains(mat_name) {
                let bind_line = mesh.material_binding_line.unwrap_or(mesh.start_line);
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: bind_line as u32, character: 0 },
                        end: Position { line: bind_line as u32, character: lines[bind_line].len() as u32 },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("invalid-material-binding".to_string())),
                    source: Some("ggen-asset-lsp".to_string()),
                    message: format!("Invalid material binding: Material '{}' is not defined in any .mtlx file", mat_name),
                    ..Default::default()
                });
            }
        }
    }

    // 3. Unreceipted prims
    for (prim_name, line_idx, is_physical) in &prim_declarations {
        if *is_physical && !receipted_prims.contains(prim_name) {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position { line: *line_idx as u32, character: 0 },
                    end: Position { line: *line_idx as u32, character: lines[*line_idx].len() as u32 },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("unreceipted-prim".to_string())),
                source: Some("ggen-asset-lsp".to_string()),
                message: format!("Unreceipted USD prim: prim '{}' is not receipted", prim_name),
                ..Default::default()
            });
        }
    }

    // 4. Headless render outputs (visual_gap_report.json)
    let report_paths = vec![
        asset_root.join("visual_gap_report.json"),
        asset_root.join("reports").join("visual_gap_report.json"),
        asset_root.join("renders").join("visual_gap_report.json"),
    ];
    for path in report_paths {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(val) = serde_json::from_str::<Value>(&content) {
                    if let Some(obj) = val.as_object() {
                        let silhouette_iou = obj.get("silhouette_iou").and_then(|v| v.as_f64());
                        let status = obj.get("status").and_then(|v| v.as_str());

                        let failed_iou = silhouette_iou.map(|val| val < 0.90).unwrap_or(false);
                        let failed_status = status == Some("FAILED");

                        // Extract VIS morphology metrics
                        let part_graph_similarity = obj.get("part_graph_similarity").and_then(|v| v.as_f64());
                        let wing_layer_count_delta = obj.get("wing_layer_count_delta").and_then(|v| v.as_f64());
                        let feather_overlap_depth_score = obj.get("feather_overlap_depth_score").and_then(|v| v.as_f64());
                        let feather_panel_curvature_score = obj.get("feather_panel_curvature_score").and_then(|v| v.as_f64());
                        let wing_panels_are_line_primitives = obj.get("wing_panels_are_line_primitives").and_then(|v| v.as_bool());
                        let core_compactness_delta = obj.get("core_compactness_delta").and_then(|v| v.as_f64());
                        let head_to_torso_ratio_delta = obj.get("head_to_torso_ratio_delta").and_then(|v| v.as_f64());
                        let blade_length_angle_delta = obj.get("blade_length_angle_delta").and_then(|v| v.as_f64());
                        let armor_shell_segmentation_score = obj.get("armor_shell_segmentation_score").and_then(|v| v.as_f64());
                        let edge_density_distribution = obj.get("edge_density_distribution").and_then(|v| v.as_f64());

                        // Find the first def line
                        let mut first_def_line_idx = None;
                        for (idx, line) in lines.iter().enumerate() {
                            if line.trim().starts_with("def ") {
                                first_def_line_idx = Some(idx);
                                break;
                            }
                        }

                        if let Some(line_idx) = first_def_line_idx {
                            let mut push_diag = |code: &str, message: &str| {
                                diagnostics.push(Diagnostic {
                                    range: Range {
                                        start: Position { line: line_idx as u32, character: 0 },
                                        end: Position { line: line_idx as u32, character: lines[line_idx].len() as u32 },
                                    },
                                    severity: Some(DiagnosticSeverity::ERROR),
                                    code: Some(NumberOrString::String(code.to_string())),
                                    source: Some("ggen-asset-lsp".to_string()),
                                    message: message.to_string(),
                                    ..Default::default()
                                });
                            };

                            // Legacy visual gap failure
                            if failed_iou || failed_status {
                                push_diag(
                                    "visual-gap-failure",
                                    &format!(
                                        "Visual gap failure: silhouette_iou is {} (expected >= 0.90) or status is FAILED",
                                        silhouette_iou.map(|v| format!("{:.4}", v)).unwrap_or_else(|| "unknown".to_string())
                                    ),
                                );
                            }

                            // VIS201: part_graph_similarity < 0.90
                            if let Some(similarity) = part_graph_similarity {
                                if similarity < 0.90 {
                                    push_diag("VIS201", "VIS201 ERROR: part-graph similarity below threshold");
                                }
                            }

                            // VIS202: wing_layer_count_delta > 0 or feather_overlap_depth_score < 0.90 or feather_panel_curvature_score < 0.90
                            let wing_layer_delta_triggered = wing_layer_count_delta.map(|d| d > 0.0).unwrap_or(false);
                            let feather_overlap_triggered = feather_overlap_depth_score.map(|s| s < 0.90).unwrap_or(false);
                            let feather_curvature_triggered = feather_panel_curvature_score.map(|s| s < 0.90).unwrap_or(false);
                            if wing_layer_delta_triggered || feather_overlap_triggered || feather_curvature_triggered {
                                push_diag("VIS202", "VIS202 ERROR: wing morphology mismatch");
                            }

                            // VIS203: wing_panels_are_line_primitives is true or feather_panel_curvature_score < 0.50
                            let line_primitives_triggered = wing_panels_are_line_primitives.unwrap_or(false);
                            let feather_curvature_low_triggered = feather_panel_curvature_score.map(|s| s < 0.50).unwrap_or(false);
                            if line_primitives_triggered || feather_curvature_low_triggered {
                                push_diag("VIS203", "VIS203 ERROR: generated wing panels are line-primitives, expected layered swept plates");
                            }

                            // VIS204: core_compactness_delta > 0.10 or head_to_torso_ratio_delta > 0.10
                            let core_compactness_triggered = core_compactness_delta.map(|d| d > 0.10).unwrap_or(false);
                            let head_torso_ratio_triggered = head_to_torso_ratio_delta.map(|d| d > 0.10).unwrap_or(false);
                            if core_compactness_triggered || head_torso_ratio_triggered {
                                push_diag("VIS204", "VIS204 ERROR: core body massing exceeds compactness bound");
                            }

                            // VIS205: blade_length_angle_delta > 0.10
                            if let Some(delta) = blade_length_angle_delta {
                                if delta > 0.10 {
                                    push_diag("VIS205", "VIS205 ERROR: blade placement/angle mismatch");
                                }
                            }

                            // VIS206: armor_shell_segmentation_score < 0.90
                            if let Some(score) = armor_shell_segmentation_score {
                                if score < 0.90 {
                                    push_diag("VIS206", "VIS206 ERROR: armor segmentation density below threshold");
                                }
                            }

                            // VIS207: edge_density_distribution < 0.90
                            if let Some(density) = edge_density_distribution {
                                if density < 0.90 {
                                    push_diag("VIS207", "VIS207 ERROR: edge-density distribution mismatch");
                                }
                            }

                            // VIS208: coarse silhouette check passed (silhouette_iou >= 0.90) but overall status is "FAILED"
                            if let Some(iou) = silhouette_iou {
                                if iou >= 0.90 && status == Some("FAILED") {
                                    push_diag("VIS208", "VIS208 ERROR: candidate passed coarse silhouette but failed morphology gate");
                                }
                            }
                        }
                    }
                }
            }
            break; // found and processed the report
        }
    }

    // 5. usdchecker logs (reports/usdchecker.log)
    let checker_paths = vec![
        asset_root.join("reports").join("usdchecker.log"),
        asset_root.join("usdchecker.log"),
    ];
    for path in checker_paths {
        if path.exists() {
            if let Ok(log_content) = std::fs::read_to_string(&path) {
                for log_line in log_content.lines() {
                    let lower_line = log_line.to_lowercase();
                    let is_issue = lower_line.contains("fail")
                        || lower_line.contains("error")
                        || lower_line.contains("warn")
                        || lower_line.contains("invalid")
                        || lower_line.contains("missing");

                    if is_issue {
                        // Scan for prim paths starting with '/'
                        let mut idx = 0;
                        while let Some(slash_idx) = log_line[idx..].find('/') {
                            let absolute_slash_idx = idx + slash_idx;
                            let end_idx = log_line[absolute_slash_idx..]
                                .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '/');
                            let path_end = match end_idx {
                                Some(e) => absolute_slash_idx + e,
                                None => log_line.len(),
                            };
                            let path_str = &log_line[absolute_slash_idx..path_end];
                            if path_str.len() > 1 {
                                if let Some(leaf) = path_str.split('/').last() {
                                    if !leaf.is_empty() {
                                        // Project onto matching prim declarations in the USDA
                                        for (prim_name, decl_line_idx, _) in &prim_declarations {
                                            if prim_name == leaf {
                                                diagnostics.push(Diagnostic {
                                                    range: Range {
                                                        start: Position { line: *decl_line_idx as u32, character: 0 },
                                                        end: Position { line: *decl_line_idx as u32, character: lines[*decl_line_idx].len() as u32 },
                                                    },
                                                    severity: Some(DiagnosticSeverity::ERROR),
                                                    code: Some(NumberOrString::String("usdchecker-failure".to_string())),
                                                    source: Some("ggen-asset-lsp".to_string()),
                                                    message: format!("usdchecker failure: {}", log_line),
                                                    ..Default::default()
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                            idx = path_end + 1;
                            if idx >= log_line.len() {
                                break;
                            }
                        }
                    }
                }
            }
            break;
        }
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::env;

    #[test]
    fn test_diagnostics_pipeline() {
        let temp_dir = env::temp_dir().join("ggen_lsp_test_dir");
        let asset_root = temp_dir.join("generated/mech_assets/reference_fabric_001");
        let _ = fs::create_dir_all(&asset_root);

        // 1. Create a .mtlx file defining "M_WhiteArmor"
        let mtlx_dir = asset_root.join("materialx");
        let _ = fs::create_dir_all(&mtlx_dir);
        let mtlx_content = r#"<?xml version="1.0"?>
<materialx version="1.38">
  <surfacematerial name="M_WhiteArmor" type="material" />
</materialx>"#;
        fs::write(mtlx_dir.join("materials.mtlx"), mtlx_content).unwrap();

        // 2. Create receipts directory and a receipt file for "prim_0001"
        let receipts_dir = asset_root.join("receipts");
        let _ = fs::create_dir_all(&receipts_dir);
        fs::write(receipts_dir.join("prim_0001.json"), "{}").unwrap();

        // 3. Create a visual_gap_report.json showing failure
        let reports_dir = asset_root.join("reports");
        let _ = fs::create_dir_all(&reports_dir);
        let report_json = r#"{"silhouette_iou": 0.85, "status": "FAILED"}"#;
        fs::write(reports_dir.join("visual_gap_report.json"), report_json).unwrap();

        // 4. Create usdchecker.log showing failure for prim_0002
        let usdchecker_log = "Failed: /SM_Torso/prim_0002 has an issue\n";
        fs::write(reports_dir.join("usdchecker.log"), usdchecker_log).unwrap();

        // 5. Create the test USDA content
        // prim_0001: has material binding M_WhiteArmor (valid), is receipted (valid), but missing payload (invalid)
        // prim_0002: has material binding M_CyanBlade (invalid material), is unreceipted (invalid), but has payload (valid)
        let usda_content = r#"#usda 1.0
def Xform "SM_Torso"
{
    def Mesh "prim_0001"
    {
        rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_WhiteArmor>
    }

    def Mesh "prim_0002"
    {
        payload = @mesh.usd@
        rel material:binding = </ASSET_ReferenceFabric_001/Materials/M_CyanBlade>
    }
}"#;

        let usd_dir = asset_root.join("usd");
        let _ = fs::create_dir_all(&usd_dir);
        let doc_path = usd_dir.join("SM_Torso.usda");
        fs::write(&doc_path, usda_content).unwrap();

        // Run diagnostics
        let diags = run_diagnostics(&doc_path, usda_content);

        // Clean up first
        let _ = fs::remove_dir_all(&temp_dir);

        // Assertions
        let mut missing_payload = false;
        let mut invalid_mat_binding = false;
        let mut unreceipted_prim = false;
        let mut visual_gap = false;
        let mut usdchecker = false;

        for diag in &diags {
            if let Some(NumberOrString::String(ref code)) = diag.code {
                match code.as_str() {
                    "missing-payload" => {
                        assert_eq!(diag.range.start.line, 3);
                        missing_payload = true;
                    }
                    "invalid-material-binding" => {
                        assert_eq!(diag.range.start.line, 11);
                        invalid_mat_binding = true;
                    }
                    "unreceipted-prim" => {
                        if diag.range.start.line == 8 {
                            unreceipted_prim = true;
                        }
                    }
                    "visual-gap-failure" => {
                        assert_eq!(diag.range.start.line, 1); // first def line
                        visual_gap = true;
                    }
                    "usdchecker-failure" => {
                        assert_eq!(diag.range.start.line, 8); // prim_0002
                        usdchecker = true;
                    }
                    _ => {}
                }
            }
        }

        assert!(missing_payload, "Expected missing-payload diagnostic");
        assert!(invalid_mat_binding, "Expected invalid-material-binding diagnostic");
        assert!(unreceipted_prim, "Expected unreceipted-prim diagnostic");
        assert!(visual_gap, "Expected visual-gap-failure diagnostic");
        assert!(usdchecker, "Expected usdchecker-failure diagnostic");
    }

    #[test]
    fn test_vis200_morphology_diagnostics() {
        let temp_dir = env::temp_dir().join("ggen_lsp_vis200_test_dir");
        let asset_root = temp_dir.join("generated/mech_assets/reference_fabric_002");
        let _ = fs::create_dir_all(&asset_root);

        // Create the reports directory
        let reports_dir = asset_root.join("reports");
        let _ = fs::create_dir_all(&reports_dir);

        // 1. Create a visual_gap_report.json that triggers VIS201-VIS208
        let report_json = r#"{
            "status": "FAILED",
            "silhouette_iou": 0.95,
            "part_graph_similarity": 0.85,
            "wing_layer_count_delta": 1.0,
            "feather_overlap_depth_score": 0.85,
            "feather_panel_curvature_score": 0.45,
            "wing_panels_are_line_primitives": true,
            "core_compactness_delta": 0.15,
            "head_to_torso_ratio_delta": 0.15,
            "blade_length_angle_delta": 0.15,
            "armor_shell_segmentation_score": 0.85,
            "edge_density_distribution": 0.85
        }"#;
        fs::write(reports_dir.join("visual_gap_report.json"), report_json).unwrap();

        // 2. Create the test USDA content with a "def Xform" line to project onto
        let usda_content = r#"#usda 1.0
def Xform "SM_Torso"
{
}"#;

        let usd_dir = asset_root.join("usd");
        let _ = fs::create_dir_all(&usd_dir);
        let doc_path = usd_dir.join("SM_Torso.usda");
        fs::write(&doc_path, usda_content).unwrap();

        // Run diagnostics
        let diags = run_diagnostics(&doc_path, usda_content);

        // Clean up first
        let _ = fs::remove_dir_all(&temp_dir);

        // Assertions
        let mut vis201 = false;
        let mut vis202 = false;
        let mut vis203 = false;
        let mut vis204 = false;
        let mut vis205 = false;
        let mut vis206 = false;
        let mut vis207 = false;
        let mut vis208 = false;

        for diag in &diags {
            if let Some(NumberOrString::String(ref code)) = diag.code {
                match code.as_str() {
                    "VIS201" => {
                        assert_eq!(diag.range.start.line, 1);
                        assert_eq!(diag.message, "VIS201 ERROR: part-graph similarity below threshold");
                        vis201 = true;
                    }
                    "VIS202" => {
                        assert_eq!(diag.range.start.line, 1);
                        assert_eq!(diag.message, "VIS202 ERROR: wing morphology mismatch");
                        vis202 = true;
                    }
                    "VIS203" => {
                        assert_eq!(diag.range.start.line, 1);
                        assert_eq!(diag.message, "VIS203 ERROR: generated wing panels are line-primitives, expected layered swept plates");
                        vis203 = true;
                    }
                    "VIS204" => {
                        assert_eq!(diag.range.start.line, 1);
                        assert_eq!(diag.message, "VIS204 ERROR: core body massing exceeds compactness bound");
                        vis204 = true;
                    }
                    "VIS205" => {
                        assert_eq!(diag.range.start.line, 1);
                        assert_eq!(diag.message, "VIS205 ERROR: blade placement/angle mismatch");
                        vis205 = true;
                    }
                    "VIS206" => {
                        assert_eq!(diag.range.start.line, 1);
                        assert_eq!(diag.message, "VIS206 ERROR: armor segmentation density below threshold");
                        vis206 = true;
                    }
                    "VIS207" => {
                        assert_eq!(diag.range.start.line, 1);
                        assert_eq!(diag.message, "VIS207 ERROR: edge-density distribution mismatch");
                        vis207 = true;
                    }
                    "VIS208" => {
                        assert_eq!(diag.range.start.line, 1);
                        assert_eq!(diag.message, "VIS208 ERROR: candidate passed coarse silhouette but failed morphology gate");
                        vis208 = true;
                    }
                    _ => {}
                }
            }
        }

        assert!(vis201, "Expected VIS201 diagnostic");
        assert!(vis202, "Expected VIS202 diagnostic");
        assert!(vis203, "Expected VIS203 diagnostic");
        assert!(vis204, "Expected VIS204 diagnostic");
        assert!(vis205, "Expected VIS205 diagnostic");
        assert!(vis206, "Expected VIS206 diagnostic");
        assert!(vis207, "Expected VIS207 diagnostic");
        assert!(vis208, "Expected VIS208 diagnostic");
    }

    #[test]
    fn test_vis200_morphology_diagnostics_passing() {
        let temp_dir = env::temp_dir().join("ggen_lsp_vis200_pass_test_dir");
        let asset_root = temp_dir.join("generated/mech_assets/reference_fabric_003");
        let _ = fs::create_dir_all(&asset_root);

        // Create the reports directory
        let reports_dir = asset_root.join("reports");
        let _ = fs::create_dir_all(&reports_dir);

        // Create a visual_gap_report.json where everything passes
        let report_json = r#"{
            "status": "PASSED",
            "silhouette_iou": 0.95,
            "part_graph_similarity": 0.95,
            "wing_layer_count_delta": 0.0,
            "feather_overlap_depth_score": 0.95,
            "feather_panel_curvature_score": 0.95,
            "wing_panels_are_line_primitives": false,
            "core_compactness_delta": 0.05,
            "head_to_torso_ratio_delta": 0.05,
            "blade_length_angle_delta": 0.05,
            "armor_shell_segmentation_score": 0.95,
            "edge_density_distribution": 0.95
        }"#;
        fs::write(reports_dir.join("visual_gap_report.json"), report_json).unwrap();

        // Create the test USDA content with a "def Xform" line to project onto
        let usda_content = r#"#usda 1.0
def Xform "SM_Torso"
{
}"#;

        let usd_dir = asset_root.join("usd");
        let _ = fs::create_dir_all(&usd_dir);
        let doc_path = usd_dir.join("SM_Torso.usda");
        fs::write(&doc_path, usda_content).unwrap();

        // Run diagnostics
        let diags = run_diagnostics(&doc_path, usda_content);

        // Clean up first
        let _ = fs::remove_dir_all(&temp_dir);

        // Assertions: no VIS201-VIS208 diagnostics should be triggered
        for diag in &diags {
            if let Some(NumberOrString::String(ref code)) = diag.code {
                if code.starts_with("VIS2") || code == "visual-gap-failure" {
                    panic!("Unexpected diagnostic emitted: {} - {}", code, diag.message);
                }
            }
        }
    }

    #[test]
    fn test_usd300_series_modularity_diagnostics() {
        let temp_dir = env::temp_dir().join("ggen_lsp_usd300_test_dir");
        let asset_root = temp_dir.join("generated/mech_assets/reference_fabric_300");
        let usd_dir = asset_root.join("usd");
        let _ = fs::create_dir_all(&usd_dir);

        // We also create empty receipts and materialx dirs to avoid other check errors
        let _ = fs::create_dir_all(asset_root.join("receipts"));
        let _ = fs::create_dir_all(asset_root.join("materialx"));

        // ==========================================
        // Test USD301 & USD306: duplicate geometry fingerprint / template expansion
        // ==========================================
        let head_content = r#"#usda 1.0
def Xform "SM_Head"
{
    def Mesh "head_mesh"
    {
    }
}"#;
        let head_path = usd_dir.join("SM_Head.usda");
        fs::write(&head_path, head_content).unwrap();

        // Write torso with identical content
        let torso_path = usd_dir.join("SM_Torso.usda");
        fs::write(&torso_path, head_content).unwrap();

        let diags_head = run_diagnostics(&head_path, head_content);
        let has_usd301 = diags_head.iter().any(|d| d.code == Some(NumberOrString::String("USD301".to_string())));
        let has_usd306 = diags_head.iter().any(|d| d.code == Some(NumberOrString::String("USD306".to_string())));
        assert!(has_usd301, "Expected USD301 duplicate fingerprint diagnostic");
        assert!(has_usd306, "Expected USD306 duplicate template expansion diagnostic");

        // Now modify SM_Torso so it is not duplicate
        let torso_content = r#"#usda 1.0
def Xform "SM_Torso"
{
    def Mesh "torso_mesh"
    {
    }
}"#;
        fs::write(&torso_path, torso_content).unwrap();
        let diags_head_clean = run_diagnostics(&head_path, head_content);
        let has_usd301_clean = diags_head_clean.iter().any(|d| d.code == Some(NumberOrString::String("USD301".to_string())));
        assert!(!has_usd301_clean, "Did not expect USD301 on clean files");

        // ==========================================
        // Test USD302: part file renders full assembly
        // ==========================================
        // Case A: contains /World
        let full_assembly_a = r#"#usda 1.0
def Xform "World"
{
    def Xform "SM_Head"
    {}
}"#;
        let diags_usd302_a = run_diagnostics(&head_path, full_assembly_a);
        let has_usd302_a = diags_usd302_a.iter().any(|d| d.code == Some(NumberOrString::String("USD302".to_string())));
        assert!(has_usd302_a, "Expected USD302 for file containing World root");

        // Case B: defines all components
        let full_assembly_b = r#"#usda 1.0
def Xform "SM_Head" {}
def Xform "SM_Torso" {}
def Xform "SM_Blade" {}
def Xform "SM_Wing" {}"#;
        let diags_usd302_b = run_diagnostics(&head_path, full_assembly_b);
        let has_usd302_b = diags_usd302_b.iter().any(|d| d.code == Some(NumberOrString::String("USD302".to_string())));
        assert!(has_usd302_b, "Expected USD302 for file defining all components");

        // ==========================================
        // Test USD303: part-local file contains foreign component prims
        // ==========================================
        let head_with_foreign = r#"#usda 1.0
def Xform "SM_Head"
{
    def Xform "SM_Torso"
    {
    }
}"#;
        let diags_usd303 = run_diagnostics(&head_path, head_with_foreign);
        let usd303_diag = diags_usd303.iter().find(|d| d.code == Some(NumberOrString::String("USD303".to_string())));
        assert!(usd303_diag.is_some(), "Expected USD303 diagnostic for foreign prim");
        assert_eq!(usd303_diag.unwrap().range.start.line, 3); // line index 3 has SM_Torso (since def is on line 3, 0-indexed)

        // ==========================================
        // Test USD304: expected part root missing
        // ==========================================
        let head_missing_root = r#"#usda 1.0
def Xform "SM_Leg"
{
}"#;
        let diags_usd304 = run_diagnostics(&head_path, head_missing_root);
        let has_usd304 = diags_usd304.iter().any(|d| d.code == Some(NumberOrString::String("USD304".to_string())));
        assert!(has_usd304, "Expected USD304 for missing expected root");

        // ==========================================
        // Test USD305: mirrored part lacks mirror transform proof
        // ==========================================
        let blade_left_content = r#"#usda 1.0
def Xform "SM_Blade_Left"
{
    double3 xformOp:translate = (10.0, 5.0, 0.0)
}"#;
        let blade_left_path = usd_dir.join("SM_Blade_Left.usda");
        fs::write(&blade_left_path, blade_left_content).unwrap();

        // Same translate, so lacks mirror proof
        let blade_right_bad = r#"#usda 1.0
def Xform "SM_Blade_Right"
{
    double3 xformOp:translate = (10.0, 5.0, 0.0)
}"#;
        let blade_right_path = usd_dir.join("SM_Blade_Right.usda");
        fs::write(&blade_right_path, blade_right_bad).unwrap();

        let diags_usd305_bad = run_diagnostics(&blade_left_path, blade_left_content);
        let has_usd305_bad = diags_usd305_bad.iter().any(|d| d.code == Some(NumberOrString::String("USD305".to_string())));
        assert!(has_usd305_bad, "Expected USD305 when left and right have identical translations");

        // Inverted translate, so mirrors correctly
        let blade_right_good = r#"#usda 1.0
def Xform "SM_Blade_Right"
{
    double3 xformOp:translate = (-10.0, 5.0, 0.0)
}"#;
        fs::write(&blade_right_path, blade_right_good).unwrap();

        let diags_usd305_good = run_diagnostics(&blade_left_path, blade_left_content);
        let has_usd305_good = diags_usd305_good.iter().any(|d| d.code == Some(NumberOrString::String("USD305".to_string())));
        assert!(!has_usd305_good, "Did not expect USD305 when translations have inverted sign");

        // ==========================================
        // Test USD307: part bounding box overlaps full-asset bounds
        // ==========================================
        let master_content = r#"#usda 1.0
def Xform "ASSET_ReferenceFabric_300"
{
    float3[] extents = [(-5, -5, -5), (5, 5, 5)]
}"#;
        let master_path = usd_dir.join("ASSET_ReferenceFabric_300.usda");
        fs::write(&master_path, master_content).unwrap();

        // Part file with same bounds/extents
        let head_bad_bounds = r#"#usda 1.0
def Xform "SM_Head"
{
    float3[] extents = [(-5.0, -5.0, -5.0), (5.0, 5.0, 5.0)]
}"#;
        let diags_usd307_bad = run_diagnostics(&head_path, head_bad_bounds);
        let has_usd307_bad = diags_usd307_bad.iter().any(|d| d.code == Some(NumberOrString::String("USD307".to_string())));
        assert!(has_usd307_bad, "Expected USD307 when part extents match master extents");

        // Part file with different bounds
        let head_good_bounds = r#"#usda 1.0
def Xform "SM_Head"
{
    float3[] extents = [(-1.0, -1.0, -1.0), (1.0, 1.0, 1.0)]
}"#;
        let diags_usd307_good = run_diagnostics(&head_path, head_good_bounds);
        let has_usd307_good = diags_usd307_good.iter().any(|d| d.code == Some(NumberOrString::String("USD307".to_string())));
        assert!(!has_usd307_good, "Did not expect USD307 when part extents differ from master");

        // ==========================================
        // Test USD308: part file contains assembly-level children
        // ==========================================
        let head_with_assembly = r#"#usda 1.0
def Xform "SM_Head"
{
    references = @./SM_Torso.usda@
}"#;
        let diags_usd308 = run_diagnostics(&head_path, head_with_assembly);
        let has_usd308 = diags_usd308.iter().any(|d| d.code == Some(NumberOrString::String("USD308".to_string())));
        assert!(has_usd308, "Expected USD308 diagnostic for assembly references");

        // ==========================================
        // Test USD309: socket emitted as attached geometry
        // ==========================================
        let head_with_geom_socket = r#"#usda 1.0
def Xform "SM_Head"
{
    def Mesh "socket_weapon"
    {
    }
}"#;
        let diags_usd309 = run_diagnostics(&head_path, head_with_geom_socket);
        let has_usd309 = diags_usd309.iter().any(|d| d.code == Some(NumberOrString::String("USD309".to_string())));
        assert!(has_usd309, "Expected USD309 diagnostic for socket declared as Mesh");

        // ==========================================
        // Test USD311: socket contains mesh payload
        // ==========================================
        let head_with_payload_socket = r#"#usda 1.0
def Xform "SM_Head"
{
    def Xform "socket_weapon"
    {
        def Mesh "payload"
        {
        }
    }
}"#;
        let diags_usd311 = run_diagnostics(&head_path, head_with_payload_socket);
        let has_usd311 = diags_usd311.iter().any(|d| d.code == Some(NumberOrString::String("USD311".to_string())));
        assert!(has_usd311, "Expected USD311 diagnostic for socket with nested Mesh");

        // ==========================================
        // Test USD312: part file references assembly root
        // ==========================================
        let head_with_asset_ref = r#"#usda 1.0
def Xform "SM_Head"
{
    references = @./ASSET_ReferenceFabric_001.usda@
}"#;
        let diags_usd312 = run_diagnostics(&head_path, head_with_asset_ref);
        let has_usd312 = diags_usd312.iter().any(|d| d.code == Some(NumberOrString::String("USD312".to_string())));
        assert!(has_usd312, "Expected USD312 diagnostic for referencing ASSET root");

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
