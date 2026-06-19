//! T3D reverse parser — converts UE4 Blueprint clipboard text (T3D format) back
//! into [`BpNode`] instances and generates Rust [`BlueprintBuilder`] code.
//!
//! This closes the round-trip: Rust → T3D (via [`crate::serializer::T3dSerializer`]) → Rust.
//!
//! # CLI usage (bpgen decompile)
//!
//! To add a `bpgen decompile` sub-command to the CLI, add to `blueprint-cli/src/main.rs`:
//!
//! ```text
//! // In the Subcommand enum:
//! Decompile {
//!     /// Path to a file containing T3D clipboard text
//!     #[arg(short, long)]
//!     input: std::path::PathBuf,
//!     /// Blueprint name to use in generated code
//!     #[arg(short = 'n', long, default_value = "MyBlueprint")]
//!     name: String,
//!     /// Parent class to use in generated code
//!     #[arg(short = 'p', long, default_value = "Actor")]
//!     parent: String,
//! },
//!
//! // In the match arm:
//! Subcommand::Decompile { input, name, parent } => {
//!     let text = std::fs::read_to_string(&input)?;
//!     let nodes = blueprint_core::parser::parse_t3d(&text)
//!         .map_err(|e| anyhow::anyhow!("{}", e))?;
//!     let code = blueprint_core::parser::generate_rust_code(&nodes, &name, &parent);
//!     println!("{}", code);
//! }
//! ```

use crate::ast::{BpNode, Pin, PinRef};
use crate::types::{ContainerType, PinCategory, PinDirection, PinType, UeGuid};
use std::collections::HashMap;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// ParseError
// ---------------------------------------------------------------------------

/// Error type returned by [`parse_t3d`] and related functions.
#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error at line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Parse UE4 Blueprint T3D clipboard text into a list of [`BpNode`]s.
///
/// The returned nodes are not yet part of a [`crate::ast::Blueprint`]/graph —
/// the caller is responsible for inserting them into the desired graph.
///
/// # Errors
///
/// Returns [`ParseError`] if a `Begin Object` block is malformed (missing
/// `Class=` or `Name=`) or if a `CustomProperties Pin` line has no parentheses.
pub fn parse_t3d(text: &str) -> Result<Vec<BpNode>, ParseError> {
    let mut nodes = Vec::new();
    let mut lines = text.lines().enumerate().peekable();

    while let Some((line_num, line)) = lines.next() {
        let trimmed = line.trim();
        if trimmed.starts_with("Begin Object") {
            let node = parse_object_block(trimmed, &mut lines, line_num)?;
            nodes.push(node);
        }
        // Lines outside Begin/End Object blocks are silently ignored (comments, etc.)
    }

    Ok(nodes)
}

// ---------------------------------------------------------------------------
// Node-handler registry — replaces large match on short_class
// ---------------------------------------------------------------------------

type NodeHandler = fn(&BpNode, &str) -> Result<String, ParseError>;

static HANDLER_REGISTRY: OnceLock<HashMap<&'static str, NodeHandler>> = OnceLock::new();

fn handler_registry() -> &'static HashMap<&'static str, NodeHandler> {
    HANDLER_REGISTRY.get_or_init(|| {
        let mut m: HashMap<&'static str, NodeHandler> = HashMap::new();
        m.insert("K2Node_Event", handle_event);
        m.insert("K2Node_CustomEvent", handle_custom_event);
        m.insert("K2Node_CallFunction", handle_call_function);
        m.insert(
            "K2Node_CommutativeAssociativeBinaryOperator",
            handle_commutative_binary_operator,
        );
        m.insert("K2Node_IfThenElse", handle_if_then_else);
        m.insert("K2Node_VariableGet", handle_variable_get);
        m.insert("K2Node_VariableSet", handle_variable_set);
        m.insert("K2Node_MacroInstance", handle_macro_instance);
        m.insert("K2Node_Select", handle_select);
        m.insert("K2Node_ForEachLoop", handle_for_each_loop);
        m
    })
}

fn handle_event(node: &BpNode, var_name: &str) -> Result<String, ParseError> {
    if let Some(ev_ref) = node.properties.get("EventReference") {
        if ev_ref.contains("ReceiveBeginPlay") {
            Ok(format!("let {var_name} = builder.begin_play_node();\n"))
        } else if ev_ref.contains("ReceiveTick") {
            Ok(format!("let {var_name} = builder.tick_node();\n"))
        } else if ev_ref.contains("ReceiveEndPlay") {
            Ok(format!("let {var_name} = builder.end_play_node();\n"))
        } else if ev_ref.contains("ReceiveHit") {
            Ok(format!("let {var_name} = builder.on_hit_node();\n"))
        } else if ev_ref.contains("ReceiveActorBeginOverlap") {
            Ok(format!(
                "let {var_name} = builder.on_overlap_begin_node();\n"
            ))
        } else if ev_ref.contains("ReceiveActorEndOverlap") {
            Ok(format!("let {var_name} = builder.on_overlap_end_node();\n"))
        } else {
            Err(ParseError {
                line: 0,
                message: format!("Unsupported K2Node_Event: {}", ev_ref),
            })
        }
    } else {
        Ok(format!("let {var_name} = builder.begin_play_node();\n"))
    }
}

