use crate::ast::*;
use crate::types::*;
use std::collections::{HashMap, HashSet};

/// A validation error found in a Blueprint graph
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub kind: ErrorKind,
    pub node_name: String,
    pub pin_name: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    /// Output pin connected to wrong-type input pin
    TypeMismatch { from_type: String, to_type: String },
    /// Exec output pin has no connections (dangling exec)
    DanglingExec,
    /// Required input pin has no value and no connection
    MissingRequiredInput,
    /// Two nodes have the same name in the same graph
    DuplicateNodeName,
    /// A connection references a node that doesn't exist
    BrokenReference { referenced_node: String },
    /// A connection references a pin that doesn't exist on the node
    BrokenPinReference { pin_id: String },
    /// Exec flow forms a cycle (infinite loop)
    ExecCycle { cycle: Vec<String> },
    /// Node has output pins but nothing is connected to them (warning level)
    UnusedOutput,
}

impl ValidationError {
    fn duplicate_name(node: &str, graph: &str) -> Self {
        ValidationError {
            kind: ErrorKind::DuplicateNodeName,
            node_name: node.to_string(),
            pin_name: None,
            message: format!("Duplicate node name '{}' in graph '{}'", node, graph),
        }
    }

    fn broken_ref(node: &str, pin: &str, missing: &str) -> Self {
        ValidationError {
            kind: ErrorKind::BrokenReference {
                referenced_node: missing.to_string(),
            },
            node_name: node.to_string(),
            pin_name: Some(pin.to_string()),
            message: format!(
                "Pin '{}' on '{}' references missing node '{}'",
                pin, node, missing
            ),
        }
    }
}

/// Validate a Blueprint and return all errors found
pub fn validate(blueprint: &Blueprint) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    for graph in &blueprint.graphs {
        errors.extend(validate_graph(graph));
    }
    errors
}

/// Validate a single graph
pub fn validate_graph(graph: &BpGraph) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // 1. Check for duplicate node names
    let mut seen_names: HashSet<&str> = HashSet::new();
    for node in &graph.nodes {
        if !seen_names.insert(node.name.as_str()) {
            errors.push(ValidationError::duplicate_name(&node.name, &graph.name));
        }
    }

    // Build name -> node map (first occurrence wins for subsequent checks)
    let node_map: HashMap<&str, &BpNode> = graph
        .nodes
        .iter()
        .map(|n| (n.name.as_str(), n))
        .collect();

    // 2. Check all pin connections
    for node in &graph.nodes {
        for pin in &node.pins {
            for link in &pin.linked_to {
                // Check target node exists
                let target_node = match node_map.get(link.node_name.as_str()) {
                    Some(n) => *n,
                    None => {
                        errors.push(ValidationError::broken_ref(
                            &node.name,
                            &pin.name,
                            &link.node_name,
                        ));
                        continue;
                    }
                };

                // Check target pin exists by GUID
                let target_pin = target_node.pins.iter().find(|p| p.id == link.pin_id);
                let target_pin = match target_pin {
                    Some(p) => p,
                    None => {
                        errors.push(ValidationError {
                            kind: ErrorKind::BrokenPinReference {
                                pin_id: link.pin_id.to_string(),
                            },
                            node_name: node.name.clone(),
                            pin_name: Some(pin.name.clone()),
                            message: format!(
                                "Pin '{}' on '{}' references non-existent pin {} on '{}'",
                                pin.name, node.name, link.pin_id, link.node_name
                            ),
                        });
                        continue;
                    }
                };

                // Check type compatibility: neither side is Wildcard, and categories differ
                if pin.pin_type.category != PinCategory::Wildcard
                    && target_pin.pin_type.category != PinCategory::Wildcard
                    && pin.pin_type.category != target_pin.pin_type.category
                {
                    errors.push(ValidationError {
                        kind: ErrorKind::TypeMismatch {
                            from_type: format!("{:?}", pin.pin_type.category),
                            to_type: format!("{:?}", target_pin.pin_type.category),
                        },
                        node_name: node.name.clone(),
                        pin_name: Some(pin.name.clone()),
                        message: format!(
                            "Type mismatch: '{}:{}'({:?}) -> '{}:{}'({:?})",
                            node.name,
                            pin.name,
                            pin.pin_type.category,
                            target_node.name,
                            target_pin.name,
                            target_pin.pin_type.category
                        ),
                    });
                }
            }
        }
    }

    // 3. Detect exec cycles using DFS
    errors.extend(detect_exec_cycles(graph, &node_map));

    errors
}

