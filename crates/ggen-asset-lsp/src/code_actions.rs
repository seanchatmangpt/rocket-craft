use std::collections::HashMap;
use std::path::Path;
use lsp_types_max::{
    CodeAction, CodeActionKind, CodeActionOrCommand,
    CodeActionParams, Position, Range, TextEdit, Url,
    WorkspaceEdit,
};
use crate::diagnostics::find_asset_root;
use crate::ocel::log_event;

fn get_payload_fix_edit(template_path: &Path) -> Vec<TextEdit> {
    if let Ok(content) = std::fs::read_to_string(template_path) {
        let mut edit_line = None;
        for (idx, line) in content.lines().enumerate() {
            if line.contains("def Mesh") {
                edit_line = Some(idx + 1); // insert after this line
                break;
            }
        }
        if let Some(line) = edit_line {
            return vec![TextEdit {
                range: Range {
                    start: Position { line: line as u32, character: 0 },
                    end: Position { line: line as u32, character: 0 },
                },
                new_text: "            payload = @mesh.usd@\n".to_string(),
            }];
        }
    }
    // Fallback: prepend to file
    vec![TextEdit {
        range: Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: 0 },
        },
        new_text: "{# TODO: Fix payload reference #}\n".to_string(),
    }]
}

fn get_material_binding_edit(ttl_path: &Path, prim_name: &str) -> Vec<TextEdit> {
    if let Ok(content) = std::fs::read_to_string(ttl_path) {
        let target = format!("mud:{}", prim_name);
        if let Some(start_idx) = content.find(&target) {
            if let Some(dot_idx) = content[start_idx..].find(" .") {
                let absolute_dot_idx = start_idx + dot_idx;
                let block = &content[start_idx..absolute_dot_idx];
                if !block.contains("mud:materialBinding") {
                    let mut line = 0;
                    let mut character = 0;
                    for (idx, c) in content.char_indices() {
                        if idx == absolute_dot_idx {
                            break;
                        }
                        if c == '\n' {
                            line += 1;
                            character = 0;
                        } else {
                            character += 1;
                        }
                    }
                    return vec![TextEdit {
                        range: Range {
                            start: Position { line, character },
                            end: Position { line, character: character + 2 }, // replace " ."
                        },
                        new_text: ";\n    mud:materialBinding mud:M_WhiteArmor .".to_string(),
                    }];
                }
            }
        }
    }
    // Fallback
    vec![TextEdit {
        range: Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: 0 },
        },
        new_text: format!("# Missing material binding for {}\n", prim_name),
    }]
}

fn get_edit_source_param_edit(ttl_path: &Path, uri: &str) -> Vec<TextEdit> {
    if let Ok(content) = std::fs::read_to_string(ttl_path) {
        if let Some(idx) = content.find(uri) {
            let mut line = 0;
            for (c_idx, c) in content.char_indices() {
                if c_idx == idx {
                    break;
                }
                if c == '\n' {
                    line += 1;
                }
            }
            return vec![TextEdit {
                range: Range {
                    start: Position { line, character: 0 },
                    end: Position { line, character: 0 },
                },
                new_text: format!("# Source parameter edit requested for {}\n", uri),
            }];
        }
    }
    // Fallback
    vec![TextEdit {
        range: Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: 0 },
        },
        new_text: format!("# Source parameter edit requested for {}\n", uri),
    }]
}