fn handle_custom_event(node: &BpNode, var_name: &str) -> Result<String, ParseError> {
    let ev_name = node
        .properties
        .get("CustomFunctionName")
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_else(|| "MyEvent".to_string());
    Ok(format!(
        "let {var_name} = builder.custom_event_node(\"{ev_name}\");\n"
    ))
}

fn handle_call_function(node: &BpNode, var_name: &str) -> Result<String, ParseError> {
    let fn_ref = node
        .properties
        .get("FunctionReference")
        .ok_or_else(|| ParseError {
            line: 0,
            message: format!(
                "K2Node_CallFunction '{}' has no FunctionReference property",
                node.name
            ),
        })?;
    if fn_ref.contains("PrintString") {
        let default_str = node
            .pins
            .iter()
            .find(|p| p.name == "InString")
            .and_then(|p| p.default_value.as_deref())
            .unwrap_or("Hello!");
        Ok(format!(
            "let {var_name} = builder.print_string(\"{default_str}\");\n"
        ))
    } else if fn_ref.contains("SetActorLocation") || fn_ref.contains("K2_SetActorLocation") {
        Ok(format!(
            "let {var_name} = builder.set_actor_location_node();\n"
        ))
    } else if fn_ref.contains("GetActorLocation") || fn_ref.contains("K2_GetActorLocation") {
        Ok(format!(
            "let {var_name} = builder.get_actor_location_node();\n"
        ))
    } else if fn_ref.contains("SetActorRotation") || fn_ref.contains("K2_SetActorRotation") {
        Ok(format!(
            "let {var_name} = builder.set_actor_rotation_node();\n"
        ))
    } else if fn_ref.contains("SpawnActor") || fn_ref.contains("BeginSpawningActorFromClass") {
        Ok(format!("let {var_name} = builder.spawn_actor_node();\n"))
    } else if fn_ref.contains("DestroyActor") || fn_ref.contains("K2_DestroyActor") {
        Ok(format!("let {var_name} = builder.destroy_actor_node();\n"))
    } else if fn_ref.contains("PlaySound") || fn_ref.contains("PlaySoundAtLocation") {
        Ok(format!("let {var_name} = builder.play_sound_node();\n"))
    } else if fn_ref.contains("ApplyDamage") {
        Ok(format!("let {var_name} = builder.apply_damage_node();\n"))
    } else if fn_ref.contains("Add_IntInt") {
        Ok(format!("let {var_name} = builder.add_int();\n"))
    } else if fn_ref.contains("Subtract_IntInt") {
        Ok(format!("let {var_name} = builder.subtract_int();\n"))
    } else {
        Err(ParseError {
            line: 0,
            message: format!(
                "Unsupported K2Node_CallFunction FunctionReference: {}",
                fn_ref
            ),
        })
    }
}

fn handle_commutative_binary_operator(node: &BpNode, var_name: &str) -> Result<String, ParseError> {
    let fn_ref = node.properties.get("FunctionReference")
        .ok_or_else(|| ParseError {
            line: 0,
            message: format!("K2Node_CommutativeAssociativeBinaryOperator '{}' has no FunctionReference property", node.name),
        })?;
    if fn_ref.contains("Add_IntInt") {
        Ok(format!("let {var_name} = builder.add_int();\n"))
    } else {
        Err(ParseError {
            line: 0,
            message: format!(
                "Unsupported K2Node_CommutativeAssociativeBinaryOperator FunctionReference: {}",
                fn_ref
            ),
        })
    }
}

fn handle_if_then_else(_node: &BpNode, var_name: &str) -> Result<String, ParseError> {
    Ok(format!("let {var_name} = builder.branch_node();\n"))
}

fn extract_var_name_prop(node: &BpNode) -> String {
    node.properties
        .get("VariableName")
        .or_else(|| node.properties.get("VariableReference"))
        .map(|s| {
            let s = s.trim();
            if let Some(pos) = s.find("MemberName=\"") {
                let after = &s[pos + 12..];
                after.split('"').next().unwrap_or(s).to_string()
            } else {
                s.trim_matches('"').to_string()
            }
        })
        .unwrap_or_else(|| node.name.clone())
}

fn handle_variable_get(node: &BpNode, var_name: &str) -> Result<String, ParseError> {
    let prop = extract_var_name_prop(node);
    Ok(format!(
        "let {var_name} = builder.variable_get_node(\"{prop}\");\n"
    ))
}