/// Detect cycles in exec flow using recursive DFS
fn detect_exec_cycles<'a>(
    graph: &'a BpGraph,
    node_map: &HashMap<&str, &'a BpNode>,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let mut visited: HashSet<&'a str> = HashSet::new();
    let mut in_stack: HashSet<&'a str> = HashSet::new();

    for node in &graph.nodes {
        if !visited.contains(node.name.as_str()) {
            let mut path = Vec::new();
            detect_cycle_dfs(
                node,
                node_map,
                &mut visited,
                &mut in_stack,
                &mut path,
                &mut errors,
            );
        }
    }

    errors
}

fn detect_cycle_dfs<'a>(
    node: &'a BpNode,
    node_map: &HashMap<&str, &'a BpNode>,
    visited: &mut HashSet<&'a str>,
    in_stack: &mut HashSet<&'a str>,
    path: &mut Vec<&'a str>,
    errors: &mut Vec<ValidationError>,
) {
    visited.insert(node.name.as_str());
    in_stack.insert(node.name.as_str());
    path.push(node.name.as_str());

    // Follow exec output connections only
    for pin in &node.pins {
        if pin.direction == PinDirection::Output
            && pin.pin_type.category == PinCategory::Exec
        {
            for link in &pin.linked_to {
                if let Some(next_node) = node_map.get(link.node_name.as_str()) {
                    if in_stack.contains(next_node.name.as_str()) {
                        // Found a cycle -- extract the cycle path
                        let cycle_start = path
                            .iter()
                            .position(|&n| n == next_node.name.as_str())
                            .unwrap_or(0);
                        let cycle: Vec<String> =
                            path[cycle_start..].iter().map(|s| s.to_string()).collect();
                        errors.push(ValidationError {
                            kind: ErrorKind::ExecCycle { cycle: cycle.clone() },
                            node_name: node.name.clone(),
                            pin_name: Some(pin.name.clone()),
                            message: format!(
                                "Exec cycle detected: {}",
                                cycle.join(" -> ")
                            ),
                        });
                    } else if !visited.contains(next_node.name.as_str()) {
                        detect_cycle_dfs(
                            next_node, node_map, visited, in_stack, path, errors,
                        );
                    }
                }
            }
        }
    }

    in_stack.remove(node.name.as_str());
    path.pop();
}

/// Check if a Blueprint is valid (no errors)
pub fn is_valid(blueprint: &Blueprint) -> bool {
    validate(blueprint).is_empty()
}

/// Format validation errors as a human-readable report
pub fn format_errors(errors: &[ValidationError]) -> String {
    if errors.is_empty() {
        return "Blueprint is valid -- no errors found.".to_string();
    }
    let mut out = format!("Blueprint has {} error(s):\n", errors.len());
    for (i, err) in errors.iter().enumerate() {
        let location = match &err.pin_name {
            Some(pin) => format!("{}::{}", err.node_name, pin),
            None => err.node_name.clone(),
        };
        let kind_label = format!("{:?}", err.kind);
        let label = kind_label.split('{').next().unwrap_or("").trim_end_matches(' ');
        out.push_str(&format!(
            "  {}. [{}] at {}: {}\n",
            i + 1,
            label,
            location,
            err.message
        ));
    }
    out
}

/// A Blueprint that has been validated and is known to be error-free
#[derive(Debug)]
pub struct ValidatedBlueprint(pub Blueprint);

impl ValidatedBlueprint {
    /// Validate and wrap a Blueprint, returning Err with all validation errors
    pub fn new(blueprint: Blueprint) -> Result<Self, Vec<ValidationError>> {
        let errors = validate(&blueprint);
        if errors.is_empty() {
            Ok(ValidatedBlueprint(blueprint))
        } else {
            Err(errors)
        }
    }

    pub fn inner(&self) -> &Blueprint {
        &self.0
    }

    pub fn into_inner(self) -> Blueprint {
        self.0
    }
}