pub fn handle_code_action(params: CodeActionParams) -> Option<Vec<CodeActionOrCommand>> {
    let uri = params.text_document.uri;
    let url = match url::Url::parse(uri.as_str()) {
        Ok(u) => u,
        Err(_) => return None,
    };
    let doc_path = match url.to_file_path() {
        Ok(path) => path,
        Err(_) => return None,
    };

    let asset_root = find_asset_root(&doc_path)?;

    // Read the document to examine the line
    let content = std::fs::read_to_string(&doc_path).ok()?;
    let lines: Vec<&str> = content.lines().collect();
    let line_idx = params.range.start.line as usize;
    if line_idx >= lines.len() {
        return None;
    }
    let current_line = lines[line_idx];

    let mut actions = Vec::new();

    // Check if the current line or file has a def Mesh
    let mut prim_name = None;
    if let Some(start_idx) = current_line.find("def Mesh \"") {
        let name_start = start_idx + "def Mesh \"".len();
        if let Some(end_idx) = current_line[name_start..].find('"') {
            prim_name = Some(&current_line[name_start..name_start + end_idx]);
        }
    }

    // 1. Fix payload reference code action
    if current_line.contains("def Mesh") {
        let part_mesh_template = asset_root.join("templates/usd/part_mesh.usda.tera");
        let asset_template = asset_root.join("templates/usd/asset.usda.tera");
        let template_path = if part_mesh_template.exists() {
            part_mesh_template
        } else {
            asset_template
        };

        let edits = get_payload_fix_edit(&template_path);
        let template_url = url::Url::from_file_path(&template_path).ok();
        let template_uri = template_url.and_then(|u| u.as_str().parse::<Url>().ok());
        if let Some(t_uri) = template_uri {
            let mut changes = HashMap::new();
            changes.insert(t_uri, edits);
            let workspace_edit = WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            };

            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Fix payload reference in template.usda.tera".to_string(),
                kind: Some(CodeActionKind::QUICKFIX),
                edit: Some(workspace_edit),
                ..Default::default()
            }));

            // Log Repair activity to OCEL
            log_event(
                &asset_root,
                "Repair",
                &template_path,
                vec![(
                    "repair_action".to_string(),
                    serde_json::json!("Fix payload reference in template.usda.tera"),
                )],
            );
        }
    }

    // 2. Add material binding code action
    if let Some(name) = prim_name {
        let ttl_path = asset_root.join("graph/generator_parameters.ttl");
        if ttl_path.exists() {
            let edits = get_material_binding_edit(&ttl_path, name);
            let ttl_url = url::Url::from_file_path(&ttl_path).ok();
            let ttl_uri = ttl_url.and_then(|u| u.as_str().parse::<Url>().ok());
            if let Some(t_uri) = ttl_uri {
                let mut changes = HashMap::new();
                changes.insert(t_uri, edits);
                let workspace_edit = WorkspaceEdit {
                    changes: Some(changes),
                    ..Default::default()
                };

                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: "Add material binding in generator_parameters.ttl".to_string(),
                    kind: Some(CodeActionKind::QUICKFIX),
                    edit: Some(workspace_edit),
                    ..Default::default()
                }));

                // Log Repair activity to OCEL
                log_event(
                    &asset_root,
                    "Repair",
                    &ttl_path,
                    vec![(
                        "repair_action".to_string(),
                        serde_json::json!("Add material binding in generator_parameters.ttl"),
                    )],
                );
            }
        }
    }

    // 3. Edit source parameter for <uri> code action
    // Scan the file for any # ggen-source: comment
    let mut source_uri = None;
    for line in &lines {
        if let Some(idx) = line.find("# ggen-source:") {
            let uri = line[idx + "# ggen-source:".len()..].trim();
            if !uri.is_empty() {
                source_uri = Some(uri.to_string());
                break;
            }
        }
    }

    if let Some(uri_str) = source_uri {
        let ttl_path = asset_root.join("graph/generator_parameters.ttl");
        if ttl_path.exists() {
            let edits = get_edit_source_param_edit(&ttl_path, &uri_str);
            let ttl_url = url::Url::from_file_path(&ttl_path).ok();
            let ttl_uri = ttl_url.and_then(|u| u.as_str().parse::<Url>().ok());
            if let Some(t_uri) = ttl_uri {
                let mut changes = HashMap::new();
                changes.insert(t_uri, edits);
                let workspace_edit = WorkspaceEdit {
                    changes: Some(changes),
                    ..Default::default()
                };

                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: format!("Edit source parameter in generator_parameters.ttl for {}", uri_str),
                    kind: Some(CodeActionKind::QUICKFIX),
                    edit: Some(workspace_edit),
                    ..Default::default()
                }));

                // Log Repair activity to OCEL
                log_event(
                    &asset_root,
                    "Repair",
                    &ttl_path,
                    vec![
                        (
                            "repair_action".to_string(),
                            serde_json::json!(format!(
                                "Edit source parameter in generator_parameters.ttl for {}",
                                uri_str
                            )),
                        ),
                        ("ggen_source_uri".to_string(), serde_json::json!(uri_str)),
                    ],
                );
            }
        }
    }

    if actions.is_empty() {
        None
    } else {
        Some(actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::env;
    use lsp_types_max::{TextDocumentIdentifier, CodeActionContext};

    #[test]
    fn test_code_actions() {
        let temp_dir = env::temp_dir().join("ggen_lsp_test_actions_dir");
        let asset_root = temp_dir.join("generated/mech_assets/reference_fabric_001");
        let _ = fs::create_dir_all(&asset_root);

        // Create empty templates and graph/generator_parameters.ttl so paths exist
        let templates_dir = asset_root.join("templates/usd");
        let _ = fs::create_dir_all(&templates_dir);
        fs::write(templates_dir.join("part_mesh.usda.tera"), "").unwrap();

        let graph_dir = asset_root.join("graph");
        let _ = fs::create_dir_all(&graph_dir);
        fs::write(graph_dir.join("generator_parameters.ttl"), "mud:prim_0001 .\n").unwrap();

        // Create USDA file
        let usda_content = r#"#usda 1.0
# ggen-source: https://example.com/source
def Mesh "prim_0001"
{
}
"#;
        let usd_dir = asset_root.join("usd");
        let _ = fs::create_dir_all(&usd_dir);
        let doc_path = usd_dir.join("SM_Torso.usda");
        fs::write(&doc_path, usda_content).unwrap();

        let doc_uri = url::Url::from_file_path(&doc_path).unwrap().as_str().parse::<Url>().unwrap();

        // Request code actions on line 2 (which is index 2, "def Mesh "prim_0001"")
        let params = CodeActionParams {
            text_document: TextDocumentIdentifier { uri: doc_uri },
            range: Range {
                start: Position { line: 2, character: 0 },
                end: Position { line: 2, character: 0 },
            },
            context: CodeActionContext::default(),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        let result = handle_code_action(params);

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);

        let actions = result.expect("Expected some code actions");
        assert!(!actions.is_empty(), "Actions list should not be empty");

        let mut has_payload_action = false;
        let mut has_material_action = false;
        let mut has_source_action = false;

        for action_or_cmd in actions {
            if let CodeActionOrCommand::CodeAction(action) = action_or_cmd {
                if action.title.contains("Fix payload reference") {
                    has_payload_action = true;
                } else if action.title.contains("Add material binding") {
                    has_material_action = true;
                } else if action.title.contains("Edit source parameter") {
                    has_source_action = true;
                    assert!(action.title.contains("https://example.com/source"));
                }
            }
        }

        assert!(has_payload_action, "Expected payload fix action");
        assert!(has_material_action, "Expected material binding action");
        assert!(has_source_action, "Expected source parameter edit action");
    }
}