fn handle_variable_set(node: &BpNode, var_name: &str) -> Result<String, ParseError> {
    let prop = extract_var_name_prop(node);
    Ok(format!(
        "let {var_name} = builder.variable_set_node(\"{prop}\");\n"
    ))
}

fn handle_macro_instance(node: &BpNode, var_name: &str) -> Result<String, ParseError> {
    let macro_name = node
        .properties
        .get("MacroGraphReference")
        .and_then(|s| {
            let pos = s.find("MacroName=\"")?;
            let after = &s[pos + 11..];
            Some(after.split('"').next().unwrap_or("Macro").to_string())
        })
        .unwrap_or_else(|| node.name.clone());
    Ok(format!(
        "let {var_name} = builder.macro_instance_node(\"{macro_name}\");\n"
    ))
}

fn handle_select(_node: &BpNode, var_name: &str) -> Result<String, ParseError> {
    Ok(format!("let {var_name} = builder.select_node();\n"))
}

fn handle_for_each_loop(_node: &BpNode, var_name: &str) -> Result<String, ParseError> {
    Ok(format!("let {var_name} = builder.for_each_loop_node();\n"))
}

/// Generate Rust [`crate::builder::BlueprintBuilder`] code from a slice of parsed nodes.
///
/// This is the "decompiler" — it turns T3D text → Rust source code that, when
/// executed, reproduces an equivalent Blueprint graph.
///
/// The generated string is valid Rust code that can be embedded in a function
/// body.  All generated identifiers start with `builder.` as required by the
/// integration tests.
///
/// # Errors
///
/// Returns an error if a node class or event/function reference is not
/// recognised and cannot be mapped to a [`crate::builder::BlueprintBuilder`] call.
pub fn generate_rust_code(
    nodes: &[BpNode],
    bp_name: &str,
    parent: &str,
) -> Result<String, ParseError> {
    let mut out = String::new();
    out.push_str(&format!(
        "let mut builder = BlueprintBuilder::new(\"{}\", \"{}\");\n",
        bp_name, parent
    ));

    let mut node_vars: HashMap<&str, String> = HashMap::new();
    let registry = handler_registry();

    for (i, node) in nodes.iter().enumerate() {
        let var_name = format!("node_{i}");
        let short_class = node.class.split('.').next_back().unwrap_or(&node.class);

        let code = registry
            .get(short_class)
            .ok_or_else(|| ParseError {
                line: 0,
                message: format!("Unsupported node class: {}", short_class),
            })
            .and_then(|handler| handler(node, &var_name))?;

        out.push_str(&code);
        node_vars.insert(&node.name, var_name);
    }

    // Emit exec_connect calls for every exec output pin that has connections.
    out.push_str("\n// Connections:\n");
    for node in nodes {
        for pin in &node.pins {
            if pin.direction == PinDirection::Output && pin.pin_type.category == PinCategory::Exec {
                for link in &pin.linked_to {
                    if let (Some(from_var), Some(to_var)) = (
                        node_vars.get(node.name.as_str()),
                        node_vars.get(link.node_name.as_str()),
                    ) {
                        out.push_str(&format!("builder.exec_connect(&{from_var}, &{to_var});\n"));
                    }
                }
            }
        }
    }

    out.push_str("\nlet t3d = builder.to_t3d();\n");
    Ok(out)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Parse a single `Begin Object … End Object` block.
///
/// `header` is the trimmed `Begin Object Class=… Name="…"` line.
/// `lines` is the iterator positioned just after that line.
fn parse_object_block<'a, I>(
    header: &str,
    lines: &mut std::iter::Peekable<I>,
    start_line: usize,
) -> Result<BpNode, ParseError>
where
    I: Iterator<Item = (usize, &'a str)>,
{
    let class = extract_attr(header, "Class=").ok_or_else(|| ParseError {
        line: start_line,
        message: "Missing Class= in Begin Object".to_string(),
    })?;
    let name = extract_quoted_attr(header, "Name=").ok_or_else(|| ParseError {
        line: start_line,
        message: "Missing Name= in Begin Object".to_string(),
    })?;

    let mut node = BpNode::new(class, name);

    for (line_num, line) in lines.by_ref() {
        let trimmed = line.trim();

        if trimmed == "End Object" {
            break;
        }

        if trimmed.starts_with("CustomProperties Pin") {
            if let Some(pin) = parse_pin_line(trimmed, line_num)? {
                node.pins.push(pin);
            }
        } else if let Some(rest) = trimmed.strip_prefix("NodePosX=") {
            if let Ok(x) = rest.trim().parse::<i32>() {
                node.pos.x = x;
            }
        } else if let Some(rest) = trimmed.strip_prefix("NodePosY=") {
            if let Ok(y) = rest.trim().parse::<i32>() {
                node.pos.y = y;
            }
        } else if let Some(rest) = trimmed.strip_prefix("NodeGuid=") {
            node.id = UeGuid(rest.trim().to_string());
        } else if !trimmed.is_empty() && !trimmed.starts_with("//") {
            // Generic key=value property
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim().to_string();
                let value = trimmed[eq_pos + 1..].trim().to_string();
                node.properties.insert(key, value);
            }
        }
    }

    Ok(node)
}