// -- Tests -------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BpGraph, BpNode, Blueprint, Pin};
    use crate::types::PinType;

    // -- helpers ---------------------------------------------------------------

    /// Build a minimal Blueprint with one EventGraph containing the given nodes.
    fn make_bp(nodes: Vec<BpNode>) -> Blueprint {
        let mut bp = Blueprint::new("TestBP", "Actor");
        bp.graphs[0].nodes = nodes;
        bp
    }

    /// Build a graph directly (bypasses Blueprint wrapper).
    fn make_graph(name: &str, nodes: Vec<BpNode>) -> BpGraph {
        BpGraph {
            name: name.to_string(),
            graph_type: crate::ast::GraphType::EventGraph,
            nodes,
        }
    }

    fn event_node(name: &str) -> BpNode {
        BpNode::new("/Script/BlueprintGraph.K2Node_Event", name)
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"))
    }

    fn print_node(name: &str) -> BpNode {
        BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", name)
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"))
            .with_pin(Pin::data_input("InString", PinType::string()))
    }

    // -- test 1: valid blueprint with no connections --------------------------

    #[test]
    fn test_valid_empty_blueprint() {
        let bp = Blueprint::new("EmptyBP", "Actor");
        let errors = validate(&bp);
        assert!(errors.is_empty(), "Empty blueprint should be error-free: {:?}", errors);
    }

    // -- test 2: valid blueprint with valid connections -----------------------

    #[test]
    fn test_valid_connected_blueprint() {
        let mut graph = make_graph("EventGraph", vec![]);
        let ev = event_node("BeginPlay");
        let pr = print_node("PrintString");
        graph.nodes.push(ev);
        graph.nodes.push(pr);

        // Connect BeginPlay.then -> PrintString.execute (both exec)
        graph.connect("BeginPlay", "then", "PrintString", "execute");

        let errors = validate_graph(&graph);
        assert!(
            errors.is_empty(),
            "Valid connected graph should have no errors: {:?}",
            errors
        );
    }

    // -- test 3: duplicate node names -----------------------------------------

    #[test]
    fn test_duplicate_node_name() {
        let graph = make_graph(
            "EventGraph",
            vec![event_node("BeginPlay"), event_node("BeginPlay")],
        );
        let errors = validate_graph(&graph);
        assert!(
            errors.iter().any(|e| e.kind == ErrorKind::DuplicateNodeName),
            "Should detect duplicate node name"
        );
    }

    // -- test 4: broken node reference ----------------------------------------

    #[test]
    fn test_broken_node_reference() {
        let mut node = event_node("BeginPlay");
        // Manually add a link to a node that doesn't exist
        node.pins[1].linked_to.push(PinRef {
            node_name: "GhostNode".to_string(),
            pin_id: UeGuid::new(),
        });

        let graph = make_graph("EventGraph", vec![node]);
        let errors = validate_graph(&graph);

        let has_broken = errors.iter().any(|e| {
            matches!(&e.kind, ErrorKind::BrokenReference { referenced_node } if referenced_node == "GhostNode")
        });
        assert!(has_broken, "Should detect broken node reference: {:?}", errors);
    }

    // -- test 5: broken pin reference -----------------------------------------

    #[test]
    fn test_broken_pin_reference() {
        let target = print_node("PrintString");
        let mut source = event_node("BeginPlay");

        // Link to PrintString but with a fake pin GUID
        source.pins[1].linked_to.push(PinRef {
            node_name: "PrintString".to_string(),
            pin_id: UeGuid::from_str("DEADBEEFDEADBEEFDEADBEEFDEADBEEF"),
        });

        let graph = make_graph("EventGraph", vec![source, target]);
        let errors = validate_graph(&graph);

        let has_broken_pin = errors
            .iter()
            .any(|e| matches!(&e.kind, ErrorKind::BrokenPinReference { .. }));
        assert!(has_broken_pin, "Should detect broken pin reference: {:?}", errors);
    }

    // -- test 6: type mismatch (exec -> data) ---------------------------------

    #[test]
    fn test_type_mismatch_exec_to_data() {
        let mut target = print_node("PrintString");
        let mut source = event_node("BeginPlay");

        // Get real pin ID for the InString data-input pin
        let data_pin_id = target
            .find_pin("InString")
            .expect("InString pin missing")
            .id
            .clone();

        // Connect exec output -> data input (wrong!)
        source.pins[1].linked_to.push(PinRef {
            node_name: "PrintString".to_string(),
            pin_id: data_pin_id.clone(),
        });
        // Bidirectional: also add reverse link
        target
            .find_pin_mut("InString")
            .unwrap()
            .linked_to
            .push(PinRef {
                node_name: "BeginPlay".to_string(),
                pin_id: source.pins[1].id.clone(),
            });

        let graph = make_graph("EventGraph", vec![source, target]);
        let errors = validate_graph(&graph);

        let has_mismatch = errors
            .iter()
            .any(|e| matches!(&e.kind, ErrorKind::TypeMismatch { .. }));
        assert!(has_mismatch, "Should detect type mismatch: {:?}", errors);
    }

    // -- test 7: type mismatch (int -> string) --------------------------------

    #[test]
    fn test_type_mismatch_int_to_string() {
        let mut source = BpNode::new("K2Node_CallFunction", "GetInt")
            .with_pin(Pin::data_output("ReturnValue", PinType::int()));
        let mut target = BpNode::new("K2Node_CallFunction", "TakeString")
            .with_pin(Pin::data_input("Value", PinType::string()));

        let target_pin_id = target.pins[0].id.clone();
        let source_pin_id = source.pins[0].id.clone();

        source.pins[0].linked_to.push(PinRef {
            node_name: "TakeString".to_string(),
            pin_id: target_pin_id,
        });
        target.pins[0].linked_to.push(PinRef {
            node_name: "GetInt".to_string(),
            pin_id: source_pin_id,
        });

        let graph = make_graph("EventGraph", vec![source, target]);
        let errors = validate_graph(&graph);

        let has_mismatch = errors
            .iter()
            .any(|e| matches!(&e.kind, ErrorKind::TypeMismatch { .. }));
        assert!(has_mismatch, "Should detect int->string type mismatch: {:?}", errors);
    }

    // -- test 8: wildcard connections are allowed -----------------------------

    #[test]
    fn test_wildcard_connection_allowed() {
        let mut source = BpNode::new("K2Node_Select", "Select")
            .with_pin(Pin::data_output("ReturnValue", PinType::wildcard()));
        let mut target = BpNode::new("K2Node_CallFunction", "TakeString")
            .with_pin(Pin::data_input("Value", PinType::string()));

        let target_pin_id = target.pins[0].id.clone();
        let source_pin_id = source.pins[0].id.clone();

        source.pins[0].linked_to.push(PinRef {
            node_name: "TakeString".to_string(),
            pin_id: target_pin_id,
        });
        target.pins[0].linked_to.push(PinRef {
            node_name: "Select".to_string(),
            pin_id: source_pin_id,
        });

        let graph = make_graph("EventGraph", vec![source, target]);
        let errors = validate_graph(&graph);

        let has_mismatch = errors
            .iter()
            .any(|e| matches!(&e.kind, ErrorKind::TypeMismatch { .. }));
        assert!(!has_mismatch, "Wildcard connections should not be flagged as mismatches");
    }

    // -- test 9: exec cycle ---------------------------------------------------

    #[test]
    fn test_exec_cycle_detected() {
        let mut node_a = event_node("NodeA");
        let mut node_b = event_node("NodeB");

        // Get pin IDs
        let a_then_id = node_a.find_pin("then").unwrap().id.clone();
        let b_then_id = node_b.find_pin("then").unwrap().id.clone();
        let a_exec_id = node_a.find_pin("execute").unwrap().id.clone();
        let b_exec_id = node_b.find_pin("execute").unwrap().id.clone();

        // A.then -> B.execute
        node_a.find_pin_mut("then").unwrap().linked_to.push(PinRef {
            node_name: "NodeB".to_string(),
            pin_id: b_exec_id,
        });
        node_b.find_pin_mut("execute").unwrap().linked_to.push(PinRef {
            node_name: "NodeA".to_string(),
            pin_id: a_then_id.clone(),
        });

        // B.then -> A.execute  (cycle!)
        node_b.find_pin_mut("then").unwrap().linked_to.push(PinRef {
            node_name: "NodeA".to_string(),
            pin_id: a_exec_id,
        });
        node_a.find_pin_mut("execute").unwrap().linked_to.push(PinRef {
            node_name: "NodeB".to_string(),
            pin_id: b_then_id,
        });

        let graph = make_graph("EventGraph", vec![node_a, node_b]);
        let errors = validate_graph(&graph);

        let has_cycle = errors
            .iter()
            .any(|e| matches!(&e.kind, ErrorKind::ExecCycle { .. }));
        assert!(has_cycle, "Should detect exec cycle: {:?}", errors);
    }

    // -- test 10: no cycle in linear flow -------------------------------------

    #[test]
    fn test_no_cycle_in_linear_flow() {
        let mut graph = make_graph("EventGraph", vec![]);
        let ev = event_node("BeginPlay");
        let pr1 = print_node("Print1");
        let pr2 = print_node("Print2");
        graph.nodes.push(ev);
        graph.nodes.push(pr1);
        graph.nodes.push(pr2);
        graph.connect("BeginPlay", "then", "Print1", "execute");
        graph.connect("Print1", "then", "Print2", "execute");

        let errors = validate_graph(&graph);
        let has_cycle = errors
            .iter()
            .any(|e| matches!(&e.kind, ErrorKind::ExecCycle { .. }));
        assert!(!has_cycle, "Linear flow should not have a cycle: {:?}", errors);
    }

    // -- test 11: is_valid returns true for valid blueprint -------------------

    #[test]
    fn test_is_valid_true_for_clean_blueprint() {
        let bp = Blueprint::new("CleanBP", "Actor");
        assert!(is_valid(&bp), "Empty blueprint should be valid");
    }

    // -- test 12: is_valid returns false for blueprint with errors ------------

    #[test]
    fn test_is_valid_false_for_broken_blueprint() {
        let mut node = event_node("BeginPlay");
        // Broken link to non-existent node
        node.pins[1].linked_to.push(PinRef {
            node_name: "Nowhere".to_string(),
            pin_id: UeGuid::new(),
        });
        let bp = make_bp(vec![node]);
        assert!(!is_valid(&bp), "Blueprint with broken reference should be invalid");
    }

    // -- test 13: format_errors on empty list ---------------------------------

    #[test]
    fn test_format_errors_valid() {
        let msg = format_errors(&[]);
        assert!(msg.contains("valid"), "format_errors on no errors should say valid");
    }

    // -- test 14: format_errors on errors produces non-empty report ----------

    #[test]
    fn test_format_errors_has_content() {
        let mut node = event_node("BeginPlay");
        node.pins[1].linked_to.push(PinRef {
            node_name: "Ghost".to_string(),
            pin_id: UeGuid::new(),
        });
        let graph = make_graph("EventGraph", vec![node]);
        let errors = validate_graph(&graph);
        let report = format_errors(&errors);
        assert!(
            report.contains("error"),
            "format_errors report should mention errors: {}",
            report
        );
        assert!(
            report.contains("BeginPlay"),
            "Report should name the offending node: {}",
            report
        );
    }

    // -- test 15: ValidatedBlueprint wraps clean blueprint -------------------

    #[test]
    fn test_validated_blueprint_ok() {
        let bp = Blueprint::new("GoodBP", "Actor");
        let result = ValidatedBlueprint::new(bp);
        assert!(result.is_ok(), "Clean blueprint should produce ValidatedBlueprint::Ok");
        let vbp = result.unwrap();
        assert_eq!(vbp.inner().name, "GoodBP");
    }

    // -- test 16: ValidatedBlueprint rejects broken blueprint ----------------

    #[test]
    fn test_validated_blueprint_err() {
        let mut node = event_node("BeginPlay");
        node.pins[1].linked_to.push(PinRef {
            node_name: "Missing".to_string(),
            pin_id: UeGuid::new(),
        });
        let bp = make_bp(vec![node]);
        let result = ValidatedBlueprint::new(bp);
        assert!(
            result.is_err(),
            "Blueprint with errors should produce ValidatedBlueprint::Err"
        );
        let errs = result.unwrap_err();
        assert!(!errs.is_empty(), "Error list should not be empty");
    }

    // -- test 17: validate across multiple graphs -----------------------------

    #[test]
    fn test_validate_across_multiple_graphs() {
        let mut bp = Blueprint::new("MultiBP", "Actor");
        // Add a broken node to the function graph
        let mut bad_node = event_node("FuncEntry");
        bad_node.pins[1].linked_to.push(PinRef {
            node_name: "Phantom".to_string(),
            pin_id: UeGuid::new(),
        });
        bp.function_graph("MyFunc").nodes.push(bad_node);

        let errors = validate(&bp);
        assert!(
            !errors.is_empty(),
            "Should find errors across all graphs"
        );
    }
}