/// Parse a `CustomProperties Pin (…)` line into a [`Pin`].
fn parse_pin_line(line: &str, line_num: usize) -> Result<Option<Pin>, ParseError> {
    let start = line.find('(').ok_or_else(|| ParseError {
        line: line_num,
        message: "Pin line missing '('".to_string(),
    })?;
    let end = line.rfind(')').ok_or_else(|| ParseError {
        line: line_num,
        message: "Pin line missing ')'".to_string(),
    })?;

    if end <= start {
        return Ok(None);
    }

    let content = &line[start + 1..end];
    let attrs = parse_pin_attrs(content);

    let pin_id = attrs
        .get("PinId")
        .map(|s| UeGuid(s.trim_matches('"').to_string()))
        .unwrap_or_default();

    let pin_name = attrs
        .get("PinName")
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_default();

    let display_name = attrs
        .get("PinFriendlyName")
        .map(|s| s.trim_matches('"').to_string());

    let direction = if attrs
        .get("Direction")
        .map(|d| d.contains("EGPD_Output"))
        .unwrap_or(false)
    {
        PinDirection::Output
    } else {
        PinDirection::Input
    };

    let category_str = attrs
        .get("PinType.PinCategory")
        .map(|s| s.trim_matches('"'))
        .unwrap_or("exec");
    let category = parse_pin_category(category_str);

    let sub_category_object = attrs.get("PinType.PinSubCategoryObject").and_then(|s| {
        let trimmed = s.trim();
        if trimmed == "None" {
            None
        } else {
            Some(trimmed.to_string())
        }
    });

    let container = match attrs.get("PinType.ContainerType").map(|s| s.trim()) {
        Some("Array") => ContainerType::Array,
        Some("Set") => ContainerType::Set,
        Some("Map") => ContainerType::Map,
        _ => ContainerType::None,
    };

    let is_reference = attrs
        .get("PinType.bIsReference")
        .map(|s| s == "True")
        .unwrap_or(false);
    let is_const = attrs
        .get("PinType.bIsConst")
        .map(|s| s == "True")
        .unwrap_or(false);

    let default_value = attrs
        .get("DefaultValue")
        .map(|s| s.trim_matches('"').to_string());

    let is_hidden = attrs.get("bHidden").map(|s| s == "True").unwrap_or(false);
    let is_not_connectable = attrs
        .get("bNotConnectable")
        .map(|s| s == "True")
        .unwrap_or(false);
    let is_advanced = attrs
        .get("bAdvancedView")
        .map(|s| s == "True")
        .unwrap_or(false);
    let is_orphaned = attrs
        .get("bOrphanedPin")
        .map(|s| s == "True")
        .unwrap_or(false);

    let linked_to = if let Some(lt) = attrs.get("LinkedTo") {
        parse_linked_to(lt)
    } else {
        Vec::new()
    };

    let pin_type = PinType {
        category,
        sub_category: None,
        sub_category_object,
        container,
        is_reference,
        is_const,
    };

    Ok(Some(Pin {
        id: pin_id,
        name: pin_name,
        display_name,
        direction,
        pin_type,
        default_value,
        linked_to,
        is_hidden,
        is_not_connectable,
        is_advanced_view: is_advanced,
        is_orphaned,
    }))
}

/// Map a UE4 pin-category string to [`PinCategory`].
fn parse_pin_category(s: &str) -> PinCategory {
    match s {
        "exec" => PinCategory::Exec,
        "bool" => PinCategory::Boolean,
        "byte" => PinCategory::Byte,
        "int" => PinCategory::Int,
        "int64" => PinCategory::Int64,
        "float" => PinCategory::Float,
        "double" => PinCategory::Double,
        "string" => PinCategory::String,
        "name" => PinCategory::Name,
        "text" => PinCategory::Text,
        "object" => PinCategory::Object,
        "class" => PinCategory::Class,
        "struct" => PinCategory::Struct,
        "delegate" => PinCategory::Delegate,
        "interface" => PinCategory::Interface,
        "softobject" => PinCategory::SoftObject,
        "softclass" => PinCategory::SoftClass,
        _ => PinCategory::Wildcard,
    }
}

/// Parse `LinkedTo=(NodeName GUID,NodeName2 GUID2,)` or `LinkedTo=(NodeName(GUID))` into a `Vec<PinRef>`.
fn parse_linked_to(s: &str) -> Vec<PinRef> {
    let mut refs = Vec::new();
    // Strip outer parentheses if present.
    let inner = s.trim();
    let inner = if inner.starts_with('(') && inner.ends_with(')') {
        &inner[1..inner.len() - 1]
    } else {
        inner
    };

    for part in inner.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        // Each part is "<NodeName> <PinGuid>" or "<NodeName>(<PinGuid>)"
        let mut iter = part.splitn(2, ' ');
        if let (Some(node_name), Some(guid)) = (iter.next(), iter.next()) {
            let node_name = node_name.trim().to_string();
            let guid = guid.trim().to_string();
            if !node_name.is_empty() && !guid.is_empty() {
                refs.push(PinRef {
                    node_name,
                    pin_id: UeGuid(guid),
                });
            }
        } else {
            if let Some(open_paren) = part.find('(') {
                if part.ends_with(')') {
                    let node_name = part[..open_paren].trim().to_string();
                    let guid = part[open_paren + 1..part.len() - 1].trim().to_string();
                    if !node_name.is_empty() && !guid.is_empty() {
                        refs.push(PinRef {
                            node_name,
                            pin_id: UeGuid(guid),
                        });
                    }
                }
            }
        }
    }
    refs
}

/// Parse the key=value pairs inside a `CustomProperties Pin (…)` string.
///
/// This tokeniser handles:
/// * Quoted strings (values may contain commas)
/// * Nested parentheses (e.g. `LinkedTo=(…)`, `PinType.PinValueType=()`)
/// * The trailing comma that UE4 appends before the closing `)`
fn parse_pin_attrs(content: &str) -> HashMap<String, String> {
    let mut attrs = HashMap::new();
    let mut depth = 0i32;
    let mut key = String::new();
    let mut value = String::new();
    let mut in_key = true;
    let mut in_quotes = false;

    for ch in content.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                if !in_key {
                    value.push(ch);
                }
            }
            '(' if !in_quotes => {
                depth += 1;
                if !in_key {
                    value.push(ch);
                }
            }
            ')' if !in_quotes => {
                depth -= 1;
                if !in_key {
                    value.push(ch);
                }
            }
            '=' if depth == 0 && !in_quotes && in_key => {
                in_key = false;
            }
            ',' if depth == 0 && !in_quotes => {
                let k = key.trim().to_string();
                let v = value.trim().to_string();
                if !k.is_empty() {
                    attrs.insert(k, v);
                }
                key = String::new();
                value = String::new();
                in_key = true;
            }
            _ => {
                if in_key {
                    key.push(ch);
                } else {
                    value.push(ch);
                }
            }
        }
    }

    // Flush the last pair (no trailing comma)
    let k = key.trim().to_string();
    let v = value.trim().to_string();
    if !k.is_empty() {
        attrs.insert(k, v);
    }

    attrs
}

/// Extract the value of a bare (unquoted) attribute like `Class=/Script/…`.
///
/// Stops at the first space after the key.
fn extract_attr(s: &str, key: &str) -> Option<String> {
    let pos = s.find(key)?;
    let after = &s[pos + key.len()..];
    // Values end at space (before the next attribute)
    let end = after.find(' ').unwrap_or(after.len());
    let raw = after[..end].trim_matches('"');
    if raw.is_empty() {
        None
    } else {
        Some(raw.to_string())
    }
}

/// Extract a double-quoted attribute value, e.g. `Name="K2Node_Event_0"`.
fn extract_quoted_attr(s: &str, key: &str) -> Option<String> {
    let pos = s.find(key)?;
    let after = &s[pos + key.len()..];
    let start = after.find('"')? + 1;
    let end = after[start..].find('"')?;
    Some(after[start..start + end].to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BpGraph, BpNode, Pin};
    use crate::types::{PinDirection, PinType};

    // Helper: serialize a single graph to T3D using the canonical t3d serializer
    // which produces the flat per-node format that parse_t3d expects.
    fn serialize_graph(graph: BpGraph) -> String {
        crate::serializer::T3dSerializer::serialize_graph(&graph)
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Minimal two-node T3D (event + print) that mirrors actual serialiser output.
    fn two_node_t3d() -> String {
        // Build a graph via the serialiser so the test data stays in sync.
        let mut graph = BpGraph::new("EventGraph");

        let event_node = BpNode::new("/Script/BlueprintGraph.K2Node_Event", "K2Node_Event_0")
            .at(64, -16)
            .with_property(
                "EventReference",
                "(MemberParent=Class'/Script/Engine.Actor',MemberName=\"ReceiveBeginPlay\")",
            )
            .with_property("bOverrideFunction", "True")
            .with_pin(Pin::exec_output("then"));

        let mut in_str = Pin::data_input("InString", PinType::string());
        in_str.default_value = Some("Hello!".to_string());
        let call_node = BpNode::new(
            "/Script/BlueprintGraph.K2Node_CallFunction",
            "K2Node_CallFunction_0",
        )
        .at(320, -16)
        .with_property(
            "FunctionReference",
            "(MemberParent=Class'/Script/Engine.KismetSystemLibrary',MemberName=\"PrintString\")",
        )
        .with_pin(Pin::exec_input("execute"))
        .with_pin(Pin::exec_output("then"))
        .with_pin(in_str);

        graph.add_node(event_node);
        graph.add_node(call_node);
        serialize_graph(graph)
    }

    // -----------------------------------------------------------------------
    // Parse correctness tests
    // -----------------------------------------------------------------------

    #[test]
    fn parse_empty_string_returns_empty_vec() {
        let nodes = parse_t3d("").expect("empty parse should succeed");
        assert!(nodes.is_empty(), "expected no nodes from empty input");
    }

    #[test]
    fn parse_comments_and_whitespace_only_returns_empty_vec() {
        let input = "// This is a comment\n\n   \n// Another comment";
        let nodes = parse_t3d(input).expect("comment-only parse should succeed");
        assert!(nodes.is_empty());
    }

    #[test]
    fn parse_two_node_t3d_yields_two_nodes() {
        let t3d = two_node_t3d();
        let nodes = parse_t3d(&t3d).expect("two-node parse should succeed");
        assert_eq!(nodes.len(), 2, "expected 2 nodes, got {}", nodes.len());
    }

    #[test]
    fn parse_two_node_t3d_correct_names() {
        let t3d = two_node_t3d();
        let nodes = parse_t3d(&t3d).unwrap();
        assert_eq!(nodes[0].name, "K2Node_Event_0");
        assert_eq!(nodes[1].name, "K2Node_CallFunction_0");
    }

    #[test]
    fn parse_two_node_t3d_correct_classes() {
        let t3d = two_node_t3d();
        let nodes = parse_t3d(&t3d).unwrap();
        assert_eq!(nodes[0].class, "/Script/BlueprintGraph.K2Node_Event");
        assert_eq!(nodes[1].class, "/Script/BlueprintGraph.K2Node_CallFunction");
    }

    #[test]
    fn parse_two_node_t3d_node_positions() {
        let t3d = two_node_t3d();
        let nodes = parse_t3d(&t3d).unwrap();
        assert_eq!(nodes[0].pos.x, 64);
        assert_eq!(nodes[0].pos.y, -16);
        assert_eq!(nodes[1].pos.x, 320);
        assert_eq!(nodes[1].pos.y, -16);
    }

    #[test]
    fn parse_two_node_t3d_event_properties() {
        let t3d = two_node_t3d();
        let nodes = parse_t3d(&t3d).unwrap();
        let ev = &nodes[0];
        let ev_ref = ev
            .properties
            .get("EventReference")
            .expect("EventReference property should be present");
        assert!(
            ev_ref.contains("ReceiveBeginPlay"),
            "EventReference should contain ReceiveBeginPlay"
        );
    }

    #[test]
    fn parse_two_node_t3d_function_reference_property() {
        let t3d = two_node_t3d();
        let nodes = parse_t3d(&t3d).unwrap();
        let call = &nodes[1];
        let fn_ref = call
            .properties
            .get("FunctionReference")
            .expect("FunctionReference should be present");
        assert!(
            fn_ref.contains("PrintString"),
            "FunctionReference should contain PrintString"
        );
    }

    #[test]
    fn parse_two_node_t3d_event_has_then_pin() {
        let t3d = two_node_t3d();
        let nodes = parse_t3d(&t3d).unwrap();
        let ev = &nodes[0];
        let then_pin = ev.find_pin("then").expect("'then' pin should be present");
        assert_eq!(then_pin.direction, PinDirection::Output);
        assert_eq!(then_pin.pin_type.category, PinCategory::Exec);
    }

    #[test]
    fn parse_two_node_t3d_call_has_execute_pin() {
        let t3d = two_node_t3d();
        let nodes = parse_t3d(&t3d).unwrap();
        let call = &nodes[1];
        let exec_pin = call
            .find_pin("execute")
            .expect("'execute' pin should be present");
        assert_eq!(exec_pin.direction, PinDirection::Input);
        assert_eq!(exec_pin.pin_type.category, PinCategory::Exec);
    }

    #[test]
    fn parse_two_node_t3d_call_has_default_value() {
        let t3d = two_node_t3d();
        let nodes = parse_t3d(&t3d).unwrap();
        let call = &nodes[1];
        let in_str = call
            .find_pin("InString")
            .expect("'InString' pin should be present");
        assert_eq!(
            in_str.default_value.as_deref(),
            Some("Hello!"),
            "InString default value should be 'Hello!'"
        );
    }

    // -----------------------------------------------------------------------
    // Exec connection (LinkedTo) tests
    // -----------------------------------------------------------------------

    #[test]
    fn parse_exec_connections_linked_to() {
        // Build a connected graph so we get real LinkedTo in the T3D.
        let mut graph = BpGraph::new("EventGraph");

        let event_node = BpNode::new("/Script/BlueprintGraph.K2Node_Event", "K2Node_Event_0")
            .at(0, 0)
            .with_pin(Pin::exec_output("then"));

        let call_node = BpNode::new(
            "/Script/BlueprintGraph.K2Node_CallFunction",
            "K2Node_CallFunction_0",
        )
        .at(250, 0)
        .with_pin(Pin::exec_input("execute"))
        .with_pin(Pin::exec_output("then"));

        graph.add_node(event_node);
        graph.add_node(call_node);
        graph.connect("K2Node_Event_0", "then", "K2Node_CallFunction_0", "execute");

        let t3d = serialize_graph(graph);
        let nodes = parse_t3d(&t3d).expect("parse should succeed");

        // The event node's "then" pin should link to the call function node.
        let ev = &nodes[0];
        let then_pin = ev.find_pin("then").expect("then pin should exist");
        assert!(
            !then_pin.linked_to.is_empty(),
            "then pin should have a LinkedTo entry"
        );
        assert_eq!(
            then_pin.linked_to[0].node_name, "K2Node_CallFunction_0",
            "LinkedTo should reference K2Node_CallFunction_0"
        );
    }

    #[test]
    fn parse_bidirectional_links() {
        let mut graph = BpGraph::new("EventGraph");

        let event_node = BpNode::new("/Script/BlueprintGraph.K2Node_Event", "K2Node_Event_0")
            .at(0, 0)
            .with_pin(Pin::exec_output("then"));

        let call_node = BpNode::new(
            "/Script/BlueprintGraph.K2Node_CallFunction",
            "K2Node_CallFunction_0",
        )
        .at(250, 0)
        .with_pin(Pin::exec_input("execute"));

        graph.add_node(event_node);
        graph.add_node(call_node);
        graph.connect("K2Node_Event_0", "then", "K2Node_CallFunction_0", "execute");

        let t3d = serialize_graph(graph);
        let nodes = parse_t3d(&t3d).expect("parse should succeed");

        // The call node's "execute" pin should also link back to the event node.
        let call = &nodes[1];
        let exec_pin = call.find_pin("execute").expect("execute pin should exist");
        assert!(
            !exec_pin.linked_to.is_empty(),
            "execute pin should have a LinkedTo entry"
        );
        assert_eq!(exec_pin.linked_to[0].node_name, "K2Node_Event_0");
    }

    // -----------------------------------------------------------------------
    // Single-node, no-pins test
    // -----------------------------------------------------------------------

    #[test]
    fn parse_single_node_with_no_pins() {
        let t3d = r#"Begin Object Class=/Script/BlueprintGraph.K2Node_Event Name="K2Node_Event_0"
   NodePosX=0
   NodePosY=0
   NodeGuid=AABBCCDD00112233AABBCCDD00112233
End Object
"#;
        let nodes = parse_t3d(t3d).expect("single node parse should succeed");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].name, "K2Node_Event_0");
        assert!(nodes[0].pins.is_empty(), "should have no pins");
        assert_eq!(nodes[0].pos.x, 0);
        assert_eq!(nodes[0].pos.y, 0);
    }

    // -----------------------------------------------------------------------
    // Round-trip test
    // -----------------------------------------------------------------------

    #[test]
    fn round_trip_serialize_parse_reserialize() {
        // Build original via the builder / serialiser.
        let original_t3d = two_node_t3d();

        // Parse it.
        let nodes = parse_t3d(&original_t3d).expect("first parse should succeed");
        assert_eq!(nodes.len(), 2, "should get 2 nodes back");

        // Re-serialize by inserting nodes into a fresh graph.
        let mut graph = BpGraph::new("EventGraph");
        for node in nodes {
            graph.add_node(node);
        }
        let round_tripped = serialize_graph(graph);

        // Both T3D texts should contain the same node names and classes.
        assert!(round_tripped.contains("K2Node_Event_0"));
        assert!(round_tripped.contains("K2Node_CallFunction_0"));
        assert!(round_tripped.contains("/Script/BlueprintGraph.K2Node_Event"));
        assert!(round_tripped.contains("/Script/BlueprintGraph.K2Node_CallFunction"));
        assert!(round_tripped.contains("ReceiveBeginPlay"));
        assert!(round_tripped.contains("PrintString"));
    }

    // -----------------------------------------------------------------------
    // Code-generator tests
    // -----------------------------------------------------------------------

    #[test]
    fn generate_rust_code_contains_builder_dot() {
        let t3d = two_node_t3d();
        let nodes = parse_t3d(&t3d).unwrap();
        let code = generate_rust_code(&nodes, "MyBlueprint", "Actor").unwrap();
        assert!(
            code.contains("builder."),
            "generated code should contain 'builder.'"
        );
    }

    #[test]
    fn generate_rust_code_has_begin_play_node() {
        let t3d = two_node_t3d();
        let nodes = parse_t3d(&t3d).unwrap();
        let code = generate_rust_code(&nodes, "MyBlueprint", "Actor").unwrap();
        assert!(
            code.contains("begin_play_node"),
            "should emit begin_play_node for K2Node_Event with ReceiveBeginPlay"
        );
    }

    #[test]
    fn generate_rust_code_has_print_string() {
        let t3d = two_node_t3d();
        let nodes = parse_t3d(&t3d).unwrap();
        let code = generate_rust_code(&nodes, "MyBlueprint", "Actor").unwrap();
        assert!(
            code.contains("print_string"),
            "should emit print_string for PrintString call-function node"
        );
    }

    #[test]
    fn generate_rust_code_has_to_t3d() {
        let t3d = two_node_t3d();
        let nodes = parse_t3d(&t3d).unwrap();
        let code = generate_rust_code(&nodes, "TestBP", "Pawn").unwrap();
        assert!(
            code.contains("to_t3d()"),
            "generated code should end with to_t3d()"
        );
    }

    #[test]
    fn generate_rust_code_empty_nodes() {
        let code = generate_rust_code(&[], "EmptyBP", "Actor").unwrap();
        assert!(code.contains("BlueprintBuilder::new"));
        assert!(code.contains("to_t3d()"));
    }

    #[test]
    fn generate_rust_code_math_nodes() {
        // Build a node vector containing Add_IntInt and Subtract_IntInt
        let mut graph = BpGraph::new("EventGraph");
        let add = crate::nodes::math::add_int("MyAdd");
        let sub = crate::nodes::math::subtract_int("MySub");
        graph.add_node(add);
        graph.add_node(sub);

        let t3d = serialize_graph(graph);
        let nodes = parse_t3d(&t3d).unwrap();
        let code = generate_rust_code(&nodes, "MathBP", "Actor").unwrap();

        assert!(
            code.contains("add_int()"),
            "should emit add_int() for Add_IntInt: {}",
            code
        );
        assert!(
            code.contains("subtract_int()"),
            "should emit subtract_int() for Subtract_IntInt: {}",
            code
        );
    }

    // -----------------------------------------------------------------------
    // Pin-attribute parser tests
    // -----------------------------------------------------------------------

    #[test]
    fn pin_attrs_basic_key_value() {
        let attrs = parse_pin_attrs(r#"PinId=AABB,PinName="execute""#);
        assert_eq!(attrs.get("PinId").map(|s| s.as_str()), Some("AABB"));
        assert_eq!(
            attrs.get("PinName").map(|s| s.as_str()),
            Some("\"execute\"")
        );
    }

    #[test]
    fn pin_attrs_quoted_value_with_comma() {
        // A value like DefaultValue="hello, world" should not split on the comma.
        let attrs = parse_pin_attrs(r#"PinId=AABB,DefaultValue="hello, world",bHidden=False"#);
        assert_eq!(
            attrs.get("DefaultValue").map(|s| s.as_str()),
            Some("\"hello, world\"")
        );
        assert_eq!(attrs.get("bHidden").map(|s| s.as_str()), Some("False"));
    }

    #[test]
    fn pin_attrs_nested_parens() {
        // LinkedTo=(NodeName GUID,) contains a comma inside parens — must not split.
        let attrs =
            parse_pin_attrs("PinId=AABB,LinkedTo=(K2Node_CallFunction_0 CCDD,),bHidden=False");
        let lt = attrs.get("LinkedTo").expect("LinkedTo should be present");
        assert!(lt.contains("K2Node_CallFunction_0"), "LinkedTo value: {lt}");
        assert_eq!(attrs.get("bHidden").map(|s| s.as_str()), Some("False"));
    }

    #[test]
    fn pin_attrs_trailing_comma_ignored() {
        // UE4 always adds a trailing comma before the closing `)`.
        let attrs = parse_pin_attrs("PinId=AABB,PinName=\"then\",");
        assert!(attrs.contains_key("PinId"));
        assert!(attrs.contains_key("PinName"));
        // The trailing comma produces an empty key which should not be inserted.
        assert!(
            !attrs.contains_key(""),
            "empty key from trailing comma should not be inserted"
        );
    }
}
